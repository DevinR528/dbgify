use std::io;

use crossterm::{input, Color, Colorize, InputEvent, KeyEvent};
pub use serde;
pub use serde_json;

pub use derive_dbg::dbgify;

pub fn step() -> io::Result<()> {
    println!("type var name or tab to auto-complete");
    let mut input = input();
    let line = input.read_line()?;
    Ok(())
}

#[macro_export]
macro_rules! bp {
    () => {
        step().unwrap();
        let dbg = DebugCollect::new();
        println!("DBG {:#?}", dbg);
    };
}
