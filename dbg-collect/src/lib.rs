// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]
// dont remove until stable
#![feature(specialization)]

use std::collections::HashMap;

use console::{Key, Term};
use serde::{Deserialize, Serialize};
use serde_json;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

pub struct PrintFn(pub std::boxed::Box<(dyn std::ops::Fn())>);
impl std::ops::Deref for PrintFn {
    type Target = (dyn Fn());

    fn deref(&self) -> &Self::Target {
        &(*self.0)
    }
}

#[derive(Debug, Clone)]
pub enum Output {
    Either,
    Not,
    Display,
    Debug,
}

trait DisplayCheck {
    fn is_display(&self) -> Output;
}
trait DebugCheck {
    fn is_debug(&self) -> Output;
}
impl<T> DisplayCheck for T {
    default fn is_display(&self) -> Output {
        Output::Not
    }
}
impl<T> DebugCheck for T {
    default fn is_debug(&self) -> Output {
        Output::Not
    }
}
impl<T: std::fmt::Display> DisplayCheck for T {
    fn is_display(&self) -> Output {
        Output::Display
    }
}
impl<T: std::fmt::Debug> DebugCheck for T {
    fn is_debug(&self) -> Output {
        Output::Debug
    }
}

pub struct Displayable<T>(std::marker::PhantomData<T>);
impl<T> Displayable<T> {
    fn cast<D: std::fmt::Display + 'static>(t: &T) -> &D {
        unsafe { std::mem::transmute(t) }
    }
    pub fn print<C: std::fmt::Display + 'static>(d: T) {
        println!("{}", Self::cast::<C>(&d))
    }
}

pub struct Debugable<T>(std::marker::PhantomData<T>);
impl<T> Debugable<T> {
    fn cast<D: std::fmt::Debug + 'static>(t: &T) -> &D {
        unsafe { std::mem::transmute(t) }
    }
    pub fn print<C: std::fmt::Debug + 'static>(d: T) {
        println!("{:?}", Self::cast::<C>(&d))
    }
}

pub fn show_fmt<T, D: std::fmt::Debug + 'static, P: std::fmt::Display + 'static>(t: &T) {
    match (DisplayCheck::is_display(t), DebugCheck::is_debug(t)) {
        (Output::Display, Output::Debug) => Debugable::print::<D>(t),
        (Output::Display, Output::Not) => Displayable::print::<P>(t),
        (Output::Not, Output::Debug) => Debugable::print::<D>(t),
        (Output::Not, Output::Not) => {
            println!("'{}' does not impl Debug or Display", stringify!(t))
        }
        err => {
            println!("{:?}", err);
            panic!("show fmt failed BUG")
        }
    }
}

fn term_input(term: &Term) -> () {}

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
            if let Some(_) = self.args.get(&line) {
                // then check if in scope?
                let mut stdout = StandardStream::stdout(ColorChoice::Auto);
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
