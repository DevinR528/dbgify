// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]
#![recursion_limit = "128"]
extern crate proc_macro;

use std::collections::HashMap;

use crossterm;
use proc_macro::TokenStream;
use proc_macro2 as pc2;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use serde_json;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, ItemFn};

use dbg_collect::*;

fn expand_ty() {}

#[proc_macro_attribute]
pub fn dbgify(args: TokenStream, function: TokenStream) -> TokenStream {
    let func = parse_macro_input!(function as ItemFn);

    println!("{:#?}", func);

    let mut dbg = DebugCollect::new();
    let mut args: Vec<(syn::PatIdent, syn::Type)> = Vec::new();

    for (i, arg) in func.decl.inputs.iter().enumerate() {
        match arg {
            syn::FnArg::Captured(syn::ArgCaptured {
                pat: syn::Pat::Ident(arg_id),
                ty,
                ..
            }) => {
                args.push((arg_id.clone(), ty.clone()));
                println!("ID {}", arg_id.ident);
                let name = arg_id.ident.clone().to_string();
                let scope = func.ident.clone().to_string();
                dbg.args.insert(name, scope);
                println!("{:#?}", dbg);
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

    let ret_ty: Punctuated<syn::Type, Comma> = args.iter().map(|(_, ty)| ty).cloned().collect();
    let : Punctuated<syn::Type, Comma> = args.iter().map(|(_, ty)| ty).cloned().collect();
    let dbg_impl = quote! {
        // #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        // struct Variables<'a> {
        //     #[serde(borrow)]
        //     inner: std::collections::HashMap<&'a str, &'a str>,
        // }
        
        // TODO capture args and locals from debugee
        const args: impl Fn() = || {
            
        };
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        pub struct DebugCollect {
            pub args: std::collections::HashMap<String, String>,
        }

        impl DebugCollect {
            fn new() -> Self {
                let d: DebugCollect = serde_json::from_str(#ser).unwrap();
                // let vals: HashMap<_, _> = d.iter().map(|(k, v| {
                //     let exp_name = syn::Ident::new(k, pc2::Span::call_site());
                //     let capture = quote!{ |#exp_name| println!("{}", #exp_name); };
                //     (k, capture)
                // })).collect();
                d
            }

            pub fn step(&self) -> std::io::Result<String> {
                println!("type var name or tab to auto-complete");
                fn print_loop(dbg: &DebugCollect) -> std::io::Result<String> {
                    let mut input = crossterm::input();
                    let line = input.read_line()?;
                    if let Some(var) = dbg.args.get(line.as_str()) {
                        return Ok(var.to_string())
                    } else {
                        println!("could not find variable in scope");
                        print_loop(&dbg)
                    }
                }
                print_loop(&self)
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
