use std::{future::Future, marker::PhantomData, pin::Pin};

use vaerdi::Value;

use crate::{
    builder::{BuildCtx, SetupCtx},
    context::Context,
    error::Error,
    InitCtx,
};

#[allow(unused)]
pub trait Module<C: Context> {
    const CONFIG_SECTION: &'static str;

    type Config: serde::Serialize + serde::de::DeserializeOwned;

    fn default_config() -> Option<Self::Config>;

    fn setup(ctx: SetupCtx<'_, C>) -> Result<(), Error> {
        Ok(())
    }

    fn build(ctx: BuildCtx<'_, C>, config: Self::Config)
        -> impl Future<Output = Result<(), Error>>;

    fn init(ctx: InitCtx<'_, C>) -> impl Future<Output = Result<(), Error>> {
        async move { Ok(()) }
    }

    fn finish(ctx: &mut C::Output) -> impl Future<Output = Result<(), Error>> {
        async move { Ok(()) }
    }
}

pub trait DynamicModule<C: Context> {
    fn config_section(&self) -> &str;

    fn default_config(&self) -> Option<Value>;

    fn setup(&self, core: SetupCtx<'_, C>) -> Result<(), Error>;

    fn build<'a>(
        &'a self,
        ctx: BuildCtx<'a, C>,
        config: Value,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;

    fn init<'a>(
        &'a self,
        ctx: InitCtx<'a, C>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;

    fn finish<'a>(
        &'a self,
        core: &'a mut C::Output,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;
}

pub fn box_module<T: Module<C> + 'static, C: Context>() -> Box<dyn DynamicModule<C>> {
    Box::new(ModuleDyn(PhantomData::<T>))
}

pub struct ModuleDyn<T>(PhantomData<T>);

impl<T, C> DynamicModule<C> for ModuleDyn<T>
where
    T: Module<C>,
    C: Context,
{
    fn config_section(&self) -> &str {
        T::CONFIG_SECTION
    }

    fn default_config(&self) -> Option<Value> {
        T::default_config().and_then(|m| vaerdi::ser::to_value(m).ok())
    }

    fn setup(&self, core: SetupCtx<'_, C>) -> Result<(), Error> {
        T::setup(core)
    }

    fn build<'a>(
        &'a self,
        ctx: BuildCtx<'a, C>,
        value: Value,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move {
            let Ok(cfg) = vaerdi::de::from_value::<T::Config>(value.clone()) else {
                panic!("invalid config: {:?}", value);
            };

            T::build(ctx, cfg).await?;
            Ok(())
        })
    }

    fn init<'a>(
        &'a self,
        ctx: InitCtx<'a, C>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move { T::init(ctx).await })
    }

    fn finish<'a>(
        &'a self,
        core: &'a mut C::Output,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move { T::finish(core).await })
    }
}
