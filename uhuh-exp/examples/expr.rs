use uhuh_exp::{
    extensions::{
        InitContext, InitList, Plugin, PluginBuildContext, PluginSetupContext, PluginsList,
    },
    BuildContext, Builder, Config, DynamicModule, Module, UhuhError,
};
use uhuh_ext::{Context as CoreContext, Extensions};
use vaerdi::Value;

#[derive(Debug, Default)]
pub struct Cfg;

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
    init: InitList<Self>,
    plugins: PluginsList<Self>,
    cfg: Cfg,
}

pub struct BuildCtx<'a> {
    extensions: &'a mut Extensions,
    plugins: &'a mut PluginsList<Context>,
}

impl InitContext<Context> for Context {
    fn initializer<T>(&mut self, init: T)
    where
        T: uhuh_exp::extensions::Initializer<Context> + 'static,
        Context: 'static,
    {
        self.init.register(init);
    }
}

impl PluginSetupContext<Context> for Context {
    fn plugin<T>(&mut self, plugin: T) -> Result<(), UhuhError>
    where
        T: 'static + uhuh_exp::extensions::Plugin<Context> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
        Context: 'static,
    {
        self.plugins.insert(plugin)
    }
}

impl<'a> PluginBuildContext<Context> for BuildCtx<'a> {
    fn configure_plugin<T>(&mut self) -> Result<&mut T, UhuhError>
    where
        T: 'static + uhuh_exp::extensions::Plugin<Context> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        self.plugins.get_mut()
    }
}

impl<'a> CoreContext for BuildCtx<'a> {
    fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.extensions.get()
    }

    fn register<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T> {
        self.extensions.insert(value)
    }
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

            let plugins = core::mem::take(&mut self.plugins);

            plugins
                .build(BuildCtx {
                    extensions: &mut self.ext,
                    plugins: &mut self.plugins,
                })
                .await?;
            Ok(())
        }
    }

    fn run_init<'a>(
        &'a mut self,
        _module: &'a [Box<dyn DynamicModule<Self>>],
    ) -> impl std::future::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move {
            self.init.run(&mut ()).await?;
            Ok(())
        }
    }

    fn build(self) -> impl std::future::Future<Output = Result<Self::Output, uhuh_exp::UhuhError>> {
        async move { Ok(self.ext) }
    }
}

struct TestModule;

impl<C: BuildContext> Module<C> for TestModule
where
    for<'a> C::Build<'a>: CoreContext + PluginBuildContext<C>,
{
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
        _config: Option<Self::Config>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async move {
            println!("Build");
            ctx.register(200u32);

            ctx.configure_plugin::<TestPlugin>()?.variable = 100;
            Ok(())
        }
    }
}

pub struct TestPlugin {
    variable: u64,
}

impl<C: BuildContext> Plugin<C> for TestPlugin {
    type Output = u64;

    type Error = UhuhError;

    fn build(
        self,
        ctx: &mut C::Build<'_>,
    ) -> impl futures::Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move { Ok(self.variable) }
    }
}

fn main() {
    futures::executor::block_on(wrapped_main()).unwrap()
}

async fn wrapped_main() -> Result<(), uhuh_exp::UhuhError> {
    let builder = Builder::new(Context::default())
        .initializer(|ctx: &mut ()| {
            //
            println!("Init");
            Ok(())
        })
        .plugin(TestPlugin { variable: 5000 })?
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
