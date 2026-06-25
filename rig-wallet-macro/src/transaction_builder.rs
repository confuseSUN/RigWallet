//! Parser and code generator for [`evm_transfer_builder!`] and [`tvm_transfer_builder!`].
//!
//! Public API and usage examples live in the crate root (`lib.rs`). This module handles:
//!
//! - Parsing `tool(description = "..."), StructName, ChainConfig [, TokenConfig]`
//! - Emitting the transfer struct and `new` / `with_request`
//! - Emitting `rig_core::tool::Tool` (parameter schema + `call` → `Transaction::send`)

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Ident, LitStr, Token, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

mod kw {
    syn::custom_keyword!(tool);
}

/// Contents of the leading `tool(...)` argument.
pub struct ToolMeta {
    /// Shown to the LLM as the tool description in `ToolDefinition`.
    pub description: String,
}

impl Parse for ToolMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut description = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            if key == "description" {
                description = Some(value.value());
            } else {
                return Err(syn::Error::new_spanned(
                    key,
                    "unknown key; only `description` is supported",
                ));
            }

            if !input.peek(Token![,]) {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            description: description.ok_or_else(|| {
                syn::Error::new(input.span(), "missing `description` in tool(...)")
            })?,
        })
    }
}

/// Parsed transfer-builder macro input (before the VM kind is set).
pub struct TransferBuilderInput {
    /// `"evm"` or `"tvm"` — filled in by the entry-point function.
    pub vm: String,
    pub tool: ToolMeta,
    /// Ident of the struct to generate (e.g. `Eip1559Transfer`, `ERC20Transfer`).
    pub name: Ident,
    /// Trait bound ident for the chain config type parameter `C`.
    pub chain_config: Ident,
    /// When `Some`, adds type parameter `T` for token configs (ERC-20 / SPL).
    pub token_config: Option<Ident>,
}

impl Parse for TransferBuilderInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<kw::tool>()?;
        let content;
        parenthesized!(content in input);
        let tool = content.parse::<ToolMeta>()?;

        input.parse::<Token![,]>()?;

        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let chain_config: Ident = input.parse()?;

        // Fourth ident is optional; `,)` trailing comma is valid.
        let token_config = if input.peek(Token![,]) {
            let lookahead = input.fork();
            lookahead.parse::<Token![,]>()?;
            if lookahead.peek(Ident) {
                input.parse::<Token![,]>()?;
                Some(input.parse::<Ident>()?)
            } else {
                None
            }
        } else {
            None
        };

        // Consume a lone trailing comma; reject any other leftover tokens.
        if !input.is_empty() {
            if input.peek(Token![,]) {
                let lookahead = input.fork();
                lookahead.parse::<Token![,]>()?;
                if lookahead.is_empty() {
                    input.parse::<Token![,]>()?;
                } else {
                    return Err(syn::Error::new(input.span(), "unexpected token"));
                }
            } else {
                return Err(syn::Error::new(input.span(), "unexpected token"));
            }
        }

        Ok(Self {
            vm: String::new(),
            tool,
            name,
            chain_config,
            token_config,
        })
    }
}

