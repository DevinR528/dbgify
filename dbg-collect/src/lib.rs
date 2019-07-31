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
use std::cell::{Cell, UnsafeCell};
use std::ptr::NonNull;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

mod format;
pub use format::*;
mod vars;
pub use vars::*;
