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
use std::cell::{Cell, UnsafeCell};
use std::ptr::NonNull;

mod format;
pub use format::*;
mod vars;
pub use vars::*;
