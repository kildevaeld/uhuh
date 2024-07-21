use std::{future::Future, pin::Pin};

use crate::{builder::InitCtx, Error};

pub trait Initializer<C> {
    fn call<'a>(
        &'a self,
        core: InitCtx<'a, C>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;
}

impl<T, C> Initializer<C> for T
where
    T: Fn(InitCtx<'_, C>) -> Result<(), Error>,
{
    fn call<'a>(
        &'a self,
        core: InitCtx<'a, C>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move { (self)(core) })
    }
}
