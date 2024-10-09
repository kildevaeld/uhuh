use crate::{Actions, BuildContext, OnBuild, OnInit, OnSetup, UhuhError};

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
    pub fn new(context: C) -> Builder<SetupPhase<C>, C> {
        Builder::from_phase(SetupPhase {
            context,
            actions: Actions::default(),
        })
    }

    pub async fn setup(self) -> Result<Builder<BuildPhase<C>, C>, UhuhError> {
        let phase = self.phase.next().await?;
        Ok(Builder::from_phase(phase))
    }
}

impl<C: BuildContext> OnSetup<C> for SetupPhase<C> {
    fn on_setup<T: crate::SetupAction<C> + 'static>(&mut self, action: T) {
        self.actions.add_setup(action);
    }
}

impl<C: BuildContext> OnBuild<C> for SetupPhase<C> {
    fn on_build<T: crate::BuildAction<C> + 'static>(&mut self, action: T) {
        self.actions.add_build(action);
    }
}

impl<C: BuildContext> OnInit<C> for SetupPhase<C> {
    fn on_init<T: crate::InitAction<C> + 'static>(&mut self, action: T) {
        self.actions.add_init(action);
    }
}
