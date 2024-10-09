// mod cmd;

use std::{future::Future, process::Output};

use uhuh_config::{ConfigResolver, FsConfigResolver};
use uhuh_exp::{
    extensions::{ConfigureSetup, Setup, SetupBuildContext},
    BuildContext, Builder, LocalBoxFuture, SetupPhase, UhuhError,
};

// pub trait CliBuildContext: BuildContext + SetupBuildContext<Self> + Sized
// // where
// //     for<'a> Self::Setup<'a>: CliSetupContext<Self>,
// {
//     fn register_command<T: Cli<Self> + 'static>(&mut self, cli: T);
// }

pub trait CliSetupContext<C: BuildContext>: ConfigureSetup<C> {
    fn register_command<T: Cli<C> + Sync + Send + 'static>(
        &mut self,
        cli: T,
    ) -> Result<(), UhuhError>;
}

impl<C: 'static, F> CliSetupContext<C> for F
where
    C: BuildContext,
    F: ConfigureSetup<C>,
{
    fn register_command<T: Cli<C> + Sync + Send + 'static>(
        &mut self,
        cli: T,
    ) -> Result<(), UhuhError> {
        self.configure_setup::<CliBuilder<C>>()?.register(cli);
        Ok(())
    }
}

pub trait BuilderExt<C>
where
    C: BuildContext,
    for<'a> C::Setup<'a>: CliSetupContext<C>,
{
    fn cli<T>(self, cli: T) -> impl Future<Output = Result<(), UhuhError>>
    where
        T: Cli<C>;
}

impl<C: 'static> BuilderExt<C> for Builder<SetupPhase<C>, C>
where
    C: BuildContext + SetupBuildContext<C> + uhuh_ext::Context,
    for<'a> C::Setup<'a>: CliSetupContext<C>,
{
    fn cli<T>(self, cli: T) -> impl Future<Output = Result<(), UhuhError>>
    where
        T: Cli<C>,
    {
        async move {
            let mut builder = self.constant(CliBuilder::default())?.setup().await?;

            let subcommands = builder
                .context()
                .get::<CliBuilder<C>>()
                .ok_or_else(|| UhuhError::new("Cli builder not registered"))?;

            let app = cli.create_command();

            let args = app.get_matches();

            // cli.prepare(builder);

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
    cmds: Vec<Box<dyn DynCli<C> + Send + Sync>>,
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
        T: Cli<C> + Sync + Send + 'static,
    {
        self.cmds.push(Box::new(CliBox(cli)))
    }
}

impl<C: BuildContext> Setup<C> for CliBuilder<C> {
    type Output = SubCommands<C>;

    type Error = UhuhError;

    fn build(
        self,
        ctx: &mut <C as BuildContext>::Setup<'_>,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> {
        async move { Ok(SubCommands(self.cmds)) }
    }
}

pub struct SubCommands<C>(Vec<Box<dyn DynCli<C> + Sync + Send>>);
