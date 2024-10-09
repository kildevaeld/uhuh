use core::future::Future;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    error::UhuhError,
    types::{Config, LocalBoxFuture},
    BuildAction, Extensions, InitAction, SetupAction,
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

    fn run_init<'a, T: InitAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn build(self) -> impl Future<Output = Result<Self::Output, UhuhError>>;
}
