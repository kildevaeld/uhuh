use core::{future::Future, pin::Pin};

use alloc::{boxed::Box, vec::Vec};
use daserror::BoxError;

use crate::{types::BoxLocalFuture, BuildContext, UhuhError};

pub trait Initializer<C: BuildContext> {
    type Error: Into<BoxError<'static>>;
    fn init<'a>(self, ctx: C::Init<'a>) -> impl Future<Output = Result<(), Self::Error>>;
}

trait DynamicInit<C: BuildContext> {
    fn init<'a>(self: Box<Self>, ctx: C::Init<'a>) -> BoxLocalFuture<'a, Result<(), UhuhError>>;
}

pub struct InitList<C: BuildContext> {
    funcs: Vec<Box<dyn DynamicInit<C>>>,
}

impl<C: BuildContext> InitList<C> {
    pub fn register<T>(&mut self, init: T)
    where
        T: Initializer<C> + 'static,
        C: 'static,
    {
        struct Dyn<T>(T);

        impl<T, C> DynamicInit<C> for Dyn<T>
        where
            T: Initializer<C> + 'static,
            C: BuildContext + 'static,
        {
            fn init<'a>(
                self: Box<Self>,
                ctx: <C as BuildContext>::Init<'a>,
            ) -> BoxLocalFuture<'a, Result<(), UhuhError>> {
                Box::pin(async move {
                    //
                    self.0.init(ctx).await.map_err(UhuhError::new)
                })
            }
        }
        self.funcs
            .push(Box::new(Dyn(init)) as Box<dyn DynamicInit<C>>)
    }

    pub async fn run<'a>(self, ctx: C::Init<'a>) -> Result<(), UhuhError>
    where
        C::Init<'a>: Clone,
    {
        for init in self.funcs {
            init.init(ctx.clone()).await?;
        }

        Ok(())
    }
}
