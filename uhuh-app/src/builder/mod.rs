use core::marker::PhantomData;

mod build;
mod init;
mod phase;
mod setup;

use crate::BuildContext;

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
