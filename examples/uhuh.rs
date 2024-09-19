use uhuh::{
    builder::{register_ext, InitCtx},
    Builder, Config, Context, Error, Mode, Module, Uhuh,
};
use vaerdi::Value;

struct Test;

impl<C: Context + 'static> Module<C> for Test {
    const CONFIG_SECTION: &'static str = "test";

    type Config = Value;

    fn default_config() -> Option<Self::Config> {
        Some("Hello, World!".into())
    }

    fn setup(mut core: uhuh::builder::SetupCtx<'_, C>) -> Result<(), Error> {
        core.add_module::<Test2>();
        core.add_module::<Test2>();
        Ok(())
    }

    fn build(
        core: uhuh::builder::BuildCtx<'_, C>,
        config: Self::Config,
    ) -> impl std::future::Future<Output = Result<(), Error>> {
        async move {
            println!("init {:?}", config);
            Ok(())
        }
    }
}

struct Test2;

impl<C: Context + 'static> Module<C> for Test2 {
    const CONFIG_SECTION: &'static str = "test2";

    type Config = Value;

    fn default_config() -> Option<Self::Config> {
        Some("Hello, World! 2".into())
    }

    fn setup(mut core: uhuh::builder::SetupCtx<'_, C>) -> Result<(), Error> {
        core.add_module::<Test>();
        Ok(())
    }

    fn build(
        core: uhuh::builder::BuildCtx<'_, C>,
        config: Self::Config,
    ) -> impl std::future::Future<Output = Result<(), Error>> {
        async move {
            println!("init2 {:?}", config);
            Ok(())
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let app = Builder::new((), "Test", Mode::Development)
        .module::<Test>()
        .configure(|cfg: &mut Config| {
            cfg.try_set("rapper", 2022)?;
            Ok(())
        })
        .initializer(register_ext::<String, _>("Hello".to_string()))
        .initializer(|core: InitCtx<()>| {
            //

            // println!("initializer: {}", *core);
            println!("ext: {:?}", core.get::<String>());
            println!("config: {:?}", core.config().get("rapper"));
            println!("Root: {}", core.root().display());

            Ok(())
        })
        .configure(|cfg: &mut Config| {
            cfg.try_set("ostelone", "Freja")?;

            Ok(())
        })
        .build()
        .await?;

    Ok(())
}
