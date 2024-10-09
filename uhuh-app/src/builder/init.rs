use crate::{Actions, BuildContext, OnInit, UhuhError};

use super::{Builder, Phase};

pub struct InitPhase<C: BuildContext> {
    pub(super) context: C,
    pub(super) actions: Actions<C>,
}

impl<C: BuildContext> Phase<C> for InitPhase<C> {
    type Next = C::Output;

    fn next(mut self) -> impl core::future::Future<Output = Result<Self::Next, crate::UhuhError>> {
        async move {
            self.context.run_init(&mut self.actions).await?;
            self.context.build().await
        }
    }

    fn context(&mut self) -> &mut C {
        todo!()
    }
}

impl<C: BuildContext> Builder<InitPhase<C>, C> {
    pub async fn init(self) -> Result<C::Output, UhuhError> {
        self.phase.next().await
    }
}

impl<C: BuildContext> OnInit<C> for InitPhase<C> {
    fn on_init<T: crate::InitAction<C> + 'static>(&mut self, action: T) {
        self.actions.add_init(action);
    }
}
