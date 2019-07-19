#![allow(dead_code)]
extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2 as pc2;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use serde_json;
use syn::{parse_macro_input, ItemFn};

#[derive(Debug, Clone)]
enum Values<'s> {
    Int(i64),
    Str(&'s str),
    Struct(Vec<Values<'s>>),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct Variables<'a> {
    #[serde(borrow)]
    inner: std::collections::HashMap<&'a str, &'a str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DebugCollect {
    // make hashmap to store name and type
    vars: std::vec::Vec<String>,
}

impl DebugCollect {
    fn new() -> Self {
        DebugCollect { vars: Vec::new() }
    }
}

#[proc_macro_attribute]
pub fn dbgify(args: TokenStream, function: TokenStream) -> TokenStream {
    let func = parse_macro_input!(function as ItemFn);

    let mut dbg = DebugCollect::new();
    for (i, arg) in func.decl.inputs.iter().enumerate() {
        match arg {
            syn::FnArg::Captured(syn::ArgCaptured {
                pat: syn::Pat::Ident(arg_id),
                ..
            }) => {
                println!("ID {}", arg_id.ident);
                let a = arg_id.ident.clone().to_string();
                dbg.vars.push(a);
            }
            syn::FnArg::SelfRef(self_ref) => {
                println!("SELF REF {:#?}", self_ref);
            }
            syn::FnArg::SelfValue(syn::ArgSelf { self_token, .. }) => {
                println!("SELF OWN {:#?}", self_token);
            }
            syn::FnArg::Inferred(pat) => {
                println!("PAT {:#?}", pat);
            }
            syn::FnArg::Ignored(ty) => {
                println!("TY {:#?}", ty);
            }
            arg_pats => println!("ARGS {:#?}", arg_pats),
        }
    }

    let ser = serde_json::to_string(&dbg).unwrap();
    println!("{}", ser);

    let dbg_impl = quote! {

        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        struct Variables<'a> {
            #[serde(borrow)]
            inner: std::collections::HashMap<&'a str, &'a str>,
        }

        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        pub struct DebugCollect {
            vars: std::vec::Vec<String>
        }

        impl DebugCollect {
            fn new() -> Self {
                let d: DebugCollect = serde_json::from_str(#ser).unwrap();
                println!("{:?}", d);
                d
            }
        }
    };

    TokenStream::from(quote! {
        #dbg_impl
        #func
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
