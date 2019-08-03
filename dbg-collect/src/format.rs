// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;
use std::fmt::{Debug, Display};

use console::{Key, Term};
use serde::{Deserialize, Serialize};
use serde_json;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use std::cell::{Cell, UnsafeCell};
use std::ptr::NonNull;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ArgMeta {
    ty_str: String,
    scope: String,
}
impl ArgMeta {
    pub fn new(ty_str: String, scope: String) -> Self {
        Self { ty_str, scope }
    }

    pub fn type_ref(&self) -> &str {
        self.ty_str.as_str()
    }

    pub fn scope_ref(&self) -> &str {
        self.scope.as_str()
    }
}

pub struct PrintFn(pub Box<(dyn Fn())>);
impl std::ops::Deref for PrintFn {
    type Target = (dyn Fn());

    fn deref(&self) -> &Self::Target {
        &(*self.0)
    }
}

pub trait Debugable {
    fn as_debug(self: &'_ Self) -> &'_ dyn Debug;
}
impl<T> Debugable for T {
    default fn as_debug(self: &'_ Self) -> &'_ dyn Debug {
        struct DefaultDebug;

        impl Debug for DefaultDebug {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "<Missing Debug Impl>")
            }
        }
        &DefaultDebug
    }
}
impl<T: Debug> Debugable for T {
    fn as_debug(self: &'_ Self) -> &'_ dyn Debug {
        self
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DebugCollect {
    pub args: HashMap<String, ArgMeta>,
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
            let input = Term::stdout();

            let line = input.read_line()?;
            // if var is saved then print value
            if let Some(arg_meta) = self.args.get(&line) {
                // then check if in scope?
                let mut stdout = match arg_meta.type_ref() {
                    "String" => {
                        let mut std_out = StandardStream::stdout(ColorChoice::Auto);
                        std_out.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                        std_out
                    }
                    "Vec" => {
                        let mut std_out = StandardStream::stdout(ColorChoice::Auto);
                        std_out.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
                        std_out
                    }
                    "Tuple" => {
                        let mut std_out = StandardStream::stdout(ColorChoice::Auto);
                        std_out.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                        std_out
                    }
                    "str" => {
                        let mut std_out = StandardStream::stdout(ColorChoice::Auto);
                        std_out.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                        std_out
                    }
                    _ => {
                        let mut std_out = StandardStream::stdout(ColorChoice::Auto);
                        std_out.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                        std_out
                    }
                };
                if let Some(cb) = cbs.get(&line) {
                    cb();
                } else {
                    stdout.reset()?;
                    eprintln!("closure map should mirror vars map. BUG");
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

fn term_input(term: &Term) -> () {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn debugable_trait() {
        fn assert_dbg<T: Debugable>(_: &T) {}

        fn make_gen<T>(t: &T) {
            let x = t.as_debug();
            assert_dbg(&x);
        }

        let num = 12.65f32;
        make_gen(&num);

        let vector = vec![1, 2, 3];
        make_gen(&vector);
    }

    #[test]
    fn show_fmt_fn() {
        assert_eq!(2 + 2, 4);
    }
}
