use crate::{initializer::Initializer, Error};
use core::future::Future;

use super::InitCtx;

pub trait Phase {
    type Next;
    fn next(self) -> impl Future<Output = Result<Self::Next, Error>>;
}

pub struct Builder<T: Phase> {
    pub(super) phase: T,
}

pub fn register_ext<T: Send + Sync + Clone + 'static, C>(value: T) -> impl Initializer<C> {
    move |mut ctx: InitCtx<'_, C>| {
        ctx.register(value.clone());
        Ok(())
    }
}
