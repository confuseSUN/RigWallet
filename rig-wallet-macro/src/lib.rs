#![warn(
    unused,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility
)]
#![forbid(unsafe_code)]

use proc_macro2::Span;
use syn::{Expr, ExprLit, Lit, LitStr, Meta};

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

#[proc_macro_derive(TokenConfig, attributes(mainnet_token, testnet_token, decimal))]
pub fn token_config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let mainnet_token =
        fetch_attr("mainnet_token", &ast.attrs).expect("Please supply a mainnet_token attribute");
    let mainnet_token = LitStr::new(&mainnet_token, Span::call_site());

    let testnet_token = fetch_attr("testnet_token", &ast.attrs).unwrap_or_default();
    let testnet_token = LitStr::new(&testnet_token, Span::call_site());

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
               const TOKEN: &str = #mainnet_token;
               #[cfg(feature = "testnet")]
               const TOKEN: &str = #testnet_token;

               const DECIMAL: Option<u8> = #decimal;
            }
        };
    };

    code.into()
}

fn fetch_attr(name: &str, attrs: &[syn::Attribute]) -> Option<String> {
    // Go over each attribute
    for attr in attrs {
        match attr.meta {
            // If the attribute's path matches `name`, and if the attribute is of
            // the form `#[name = "value"]`, return `value`
            Meta::NameValue(ref nv) if nv.path.is_ident(name) => {
                // Extract and return the string value.
                // If `value` is not a string, return an error
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(ref s),
                    ..
                }) = nv.value
                {
                    return Some(s.value());
                }
                panic!("attribute {name} should be a string")
            }
            _ => {}
        }
    }
    None
}
