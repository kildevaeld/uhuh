use core::future::Future;

use crate::error::UhuhError;

pub trait Phase {
    type Next;
    fn next(self) -> impl Future<Output = Result<Self::Next, UhuhError>>;
}
