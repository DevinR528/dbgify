use crossterm::{input, Color, Colorize, InputEvent, KeyEvent};
use std::io;

pub use derive_dbg::dbgify;

pub fn step() -> io::Result<()> {
    println!("type var name or tab to auto-complete");
    let mut input = input();
    let line = input.read_line()?;
    println!("{}", line);
    Ok(())
}

#[macro_export]
macro_rules! bp {
    () => {
        step().unwrap();
    };
}
