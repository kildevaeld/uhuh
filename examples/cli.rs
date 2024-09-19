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
        core.cmd(clap::Command::new("test"), |app, args| async move {
            println!("Test");
            Ok(())
        });
        Ok(())
    }

    fn build(
        _core: uhuh::builder::BuildCtx<'_, C>,
        config: Self::Config,
    ) -> impl std::future::Future<Output = Result<(), Error>> {
        async move {
            println!("init {:?}", config);
            Ok(())
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    Builder::new((), "Test", Mode::Development)
        .module::<Test>()
        .configure(|cfg: &mut Config| {
            cfg.try_set("rapper", 2022)?;
            Ok(())
        })
        .initializer(register_ext::<String, _>("Hello".to_string()))
        .initializer(|core: InitCtx<()>| {
            //

            println!("ext: {:?}", core.get::<String>());
            println!("config: {:?}", core.config().get("rapper"));
            println!("Root: {}", core.root().display());

            Ok(())
        })
        .configure(|cfg: &mut Config| {
            cfg.try_set("ostelone", "Freja")?;

            Ok(())
        })
        .setup()
        .await?
        .cli(|app: Uhuh, _args| async move {
            println!("App: {:?}", app.config());
            Ok(())
        })
        .await
}
