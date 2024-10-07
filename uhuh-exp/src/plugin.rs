use core::{
    any::{Any, TypeId},
    future::Future,
};

use alloc::{boxed::Box, collections::btree_map::BTreeMap, format};
use daserror::BoxError;

use crate::{types::BoxFuture, BuildContext, UhuhError};

pub trait Plugin<C> {
    type Output;
    type Error: Into<BoxError<'static>>;
    fn build(self) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

pub(crate) trait DynamicPlugin<C> {
    fn build<'a>(self: Box<Self>, context: &'a mut C) -> BoxFuture<'a, Result<(), UhuhError>>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(crate) struct PluginBox<T> {
    inner: T,
}

impl<T, C> DynamicPlugin<C> for PluginBox<T>
where
    C: BuildContext,
    T: 'static + Plugin<C> + Send,
    T::Output: Send + Sync + 'static,
    T::Error: 'static,
{
    fn build<'a>(self: Box<Self>, context: &'a mut C) -> BoxFuture<'a, Result<(), UhuhError>> {
        Box::pin(async move {
            let ret = self.inner.build().await.map_err(UhuhError::new)?;
            extensions.insert(ret);
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

pub type BoxPlugin<C> = Box<dyn DynamicPlugin<C> + Send + Sync>;

pub fn plugin_box<T, C>(extension: T) -> BoxPlugin<C>
where
    T: 'static + Plugin<C> + Send + Sync,
    T::Output: Send + Sync + 'static,
    T::Error: 'static,
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
impl<C> PluginsList<C> {
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

    pub async fn build<'a>(self, context: &mut C) -> Result<(), UhuhError> {
        for plugin in self.plugins.into_values() {
            plugin.build(context).await?;
        }
        Ok(())
    }
}
