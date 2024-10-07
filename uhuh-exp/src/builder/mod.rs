use phase::Phase;

mod build;
mod init;
mod phase;
mod setup;

pub struct Builder<P: Phase> {
    phase: P,
}
