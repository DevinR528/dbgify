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
mod parse;
mod visit;
mod args;

// use func::*;
use parse::DebugKind;
use visit::VisitDebug;

#[proc_macro_attribute]
pub fn dbgify(attrs: TokenStream, function: TokenStream) -> TokenStream {
    assert!(attrs.is_empty());
    let func = parse_macro_input!(function as DebugKind);

    let visited = VisitDebug::visit_mut(func);

    // let mut visit_fn = Func::new(&func);
    // visit_fn.insert_bp();

    let func_dbg = visited.debugable();

    TokenStream::from(quote! {
        #func_dbg
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
