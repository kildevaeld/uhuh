use alloc::boxed::Box;
use core::{future::Future, pin::Pin};
use daserror::BoxError;

use crate::error::UhuhError;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait Config {
    type Error: Into<BoxError<'static>>;
    fn contains(&self, key: &str) -> bool;
    fn try_get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, Self::Error>;
}

impl Config for () {
    type Error = UhuhError;

    fn contains(&self, _key: &str) -> bool {
        false
    }

    fn try_get<T: serde::de::DeserializeOwned>(&self, _key: &str) -> Result<T, Self::Error> {
        Err(UhuhError::new("not found"))
    }
}
