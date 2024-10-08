use super::{init::InitPhase, phase::Phase, Builder};
use crate::{context::BuildContext, error::UhuhError, module::DynamicModule};
use alloc::{boxed::Box, vec::Vec};
use core::{future::Future, marker::PhantomData};

impl<C: BuildContext> Builder<BuildPhase<C>, C> {
    pub async fn build(self) -> Result<Builder<InitPhase<C>, C>, UhuhError> {
        Ok(Builder {
            phase: self.phase.next().await?,
            _c: PhantomData,
        })
    }
}

pub struct BuildPhase<C> {
    pub(super) context: C,
    pub(super) modules: Vec<Box<dyn DynamicModule<C>>>,
}

impl<C: BuildContext> Phase<C> for BuildPhase<C> {
    type Next = InitPhase<C>;

    fn next(mut self) -> impl Future<Output = Result<Self::Next, UhuhError>> {
        async move {
            self.context.run_build(&self.modules).await?;

            let next = InitPhase {
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
