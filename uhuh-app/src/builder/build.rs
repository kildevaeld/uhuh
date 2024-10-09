use crate::{Actions, BuildContext, UhuhError};

use super::{init::InitPhase, Builder, Phase};

pub struct BuildPhase<C: BuildContext> {
    pub(super) context: C,
    pub(super) actions: Actions<C>,
}

impl<C: BuildContext> Phase<C> for BuildPhase<C> {
    type Next = InitPhase<C>;

    fn next(mut self) -> impl core::future::Future<Output = Result<Self::Next, crate::UhuhError>> {
        async move {
            self.context.run_build(&mut self.actions).await?;
            Ok(InitPhase {
                context: self.context,
                actions: self.actions,
            })
        }
    }

    fn context(&mut self) -> &mut C {
        todo!()
    }
}

impl<C: BuildContext> Builder<BuildPhase<C>, C> {
    pub async fn build(self) -> Result<Builder<InitPhase<C>, C>, UhuhError> {
        let phase = self.phase.next().await?;
        Ok(Builder::from_phase(phase))
    }
}
