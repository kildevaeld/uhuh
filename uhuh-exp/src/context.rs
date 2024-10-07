use core::future::Future;

use alloc::boxed::Box;

use crate::{error::UhuhError, module::DynamicModule, types::Config};

pub trait BuildContext: Sized {
    type Setup<'a>;
    type Build<'a>;
    type Init<'a>;

    type Config: Config;

    type Output;

    fn run_setup<'a>(
        &'a mut self,
        module: &'a [Box<dyn DynamicModule<Self>>],
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn run_build<'a>(
        &'a mut self,
        module: &'a [Box<dyn DynamicModule<Self>>],
        // config: &'a Config,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn run_init<'a>(
        &'a mut self,
        module: &'a [Box<dyn DynamicModule<Self>>],
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn build(self) -> impl Future<Output = Result<Self::Output, UhuhError>>;
}

pub trait CliSetupContext {}

pub trait CliContext: BuildContext
where
    for<'a> Self::Setup<'a>: CliSetupContext,
{
}
