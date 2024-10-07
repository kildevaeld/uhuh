use core::{future::Future, pin::Pin};

use alloc::boxed::Box;

pub type BoxLocalFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
