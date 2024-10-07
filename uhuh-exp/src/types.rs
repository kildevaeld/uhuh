use core::{future::Future, pin::Pin};

use alloc::boxed::Box;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
