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
    parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, ItemFn, Token, Type,
};

use dbg_collect;

#[derive(Debug, Clone)]
pub(crate) struct Func {
    dbg_col: dbg_collect::DebugCollect,
    _fn: syn::ItemFn,
    args: Vec<syn::FnArg>,
    locals: Vec<syn::Local>,
    stmts: Vec<syn::Stmt>,
}

impl Func {
    pub fn new(function: &ItemFn) -> Self {
        Func {
            dbg_col: dbg_collect::DebugCollect::default(),
            _fn: function.clone(),
            args: Vec::default(),
            locals: Vec::default(),
            stmts: function.block.stmts.clone(),
        }
    }

    pub fn item_fn(mut self) -> syn::ItemFn {
        self.insert_bp();
        self.visit_stmts_mut();
        self.set_body();
        self._fn
    }

    fn expand_path(&self, path: &syn::Path) -> Option<syn::Ident> {
        if let Some(ps) = path.segments.last() {
            Some(ps.value().ident.clone())
        } else {
            None
        }
    }

    fn expand_bounds(
        &self,
        bounds: &Punctuated<syn::TypeParamBound, syn::token::Add>,
    ) -> Option<syn::Ident> {
        if let Some(b) = bounds.last() {
            if let syn::TypeParamBound::Trait(tb) = b.value() {
                self.expand_path(&tb.path)
            } else {
                None
            }
        } else {
            None
        }
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
                if let Some(id) = self.expand_path(&ty.path) {
                    Some(id.to_string())
                } else {
                    None
                }
            }
            Type::TraitObject(ty) => {
                if let Some(trait_obj) = self.expand_bounds(&ty.bounds) {
                    // TODO make know its a trait
                    Some(trait_obj.to_string())
                } else {
                    eprintln!("found no trait");
                    None
                }
            }
            Type::ImplTrait(ty) => {
                if let Some(trait_impl) = self.expand_bounds(&ty.bounds) {
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
            }
        }
    }

    fn capture_args(
        &mut self,
    ) -> (
        Vec<syn::PatIdent>,
        Vec<syn::PatIdent>,
        Vec<syn::PatIdent>,
        Vec<String>,
    ) {
        let mut p_args: Vec<(syn::PatIdent, syn::Type)> = Vec::new();
        for arg in self._fn.decl.inputs.iter() {
            match &arg {
                syn::FnArg::Captured(syn::ArgCaptured {
                    pat: syn::Pat::Ident(arg_id),
                    ty,
                    ..
                }) => {
                    // println!("{:#?}", ty);
                    p_args.push((arg_id.clone(), ty.clone()));
                    let name = arg_id.ident.clone().to_string();
                    let scope = self._fn.ident.clone().to_string();
                    // TODO remove expect for something better
                    let am = dbg_collect::ArgMeta::new(
                        self.expand_arg_ty(&ty).expect("expand type failed"),
                        scope,
                    );
                    self.dbg_col.args.insert(name, am);
                }
                ar => println!("ARGS {:#?}", ar),
            }
        }
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
        // TODO this is terible why do i have to do this
        (capt_arg.clone(), arg_id, capt_arg.clone(), arg_str)
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
    pub fn insert_bp(&mut self) {
        self.stmts.iter_mut().for_each(|s| match s {
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

    pub fn set_body(&mut self) {
        let ret = match &self._fn.decl.output {
            syn::ReturnType::Default => quote!(-> ()),
            out @ syn::ReturnType::Type(..) => quote!(#out),
        };

        // serialize the obj to 'send' to the running program
        let ser = serde_json::to_string(&self.dbg_col).unwrap();

        let (capt_arg, arg_id, capt_clone, arg_str) = self.capture_args();

        let body = self._fn.block.clone();
        self._fn.block = Box::new(parse_quote! ({
            std::thread_local! {
                static VARS: dbg_collect::VarRef = dbg_collect::VarRef::new();
            }
            let __result = (|| #ret {

                let mut print_map: std::collections::HashMap<String, PrintFn> = std::collections::HashMap::new();
                #(
                    let #capt_arg = std::cell::UnsafeCell::new(#arg_id);
                    VARS.with(|var| var.set_ref(&#capt_clone));
                    let print_fn = dbg_collect::PrintFn(Box::new(move || {
                        VARS.with(|var| unsafe {
                            if let Some(v) = var.0.get() {
                                let ptr = &*(v.as_ref().get());
                                println!("{:?}", ptr.as_debug());
                            } else {
                                eprintln!("accessed var after drop unsafe be careful");
                            }
                        });
                    }));
                    print_map.insert(#arg_str.into(), print_fn);
                )*
                let dbg = dbg_collect::DebugCollect::deserialize(#ser);
                #body
            })();
        }));
    }

    fn capture_mut_args(&self, arg: &syn::FnArg) -> bool {
        match arg {
            syn::FnArg::Captured(ac) => {
                if let syn::Type::Reference(ty_ref) = &ac.ty {
                    if let Some(_ty_mut) = ty_ref.mutability {
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

    fn check_mut_args(&self, f: Box<dyn FnMut(&syn::FnArg)>) {
        // mut args
        self._fn
            .decl
            .inputs
            .iter()
            .filter(|arg| self.capture_mut_args(arg))
            .for_each(|arg| (*f)(arg));
    }

    fn visit_stmts_mut(&mut self) {
        let visit = |arg: &syn::FnArg| {
            let (id, _) = expand_fn_arg(arg);
            self.stmts.iter_mut().for_each(|s| {
                // TODO
                if is_mut_var(s, id) {
                    insert_stmt(s);
                }
            })
        };
        let mut_args = self.check_mut_args(Box::new(visit));
    }

    fn capture_local(&self) {
        println!("{:#?}", self.stmts);
    }

    fn collect_type(&self, p_args: &Vec<(syn::PatIdent, syn::Type)>) {
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
    }
}

fn is_mut_var(stmt: &syn::Stmt, id: &syn::Ident) -> bool {
    true
}

fn insert_stmt(stmt: &mut syn::Stmt) {}

fn expand_fn_arg(arg: &syn::FnArg) -> (syn::PatIdent, syn::Type) {
    match arg {
        syn::FnArg::Captured(syn::ArgCaptured {
            pat: syn::Pat::Ident(arg_id),
            ty,
            ..
        }) => (arg_id.clone(), ty.clone()),
        ar => {
            println!("ARGS {:#?}", ar);
            panic!()
        }
    }
}

fn expand_macro(path: &syn::Path) -> Option<syn::Ident> {
    let syn::Path { segments, .. } = path;
    // make sure it is always last, cause trouble?
    if let Some(ps) = segments.last() {
        Some(ps.value().ident.clone())
    } else {
        None
    }
}
