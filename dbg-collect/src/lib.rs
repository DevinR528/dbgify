// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

use crossterm::{input, InputEvent, TerminalColor, TerminalInput};
use serde::{Deserialize, Serialize};
use serde_json;

fn term_input(term: &TerminalInput) -> () {}

pub struct Cb(pub std::boxed::Box<(dyn std::ops::Fn())>);
impl std::ops::Deref for Cb {
    type Target = (dyn Fn());

    fn deref(&self) -> &Self::Target {
        &(*self.0)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DebugCollect {
    pub args: HashMap<String, String>,
}
impl DebugCollect {
    pub fn new() -> Self {
        DebugCollect {
            args: HashMap::default(),
        }
    }
    pub fn deserialize(s: &str) -> Self {
        let d: DebugCollect = serde_json::from_str(s).unwrap();
        d
    }
    pub fn step(&self, cbs: &HashMap<String, Cb>) -> std::io::Result<()> {
        println!("type var name or tab to auto-complete");
        let print_loop = || -> std::io::Result<bool> {
            let mut input = crossterm::input();
            let line = input.read_line()?;
            // if var is saved then print value
            if let Some(_) = self.args.get(&line) {
                // then check if in scope?
                (*cbs.get(&line).expect("closure map should = vars map BUG"))();
                Ok(true)
            } else {
                println!("could not find variable '{}' in scope", line);
                Ok(false)
            }
        };
        while let Ok(t) = print_loop() {
            if t == false {
                continue;
            } else {
                return Ok(());
            }
        }
        unreachable!("in fn step()")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
