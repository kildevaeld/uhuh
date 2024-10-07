use core::{
    any::{Any, TypeId},
    future::Future,
};

use alloc::{boxed::Box, collections::btree_map::BTreeMap, format};
use daserror::BoxError;
use uhuh_ext::Context as _;

use crate::{types::BoxLocalFuture, BuildContext, UhuhError};

pub trait Plugin<C: BuildContext> {
    type Output;
    type Error: Into<BoxError<'static>>;
    fn build(
        self,
        ctx: &mut C::Build<'_>,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

pub(crate) trait DynamicPlugin<C: BuildContext> {
    fn build<'a, 'b>(
        self: Box<Self>,
        context: &'a mut C::Build<'b>,
    ) -> BoxLocalFuture<'a, Result<(), UhuhError>>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(crate) struct PluginBox<T> {
    inner: T,
}

impl<T, C> DynamicPlugin<C> for PluginBox<T>
where
    C: BuildContext + 'static,
    for<'a> C::Build<'a>: uhuh_ext::Context,
    T: 'static + Plugin<C> + Send,
    T::Output: Send + Sync + 'static,
    T::Error: 'static,
{
    fn build<'a, 'b>(
        self: Box<Self>,
        mut context: &'a mut C::Build<'b>,
    ) -> BoxLocalFuture<'a, Result<(), UhuhError>> {
        Box::pin(async move {
            let ret = self
                .inner
                .build(&mut context)
                .await
                .map_err(UhuhError::new)?;

            context.register(ret);
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        &self.inner
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.inner
    }
}

pub(crate) type BoxPlugin<C> = Box<dyn DynamicPlugin<C> + Send + Sync>;

pub(crate) fn plugin_box<T, C>(extension: T) -> BoxPlugin<C>
where
    T: 'static + Plugin<C> + Send + Sync,
    T::Output: Send + Sync + 'static,
    T::Error: 'static,
    C: BuildContext + 'static,
    for<'a> C::Build<'a>: uhuh_ext::Context,
{
    Box::new(PluginBox { inner: extension })
}

pub struct PluginsList<C> {
    plugins: BTreeMap<TypeId, BoxPlugin<C>>,
}

impl<C> Default for PluginsList<C> {
    fn default() -> Self {
        PluginsList {
            plugins: Default::default(),
        }
    }
}
impl<C: BuildContext> PluginsList<C>
where
    C: 'static,
    for<'a> C::Build<'a>: uhuh_ext::Context,
{
    pub fn insert<T>(&mut self, plugin: T) -> Result<(), UhuhError>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        if self.plugins.contains_key(&id) {
            return Err(UhuhError::new(format!(
                "Plugin '{}' already defined",
                core::any::type_name::<T>()
            )));
        }

        self.plugins.insert(id, plugin_box(plugin));

        Ok(())
    }

    pub fn get<T>(&self) -> Result<&T, UhuhError>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        let Some(plugin) = self.plugins.get(&id) else {
            return Err(UhuhError::new("Plugin not registered"));
        };

        plugin
            .as_any()
            .downcast_ref()
            .ok_or_else(|| UhuhError::new("Plugin not registered"))
    }

    pub fn get_mut<T>(&mut self) -> Result<&mut T, UhuhError>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        let Some(plugin) = self.plugins.get_mut(&id) else {
            return Err(UhuhError::new("Plugin not registered"));
        };

        plugin
            .as_any_mut()
            .downcast_mut()
            .ok_or_else(|| UhuhError::new("Plugin not registered"))
    }

    pub async fn build<'a>(self, mut context: C::Build<'a>) -> Result<(), UhuhError> {
        for plugin in self.plugins.into_values() {
            plugin.build(&mut context).await?;
        }
        Ok(())
    }
}

pub trait PluginSetupContext<C: BuildContext> {
    fn plugin<T>(&mut self, plugin: T) -> Result<(), UhuhError>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
        C: 'static;
}

pub trait PluginBuildContext<C: BuildContext> {
    fn configure_plugin<T>(&mut self) -> Result<&mut T, UhuhError>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static;
}
