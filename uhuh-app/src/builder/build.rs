use crate::{factory::Factories, Actions, BuildContext, OnBuild, OnInit, UhuhError};

use super::{init::InitPhase, Builder, Phase};

pub struct BuildPhase<C: BuildContext> {
    pub(super) context: C,
    pub(super) actions: Actions<C>,
    pub(super) factories: Factories<C>,
}

impl<C: BuildContext> Phase<C> for BuildPhase<C> {
    type Next = InitPhase<C>;

    fn next(mut self) -> impl core::future::Future<Output = Result<Self::Next, crate::UhuhError>> {
        async move {
            self.context.run_build(&mut self.actions).await?;
            self.context.run_build(&mut self.factories).await?;

            Ok(InitPhase {
                context: self.context,
                actions: self.actions,
                factories: self.factories,
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

impl<C: BuildContext> OnBuild<C> for BuildPhase<C> {
    fn on_build<T: crate::BuildAction<C> + 'static>(&mut self, action: T) {
        self.actions.add_build(action);
    }
}

impl<C: BuildContext> OnInit<C> for BuildPhase<C> {
    fn on_init<T: crate::InitAction<C> + 'static>(&mut self, action: T) {
        self.actions.add_init(action);
    }
}
