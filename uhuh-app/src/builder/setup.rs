use crate::{Actions, BuildContext, UhuhError};

use super::{BuildPhase, Builder, Phase};

pub struct SetupPhase<C: BuildContext> {
    context: C,
    actions: Actions<C>,
}

impl<C: BuildContext> Phase<C> for SetupPhase<C> {
    type Next = BuildPhase<C>;

    fn next(mut self) -> impl core::future::Future<Output = Result<Self::Next, crate::UhuhError>> {
        async move {
            self.context.run_setup(&mut self.actions).await?;
            Ok(BuildPhase {
                context: self.context,
                actions: self.actions,
            })
        }
    }

    fn context(&mut self) -> &mut C {
        todo!()
    }
}

impl<C: BuildContext> Builder<SetupPhase<C>, C> {
    pub async fn setup(self) -> Result<Builder<BuildPhase<C>, C>, UhuhError> {
        let phase = self.phase.next().await?;
        Ok(Builder::from_phase(phase))
    }
}
