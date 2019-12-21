// // TODO remove
// #![allow(dead_code)]
// #![allow(unused_imports)]

// extern crate proc_macro;

// use std::collections::HashMap;

// use proc_macro::TokenStream;
// use proc_macro2 as pc2;
// use proc_macro2::Span;
// use quote::{quote, ToTokens};
// use serde::{Deserialize, Serialize};
// use serde_json;
// use syn::parse::{Parse, ParseStream};
// use syn::{
//     parse_macro_input, parse_quote,
//     punctuated::{IntoIter, Punctuated},
//     token::Comma,
//     ItemFn, Token, Type,
// };

// use crate::args::{Args, ArgIdents};
// use dbg_collect;

// ///
// ///
// ///
// #[derive(Debug, Clone)]
// struct VisitStmt {
//     mut_args: Vec<syn::FnArg>,
//     stmts: Vec<RefCell<syn::Stmt>>,
//     curr_stmt: Option<syn::Stmt>,
//     mut_idx: Vec<usize>,
// }

// use std::cell::RefCell;
// impl VisitStmt {
//     fn new(stmts: Vec<syn::Stmt>) -> Self {
//         VisitStmt {
//             mut_args: Vec::default(),
//             stmts: stmts.into_iter().map(|s| RefCell::new(s)).collect(),
//             curr_stmt: None,
//             mut_idx: Vec::default(),
//         }
//     }

//     fn set_mut_args(&mut self, mut_args: Vec<syn::FnArg>) {
//         self.mut_args = mut_args;
//     }

//     /// Insert step debug code in place of bp!()
//     ///
//     /// Examples
//     ///
//     /// ```
//     /// fn fake_main() {
//     ///     let mut x = 0;
//     ///     bp!();
//     ///     x = 10;
//     /// }
//     /// ```
//     /// allows you to inspect every change in the value of x or any other argument,
//     /// local, or captured variable.
//     pub fn insert_bp(&self) {
//         self.stmts.iter().for_each(|s| {
//             let mut stmt_ref_mut = s.borrow_mut();
//             match &*stmt_ref_mut {
//                 syn::Stmt::Semi(syn::Expr::Macro(syn::ExprMacro { mac, .. }), _) => {
//                     if let Some(mac_path) = expand_macro(&mac.path) {
//                         if mac_path.to_string() == "bp" {
//                             let dbg_step: syn::Stmt = parse_quote! {
//                                 dbg.step(&print_map).unwrap();
//                             };
//                             *stmt_ref_mut = dbg_step;
//                         }
//                     }
//                 }
//                 _ => {}
//             }
//         });
//     }

//     fn expand_expr(&self, e: &syn::Expr) -> Option<syn::Ident> {
//         match e {
//             syn::Expr::Path(e_path) => expand_path(&e_path.path),
//             syn::Expr::MethodCall(e_meth) => self.expand_expr(&e_meth.receiver),
//             expr => {
//                 println!("IN expand_expr {:?}", expr);
//                 panic!()
//             }
//         }
//     }

//     fn expand_stmt(&self, s: &syn::Stmt) -> Option<syn::Ident> {
//         match s {
//             syn::Stmt::Semi(expr, s) => self.expand_expr(&expr),
//             syn::Stmt::Local(loc) => panic!("impl LOCALS"),
//             syn::Stmt::Expr(expr) => self.expand_expr(&expr),
//             syn::Stmt::Item(item) => panic!("impl LOCALS"),
//         }
//     }

//     fn is_mut_var(&self, stmt: &syn::Stmt) -> bool {
//         self.mut_args.iter().any(|arg| {
//             let (id, ty) = expand_fn_arg(arg);
//             if let Some(vars_used) = self.expand_stmt(stmt) {
//                 id.ident == vars_used
//             } else {
//                 false
//             }
//         })
//     }

//     fn split_for_meth_call(&self, s: &syn::Stmt) -> Option<syn::Stmt> {
//         // println!("METHCALL {:#?}", s);
//         if let syn::Stmt::Semi(syn::Expr::MethodCall(m_call), semi) = s {
//             let mut ident = self.expand_expr(&m_call.receiver).unwrap();
//             ident = syn::Ident::new(&format!("_{}", ident), pc2::Span::call_site());

