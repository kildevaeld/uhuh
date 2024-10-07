use core::future::Future;

use alloc::{boxed::Box, vec::Vec};

use super::{phase::Phase, Builder};
use crate::{context::BuildContext, error::UhuhError, module::DynamicModule};

impl<C: BuildContext> Builder<Init<C>> {
    pub async fn init(self) -> Result<C::Output, UhuhError> {
        self.phase.next().await
    }
}

pub struct Init<C> {
    pub(super) context: C,
    pub(super) modules: Vec<Box<dyn DynamicModule<C>>>,
}

impl<C: BuildContext> Phase for Init<C> {
    type Next = C::Output;

    fn next(mut self) -> impl Future<Output = Result<Self::Next, UhuhError>> {
        async move {
            self.context.run_init(&self.modules).await?;
            self.context.build().await
        }
    }
}
