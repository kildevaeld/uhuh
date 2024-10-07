mod cmd;

use std::future::Future;

use uhuh_exp::{BuildContext, Builder, Setup, UhuhError};

pub trait CliBuildContext: BuildContext
where
    for<'a> Self::Setup<'a>: CliSetupContext<Self>,
{
}

pub trait CliSetupContext<C> {
    fn command(&mut self, cmd: clap::Command);
}

pub trait BuilderExt<C>
where
    C: CliBuildContext,
    for<'a> C::Setup<'a>: CliSetupContext<C>,
{
    fn cli<F, U>(self, func: F) -> impl Future<Output = Result<(), UhuhError>>
    where
        F: FnOnce(C::Output) -> U,
        U: Future<Output = Result<(), UhuhError>>;
}

impl<C> BuilderExt<C> for Builder<Setup<C>>
where
    C: CliBuildContext,
    for<'a> C::Setup<'a>: CliSetupContext<C>,
{
    fn cli<F, U>(self, func: F) -> impl Future<Output = Result<(), UhuhError>>
    where
        F: FnOnce(C::Output, clap::ArgMatches) -> U,
        U: Future<Output = Result<(), UhuhError>>,
    {
        async move {
            let builder = self.setup().await?;

            Ok(())
        }
    }
}
