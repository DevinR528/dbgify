// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]
#![recursion_limit = "128"]
extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2 as pc2;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use serde_json;
use syn::{parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, ItemFn, Token};

use dbg_collect;

fn expand_ty(path: &syn::Path) -> Option<syn::Ident> {
    let syn::Path { segments, .. } = path;
    if let Some(ps) = segments.first() {
        Some(ps.value().ident.clone())
    } else {
        None
    }
}

fn capture_local(stmts: &Vec<syn::Stmt>) {
    println!("{:#?}", stmts);
}

/// Insert step debug code in place of bp!()
///
/// Examples
///
/// ```
/// fn fake_main() {
///     let mut x = 0;
///     bp!();
///     x = 10;
/// }
/// ```
/// allows you to inspect every change in the value of x or any other argument,
/// local, or captured variable.
fn insert_bp(stmts: &mut Vec<syn::Stmt>) {
    stmts.iter_mut().for_each(|s| match s {
        syn::Stmt::Semi(syn::Expr::Macro(syn::ExprMacro { mac, .. }), _) => {
            if let Some(mac_path) = expand_ty(&mac.path) {
                if mac_path.to_string() == "bp" {
                    let dbg_step: syn::Stmt = parse_quote! {
                        dbg.step(&print_map).unwrap();
                    };
                    *s = dbg_step;
                }
            }
        }
        _ => {}
    });
}

fn insert_lifetime(arg: &mut syn::FnArg) {
    match arg {
        syn::FnArg::Captured(ac) => {
            if let syn::Type::Reference(ty_ref) = &mut ac.ty {
                match ty_ref.lifetime {
                    None => {
                        ty_ref.lifetime =
                            Some(syn::Lifetime::new("'static", pc2::Span::call_site()));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

#[proc_macro_attribute]
pub fn dbgify(args: TokenStream, function: TokenStream) -> TokenStream {
    assert!(args.is_empty());
    let mut func = parse_macro_input!(function as ItemFn);

    // println!("{:#?}", func);
    insert_bp(&mut func.block.stmts);

    let mut dbg = dbg_collect::DebugCollect::default();
    let mut p_args: Vec<(syn::PatIdent, syn::Type)> = Vec::new();

    for arg in func.decl.inputs.iter_mut() {
        match &arg {
            syn::FnArg::Captured(syn::ArgCaptured {
                pat: syn::Pat::Ident(arg_id),
                ty,
                ..
            }) => {
                p_args.push((arg_id.clone(), ty.clone()));
                let name = arg_id.ident.clone().to_string();
                let scope = func.ident.clone().to_string();
                dbg.args.insert(name, scope);
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
            ar => println!("ARGS {:#?}", ar),
        }
    }

    let arg_ty: Vec<syn::Type> = p_args.iter().map(|(_, ty)| ty).cloned().collect();

    // vec of ident strings for print Fn map
    let arg_str: Vec<String> = p_args
        .iter()
        .map(|(arg, _)| arg.ident.to_string())
        .collect();

    // unchanged arg ident
    let arg_id: Vec<syn::PatIdent> = p_args.iter().map(|(arg, _)| arg).cloned().collect();

    // changed arg ident: '_arg'
    let capt_arg: Vec<syn::PatIdent> = p_args
        .iter()
        .map(|(arg, _)| {
            let mut a = arg.clone();
            a.ident = syn::Ident::new(&format!("_{}", arg.ident), pc2::Span::call_site());
            a
        })
        .collect();

    // clone for use in print Fn as ident
    let ca_clone = capt_arg.clone();

    let ret = match &func.decl.output {
        syn::ReturnType::Default => quote!(-> ()),
        out @ syn::ReturnType::Type(..) => quote!(#out),
    };

    // serialize the obj to 'send' to the running program
    let ser = serde_json::to_string(&dbg).unwrap();

    let body = func.block;
    func.block = Box::new(parse_quote! ({
        let __result = (|| #ret {

            let mut print_map: std::collections::HashMap<String, PrintFn> = std::collections::HashMap::new();
            #(
                // must re-bind or borrow check complains
                // and to move into closure must clone
                let #capt_arg = #arg_id.clone();

                let print_fn = dbg_collect::PrintFn(Box::new(move || println!("{}", #ca_clone)));

                print_map.insert(#arg_str.into(), print_fn);
            )*

            let dbg = dbg_collect::DebugCollect::deserialize(#ser);
            #body
        })();
    }));

    TokenStream::from(quote! {
        #func
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
