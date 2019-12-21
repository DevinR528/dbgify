// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TknStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use serde::{Deserialize, Serialize};
use serde_json;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block,
    Expr,
    ExprClosure,
    Item,
    Macro,
    Result as SynResult,
    Stmt,
    punctuated::{IntoIter, Punctuated},
    token::Comma,
    ItemFn,
    Token,
    Type,
    PatIdent,
};

pub enum DebugKind {
    Func(ItemFn),
    Closure(ExprClosure),
}

impl Parse for DebugKind {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let item: DebugKind;
        if input.peek(Token![fn]) {
            let func = input.parse::<ItemFn>()?;
            item = DebugKind::Func(func);
        } else {
            let closure = input.parse::<ExprClosure>()?;
            item = DebugKind::Closure(closure);
        }
        Ok(item)
    }
}

impl ToTokens for DebugKind {
    fn to_tokens(&self, tokens: &mut TknStream2) {
        match self {
            Self::Func(f) => f.to_tokens(tokens),
            Self::Closure(cls) => cls.to_tokens(tokens),
        }
    }
}
