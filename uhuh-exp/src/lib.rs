#![no_std]

extern crate alloc;

mod builder;
// mod config;
mod context;
mod error;
mod module;
// mod plugin;
pub mod extensions;
mod types;

pub use self::types::{Config, LocalBoxFuture};

pub use self::{
    builder::{BuildPhase, Builder, InitPhase, Phase, SetupPhase},
    // config::Config,
    context::*,
    error::*,
    module::{DynamicModule, Module},
};

pub use serde;