//             let method = m_call.method.clone();
//             let args = m_call.args.clone();
//             Some(parse_quote! { unsafe { (*#ident.get()).#method(#args); } })
//         } else {
//             None
//         }
//     }

//     fn replace_stmt(&self, s: &mut syn::Stmt) {
//         // TODO
//         let new_stmt = self.split_for_meth_call(&s).unwrap();
//         *s = new_stmt;
//     }

//     fn visit_stmts_mut(&self) {
//         self.stmts.iter().for_each(|s| {
//             if self.is_mut_var(&s.borrow()) {
//                 let s_mut: &mut syn::Stmt = &mut s.borrow_mut();
//                 self.replace_stmt(s_mut)
//             }
//         });
//     }

//     fn body(&self) -> syn::Block {
//         let s: Vec<syn::Stmt> = self
//             .stmts
//             .iter()
//             .map(|s| {
//                 let stmt = s.borrow().clone();
//                 stmt
//             })
//             .collect();

//         parse_quote! { { #(#s)* } }
//     }

//     fn iter(&self) -> impl Iterator<Item = &RefCell<syn::Stmt>> {
//         self.stmts.iter()
//     }
// }

// ///
// ///
// ///
// #[derive(Debug, Clone)]
// pub(crate) struct Func {
//     dbg_col: dbg_collect::DebugCollect,
//     _fn: syn::ItemFn,
//     args: Args,
//     locals: Vec<syn::Local>,
//     stmts: VisitStmt,
// }

// impl Func {
//     pub fn new(function: &ItemFn) -> Self {
//         let mut ret = Func {
//             dbg_col: dbg_collect::DebugCollect::default(),
//             _fn: function.clone(),
//             args: Args::new(function.sig.ident.to_string(), function.sig.inputs.clone()),
//             locals: Vec::default(),
//             stmts: VisitStmt::new(function.block.stmts.clone()),
//         };
//         ret.stmts.set_mut_args(ret.args.mut_args.clone());
//         ret
//     }

//     pub fn item_fn(mut self) -> syn::ItemFn {
//         self.insert_bp();
//         self.visit_stmts_mut();
//         self.set_body();
//         self._fn
//     }

//     fn capture_args(&mut self) -> ArgIdents {
//         self.args.capture_args(&mut self.dbg_col)
//     }

//     pub fn insert_bp(&self) {
//         self.stmts.insert_bp()
//     }

//     fn set_body(&mut self) {
//         // must be done before serialization as it fills dbg_collect
//         let args = self.capture_args();
//         // serialize the obj to 'send' to the running program
//         let ser = serde_json::to_string(&self.dbg_col).unwrap();

//         let ret = match &self._fn.sig.output {
//             syn::ReturnType::Default => quote!{ -> () },
//             out @ syn::ReturnType::Type(..) => quote!{ #out },
//         };
//         let (mut_arg, under_mut, mut_str) = args.new_mut_decl();
//         // TODO
//         // anyway to avoid this
//         let under_mut2 = under_mut.clone();

//         let (org_arg, under_arg, arg_str) = args.new_decl();
//         let under_arg2 = under_arg.clone();

//         let body = self.stmts.body();
//         self._fn.block = Box::new(parse_quote! ({
//             std::thread_local! {
//                 static VARS: dbg_collect::VarRef = dbg_collect::VarRef::new();
//             }
//             let __result = (move || #ret {

//                 let mut print_map: std::collections::HashMap<String, PrintFn> = std::collections::HashMap::new();
//                 #(
//                     let #under_arg = #org_arg;

//                     let print_fn = dbg_collect::PrintFn(Box::new(move || {
//                         println!("{:?}", #under_arg2.as_debug())
//                     }));

//                     print_map.insert(#arg_str.into(), print_fn);

//                 )*

//                 #(
//                     let #under_mut = std::cell::UnsafeCell::new(#mut_arg);
//                     VARS.with(|var| var.set_ref(&#under_mut2));
//                     let print_fn = dbg_collect::PrintFn(Box::new(|| {
//                         VARS.with(|var| unsafe {
//                             if let Some(v) = var.0.get() {
//                                 let ptr = &*(v.as_ref().get());
//                                 println!("{:?}", ptr.as_debug());
//                             } else {
//                                 eprintln!("accessed var after drop unsafe be careful");
//                             }
//                         });
//                     }));
//                     print_map.insert(#mut_str.into(), print_fn);
//                 )*
//                 let dbg = dbg_collect::DebugCollect::deserialize(#ser);
//                 #body
//             })();
//             __result
//         }));
//     }

