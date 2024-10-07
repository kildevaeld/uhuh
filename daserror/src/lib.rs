#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg_attr(ERROR_IN_STD, path = "./error.rs")]
#[cfg_attr(ERROR_IN_CORE, path = "./error_v181.rs")]
mod error;

pub use self::error::*;
