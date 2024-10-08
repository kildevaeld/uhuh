#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod builder;
mod configure;
mod context;
mod resolver;

pub use self::{builder::*, configure::*, context::*, resolver::*};
