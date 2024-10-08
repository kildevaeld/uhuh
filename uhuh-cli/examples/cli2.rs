use uhuh_cli::{BuilderExt, Cli, CliBuildContext, CliBuilder, CliSetupContext};
use uhuh_exp::{
    extensions::{PluginsList, SetupBuildContext, SetupList},
    serde, BuildContext, Builder, Config, DynamicModule, Module, UhuhError,
};
use uhuh_ext::Extensions;
use vaerdi::{Map, Value};

#[derive(Debug, Default)]
pub struct Cfg {
    name: Map,
}

impl Config for Cfg {
    type Error = UhuhError;

    fn contains(&self, key: &str) -> bool {
        self.name.contains(key)
    }

    fn try_get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, Self::Error> {
        Ok(vaerdi::de::from_value(
            self.name
                .get(key)
                .ok_or(UhuhError::new("Not found"))
                .cloned()?,
        )
        .unwrap())
    }
}

#[derive(Default)]
pub struct Context {
    ext: Extensions,
    plugins: PluginsList<Self>,
    cmds: CliBuilder<Self>,
    setup: SetupList<Self>,
    cfg: Cfg,
}

impl CliBuildContext for Context {
    fn register_command<T: uhuh_cli::Cli<Self> + 'static>(&mut self, cli: T) {
        todo!()
    }
}

impl SetupBuildContext<Context> for Context {
    fn register_constant<T>(&mut self, setup: T) -> Result<(), UhuhError>
    where
        T: 'static + uhuh_exp::extensions::Setup<Context> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
        Context: 'static,
    {
        self.setup.insert(setup)
    }
}

pub struct SetupCtx<'a> {
    cmds: &'a mut CliBuilder<Context>,
    setup: &'a mut SetupList<Context>,
    ext: &'a mut Extensions,
}

impl<'a> CliSetupContext<Context> for SetupCtx<'a> {
    fn register_command<T: uhuh_cli::Cli<Context> + 'static>(&mut self, cli: T) {}
}

impl<'a> uhuh_ext::Context for SetupCtx<'a> {
    fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.ext.get()
    }

    fn register<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T> {
        self.ext.insert(value)
    }
}

pub struct BuildCtx<'a> {
    extensions: &'a mut Extensions,
    plugins: &'a mut PluginsList<Context>,
}

impl BuildContext for Context {
    type Build<'a> = BuildCtx<'a>;
    type Setup<'a> = SetupCtx<'a>;
    type Init<'a> = ();
    type Output = Extensions;

    type Config = Cfg;

    fn run_setup<'a>(
        &'a mut self,
        modules: &'a [Box<dyn DynamicModule<Self>>],
    ) -> impl std::future::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move {
            for module in modules {
                module
                    .setup(SetupCtx {
                        cmds: &mut self.cmds,
                        setup: &mut self.setup,
                        ext: &mut self.ext,
                    })
                    .await?;
            }
            Ok(())
        }
    }

    fn run_build<'a>(
        &'a mut self,
        modules: &'a [Box<dyn DynamicModule<Self>>],
    ) -> impl std::future::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move {
            for module in modules {
                module
                    .build(
                        BuildCtx {
                            extensions: &mut self.ext,
                            plugins: &mut self.plugins,
                        },
                        &self.cfg,
                    )
                    .await?;
            }

            Ok(())
        }
    }

    fn run_init<'a>(
        &'a mut self,
        _module: &'a [Box<dyn DynamicModule<Self>>],
    ) -> impl std::future::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move { Ok(()) }
    }

    fn build(self) -> impl std::future::Future<Output = Result<Self::Output, uhuh_exp::UhuhError>> {
        async move { Ok(self.ext) }
    }
}

struct TestModule;

impl<C: BuildContext> Module<C> for TestModule {
    const CONFIG_SECTION: &'static str = "test";

    type Config = Value;

    type Error = UhuhError;

    fn setup(
        _ctx: <C as BuildContext>::Setup<'_>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async move {
            println!("Setup");
            Ok(())
        }
    }

    fn build(
        mut ctx: <C as BuildContext>::Build<'_>,
        config: Option<Self::Config>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async move {
            println!("Build: {:?}", config);

            Ok(())
        }
    }
}

pub struct App;

impl Cli<Context> for App {
    fn create_command(&self) -> clap::Command {
        clap::Command::new("App")
    }

    fn run(
        self,
        ctx: Extensions,
        args: &clap::ArgAction,
    ) -> impl futures::Future<Output = Result<(), UhuhError>> {
        async move {
            println!("Worm");
            Ok(())
        }
    }
}

fn main() {
    futures::executor::block_on(wrapped_main()).unwrap()
}

async fn wrapped_main() -> Result<(), uhuh_exp::UhuhError> {
    Builder::new(Context::default())
        .module::<TestModule>()
        .cli(App)
        .await
}
