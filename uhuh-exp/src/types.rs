use core::{convert::Infallible, future::Future, pin::Pin};

use alloc::boxed::Box;
use daserror::BoxError;

use crate::UhuhError;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait Config {
    type Error: Into<BoxError<'static>>;
    fn contains(&self, key: &str) -> bool;
    fn try_get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, Self::Error>;
}

impl Config for () {
    type Error = UhuhError;

    fn contains(&self, key: &str) -> bool {
        false
    }

    fn try_get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, Self::Error> {
        Err(UhuhError::new("not found"))
    }
}
