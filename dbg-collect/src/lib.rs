// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

use console::{Key, Term};
use serde::{Deserialize, Serialize};
use serde_json;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn term_input(term: &Term) -> () {}

pub struct PrintFn(pub std::boxed::Box<(dyn std::ops::Fn())>);
impl std::ops::Deref for PrintFn {
    type Target = (dyn Fn());

    fn deref(&self) -> &Self::Target {
        &(*self.0)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DebugCollect {
    pub args: HashMap<String, String>,
}
impl Default for DebugCollect {
    fn default() -> Self {
        DebugCollect {
            args: Default::default(),
        }
    }
}
impl DebugCollect {
    pub fn deserialize(s: &str) -> Self {
        let d: DebugCollect = serde_json::from_str(s).unwrap();
        d
    }
    pub fn step(&self, cbs: &HashMap<String, PrintFn>) -> std::io::Result<()> {
        println!("type var name or tab to auto-complete");
        let print_loop = || -> std::io::Result<bool> {
            // put this in struct
            let mut input = Term::stdout();
            let line = input.read_line()?;
            // if var is saved then print value
            if let Some(_) = self.args.get(&line) {
                // then check if in scope?
                let mut stdout = StandardStream::stdout(ColorChoice::Auto);
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;

                if let Some(cb) = cbs.get(&line) {
                    cb();
                } else {
                    stdout.reset()?;
                    eprintln!("closure map should = vars map BUG");
                }

                stdout.reset()?;
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
