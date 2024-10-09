use core::future::Future;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    error::UhuhError,
    types::{Config, LocalBoxFuture},
};

pub trait BuildContext: Sized {
    type Setup<'a>;
    type Build<'a>;
    type Init<'a>;

    type Config: Config;

    type Output;

    fn run_setup<'a, T: SetupAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn run_build<'a, T: BuildAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn run_init<'a, T: BuildAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn build(self) -> impl Future<Output = Result<Self::Output, UhuhError>>;
}

pub trait BuildAction<C: BuildContext> {
    fn run<'a, 'b>(self, ctx: &'a mut C::Build<'b>) -> impl Future<Output = Result<(), UhuhError>>;
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
        self.build.push(Box::new(move |ctx: &mut C::Build<'_>| {
            Box::pin(async move { action.run(ctx).await })
        }))
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

    //     pub async fn build(&mut self, ctx: &mut C::Build<'_>) -> Result<(), UhuhError> {
    //         for action in self.build.drain(..) {
    //             (action)(ctx).await?;
    //         }
    //         Ok(())
    //     }

    //     pub async fn init(&mut self, ctx: &mut C::Init<'_>) -> Result<(), UhuhError> {
    //         for action in self.init.drain(..) {
    //             (action)(ctx).await?;
    //         }
    //         Ok(())
    //     }

    //     pub async fn setup(&mut self, ctx: &mut C::Setup<'_>) -> Result<(), UhuhError> {
    //         for action in self.setup.drain(..) {
    //             (action)(ctx).await?;
    //         }
    //         Ok(())
    //     }
}

impl<'t, C: BuildContext> BuildAction<C> for &'t mut Actions<C> {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Build<'b>,
    ) -> impl Future<Output = Result<(), UhuhError>> {
        async move {
            for action in self.build.drain(..) {
                (action)(ctx).await?;
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
