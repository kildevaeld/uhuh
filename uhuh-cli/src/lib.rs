// mod cmd;

use std::{future::Future, process::Output};

use uhuh_config::{ConfigResolver, FsConfigResolver};
use uhuh_exp::{
    extensions::SetupBuildContext, BuildContext, Builder, LocalBoxFuture, Phase, SetupPhase,
    UhuhError,
};

pub trait CliBuildContext: BuildContext + SetupBuildContext<Self> + Sized
// where
//     for<'a> Self::Setup<'a>: CliSetupContext<Self>,
{
    fn register_command<T: Cli<Self> + 'static>(&mut self, cli: T);
}

pub trait CliSetupContext<C: BuildContext> {
    fn register_command<T: Cli<C> + 'static>(&mut self, cli: T);
}

pub trait BuilderExt<C>
where
    C: CliBuildContext,
    for<'a> C::Setup<'a>: CliSetupContext<C>,
{
    fn cli<T>(self, cli: T) -> impl Future<Output = Result<(), UhuhError>>
    where
        T: Cli<C>;
}

impl<C: 'static> BuilderExt<C> for Builder<SetupPhase<C>, C>
where
    C: CliBuildContext,
    for<'a> C::Setup<'a>: CliSetupContext<C>,
{
    fn cli<T>(self, cli: T) -> impl Future<Output = Result<(), UhuhError>>
    where
        T: Cli<C>,
    {
        async move {
            let builder = self.constant(init)?.setup().await?;

            let app = cli.create_command();

            Ok(())
        }
    }
}

#[allow(unused)]
pub trait Cli<C: BuildContext> {
    fn create_command(&self) -> clap::Command;
    fn prepare<'a, 'b>(
        &'a self,
        builder: &'a mut C::Build<'b>,
    ) -> impl Future<Output = Result<(), UhuhError>> + 'a {
        async move { Ok(()) }
    }
    fn run(
        self,
        ctx: C::Output,
        args: &clap::ArgAction,
    ) -> impl Future<Output = Result<(), UhuhError>>;
}

trait DynCli<C: BuildContext> {
    fn create_command(&self) -> clap::Command;
    fn prepare<'a, 'b>(
        &'a self,
        builder: &'a mut C::Build<'b>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>>;

    fn run<'a>(
        self,
        ctx: C::Output,
        args: &'a clap::ArgAction,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>>;
}

struct CliBox<T>(T);

impl<C: 'static, T: 'static> DynCli<C> for CliBox<T>
where
    C: BuildContext,
    T: Cli<C>,
{
    fn create_command(&self) -> clap::Command {
        self.0.create_command()
    }

    fn prepare<'a, 'b>(
        &'a self,
        builder: &'a mut C::Build<'b>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>> {
        Box::pin(async move { self.0.prepare(builder).await })
    }

    fn run<'a>(
        self,
        ctx: <C as BuildContext>::Output,
        args: &'a clap::ArgAction,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>> {
        Box::pin(async move { self.0.run(ctx, args).await })
    }
}

pub struct CliBuilder<C: BuildContext> {
    cmds: Vec<Box<dyn DynCli<C>>>,
}

impl<C: BuildContext> Default for CliBuilder<C> {
    fn default() -> Self {
        CliBuilder {
            cmds: Default::default(),
        }
    }
}

impl<C: BuildContext + 'static> CliBuilder<C> {
    pub fn register<T>(&mut self, cli: T)
    where
        T: Cli<C> + 'static,
    {
        self.cmds.push(Box::new(CliBox(cli)))
    }
}
