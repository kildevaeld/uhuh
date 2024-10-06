use std::{
    any::{Any, TypeId},
    collections::HashMap,
    path::Path,
};

use extensions::concurrent::Extensions;
use futures_core::{future::BoxFuture, Future};

use crate::Error;

pub trait Plugin<C> {
    type Output;
    type Error: std::error::Error + Send + Sync;
    fn build(self, root: &Path) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

pub(crate) trait DynamicPlugin<C> {
    fn build<'a>(
        self: Box<Self>,
        extensions: &'a mut Extensions,
        root: &'a Path,
    ) -> BoxFuture<'a, Result<(), Error>>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(crate) struct PluginBox<T> {
    inner: T,
}

impl<T, C> DynamicPlugin<C> for PluginBox<T>
where
    T: 'static + Plugin<C> + Send,
    T::Output: Send + Sync + 'static,
    T::Error: 'static,
{
    fn build<'a>(
        self: Box<Self>,
        extensions: &'a mut Extensions,
        path: &'a Path,
    ) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let ret = self.inner.build(path).await.map_err(Error::new)?;
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
    plugins: HashMap<TypeId, BoxPlugin<C>>,
}

impl<C> Default for PluginsList<C> {
    fn default() -> Self {
        PluginsList {
            plugins: Default::default(),
        }
    }
}
impl<C> PluginsList<C> {
    pub fn insert<T>(&mut self, plugin: T) -> Result<(), Error>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        if self.plugins.contains_key(&id) {
            return Err(Error::new(format!(
                "Plugin '{}' already defined",
                core::any::type_name::<T>()
            )));
        }

        self.plugins.insert(id, plugin_box(plugin));

        Ok(())
    }

    pub fn get<T>(&self) -> Result<&T, Error>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        let Some(plugin) = self.plugins.get(&id) else {
            return Err(Error::new("Plugin not registered"));
        };

        plugin
            .as_any()
            .downcast_ref()
            .ok_or_else(|| Error::new("Plugin not registered"))
    }

    pub fn get_mut<T>(&mut self) -> Result<&mut T, Error>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        let Some(plugin) = self.plugins.get_mut(&id) else {
            return Err(Error::new("Plugin not registered"));
        };

        plugin
            .as_any_mut()
            .downcast_mut()
            .ok_or_else(|| Error::new("Plugin not registered"))
    }

    pub async fn build<'a>(
        self,
        extensions: &'a mut Extensions,
        root: &'a Path,
    ) -> Result<(), Error> {
        for plugin in self.plugins.into_values() {
            plugin.build(extensions, root).await?;
        }
        Ok(())
    }
}
