use alloc::{boxed::Box, collections::btree_set::BTreeSet, vec::Vec};
use core::{any::TypeId, future::Future};

use super::{build::Build, phase::Phase, Builder};
use crate::{
    context::BuildContext,
    error::UhuhError,
    extensions::{InitContext, Initializer, Plugin, PluginSetupContext},
    module::{box_module, DynamicModule},
    Module,
};

impl<C: BuildContext + 'static> Builder<Setup<C>> {
    pub fn new(context: C) -> Builder<Setup<C>> {
        Self {
            phase: Setup {
                context,
                modules: Default::default(),
                module_map: Default::default(),
            },
        }
    }

    pub fn module<T: Module<C> + 'static>(mut self) -> Self {
        self.phase.add_module::<T>();
        self
    }

    pub async fn build(self) -> Result<C::Output, UhuhError> {
        self.setup().await?.build().await?.init().await
    }

    pub async fn setup(self) -> Result<Builder<Build<C>>, UhuhError> {
        Ok(Builder {
            phase: self.phase.next().await?,
        })
    }
}

impl<C: BuildContext> Builder<Setup<C>>
where
    C: InitContext<C> + 'static,
{
    pub fn initializer<T>(mut self, init: T) -> Self
    where
        T: Initializer<C> + 'static,
    {
        self.phase.context.initializer(init);
        self
    }
}

impl<C: BuildContext> Builder<Setup<C>>
where
    C: PluginSetupContext<C> + 'static,
{
    pub fn plugin<T>(mut self, init: T) -> Result<Self, UhuhError>
    where
        T: Plugin<C> + Send + Sync + 'static,
        T::Output: Send + Sync,
    {
        self.phase.context.plugin(init)?;
        Ok(self)
    }
}

pub struct Setup<C> {
    context: C,
    modules: Vec<Box<dyn DynamicModule<C>>>,
    module_map: BTreeSet<TypeId>,
}

impl<C: BuildContext + 'static> Setup<C> {
    pub fn add_module<T: Module<C> + 'static>(&mut self) -> &mut Self {
        if !self.module_map.contains(&TypeId::of::<T>()) {
            self.modules.push(box_module::<T, C>());
            self.module_map.insert(TypeId::of::<T>());
        }
        self
    }
}

impl<C> Phase for Setup<C>
where
    C: BuildContext,
{
    type Next = Build<C>;

    fn next(mut self) -> impl Future<Output = Result<Self::Next, UhuhError>> {
        async move {
            self.context.run_setup(&self.modules).await?;

            let next = Build {
                context: self.context,
                modules: self.modules,
            };

            Ok(next)
        }
    }
}
