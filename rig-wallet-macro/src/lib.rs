#![warn(
    unused,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility
)]
#![forbid(unsafe_code)]

//! Proc-macros used by the RigWallet workspace.
//!
//! This crate has two responsibilities:
//!
//! 1. **Configuration derives** — turn declarative attributes into `ChainConfig` /
//!    `TokenConfig` trait implementations.
//! 2. **Transfer builders** — generate a transfer struct, constructor helpers, and a
//!    [`rig_core::tool::Tool`](https://docs.rs/rig-core) implementation for AI agents.
//!
//! Transaction signing and broadcasting are **not** generated here; they are implemented
//! manually in `rig-wallet-executor` via the `Transaction` trait.
//!
//! # Macro overview
//!
//! | Macro | Kind | Output |
//! |-------|------|--------|
//! | [`ChainConfig`] derive | `#[derive(ChainConfig)]` | `RPC`, `CHAIN_ID` constants |
//! | [`TokenConfig`] derive | `#[derive(TokenConfig)]` | `TOKEN`, `DECIMAL`, `TOOL_NAME` |
//! | [`evm_transfer_builder!`] | `macro` | EVM transfer struct + `Tool` |
//! | [`tvm_transfer_builder!`] | `macro` | Solana transfer struct + `Tool` |
//!
//! # Typical layout
//!
//! ```text
//! rig-wallet-executor          rig-wallet (facade)
//! ┌─────────────────────┐      ┌──────────────────────────┐
//! │ evm_transfer_builder!│      │ #[derive(ChainConfig)]   │
//! │   → ERC20Transfer    │◄─────│ pub type ETHUSDC =       │
//! │ impl Transaction     │      │   ERC20Transfer<C, T>    │
//! └─────────────────────┘      │ #[derive(TokenConfig)]   │
//!                                └──────────────────────────┘
//! ```
//!
//! # `ChainConfig` derive
//!
//! Attach to a unit struct in the facade crate (`rig-wallet`):
//!
//! ```ignore
//! #[derive(Clone, ChainConfig)]
//! #[mainnet_rpc = "https://…"]
//! #[testnet_rpc = "https://…"]          // optional; defaults to ""
//! #[mainnet_chainid = "1"]               // optional
//! #[testnet_chainid = "11155111"]       // optional
//! pub struct ETHConfig;
//! ```
//!
//! `RPC` and `CHAIN_ID` switch on the `testnet` feature flag of the consuming crate.
//!
//! # `TokenConfig` derive
//!
//! One struct per token; used as the `T` generic on token transfer builders:
//!
//! ```ignore
//! #[derive(Clone, TokenConfig)]
//! #[mainnet_token = "0x…"]
//! #[testnet_token = "0x…"]              // optional; falls back to mainnet_token
//! #[decimal = "6"]                      // optional; required for SPL
//! #[tool_name = "ETHUSDC"]              // optional; defaults to struct ident
//! pub struct USDCConfig;
//! ```
//!
//! `TOOL_NAME` must be unique per token so each `ERC20Transfer<C, T>` / `SplTransfer<C, T>`
//! instance registers as a distinct agent tool.
//!
//! # Transfer builder macros
//!
//! Invoked once per transfer type inside `rig-wallet-executor`:
//!
//! ```ignore
//! // Native asset (3 arguments)
//! evm_transfer_builder!(
//!     tool(description = "Transfer ETH to a given address."),
//!     Eip1559Transfer,   // generated struct name
//!     ChainConfig,       // trait bound for C
//! );
//!
//! // Token transfer (4 arguments)
//! evm_transfer_builder!(
//!     tool(description = "Transfer ERC20 tokens to a given address."),
//!     ERC20Transfer,
//!     ChainConfig,
//!     TokenConfig,       // trait bound for T
//! );
//! ```
//!
//! `tvm_transfer_builder!` has the same shape for Solana (`SolTransfer`, `SplTransfer`).
//!
//! ## Generated per invocation
//!
//! - `pub struct Name<C>` or `pub struct Name<C, T>` with `request`, `tx_unsigned`, `PhantomData`
//! - `new()` and `with_request(TxRequest)`
//! - `impl Tool` with JSON schema `{ to: string, value: integer }`
//!
//! ## `Tool::NAME` rules
//!
//! | Transfer kind | `Tool::NAME` source |
//! |---------------|---------------------|
//! | Native (`Eip1559Transfer`, `SolTransfer`) | Struct ident, e.g. `"Eip1559Transfer"` |
//! | Token (`ERC20Transfer`, `SplTransfer`) | `T::TOOL_NAME`, e.g. `"ETHUSDC"` |
//!
//! The facade crate then aliases concrete types:
//!
//! ```ignore
//! pub type ETHUSDC = ERC20Transfer<ETHConfig, USDCConfig>;
//! // ETHUSDC::new() → Tool::NAME == "ETHUSDC"
//! ```
//!
//! A single generic `impl Tool for ERC20Transfer<C, T>` is monomorphized per `T`; type
//! aliases do not get separate impls.

use proc_macro2::Span;
use syn::{Expr, ExprLit, Lit, LitStr, Meta};

mod transaction_builder;

