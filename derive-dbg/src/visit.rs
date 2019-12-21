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
use syn::{
    parse_quote,
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block,
    Expr,
    ExprClosure,
    Item,
    Macro,
    Stmt,
    Token,
    ExprMethodCall,
    FnArg,
    Receiver,
    PatType,
    Type,
    punctuated::{IntoIter, Punctuated},
    token::Comma,
    ItemFn,
};

use std::collections::VecDeque;

use crate::parse::DebugKind;
use crate::args::{Args, ArgIdents};

pub struct VisitDebug {
    is_function: bool,
    func: Option<ItemFn>,
    cls: Option<ExprClosure>,
    stmts: VecDeque<Stmt>,
    args: Option<Args>,
    dbg_col: dbg_collect::DebugCollect,
}

impl Default for VisitDebug {
    fn default() -> Self {
        Self {
            is_function: false,
            func: None,
            cls: None,
            stmts: VecDeque::default(),
            args: None,
            dbg_col: dbg_collect::DebugCollect::default(),
        }
    }
}

impl VisitDebug {
    /// Creates an instance of VisitDebug. This starts parsing
    /// and mutation of the original function or closure.
    pub fn visit_mut(item: DebugKind) -> VisitDebug {
        let mut dbg = Self::default();
        match item {
            DebugKind::Func(mut func) => {
                dbg.is_function = true;
                dbg.func = Some(func.clone());
                dbg.args = Some(Args::new(func.sig.ident.to_string(), func.sig.inputs.clone()));
                dbg.visit_item_fn_mut(&mut func)
            },
            DebugKind::Closure(mut cls) => {
                dbg.is_function = false;
                dbg.cls = Some(cls.clone());
                dbg.visit_expr_closure_mut(&mut cls)
            },
        }
        dbg
    }

    pub fn is_func(&self) -> bool {
        self.is_function
    }
    /// Replaces the most recent stmt this should be index 0.
    fn replace_stmt(&mut self, new: Stmt) -> bool {
        if let Some(mut stmt) = self.stmts.get_mut(0) {
            *stmt = new;
            return true;
        }
        false
    }

    fn call_name(&self, e: &syn::Expr) -> Option<syn::Ident> {
        match e {
            syn::Expr::Path(e_path) => expand_path(&e_path.path),
            syn::Expr::MethodCall(e_meth) => self.call_name(&e_meth.receiver),
            expr => {
                println!("IN call_name {:?}", expr);
                panic!()
            }
        }
    }

    fn fn_body(&self) -> syn::Block {
        let s: Vec<syn::Stmt> = self
            .stmts
            .iter()
            .cloned()
            .collect();

        parse_quote! { { #(#s)* } }
    }

    fn capture_args(&mut self) -> ArgIdents {
        self.args.take().unwrap().capture_args(&mut self.dbg_col)
    }

    fn build_fn(&mut self) {
        // must be done before serialization as it fills dbg_collect
        let args = self.capture_args();
        // serialize the obj to 'send' to the running program
        let ser = serde_json::to_string(&self.dbg_col).unwrap();

        let func = self.func.clone();
        let ret = match func.unwrap().sig.output {
            syn::ReturnType::Default => quote!{ -> () },
            out @ syn::ReturnType::Type(..) => quote!{ #out },
        };
        let (mut_arg, under_mut, mut_str) = args.new_mut_decl();
        // TODO
        // anyway to avoid this
        let under_mut2 = under_mut.clone();

        let (org_arg, under_arg, arg_str) = args.new_decl();
        let under_arg2 = under_arg.clone();

        let body = self.fn_body();
        self.func.as_mut().unwrap().block = Box::new(parse_quote! ({
            std::thread_local! {
                static VARS: dbg_collect::VarRef = dbg_collect::VarRef::new();
            }
            let __result = (move || #ret {

                let mut print_map: std::collections::HashMap<String, PrintFn> = std::collections::HashMap::new();
                #(
                    let #under_arg = #org_arg;

                    let print_fn = dbg_collect::PrintFn(Box::new(move || {
                        println!("{:?}", #under_arg2.as_debug())
                    }));

                    print_map.insert(#arg_str.into(), print_fn);

                )*

                #(
                    let #under_mut = std::cell::UnsafeCell::new(#mut_arg);
                    VARS.with(|var| var.set_ref(&#under_mut2));
                    let print_fn = dbg_collect::PrintFn(Box::new(|| {
                        VARS.with(|var| unsafe {
                            if let Some(v) = var.0.get() {
                                let ptr = &*(v.as_ref().get());
                                println!("{:?}", ptr.as_debug());
                            } else {
                                eprintln!("accessed var after drop unsafe be careful");
                            }
                        });
                    }));
                    print_map.insert(#mut_str.into(), print_fn);
                )*
                let dbg = dbg_collect::DebugCollect::deserialize(#ser);
                #body
            })();
            __result
        }));
    }

    fn build_cls(&mut self) {}

    pub fn debugable(mut self) -> DebugKind {
        if self.is_func() {
            self.build_fn();
            DebugKind::Func(self.func.unwrap())
        } else {
            self.build_cls();
            DebugKind::Closure(self.cls.unwrap())
        }
    }
}

impl VisitMut for VisitDebug {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        self.stmts.push_front(stmt.clone());

        visit_mut::visit_stmt_mut(self, stmt);
    }

    fn visit_expr_method_call_mut(
        &mut self, 
        node: &mut ExprMethodCall
    ) {
        let mut ident = self.call_name(&node.receiver).unwrap();
        ident = syn::Ident::new(&format!("_{}", ident), pc2::Span::call_site());
        let method = node.method.clone();
        let args = node.args.clone();
        let replace: Stmt = parse_quote! { unsafe { (*#ident.get()).#method(#args); } };
        self.replace_stmt(replace);
    }

    // fn visit_expr_mut(&mut self, expr: &mut Expr) {

    //     VisitMut::visit_expr_mut(self, expr);
    // }

    // fn visit_fn_arg_mut(&mut self, arg: &mut FnArg) {
        
    //     VisitMut::visit_fn_arg_mut(self, arg);
    // }

    /// visits the non self args of any function.
    fn visit_pat_type_mut(&mut self, arg: &mut PatType) {
        
        visit_mut::visit_pat_type_mut(self, arg);
    }

    /// visits any self args of a method.
    fn visit_receiver_mut(&mut self, this: &mut Receiver) {

        visit_mut::visit_receiver_mut(self, this);
    }

    fn visit_macro_mut(&mut self, mac: &mut Macro) {
        if let Some(mac_path) = expand_macro(&mac.path) {
            if mac_path.to_string() == "bp" {
                let dbg_step: syn::Stmt = parse_quote! {
                    dbg.step(&print_map).unwrap();
                };
                self.replace_stmt(dbg_step);
            }
        }

        visit_mut::visit_macro_mut(self, mac);
    }
}

fn expand_path(path: &syn::Path) -> Option<syn::Ident> {
    Some(path.segments.last()?.ident.clone())
}

fn expand_macro(path: &syn::Path) -> Option<syn::Ident> {
    let syn::Path { segments, .. } = path;
    // make sure it is always last, cause trouble?
    let ps = segments.last()?;
    Some(ps.ident.clone())
}
