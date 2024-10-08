use core::{
    any::{Any, TypeId},
    future::Future,
};

use alloc::{boxed::Box, collections::btree_map::BTreeMap, format};
use daserror::BoxError;
use uhuh_ext::Context as _;

use crate::{types::LocalBoxFuture, BuildContext, UhuhError};

pub trait Setup<C: BuildContext> {
    type Output;
    type Error: Into<BoxError<'static>>;
    fn build(
        self,
        ctx: &mut C::Setup<'_>,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

pub(crate) trait DynamicSetup<C: BuildContext> {
    fn build<'a, 'b>(
        self: Box<Self>,
        context: &'a mut C::Setup<'b>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(crate) struct SetupBox<T> {
    inner: T,
}

impl<T, C> DynamicSetup<C> for SetupBox<T>
where
    C: BuildContext + 'static,
    for<'a> C::Setup<'a>: uhuh_ext::Context,
    T: 'static + Setup<C> + Send,
    T::Output: Send + Sync + 'static,
    T::Error: 'static,
{
    fn build<'a, 'b>(
        self: Box<Self>,
        mut context: &'a mut C::Setup<'b>,
    ) -> LocalBoxFuture<'a, Result<(), UhuhError>> {
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

pub(crate) type BoxSetup<C> = Box<dyn DynamicSetup<C> + Send + Sync>;

pub(crate) fn setup_box<T, C>(extension: T) -> BoxSetup<C>
where
    T: 'static + Setup<C> + Send + Sync,
    T::Output: Send + Sync + 'static,
    T::Error: 'static,
    C: BuildContext + 'static,
    for<'a> C::Setup<'a>: uhuh_ext::Context,
{
    Box::new(SetupBox { inner: extension })
}

pub struct SetupList<C> {
    tree: BTreeMap<TypeId, BoxSetup<C>>,
}

impl<C> Default for SetupList<C> {
    fn default() -> Self {
        SetupList {
            tree: Default::default(),
        }
    }
}

impl<C: BuildContext> SetupList<C>
where
    C: 'static,
    for<'a> C::Setup<'a>: uhuh_ext::Context,
{
    pub fn insert<T>(&mut self, Setup: T) -> Result<(), UhuhError>
    where
        T: 'static + Setup<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        if self.tree.contains_key(&id) {
            return Err(UhuhError::new(format!(
                "Setup '{}' already defined",
                core::any::type_name::<T>()
            )));
        }

        self.tree.insert(id, setup_box(Setup));

        Ok(())
    }

    pub fn get<T>(&self) -> Result<&T, UhuhError>
    where
        T: 'static + Setup<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        let Some(setup) = self.tree.get(&id) else {
            return Err(UhuhError::new("Setup not registered"));
        };

        setup
            .as_any()
            .downcast_ref()
            .ok_or_else(|| UhuhError::new("Setup not registered"))
    }

    pub fn get_mut<T>(&mut self) -> Result<&mut T, UhuhError>
    where
        T: 'static + Setup<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        let id = TypeId::of::<T>();
        let Some(setup) = self.tree.get_mut(&id) else {
            return Err(UhuhError::new("Setup not registered"));
        };

        setup
            .as_any_mut()
            .downcast_mut()
            .ok_or_else(|| UhuhError::new("Setup not registered"))
    }

    pub async fn build<'a>(self, mut context: C::Setup<'a>) -> Result<(), UhuhError> {
        for setup in self.tree.into_values() {
            setup.build(&mut context).await?;
        }
        Ok(())
    }
}

pub trait SetupBuildContext<C: BuildContext> {
    fn register_constant<T>(&mut self, setup: T) -> Result<(), UhuhError>
    where
        T: 'static + Setup<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
        C: 'static;
}

// pub trait SetupBuildContext<C: BuildContext> {
//     fn constant<T>(&mut self) -> Result<&mut T, UhuhError>
//     where
//         T: 'static + Setup<C> + Send + Sync,
//         T::Output: Send + Sync + 'static,
//         T::Error: 'static;
// }
