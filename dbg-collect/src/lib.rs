// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]
// dont remove until stable
#![feature(specialization, raw)]

use std::collections::HashMap;
use std::fmt::{Debug, Display};

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

trait Debugable {
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

use std::cell::{Cell, UnsafeCell};
use std::ptr::NonNull;

pub struct VarRef(Cell<Option<NonNull<UnsafeCell<dyn Debugable>>>>);
impl VarRef {
    fn new() -> Self {
        Self(Cell::new(None))
    }
    fn set_ref<T: Debugable>(self: &'_ Self, ptr: &'_ UnsafeCell<T>) {
        self.0.set(Some(unsafe {
            // create a null ptr as a &Debugable trait object
            let mut raw_trait_obj: std::raw::TraitObject =
                std::mem::transmute(std::ptr::null::<T>() as *const dyn Debugable);
            // fill trait object with ptr to local or arg value
            raw_trait_obj.data = ptr as *const _ as *mut ();
            // wrapper to indicate unsafe interior, mutable aliasable value
            // (a ptr to a &mut arg or local)
            let ptr: &UnsafeCell<dyn Debugable> = std::mem::transmute(raw_trait_obj);

            ptr.into()
        }));
    }
    fn drop_ref(self: &'_ Self) {
        self.0.set(None)
    }
}

// pub fn show_fmt<T, D: Debug, P: Display>(t: &T) {
//     match (DisplayCheck::is_display(t), DebugCheck::is_debug(t)) {
//         (Output::Display, Output::Debug) => Debugable::print::<D>(t),
//         (Output::Display, Output::Not) => Displayable::print::<P>(t),
//         (Output::Not, Output::Debug) => Debugable::print::<D>(t),
//         (Output::Not, Output::Not) => {
//             println!("'{}' does not impl Debug or Display", stringify!(t))
//         }
//         err => {
//             println!("{:?}", err);
//             panic!("show fmt failed BUG")
//         }
//     }
// }

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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn unsafe_cast_fn() {
        fn assert_dbg<T: Debug>(_: &T) {}
        fn make_gen<T, D: Debug + 'static>(t: &T) {
            let x = unsafe { Debugable::cast::<D>(&t) };
            assert_dbg(&x);
        }
        let num = 12.65f32;
        make_gen::<_, f32>(&num);
    }

    #[test]
    fn unsafe_show_fmt() {
        assert_eq!(2 + 2, 4);
    }
}
