use core::future::Future;

use alloc::{boxed::Box, vec::Vec};
use daserror::BoxError;

use crate::{types::LocalBoxFuture, BuildContext, UhuhError};

pub trait Initializer<C: BuildContext> {
    type Error: Into<BoxError<'static>>;
    fn init<'a, 'b>(
        self,
        ctx: &'a mut C::Init<'b>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a;
}

impl<C: BuildContext, F> Initializer<C> for F
where
    C: 'static,
    F: FnOnce(&mut C::Init<'_>) -> Result<(), UhuhError> + 'static,
{
    type Error = UhuhError;
    fn init<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Init<'b>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async move { (self)(ctx) }
    }
}

trait DynamicInit<C: BuildContext> {
    fn init<'a, 'b>(
        self: Box<Self>,
        ctx: &'a mut C::Init<'b>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>>;
}

pub struct InitList<C: BuildContext> {
    funcs: Vec<Box<dyn DynamicInit<C>>>,
}

impl<C: BuildContext> Default for InitList<C> {
    fn default() -> Self {
        InitList {
            funcs: Default::default(),
        }
    }
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
            fn init<'a, 'b>(
                self: Box<Self>,
                ctx: &'a mut <C as BuildContext>::Init<'b>,
            ) -> LocalBoxFuture<'a, Result<(), UhuhError>> {
                Box::pin(async move {
                    //
                    self.0.init(ctx).await.map_err(UhuhError::new)
                })
            }
        }
        self.funcs
            .push(Box::new(Dyn(init)) as Box<dyn DynamicInit<C>>)
    }

    pub async fn run<'a>(&mut self, ctx: &mut C::Init<'_>) -> Result<(), UhuhError> {
        for init in self.funcs.drain(..) {
            init.init(ctx).await?;
        }

        Ok(())
    }
}

pub trait InitContext<C: BuildContext> {
    fn initializer<T>(&mut self, init: T)
    where
        T: Initializer<C> + 'static,
        C: 'static;
}
