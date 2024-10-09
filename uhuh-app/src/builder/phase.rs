use core::future::Future;

use crate::error::UhuhError;

pub trait Phase<C> {
    type Next;
    fn next(self) -> impl Future<Output = Result<Self::Next, UhuhError>>;

    fn context(&mut self) -> &mut C;
}
