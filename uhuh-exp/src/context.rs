use core::future::Future;

use crate::{error::UhuhError, module::DynamicModule, plugin::DynamicPlugin, Config};

pub trait BuildContext {
    type Setup<'a>;
    type Build<'a>;
    type Init<'a>;
    type Plugin<'a>;

    type Output;

    fn run_setup<'a>(
        &'a mut self,
        module: &'a dyn DynamicModule<Self>,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn run_build<'a>(
        &'a mut self,
        module: &'a dyn DynamicModule<Self>,
        config: &'a Config,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn run_init<'a>(
        &'a mut self,
        module: &'a dyn DynamicModule<Self>,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn run_build_plugin<'a>(
        &'a self,
        plugin: &'a dyn DynamicPlugin<Self>,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a;

    fn build(self) -> impl Future<Output = Result<Self::Output, UhuhError>>;
}

pub trait CliSetupContext {}

pub trait CliContext: BuildContext
where
    for<'a> Self::Setup<'a>: CliSetupContext,
{
}
