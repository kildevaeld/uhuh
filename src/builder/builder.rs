use crate::{initializer::Initializer, Error};
use core::future::Future;

use super::InitCtx;

// pub struct Uhuh<C> {
//     pub ctx: C,
//     pub extensions: Extensions,
//     pub config: Config,
//     pub mode: Mode,
//     pub name: String,
//     pub root: PathBuf,
// }

// impl<C> Uhuh<C> {
//     pub fn root(&self) -> &Path {
//         &self.root
//     }

//     pub fn register<T: Send + Sync + Clone + 'static>(&mut self, value: T) -> &mut Self {
//         self.extensions.insert(value);
//         self
//     }

//     pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
//         self.extensions.get::<T>()
//     }

//     pub fn config(&self) -> &Config {
//         &self.config
//     }
// }

pub trait Phase {
    type Next;
    fn next(self) -> impl Future<Output = Result<Self::Next, Error>>;
}

pub struct Builder<T: Phase> {
    pub(super) phase: T,
}

pub fn register_ext<T: Send + Sync + Clone + 'static, C>(value: T) -> impl Initializer<C> {
    move |mut ctx: InitCtx<'_, C>| {
        ctx.register(value.clone());
        Ok(())
    }
}