/// Derives `ChainConfig` from struct attributes.
///
/// # Attributes
///
/// | Attribute | Required | Notes |
/// |-----------|----------|-------|
/// | `mainnet_rpc` | yes | Mainnet JSON-RPC URL |
/// | `testnet_rpc` | no | Defaults to `""` |
/// | `mainnet_chainid` | no | EVM chain ID on mainnet |
/// | `testnet_chainid` | no | EVM chain ID on testnet |
#[proc_macro_derive(
    ChainConfig,
    attributes(mainnet_rpc, testnet_rpc, mainnet_chainid, testnet_chainid)
)]
pub fn chain_config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let mainnet_rpc =
        fetch_attr("mainnet_rpc", &ast.attrs).expect("Please supply a mainnet_rpc attribute");

    let testnet_rpc: String = fetch_attr("testnet_rpc", &ast.attrs).unwrap_or_default();

    let mainnet_chainid: Option<u64> = fetch_attr("mainnet_chainid", &ast.attrs)
        .map(|x| x.parse().expect("mainnet_chainid should be a number"));

    let testnet_chainid: Option<u64> = fetch_attr("testnet_chainid", &ast.attrs)
        .map(|x| x.parse().expect("testnet_chainid should be a number"));

    chain_config_helper(
        mainnet_rpc,
        testnet_rpc,
        mainnet_chainid,
        testnet_chainid,
        ast.ident,
    )
    .into()
}

/// Emits a `ChainConfig` impl inside a const anonymous scope.
pub(crate) fn chain_config_helper(
    mainnet_rpc: String,
    testnet_rpc: String,
    mainnet_chainid: Option<u64>,
    testnet_chainid: Option<u64>,
    config_name: proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let mainnet_rpc = LitStr::new(&mainnet_rpc, Span::call_site());

    let testnet_rpc = LitStr::new(&testnet_rpc, Span::call_site());

    let mainnet_chainid = match mainnet_chainid {
        Some(id) => {
            let lit_id = proc_macro2::Literal::u64_unsuffixed(id);
            quote::quote! { Some(#lit_id) }
        }
        None => quote::quote! { None },
    };

    let testnet_chainid = match testnet_chainid {
        Some(id) => {
            let lit_id = proc_macro2::Literal::u64_unsuffixed(id);
            quote::quote! { Some(#lit_id) }
        }
        None => quote::quote! { None },
    };

    quote::quote! {
        const _: () = {
           #[automatically_derived]
           impl ChainConfig for #config_name {
            #[cfg(not(feature = "testnet"))]
            const RPC: &'static str = #mainnet_rpc;
            #[cfg(feature = "testnet")]
            const RPC: &'static str = #testnet_rpc;

            #[cfg(not(feature = "testnet"))]
            const CHAIN_ID: Option<u64> = #mainnet_chainid;
            #[cfg(feature = "testnet")]
            const CHAIN_ID: Option<u64> = #testnet_chainid;
            }
        };
    }
}

/// Derives `TokenConfig` from struct attributes.
///
/// # Attributes
///
/// | Attribute | Required | Notes |
/// |-----------|----------|-------|
/// | `mainnet_token` | yes | Token mint / contract address on mainnet |
/// | `testnet_token` | no | Falls back to `mainnet_token` when omitted |
/// | `decimal` | no | Token decimals; required for SPL `transfer_checked` |
/// | `tool_name` | no | Agent tool name; defaults to the struct ident |
#[proc_macro_derive(
    TokenConfig,
    attributes(mainnet_token, testnet_token, decimal, tool_name),
)]
pub fn token_config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let mainnet_token =
        fetch_attr("mainnet_token", &ast.attrs).expect("Please supply a mainnet_token attribute");
    let mainnet_token_lit = LitStr::new(&mainnet_token, Span::call_site());

    let testnet_token = fetch_attr("testnet_token", &ast.attrs).unwrap_or(mainnet_token);
    let testnet_token_lit = LitStr::new(&testnet_token, Span::call_site());

    let tool_name = fetch_attr("tool_name", &ast.attrs).unwrap_or_else(|| ast.ident.to_string());
    let tool_name = LitStr::new(&tool_name, Span::call_site());

    let decimal: Option<u8> =
        fetch_attr("decimal", &ast.attrs).map(|x| x.parse().expect("decimal should be a number"));
    let decimal = match decimal {
        Some(id) => {
            let lit_id = proc_macro2::Literal::u8_unsuffixed(id);
            quote::quote! { Some(#lit_id) }
        }
        None => quote::quote! { None },
    };

    let name = ast.ident;
    let code = quote::quote! {
        const _: () = {
           #[automatically_derived]
           impl TokenConfig for #name {
               #[cfg(not(feature = "testnet"))]
               const TOKEN: &str = #mainnet_token_lit;
               #[cfg(feature = "testnet")]
               const TOKEN: &str = #testnet_token_lit;

               const DECIMAL: Option<u8> = #decimal;
               const TOOL_NAME: &'static str = #tool_name;
            }
        };
    };

    code.into()
}

/// Generates an EVM transfer struct and `Tool` impl.
///
/// See the [crate-level documentation](self) for syntax and naming rules.
///
/// `Transaction` (`build` / `sign` / `send_signed`) must be implemented separately in
/// `rig-wallet-executor`.
#[proc_macro]
pub fn evm_transfer_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    transaction_builder::generate_evm_transfer_builder(input)
}

/// Generates a Solana (TVM) transfer struct and `Tool` impl.
///
/// Same input shape as [`evm_transfer_builder!`]; sets `tx_unsigned` to
/// `solana_sdk::transaction::Transaction` instead of an Alloy typed transaction.
#[proc_macro]
pub fn tvm_transfer_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    transaction_builder::generate_tvm_transfer_builder(input)
}

/// Reads `#[attr = "value"]` from a derive input's attribute list.
fn fetch_attr(name: &str, attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if let Meta::NameValue(ref nv) = attr.meta
            && nv.path.is_ident(name)
        {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(ref s),
                ..
            }) = nv.value
            {
                return Some(s.value());
            }
            panic!("attribute {name} should be a string");
        }
    }
    None
}
