use uhuh_exp::{BuildContext, Builder, Config, DynamicModule, Module, UhuhError};
use uhuh_ext::{Context as CoreContext, Extensions};
use vaerdi::Value;

#[derive(Default)]
pub struct Context {
    ext: Extensions,
}

pub struct BuildCtx<'a> {
    extensions: &'a mut Extensions,
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
        config: &'a Config,
    ) -> impl std::future::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move {
            for module in modules {
                module
                    .build(
                        BuildCtx {
                            extensions: &mut self.ext,
                        },
                        config,
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

impl<C: BuildContext> Module<C> for TestModule
where
    for<'a> C::Build<'a>: CoreContext,
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
        .setup()
        .await?
        .build()
        .await?
        .init()
        .await?;

    println!("Ret {:?}", builder.get::<u32>());

    Ok(())
}
