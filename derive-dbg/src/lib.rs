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
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, ItemFn, Token, Type,
};

use dbg_collect;
mod func;
use func::*;

// fn is_mut_vars(p: &syn::Path, other: &Vec<String>) -> bool {
//     if let Some(id) = expand_path(p.path) {
//         other.iter().any(|)
//     } else {
//         false
//     }
// }

// fn expand_expr(e: &syn::Expr, mut_vars: &Vec<String>) -> Option<syn::Ident> {
//     match e {
//         syn::Expr::Path(e_path) => expand_path(&e_path.path),
//         syn::Expr::MethodCall(e_meth) => expand_expr(&e_meth.receiver),
//         syn::Expr::Call(e_call) => {
//             let muts = e_call
//                 .args
//                 .iter()
//                 .filter(|arg| is_mut_vars(arg, mut_vars))
//                 .collect();
//         }
//         // syn::Expr::Box(e_box) => {}
//         // syn::Expr::Tuple(e_tup) => {}
//         // syn::Expr::If(e_if) => {}
//         // syn::Expr::Lit(e_lit) => {}
//         // syn::Expr::While(e_while) => {}
//         // syn::Expr::ForLoop(e_for) => {}
//         // syn::Expr::Loop(e_loop) => {}
//         // syn::Expr::Match(e_match) => {}
//         // syn::Expr::Closure(e_cb) => {}
//         // syn::Expr::Unsafe(e_uns) => {}
//         // syn::Expr::Block(e_blk) => {}
//         // syn::Expr::Assign(e_a) => {}
//         // syn::Expr::AssignOp(e_ao) => {}
//         // syn::Expr::Field(e_f) => {}
//         // syn::Expr::Index(e_idx) => {}
//         // syn::Expr::Range(e_r) => {}
//         // syn::Expr::Reference(e_ref) => {}
//         // syn::Expr::Break(e_brk) => {}
//         // syn::Expr::Continue(e_cnt) => {}
//         // syn::Expr::Return(e_ret) => {}
//         // syn::Expr::Macro(e_mac) => {}
//         // syn::Expr::Yield(e_yield) => {}
//         expr => {
//             println!("IN expand_expr {:?}", expr);
//             panic!()
//         }
//     }
// }

// fn morph_stmts(mut_vars: &Vec<String>, stmts: &mut Vec<syn::Stmt>) {
//     let mut res_stmts = stmts.clone();
//     for (i, stmt) in stmts.iter().enumerate() {
//         match stmt {
//             syn::Stmt::Semi(expr, s) => {
//                 expand_expr(expr, mut_vars);
//             }
//             syn::Stmt::Local(loc) => {}
//             syn::Stmt::Expr(expr) => {}
//             syn::Stmt::Item(item) => {}
//         };
//     }
// }

#[proc_macro_attribute]
pub fn dbgify(attrs: TokenStream, function: TokenStream) -> TokenStream {
    assert!(attrs.is_empty());
    let func = parse_macro_input!(function as ItemFn);
    let mut visit_fn = Func::new(&func);
    visit_fn.insert_bp();

    //println!("{:#?}", func);

    let func_dbg = visit_fn.item_fn();

    TokenStream::from(quote! {
        #func_dbg
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
