// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]

use std::cell::{Cell, UnsafeCell};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ptr::NonNull;

use console::{Key, Term};
use serde::{Deserialize, Serialize};
use serde_json;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::format::Debugable;

// look into Box::leak()
pub struct VarRef(pub Cell<Option<NonNull<UnsafeCell<dyn Debugable>>>>);
impl VarRef {
    pub fn new() -> Self {
        Self(Cell::new(None))
    }
    pub fn set_ref<T: Debugable>(self: &'_ Self, ptr: &'_ UnsafeCell<T>) {
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
    pub fn drop_ref(self: &'_ Self) {
        self.0.set(None)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_set_ref() {
        assert_eq!(2 + 2, 4);
    }
}
