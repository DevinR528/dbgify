// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]

use std::io;

pub use crossterm::{input, Color, Colorize, InputEvent, KeyEvent};
use serde;
use serde_json;

use dbg_collect::DebugCollect;
use derive_dbg::dbgify;

#[macro_export]
macro_rules! bp {
    () => {};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
