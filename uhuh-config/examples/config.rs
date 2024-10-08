use uhuh_config::{ConfigBuildContext, ConfigBuilder, SimpleResolver};
use uhuh_exp::{
    extensions::{
        InitContext, InitList, Plugin, PluginBuildContext, PluginSetupContext, PluginsList,
    },
    serde, BuildContext, Builder, Config, DynamicModule, Module, UhuhError,
};
use uhuh_ext::Extensions;

#[derive(Debug, Default)]
pub struct Cfg {
    name: String,
}

impl Config for Cfg {
    type Error = UhuhError;

    fn contains(&self, key: &str) -> bool {
        false
    }

    fn try_get<'a, T: serde::Deserialize<'a>>(&'a self, key: &str) -> Result<T, Self::Error> {
        todo!()
    }
}

#[derive(Default)]
pub struct Context {
    ext: Extensions,
    plugins: PluginsList<Self>,
    cfg: Cfg,
    config_builder: ConfigBuilder<Self, SimpleResolver<Cfg>>,
}

impl ConfigBuildContext<SimpleResolver<Cfg>> for Context {
    fn configure<F>(&mut self, func: F)
    where
        F: uhuh_config::Configure<SimpleResolver<Cfg>, Self::Config> + 'static,
    {
        self.config_builder.configure(func);
    }
}

pub struct BuildCtx<'a> {
    extensions: &'a mut Extensions,
    plugins: &'a mut PluginsList<Context>,
}

impl BuildContext for Context {
    type Build<'a> = BuildCtx<'a>;
    type Setup<'a> = ();
    type Init<'a> = ();
    type Output = Extensions;

    type Config = Cfg;

    fn run_setup<'a>(
        &'a mut self,
        modules: &'a [Box<dyn DynamicModule<Self>>],
    ) -> impl std::future::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move {
            for module in modules {
                module.setup(()).await?;
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

    type Config = ();

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
        _config: Option<Self::Config>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async move {
            println!("Build");

            Ok(())
        }
    }
}

fn main() {
    futures::executor::block_on(wrapped_main()).unwrap()
}

async fn wrapped_main() -> Result<(), uhuh_exp::UhuhError> {
    let builder = Builder::new(Context::default())
        .module::<TestModule>()
        .build()
        .await?;

    println!(
        "Ret {:?}, Ret2: {:?}",
        builder.get::<u32>(),
        builder.get::<u64>()
    );

    Ok(())
}