//     fn capture_local(&self) {
//         println!("{:#?}", self.stmts);
//     }

//     fn collect_type(&self, p_args: &Vec<(syn::PatIdent, syn::Type)>) {
//         // vec of types for casting/transmute
//         let arg_ty: Vec<syn::Type> = p_args.iter().map(|(_, ty)| ty).cloned().collect();
//         // strip any & or &mut from type so we don't double ref when we unsafe cast
//         // with show_fmt
//         let base_ty: Vec<syn::Type> = p_args
//             .iter()
//             .map(|(_, ty)| match ty {
//                 syn::Type::Reference(ty_ref) => {
//                     let t = ty_ref.elem.clone();
//                     *t
//                 }
//                 t => t.clone(),
//             })
//             .collect();
//     }

//     fn visit_stmts_mut(&mut self) {
//         self.stmts.visit_stmts_mut()
//     }
// }

// ///
// ///
// ///
// fn expand_fn_arg(arg: &syn::FnArg) -> (syn::Pat, syn::Type) {
//     match arg {
//         syn::FnArg::Typed(syn::PatType {
//             pat,
//             ty,
//             ..
//         }) => (*pat.clone(), *ty.clone()),
//         ar => {
//             println!("ARGS {:#?}", ar);
//             panic!()
//         }
//     }
// }

// fn expand_macro(path: &syn::Path) -> Option<syn::Ident> {
//     let syn::Path { segments, .. } = path;
//     // make sure it is always last, cause trouble?
//     let ps = segments.last()?;
//     Some(ps.ident.clone())
// }

// fn expand_path(path: &syn::Path) -> Option<syn::Ident> {
//     Some(path.segments.last()?.ident)
// }

// fn expand_bounds(bounds: &Punctuated<syn::TypeParamBound, syn::token::Add>) -> Option<syn::Ident> {
//     if let Some(b) = bounds.last() {
//         if let syn::TypeParamBound::Trait(tb) = b {
//             expand_path(&tb.path)
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }

// fn expand_arg_ty(ty: &syn::Type) -> Option<String> {
//     match ty {
//         Type::Slice(_ty) => Some("Vec".to_string()),
//         Type::Array(_ty) => Some("Vec".into()),
//         Type::Ptr(_ty) => Some("Ptr".into()),
//         Type::Reference(ty) => expand_arg_ty(&ty.elem),
//         Type::BareFn(_ty) => Some("Fn".into()),
//         Type::Never(_ty) => Some("Never".into()),
//         Type::Tuple(_ty) => Some("Tuple".into()),
//         Type::Path(ty) => {
//             if let Some(id) = expand_path(&ty.path) {
//                 Some(id.to_string())
//             } else {
//                 None
//             }
//         }
//         Type::TraitObject(ty) => {
//             if let Some(trait_obj) = expand_bounds(&ty.bounds) {
//                 // TODO make know its a trait
//                 Some(trait_obj.to_string())
//             } else {
//                 eprintln!("found no trait");
//                 None
//             }
//         }
//         Type::ImplTrait(ty) => {
//             if let Some(trait_impl) = expand_bounds(&ty.bounds) {
//                 // TODO make know its a trait
//                 Some(trait_impl.to_string())
//             } else {
//                 eprintln!("found no impl'ed trait");
//                 None
//             }
//         }
//         Type::Paren(ty) => expand_arg_ty(&ty.elem),
//         Type::Group(ty) => expand_arg_ty(&ty.elem),
//         Type::Infer(_ty) => Some("Underscore".into()),
//         Type::Macro(ty) => {
//             if let Some(id) = expand_macro(&ty.mac.path) {
//                 Some(id.to_string())
//             } else {
//                 None
//             }
//         }
//         Type::Verbatim(ty) => {
//             eprintln!("VERBATIM {:#?}", ty);
//             panic!()
//         }
//     }
// }
