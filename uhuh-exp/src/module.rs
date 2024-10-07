use alloc::boxed::Box;
use core::{future::Future, marker::PhantomData, pin::Pin};
use vaerdi::Value;

use super::error::BoxError;
use crate::{context::BuildContext, error::UhuhError, ResultContext};

#[allow(unused)]
pub trait Module<C: BuildContext> {
    const CONFIG_SECTION: &'static str;

    type Config: serde::Serialize + serde::de::DeserializeOwned;
    type Error: Into<BoxError<'static>>;

    fn default_config() -> Option<Self::Config> {
        None
    }

    fn setup(ctx: C::Setup<'_>) -> impl Future<Output = Result<(), Self::Error>> {
        async move { Ok(()) }
    }

    fn build(
        ctx: C::Build<'_>,
        config: Option<Self::Config>,
    ) -> impl Future<Output = Result<(), Self::Error>>;

    fn init(ctx: C::Init<'_>) -> impl Future<Output = Result<(), Self::Error>> {
        async move { Ok(()) }
    }
}

pub trait DynamicModule<C: BuildContext> {
    fn config_section(&self) -> &str;

    fn default_config(&self) -> Option<Value>;

    fn setup<'a>(
        &'a self,
        core: C::Setup<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<(), UhuhError>> + 'a>>;

    fn build<'a>(
        &'a self,
        ctx: C::Build<'a>,
        config: Option<Value>,
    ) -> Pin<Box<dyn Future<Output = Result<(), UhuhError>> + 'a>>;

    fn init<'a>(
        &'a self,
        ctx: C::Init<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<(), UhuhError>> + 'a>>;
}

pub fn box_module<T: Module<C> + 'static, C: BuildContext + 'static>() -> Box<dyn DynamicModule<C>>
{
    Box::new(ModuleDyn(PhantomData::<T>))
}

pub struct ModuleDyn<T>(PhantomData<T>);

impl<T, C> DynamicModule<C> for ModuleDyn<T>
where
    T: Module<C>,
    C: BuildContext + 'static,
{
    fn config_section(&self) -> &str {
        T::CONFIG_SECTION
    }

    fn default_config(&self) -> Option<Value> {
        T::default_config().and_then(|m| vaerdi::ser::to_value(m).ok())
    }

    fn setup<'a>(
        &'a self,
        core: C::Setup<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<(), UhuhError>> + 'a>> {
        Box::pin(async move { T::setup(core).await.map_err(UhuhError::new) })
    }

    fn build<'a>(
        &'a self,
        ctx: C::Build<'a>,
        value: Option<Value>,
    ) -> Pin<Box<dyn Future<Output = Result<(), UhuhError>> + 'a>> {
        Box::pin(async move {
            let cfg = if let Some(cfg) = value {
                let cfg = vaerdi::de::from_value::<T::Config>(cfg)
                    .map_err(UhuhError::new)
                    .with_context("Could not unmarshal config")?;

                Some(cfg)
            } else {
                None
            };

            T::build(ctx, cfg).await.map_err(UhuhError::new)?;
            Ok(())
        })
    }

    fn init<'a>(
        &'a self,
        ctx: C::Init<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<(), UhuhError>> + 'a>> {
        Box::pin(async move { T::init(ctx).await.map_err(UhuhError::new) })
    }
}
