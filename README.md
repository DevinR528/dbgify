#Dbgify
###Step wise debugging as a rust attribute macro. (Major WIP)
###Tab complete your bugs away!

[![Build Status](https://travis-ci.com/DevinR528/dbgify?branch=master)](https://travis-ci.com/DevinR528/dbgify)
[![Latest Version](https://img.shields.io/crates/v/dbgify.svg)](https://crates.io/crates/toml)

An attribute macro that allows you to step through and inspect rust code. This macro captures variables
and inserts 'breakpoints' to search and print running rust programs.

## Use
Include in Cargo.toml
[toml]: https://github.com/toml-lang/toml
```toml
["dependencies"]
dbgify="0.1"
```

then use like so, the number of places the attribute can be used is limited now but will grow.
```rust
use dbgify::*;

fn main() {
    #[dbgify]
    fn test<'a>(x: &'a mut String) {
        let _y = 0;
        bp!();
        x.push_str(" world");
    }
    let mut x = "hello".to_string();
    test(&mut x);
}

```
this will pause at bp!() and you can tab complete

