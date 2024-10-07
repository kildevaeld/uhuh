use alloc::{boxed::Box, collections::btree_set::BTreeSet, vec::Vec};
use core::{any::TypeId, future::Future};

use super::{build::Build, phase::Phase, Builder};
use crate::{
    context::BuildContext,
    error::UhuhError,
    module::{box_module, DynamicModule},
    Config, Module,
};

impl<C: BuildContext + 'static> Builder<Setup<C>> {
    pub fn new(context: C) -> Builder<Setup<C>> {
        Self {
            phase: Setup {
                context,
                modules: Default::default(),
                module_map: Default::default(),
                config: Config::default(),
            },
        }
    }

    pub fn module<T: Module<C> + 'static>(mut self) -> Self {
        self.phase.add_module::<T>();
        self
    }

    pub async fn setup(self) -> Result<Builder<Build<C>>, UhuhError> {
        Ok(Builder {
            phase: self.phase.next().await?,
        })
    }
}

pub struct Setup<C> {
    context: C,
    modules: Vec<Box<dyn DynamicModule<C>>>,
    module_map: BTreeSet<TypeId>,
    config: Config,
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
                config: self.config,
            };

            Ok(next)
        }
    }
}
