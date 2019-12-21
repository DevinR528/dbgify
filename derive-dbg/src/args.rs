// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2 as pc2;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use serde_json;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, parse_quote,
    punctuated::{IntoIter, Punctuated},
    token::Comma,
    ItemFn, Token, Type, FnArg, Ident, Pat, PatIdent
};

use dbg_collect;

pub(crate) struct ArgIdents {
    pub(crate) non_mut_args: Vec<PatIdent>,
    pub(crate) mut_args: Vec<PatIdent>,
    pub(crate) mut_under: Vec<PatIdent>,
    pub(crate) non_mut_under: Vec<PatIdent>,
    pub(crate) mut_string: Vec<String>,
    pub(crate) non_mut_string: Vec<String>,
}

impl ArgIdents {
    fn new(non_mut_args: Vec<PatIdent>, mut_args: Vec<PatIdent>) -> Self {
        let non_mut_under: Vec<PatIdent> = non_mut_args
            .iter()
            .map(|arg| {
                let mut a = arg.clone();
                a.ident = syn::Ident::new(&format!("_{}", arg.ident), pc2::Span::call_site());
                a
            })
            .collect();
        // vec of ident strings for print Fn map
        let non_mut_string: Vec<String> = non_mut_args
            .iter()
            .map(|arg| arg.ident.to_string())
            .collect();

        let mut_under: Vec<PatIdent> = mut_args
            .iter()
            .map(|arg| {
                let mut a = arg.clone();
                a.ident = syn::Ident::new(&format!("_{}", arg.ident), pc2::Span::call_site());
                a
            })
            .collect();
        // vec of ident strings for print Fn map
        let mut_string: Vec<String> = mut_args.iter().map(|arg| arg.ident.to_string()).collect();

        Self {
            non_mut_args,
            mut_args,
            mut_under,
            non_mut_under,
            mut_string,
            non_mut_string,
        }
    }

    fn arg_string(&self) -> (Vec<String>, Vec<String>) {
        (self.mut_string.clone(), self.non_mut_string.clone())
    }

    fn arg_private(&self) -> (Vec<PatIdent>, Vec<PatIdent>) {
        (self.mut_under.clone(), self.non_mut_under.clone())
    }

    fn arg_original(&self) -> (Vec<PatIdent>, Vec<PatIdent>) {
        (self.mut_args.clone(), self.non_mut_args.clone())
    }

    pub(crate) fn new_mut_decl(&self) -> (Vec<PatIdent>, Vec<PatIdent>, Vec<String>) {
        let (m_a, _) = self.arg_original();
        let (p_a, _) = self.arg_private();
        let (s_a, _) = self.arg_string();
        (m_a, p_a, s_a)
    }

    pub(crate) fn new_decl(&self) -> (Vec<PatIdent>, Vec<PatIdent>, Vec<String>) {
        let (_, o_a) = self.arg_original();
        let (_, p_a) = self.arg_private();
        let (_, s_a) = self.arg_string();
        (o_a, p_a, s_a)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Args {
    pub(crate) orig_args: Vec<syn::FnArg>,
    pub(crate) mut_args: Vec<syn::FnArg>,
    pub(crate) args: Vec<syn::FnArg>,
    pub(crate) mut_under: Vec<PatIdent>,
    pub(crate) scope: String,
}

impl Args {
    pub(crate) fn new(scope: String, args: Punctuated<FnArg, Comma>) -> Self {
        let orig_args = args.iter().cloned().collect();
        let mut ret = Args {
            orig_args,
            mut_args: Vec::new(),
            mut_under: Vec::new(),
            args: Vec::new(),
            scope,
        };
        args.into_iter().for_each(|arg| ret.push_mut_else(arg));
        ret
    }

    fn is_mut_arg(&self, arg: &syn::FnArg) -> bool {
        match arg {
            syn::FnArg::Receiver(slf) => slf.mutability.is_some(),
            syn::FnArg::Typed(a) => {
                match &*a.ty {
                    Type::Reference(ty) => ty.mutability.is_some(),
                    _ => false
                }
            },
            _ => false,
        }
    }

    fn push_mut_else(&mut self, arg: syn::FnArg) {
        if self.is_mut_arg(&arg) {
            self.mut_args.push(arg);
        } else {
            self.args.push(arg);
        }
    }

    pub(crate) fn capture_args(&self, dbg: &mut dbg_collect::DebugCollect) -> ArgIdents {
        self.orig_args()
            .iter()
            .map(expand_fn_arg)
            .for_each(|(arg_id, ty)| {
                let name = arg_id.ident.clone().to_string();
                let scope = self.scope.clone();
                // TODO remove expect for something better
                let am = dbg_collect::ArgMeta::new(
                    self.expand_arg_ty(&ty).expect("expand type failed"),
                    scope,
                );
                dbg.args.insert(name, am);
            });

        let m_args: Vec<PatIdent> = self
            .mut_args()
            .iter()
            .map(expand_fn_arg)
            .map(|(id, _ty)| id)
            .collect();

        let non_m_args: Vec<PatIdent> = self
            .non_mut_args()
            .iter()
            .map(expand_fn_arg)
            .map(|(id, _ty)| id)
            .collect();

        ArgIdents::new(non_m_args, m_args)
    }

    fn expand_arg_ty(&self, ty: &syn::Type) -> Option<String> {
        match ty {
            Type::Slice(_ty) => Some("Vec".to_string()),
            Type::Array(_ty) => Some("Vec".into()),
            Type::Ptr(_ty) => Some("Ptr".into()),
            Type::Reference(ty) => self.expand_arg_ty(&ty.elem),
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
            Type::Paren(ty) => self.expand_arg_ty(&ty.elem),
            Type::Group(ty) => self.expand_arg_ty(&ty.elem),
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
            },
            _ => panic!("WHY DAVID")
        }
    }

    pub(crate) fn mut_args(&self) -> Vec<syn::FnArg> {
        self.mut_args.clone()
    }

    pub(crate) fn non_mut_args(&self) -> Vec<syn::FnArg> {
        self.args.clone()
    }

    pub(crate) fn orig_args(&self) -> Vec<syn::FnArg> {
        self.orig_args.clone()
    }
}

fn expand_fn_arg(arg: &syn::FnArg) -> (PatIdent, syn::Type) {
    match arg {
        syn::FnArg::Typed(syn::PatType {
            pat,
            ty,
            ..
        }) => {
            match &**pat {
                Pat::Ident(p) => (p.clone(), *ty.clone()),
                a @ _ => { println!("{:?}", a); panic!() },
            }
        },
        ar => {
            println!("ARGS {:#?}", ar);
            panic!()
        }
    }
}

fn expand_macro(path: &syn::Path) -> Option<syn::Ident> {
    let syn::Path { segments, .. } = path;
    // make sure it is always last, cause trouble?
    let ps = segments.last()?;
    Some(ps.ident.clone())
}

fn expand_path(path: &syn::Path) -> Option<syn::Ident> {
    Some(path.segments.last()?.ident.clone())
}

fn expand_bounds(bounds: &Punctuated<syn::TypeParamBound, syn::token::Add>) -> Option<syn::Ident> {
    if let Some(b) = bounds.last() {
        if let syn::TypeParamBound::Trait(tb) = b {
            expand_path(&tb.path)
        } else {
            None
        }
    } else {
        None
    }
}
