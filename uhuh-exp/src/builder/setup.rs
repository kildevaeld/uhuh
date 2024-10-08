use alloc::{boxed::Box, collections::btree_set::BTreeSet, vec::Vec};
use core::{any::TypeId, future::Future, marker::PhantomData};

use super::{build::BuildPhase, phase::Phase, Builder};
use crate::{
    context::BuildContext,
    error::UhuhError,
    extensions::{InitContext, Initializer, Plugin, PluginSetupContext, Setup, SetupBuildContext},
    module::{box_module, DynamicModule},
    Module,
};

impl<C: BuildContext + 'static> Builder<SetupPhase<C>, C> {
    pub fn new(context: C) -> Builder<SetupPhase<C>, C> {
        Self {
            phase: SetupPhase {
                context,
                modules: Default::default(),
                module_map: Default::default(),
            },
            _c: PhantomData,
        }
    }

    pub fn module<T: Module<C> + 'static>(mut self) -> Self {
        self.phase.add_module::<T>();
        self
    }

    pub async fn build(self) -> Result<C::Output, UhuhError> {
        self.setup().await?.build().await?.init().await
    }

    pub async fn setup(self) -> Result<Builder<BuildPhase<C>, C>, UhuhError> {
        Ok(Builder {
            phase: self.phase.next().await?,
            _c: PhantomData,
        })
    }
}

impl<C: BuildContext> Builder<SetupPhase<C>, C>
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

impl<C: BuildContext> Builder<SetupPhase<C>, C>
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

impl<C: BuildContext> Builder<SetupPhase<C>, C>
where
    C: SetupBuildContext<C> + 'static,
{
    pub fn constant<T>(mut self, init: T) -> Result<Self, UhuhError>
    where
        T: Setup<C> + Send + Sync + 'static,
        T::Output: Send + Sync,
    {
        self.phase.context.register_constant(init)?;
        Ok(self)
    }
}

pub struct SetupPhase<C> {
    context: C,
    modules: Vec<Box<dyn DynamicModule<C>>>,
    module_map: BTreeSet<TypeId>,
}

impl<C: BuildContext + 'static> SetupPhase<C> {
    pub fn add_module<T: Module<C> + 'static>(&mut self) -> &mut Self {
        if !self.module_map.contains(&TypeId::of::<T>()) {
            self.modules.push(box_module::<T, C>());
            self.module_map.insert(TypeId::of::<T>());
        }
        self
    }
}

impl<C> Phase<C> for SetupPhase<C>
where
    C: BuildContext,
{
    type Next = BuildPhase<C>;

    fn next(mut self) -> impl Future<Output = Result<Self::Next, UhuhError>> {
        async move {
            self.context.run_setup(&self.modules).await?;

            let next = BuildPhase {
                context: self.context,
                modules: self.modules,
            };

            Ok(next)
        }
    }

    fn context(&mut self) -> &mut C {
        &mut self.context
    }
}
