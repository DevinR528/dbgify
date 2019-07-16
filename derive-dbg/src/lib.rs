extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2 as pc2;
use proc_macro2::Span;

use quote::{quote, ToTokens};

use syn::{parse_macro_input, ItemFn};

struct Variables<'a, T> (HashMap<&'a str, T>);

struct DebugCollect<'a, T> {
    vars: HashMap<&'a str, T>,
}

#[proc_macro_attribute]
pub fn dbgify(args: TokenStream, function: TokenStream) -> TokenStream {
    let func = parse_macro_input!(function as ItemFn);

    println!("{:#?}", func);

    TokenStream::from(quote! { #func })
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        
    }
}
