use phase::Phase;

mod build;
mod init;
mod phase;
mod setup;

pub use self::{build::Build, init::Init, setup::Setup};

pub struct Builder<P: Phase> {
    phase: P,
}
