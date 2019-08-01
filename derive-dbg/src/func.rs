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
    ItemFn, Token, Type,
};

use dbg_collect;

#[derive(Debug, Clone)]
struct Args {
    org_args: Vec<syn::FnArg>,
    mut_args: Vec<syn::FnArg>,
    args: Vec<syn::FnArg>,
    scope: String,
}

impl Args {
    fn new(scope: String, args: Punctuated<syn::FnArg, Comma>) -> Self {
        let org_args = args.iter().cloned().collect();
        let mut ret = Args {
            org_args,
            mut_args: Vec::new(),
            args: Vec::new(),
            scope,
        };
        args.into_iter().for_each(|arg| ret.push_mut_else(arg));
        ret
    }

    fn is_mut_arg(&self, arg: &syn::FnArg) -> bool {
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

    fn push_mut_else(&mut self, arg: syn::FnArg) {
        if self.is_mut_arg(&arg) {
            self.mut_args.push(arg);
        } else {
            self.args.push(arg);
        }
    }

    fn capture_args(
        &mut self,
        dbg: &mut dbg_collect::DebugCollect,
    ) -> (
        Vec<syn::PatIdent>,
        Vec<syn::PatIdent>,
        Vec<syn::PatIdent>,
        Vec<String>,
    ) {
        let mut p_args: Vec<(syn::PatIdent, syn::Type)> = Vec::new();
        for arg in self.org_args.iter() {
            match &arg {
                syn::FnArg::Captured(syn::ArgCaptured {
                    pat: syn::Pat::Ident(arg_id),
                    ty,
                    ..
                }) => {
                    // println!("{:#?}", ty);
                    p_args.push((arg_id.clone(), ty.clone()));
                    let name = arg_id.ident.clone().to_string();
                    let scope = self.scope.clone();
                    // TODO remove expect for something better
                    let am = dbg_collect::ArgMeta::new(
                        self.expand_arg_ty(&ty).expect("expand type failed"),
                        scope,
                    );
                    dbg.args.insert(name, am);
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
            }
        }
    }
}

///
///
///
#[derive(Debug, Clone)]
struct VisitStmt {
    mut_args: Vec<syn::FnArg>,
    stmts: Vec<RefCell<syn::Stmt>>,
    curr_stmt: Option<syn::Stmt>,
    mut_idx: Vec<usize>,
}

use std::cell::RefCell;
impl VisitStmt {
    fn new(stmts: Vec<syn::Stmt>) -> Self {
        VisitStmt {
            mut_args: Vec::default(),
            stmts: stmts.into_iter().map(|s| RefCell::new(s)).collect(),
            curr_stmt: None,
            mut_idx: Vec::default(),
        }
    }

    fn set_mut_args(&mut self, mut_args: Vec<syn::FnArg>) {
        self.mut_args = mut_args;
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
    pub fn insert_bp(&self) {
        self.stmts.iter().for_each(|s| {
            let mut stmt_ref_mut = s.borrow_mut();
            match &*stmt_ref_mut {
                syn::Stmt::Semi(syn::Expr::Macro(syn::ExprMacro { mac, .. }), _) => {
                    if let Some(mac_path) = expand_macro(&mac.path) {
                        if mac_path.to_string() == "bp" {
                            let dbg_step: syn::Stmt = parse_quote! {
                                dbg.step(&print_map).unwrap();
                            };
                            *stmt_ref_mut = dbg_step;
                        }
                    }
                }
                _ => {}
            }
        });
    }

    fn expand_expr(&self, e: &syn::Expr) -> Option<syn::Ident> {
        match e {
            syn::Expr::Path(e_path) => expand_path(&e_path.path),
            syn::Expr::MethodCall(e_meth) => self.expand_expr(&e_meth.receiver),
            expr => {
                println!("IN expand_expr {:?}", expr);
                panic!()
            }
        }
    }

    fn expand_stmt(&self, s: &syn::Stmt) -> Option<syn::Ident> {
        match s {
            syn::Stmt::Semi(expr, s) => self.expand_expr(&expr),
            syn::Stmt::Local(loc) => panic!("impl LOCALS"),
            syn::Stmt::Expr(expr) => self.expand_expr(&expr),
            syn::Stmt::Item(item) => panic!("impl LOCALS"),
        }
    }

    fn is_mut_var(&self, stmt: &syn::Stmt) -> bool {
        self.mut_args.iter().enumerate().any(|(i, arg)| {
            let (id, ty) = expand_fn_arg(arg);
            if let Some(vars_used) = self.expand_stmt(stmt) {
                if id.ident == vars_used {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        })
    }

    // fn split_for_use(&self, s: &syn::Stmt) -> (syn::Ident, syn::Expr) {
    //     let id = self.expand_stmt(s);

    //     (id, rest)
    // }

    fn replace_stmt(&self, s: &mut syn::Stmt) {
        let x = s.clone();
        //let (id, action) = self.split_for_use(&s);
        let new_stmt: syn::Stmt = parse_quote! {
            #x
        };
        *s = new_stmt;
    }

    fn compare_mut_stmt(&self, f: Box<(dyn FnMut(Vec<&syn::FnArg>) + '_)>) {}

    fn visit_stmts_mut(&self) {
        self.stmts.iter().for_each(|s| {
            if self.is_mut_var(&s.borrow()) {
                let s_mut: &mut syn::Stmt = &mut s.borrow_mut();
                self.replace_stmt(s_mut)
            }
        });
    }

    fn iter(&self) -> impl Iterator<Item = &RefCell<syn::Stmt>> {
        self.stmts.iter()
    }
}

///
///
///
#[derive(Debug, Clone)]
pub(crate) struct Func {
    dbg_col: dbg_collect::DebugCollect,
    _fn: syn::ItemFn,
    args: Args,
    locals: Vec<syn::Local>,
    stmts: VisitStmt,
}

impl Func {
    pub fn new(function: &ItemFn) -> Self {
        let mut ret = Func {
            dbg_col: dbg_collect::DebugCollect::default(),
            _fn: function.clone(),
            args: Args::new(function.ident.to_string(), function.decl.inputs.clone()),
            locals: Vec::default(),
            stmts: VisitStmt::new(function.block.stmts.clone()),
        };
        ret.stmts.set_mut_args(ret.args.mut_args.clone());
        ret
    }

    pub fn item_fn(mut self) -> syn::ItemFn {
        self.insert_bp();
        self.visit_stmts_mut();
        self.set_body();
        self._fn
    }

    fn capture_args(
        &mut self,
    ) -> (
        Vec<syn::PatIdent>,
        Vec<syn::PatIdent>,
        Vec<syn::PatIdent>,
        Vec<String>,
    ) {
        self.args.capture_args(&mut self.dbg_col)
    }

    pub fn insert_bp(&self) {
        self.stmts.insert_bp()
    }

    fn set_body(&mut self) {
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

    fn visit_stmts_mut(&mut self) {}
}

///
///
///
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
