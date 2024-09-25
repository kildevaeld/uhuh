use futures_core::{future::LocalBoxFuture, Future};
use std::marker::PhantomData;

use crate::{context::Context, Error};

pub struct Cmd<C> {
    pub(super) cmd: clap::Command,
    pub(super) action: Box<dyn CmdAction<C, Future = LocalBoxFuture<'static, Result<(), Error>>>>,
}

pub trait CmdAction<C: Context> {
    type Future: Future<Output = Result<(), Error>>;
    fn call(self: Box<Self>, ctx: C::Output, args: clap::ArgMatches) -> Self::Future;
}

impl<T, U, C> CmdAction<C> for T
where
    C: Context,
    T: Fn(C::Output, clap::ArgMatches) -> U,
    U: Future<Output = Result<(), Error>>,
{
    type Future = U;
    fn call(self: Box<Self>, ctx: C::Output, args: clap::ArgMatches) -> Self::Future {
        (self)(ctx, args)
    }
}

pub fn box_action<C, T>(
    action: T,
) -> Box<dyn CmdAction<C, Future = LocalBoxFuture<'static, Result<(), Error>>>>
where
    T: CmdAction<C> + 'static,
    C: Context + 'static,
{
    struct Impl<C, T1>(Box<T1>, PhantomData<C>);

    impl<C: Context, T1> CmdAction<C> for Impl<C, T1>
    where
        T1: CmdAction<C> + 'static,
        T1::Future: 'static,
    {
        type Future = LocalBoxFuture<'static, Result<(), Error>>;

        fn call(self: Box<Self>, ctx: C::Output, args: clap::ArgMatches) -> Self::Future {
            Box::pin(self.0.call(ctx, args))
        }
    }

    Box::new(Impl(Box::new(action), PhantomData))
}