/// Emits `impl Tool for StructName<…>`.
///
/// Native transfers use the struct ident as `NAME`; token transfers use `T::TOOL_NAME`
/// so multiple tokens can share one generic struct without colliding in the agent toolset.
fn generate_tool_impl(
    name: &Ident,
    bounded_generics: &TokenStream2,
    ty_generics: &TokenStream2,
    tool: &ToolMeta,
    has_token: bool,
) -> TokenStream2 {
    let struct_name_lit = LitStr::new(&name.to_string(), Span::call_site());
    let description_lit = LitStr::new(&tool.description, Span::call_site());
    let value_description = LitStr::new(
        "Amount expressed in the token's smallest unit (according to its decimals)",
        Span::call_site(),
    );

    let name_expr = if has_token {
        quote! { T::TOOL_NAME }
    } else {
        quote! { #struct_name_lit }
    };

    quote! {
        impl #bounded_generics ::rig_core::tool::Tool for #name #ty_generics {
            const NAME: &'static str = #name_expr;
            type Error = ::rig_wallet_types::errors::Error;
            type Args = ::rig_wallet_types::request::TransferToolArgs;
            type Output = String;

            fn definition(
                &self,
                _prompt: String,
            ) -> impl ::std::future::Future<
                Output = ::rig_core::completion::ToolDefinition,
            > + ::std::marker::Send {
                async move {
                    ::rig_core::completion::ToolDefinition {
                        name: Self::NAME.to_string(),
                        description: #description_lit.to_string(),
                        parameters: ::serde_json::json!({
                            "type": "object",
                            "properties": {
                                "to": {
                                    "type": "string",
                                    "description": "Recipient address"
                                },
                                "value": {
                                    "type": "integer",
                                    "description": #value_description
                                }
                            },
                            "required": ["to", "value"]
                        }),
                    }
                }
            }

            fn call(
                &self,
                args: Self::Args,
            ) -> impl ::std::future::Future<
                Output = ::rig_wallet_types::errors::Result<Self::Output>,
            > + ::std::marker::Send {
                use ::rig_wallet_types::transaction::Transaction;

                let transfer = self.clone();
                async move {
                    transfer
                        .with_request(::rig_wallet_types::request::TxRequest::new(
                            args.to, args.value,
                        ))
                        .send()
                        .await
                }
            }
        }
    }
}

/// Builds the full expansion: struct + inherent impl + `Tool` impl.
pub fn generate_builder_impl(args: &TransferBuilderInput) -> TokenStream2 {
    let name = &args.name;
    let chain_config = &args.chain_config;
    let token_config = args.token_config.as_ref();
    let has_token = token_config.is_some();

    let (bounded_generics, ty_generics, phantom_field_defs, phantom_field_inits) =
        if let Some(token_config) = token_config {
            (
                quote! { <C: #chain_config, T: #token_config> },
                quote! { <C, T> },
                quote! {
                    _c: std::marker::PhantomData<C>,
                    _t: std::marker::PhantomData<T>,
                },
                quote! {
                    _c: std::marker::PhantomData,
                    _t: std::marker::PhantomData,
                },
            )
        } else {
            (
                quote! { <C: #chain_config> },
                quote! { <C> },
                quote! { _c: std::marker::PhantomData<C>, },
                quote! { _c: std::marker::PhantomData, },
            )
        };

    let (tx_unsigned_ty, tx_unsigned_init) = match args.vm.as_str() {
        "evm" => (
            quote! { Option<alloy::consensus::TypedTransaction> },
            quote! { tx_unsigned: None },
        ),
        "tvm" => (
            quote! { solana_sdk::transaction::Transaction },
            quote! { tx_unsigned: solana_sdk::transaction::Transaction::default() },
        ),
        other => {
            panic!("unsupported vm \"{other}\"; use evm_transfer_builder! or tvm_transfer_builder!")
        }
    };

    let tool_impl =
        generate_tool_impl(name, &bounded_generics, &ty_generics, &args.tool, has_token);

    quote! {
        #[derive(Clone, Default)]
        pub struct #name #bounded_generics {
            pub request: rig_wallet_types::request::TxRequest,
            tx_unsigned: #tx_unsigned_ty,
            #phantom_field_defs
        }

        impl #bounded_generics #name #ty_generics {
            pub fn new() -> Self {
                Self {
                    request: rig_wallet_types::request::TxRequest::default(),
                    #tx_unsigned_init,
                    #phantom_field_inits
                }
            }

            pub fn with_request(
                mut self,
                request: rig_wallet_types::request::TxRequest,
            ) -> Self {
                self.request = request;
                self
            }
        }

        #tool_impl
    }
}

pub fn generate_evm_transfer_builder(input: TokenStream) -> TokenStream {
    let mut args = parse_macro_input!(input as TransferBuilderInput);
    args.vm = "evm".to_string();
    TokenStream::from(generate_builder_impl(&args))
}

pub fn generate_tvm_transfer_builder(input: TokenStream) -> TokenStream {
    let mut args = parse_macro_input!(input as TransferBuilderInput);
    args.vm = "tvm".to_string();
    TokenStream::from(generate_builder_impl(&args))
}
