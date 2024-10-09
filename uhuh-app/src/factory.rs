use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use daserror::BoxError;

use crate::{BuildAction, BuildContext, InitAction, LocalBoxFuture, SetupAction, UhuhError};
use core::future::Future;

pub struct HookCtx<'a, C: BuildContext, T> {
    factories: &'a mut VecDeque<Box<dyn DynFactory<C>>>,
    ctx: &'a mut T,
}

impl<'a, C: BuildContext, T> HookCtx<'a, C, T> {
    pub fn register<F>(&mut self, factory: F) -> &mut Self
    where
        F: Factory<C> + 'static,
    {
        self.factories.push_back(Box::new(FactoryBox(factory)));
        self
    }
}

impl<'a, C: BuildContext, T> core::ops::Deref for HookCtx<'a, C, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a, C: BuildContext, T> core::ops::DerefMut for HookCtx<'a, C, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}

#[allow(used)]
pub trait Factory<C: BuildContext> {
    type Error: Into<BoxError<'static>>;
    fn on_setup<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Setup<'b>>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async move { Ok(()) }
    }

    fn on_build<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Build<'b>>,
        config: &'a C::Config,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async move { Ok(()) }
    }

    fn on_init<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Init<'b>>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async move { Ok(()) }
    }
}

pub trait DynFactory<C: BuildContext> {
    fn on_setup<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Setup<'b>>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>>;

    fn on_build<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Build<'b>>,
        config: &'a C::Config,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>>;

    fn on_init<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Init<'b>>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>>;
}

struct FactoryBox<T>(T);

impl<C, T> DynFactory<C> for FactoryBox<T>
where
    C: BuildContext,
    T: Factory<C>,
{
    fn on_setup<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Setup<'b>>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>> {
        Box::pin(async move { self.0.on_setup(ctx).await.map_err(UhuhError::new) })
    }

    fn on_build<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Build<'b>>,
        config: &'a C::Config,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>> {
        Box::pin(async move { self.0.on_build(ctx, config).await.map_err(UhuhError::new) })
    }

    fn on_init<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, C::Init<'b>>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>> {
        Box::pin(async move { self.0.on_init(ctx).await.map_err(UhuhError::new) })
    }
}

pub struct Factories<C: BuildContext> {
    i: Vec<Box<dyn DynFactory<C>>>,
}

impl<C: BuildContext> Default for Factories<C> {
    fn default() -> Self {
        Factories {
            i: Default::default(),
        }
    }
}

impl<C> Factories<C>
where
    C: BuildContext,
{
    pub fn register<T>(&mut self, factory: T)
    where
        T: Factory<C> + 'static,
    {
        self.i.push(Box::new(FactoryBox(factory)));
    }
}

impl<'t, C: BuildContext> SetupAction<C> for &'t mut Factories<C> {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Setup<'b>,
    ) -> impl Future<Output = Result<(), UhuhError>> {
        async move {
            let mut factories = VecDeque::from_iter(self.i.drain(..));

            loop {
                let Some(mut action) = factories.pop_front() else {
                    break;
                };
                action
                    .on_setup(HookCtx {
                        factories: &mut factories,
                        ctx,
                    })
                    .await?;

                self.i.push(action);
            }

            Ok(())
        }
    }
}

impl<'t, C: BuildContext> BuildAction<C> for &'t mut Factories<C> {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Build<'b>,
        config: &'a C::Config,
    ) -> impl Future<Output = Result<(), UhuhError>> {
        async move {
            let mut factories = VecDeque::from_iter(self.i.drain(..));

            loop {
                let Some(mut action) = factories.pop_front() else {
                    break;
                };
                action
                    .on_build(
                        HookCtx {
                            factories: &mut factories,
                            ctx,
                        },
                        config,
                    )
                    .await?;

                self.i.push(action);
            }

            Ok(())
        }
    }
}

impl<'t, C: BuildContext> InitAction<C> for &'t mut Factories<C> {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Init<'b>,
    ) -> impl Future<Output = Result<(), UhuhError>> {
        async move {
            let mut factories = VecDeque::from_iter(self.i.drain(..));

            loop {
                let Some(mut action) = factories.pop_front() else {
                    break;
                };
                action
                    .on_init(HookCtx {
                        factories: &mut factories,
                        ctx,
                    })
                    .await?;

                self.i.push(action);
            }

            Ok(())
        }
    }
}
