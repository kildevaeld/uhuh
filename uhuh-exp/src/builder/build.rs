use super::{init::Init, phase::Phase, Builder};
use crate::{context::BuildContext, error::UhuhError, module::DynamicModule};
use alloc::{boxed::Box, vec::Vec};
use core::future::Future;

impl<C: BuildContext> Builder<Build<C>> {
    pub async fn build(self) -> Result<Builder<Init<C>>, UhuhError> {
        Ok(Builder {
            phase: self.phase.next().await?,
        })
    }
}

pub struct Build<C> {
    pub(super) context: C,
    pub(super) modules: Vec<Box<dyn DynamicModule<C>>>,
}

impl<C: BuildContext> Phase for Build<C> {
    type Next = Init<C>;

    fn next(mut self) -> impl Future<Output = Result<Self::Next, UhuhError>> {
        async move {
            self.context.run_build(&self.modules).await?;

            let next = Init {
                context: self.context,
                modules: self.modules,
            };

            Ok(next)
        }
    }
}
