// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]
#![recursion_limit = "512"]

extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2 as pc2;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use serde_json;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, ItemFn, Token, Type,
};

use dbg_collect;

fn expand_macro(path: &syn::Path) -> Option<syn::Ident> {
    let syn::Path { segments, .. } = path;
    // make sure it is always last, cause trouble?
    if let Some(ps) = segments.last() {
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
            if let Some(mac_path) = expand_macro(&mac.path) {
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

fn insert_lifetime {
    
}

fn expand_path(path: &syn::Path) -> Option<syn::Ident> {
    if let Some(ps) = path.segments.last() {
        Some(ps.value().ident.clone())
    } else {
        None
    }
}

fn expand_bounds(bounds: &Punctuated<syn::TypeParamBound, syn::token::Add>) -> Option<syn::Ident> {
    if let Some(b) = bounds.last() {
        if let syn::TypeParamBound::Trait(tb) = b.value() {
            expand_path(&tb.path)
        } else {
            None
        }
    } else {
        None
    }
}

fn expand_arg_ty(ty: &syn::Type) -> Option<String> {
    match ty {
        Type::Slice(_ty) => Some("Vec".to_string()),
        Type::Array(_ty) => Some("Vec".into()),
        Type::Ptr(_ty) => Some("Ptr".into()),
        Type::Reference(ty) => expand_arg_ty(&ty.elem),
        Type::BareFn(_ty) => Some("Fn".into()),
        Type::Never(_ty) => Some("Never".into()),
        Type::Tuple(_ty) => Some("Tuple".into()),
        Type::Path(ty) => {
            if let Some(id) = expand_path(&ty.path) {
                Some(id.to_string())
            } else {
                None
            }
        }
        Type::TraitObject(ty) => {
            if let Some(trait_obj) = expand_bounds(&ty.bounds) {
                // TODO make know its a trait
                Some(trait_obj.to_string())
            } else {
                eprintln!("found no trait");
                None
            }
        }
        Type::ImplTrait(ty) => {
            if let Some(trait_impl) = expand_bounds(&ty.bounds) {
                // TODO make know its a trait
                Some(trait_impl.to_string())
            } else {
                eprintln!("found no impl'ed trait");
                None
            }
        }
        Type::Paren(ty) => expand_arg_ty(&ty.elem),
        Type::Group(ty) => expand_arg_ty(&ty.elem),
        Type::Infer(_ty) => Some("Underscore".into()),
        Type::Macro(ty) => {
            if let Some(id) = expand_macro(&ty.mac.path) {
                Some(id.to_string())
            } else {
                None
            }
        }
        Type::Verbatim(ty) => {
            eprintln!("VERBATIM {:#?}", ty);
            panic!()
        }
    }
}

fn capture_mut_args(arg: syn::FnArg) -> bool {
    match arg {
        syn::FnArg::Captured(ac) => {
            if let syn::Type::Reference(ty_ref) = &mut ac.ty {
                if let Some(ty_mut) = ty_ref.mutability {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

fn capture_mut_locals() {}

#[proc_macro_attribute]
pub fn dbgify(args: TokenStream, function: TokenStream) -> TokenStream {
    assert!(args.is_empty());
    let mut func = parse_macro_input!(function as ItemFn);

    // println!("{:#?}", func);
    insert_bp(&mut func.block.stmts);

    let mut dbg = dbg_collect::DebugCollect::default();
    let mut p_args: Vec<(syn::PatIdent, syn::Type)> = Vec::new();

    for arg in func.decl.inputs.iter() {
        match &arg {
            syn::FnArg::Captured(syn::ArgCaptured {
                pat: syn::Pat::Ident(arg_id),
                ty,
                ..
            }) => {
                // println!("{:#?}", ty);
                p_args.push((arg_id.clone(), ty.clone()));
                let name = arg_id.ident.clone().to_string();
                let scope = func.ident.clone().to_string();
                // TODO remove expect for something better
                let am = dbg_collect::ArgMeta::new(
                    expand_arg_ty(&ty).expect("expand type failed"),
                    scope,
                );
                dbg.args.insert(name, am);
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

    // vec of types for casting/transmute
    let arg_ty: Vec<syn::Type> = p_args.iter().map(|(_, ty)| ty).cloned().collect();;
    // strip any & or &mut from type so we don't double ref when we unsafe cast
    // with show_fmt
    let base_ty: Vec<syn::Type> = p_args
        .iter()
        .map(|(_, ty)| match ty {
            syn::Type::Reference(ty_ref) => {
                let t = ty_ref.elem.clone();
                *t
            }
            t => t.clone(),
        })
        .collect();

    println!("{:#?}", base_ty);

    // vec of ident strings for print Fn map
    let arg_str: Vec<String> = p_args
        .iter()
        .map(|(arg, _)| arg.ident.to_string())
        .collect();
    // unchanged arg ident
    let arg_id: Vec<syn::PatIdent> = p_args.iter().map(|(arg, _)| arg).cloned().collect();
    let mut_args: Vec<syn::PatIdent> = p_args.iter().filter(|(arg, _)| capture_mut_args(arg)).cloned().collect();
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
    // TODO this is terible why do i have to do this
    let capt_clone = capt_arg.clone();

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
                 
                let print_fn = dbg_collect::PrintFn(Box::new(move || {
                    // TODO clean up type signature
                    dbg_collect::show_fmt::<_, #base_ty, usize>(&#capt_clone)
                }));
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
