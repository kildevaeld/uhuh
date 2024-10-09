use core::marker::PhantomData;

mod build;
mod init;
mod phase;
mod setup;

use crate::{BuildContext, OnBuild, OnInit, OnSetup};

pub use self::{build::BuildPhase, init::InitPhase, phase::Phase, setup::SetupPhase};

pub struct Builder<P: Phase<C>, C: BuildContext> {
    phase: P,
    _c: PhantomData<C>,
}

impl<P: Phase<C>, C: BuildContext> Builder<P, C> {
    pub fn context(&mut self) -> &mut C {
        self.phase.context()
    }
}

impl<P: Phase<C>, C: BuildContext> Builder<P, C> {
    pub(super) fn from_phase(phase: P) -> Builder<P, C> {
        Builder {
            phase,
            _c: PhantomData,
        }
    }
}

impl<P, C: BuildContext> Builder<P, C>
where
    P: Phase<C> + OnBuild<C>,
{
    pub fn on_build<T: crate::BuildAction<C> + 'static>(mut self, action: T) -> Self {
        self.phase.on_build(action);
        self
    }
}

impl<P, C: BuildContext> Builder<P, C>
where
    P: Phase<C> + OnSetup<C>,
{
    pub fn on_setup<T: crate::SetupAction<C> + 'static>(mut self, action: T) -> Self {
        self.phase.on_setup(action);
        self
    }
}

impl<P, C: BuildContext> Builder<P, C>
where
    P: Phase<C> + OnInit<C>,
{
    pub fn on_init<T: crate::InitAction<C> + 'static>(mut self, action: T) -> Self {
        self.phase.on_init(action);
        self
    }
}
