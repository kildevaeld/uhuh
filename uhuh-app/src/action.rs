use alloc::{boxed::Box, vec::Vec};
use core::future::Future;

use crate::{BuildContext, Extensions, LocalBoxFuture, UhuhError};

pub struct ActionCtx<'a, C: BuildContext, T> {
    pub ctx: &'a mut T,
    pub actions: &'a mut Actions<C>,
}

impl<'a, C: BuildContext, T> ActionCtx<'a, C, T> {}

pub trait BuildAction<C: BuildContext> {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut C::Build<'b>,
        config: &'a C::Config,
    ) -> impl Future<Output = Result<(), UhuhError>>;
}

pub trait SetupAction<C: BuildContext> {
    fn run<'a, 'b>(self, ctx: &'a mut C::Setup<'b>) -> impl Future<Output = Result<(), UhuhError>>;
}

pub trait InitAction<C: BuildContext> {
    fn run<'a, 'b>(self, ctx: &'a mut C::Init<'b>) -> impl Future<Output = Result<(), UhuhError>>;
}

pub struct Actions<C: BuildContext> {
    build: Vec<
        Box<
            dyn for<'a, 'b> FnOnce(
                &'a mut C::Build<'b>,
                &'a C::Config,
            ) -> LocalBoxFuture<'a, Result<(), UhuhError>>,
        >,
    >,
    init: Vec<
        Box<
            dyn for<'a, 'b> FnOnce(
                &'a mut C::Init<'b>,
            ) -> LocalBoxFuture<'a, Result<(), UhuhError>>,
        >,
    >,
    setup: Vec<
        Box<
            dyn for<'a, 'b> FnOnce(
                &'a mut C::Setup<'b>,
            ) -> LocalBoxFuture<'a, Result<(), UhuhError>>,
        >,
    >,
}

impl<C: BuildContext> Default for Actions<C> {
    fn default() -> Self {
        Actions {
            build: Default::default(),
            init: Default::default(),
            setup: Default::default(),
        }
    }
}

impl<C: BuildContext> Actions<C> {
    pub fn add_build<T: BuildAction<C> + 'static>(&mut self, action: T) {
        self.build.push(Box::new(
            move |ctx: &mut C::Build<'_>, config: &C::Config| {
                Box::pin(async move { action.run(ctx, config).await })
            },
        ))
    }

    pub fn add_setup<T: SetupAction<C> + 'static>(&mut self, action: T) {
        self.setup.push(Box::new(move |ctx: &mut C::Setup<'_>| {
            Box::pin(async move { action.run(ctx).await })
        }))
    }

    pub fn add_init<T: InitAction<C> + 'static>(&mut self, action: T) {
        self.init.push(Box::new(move |ctx: &mut C::Init<'_>| {
            Box::pin(async move { action.run(ctx).await })
        }))
    }
}

impl<'t, C: BuildContext> BuildAction<C> for &'t mut Actions<C> {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Build<'b>,
        config: &'a C::Config,
    ) -> impl Future<Output = Result<(), UhuhError>> {
        async move {
            for action in self.build.drain(..) {
                (action)(ctx, config).await?;
            }
            Ok(())
        }
    }
}

impl<'t, C: BuildContext> SetupAction<C> for &'t mut Actions<C> {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Setup<'b>,
    ) -> impl Future<Output = Result<(), UhuhError>> {
        async move {
            for action in self.setup.drain(..) {
                (action)(ctx).await?;
            }
            Ok(())
        }
    }
}

impl<'t, C: BuildContext> InitAction<C> for &'t mut Actions<C> {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Init<'b>,
    ) -> impl Future<Output = Result<(), UhuhError>> {
        async move {
            for action in self.init.drain(..) {
                (action)(ctx).await?;
            }
            Ok(())
        }
    }
}

pub trait OnBuild<C: BuildContext> {
    fn on_build<T: BuildAction<C> + 'static>(&mut self, action: T);
}

pub trait OnSetup<C: BuildContext> {
    fn on_setup<T: SetupAction<C> + 'static>(&mut self, action: T);
}

pub trait OnInit<C: BuildContext> {
    fn on_init<T: InitAction<C> + 'static>(&mut self, action: T);
}
