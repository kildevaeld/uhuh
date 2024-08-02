use crate::{initializer::Initializer, module::DynamicModule, uhuh::Uhuh, Error, Mode};
use extensions::concurrent::Extensions;
use johnfig::Config;
use std::{
    future::Future,
    path::{Path, PathBuf},
};

use super::Phase;

pub struct Init<C> {
    pub(super) ctx: C,
    pub(super) initializers: Vec<Box<dyn Initializer<C>>>,
    pub(super) extensions: Extensions,
    pub(super) config: Config,
    pub(super) name: String,
    pub(super) mode: Mode,
    pub(super) root: PathBuf,
    pub(super) modules: Vec<Box<dyn DynamicModule<C>>>,
}

impl<C> Phase for Init<C> {
    type Next = Uhuh<C>;
    fn next(mut self) -> impl Future<Output = Result<Self::Next, Error>> {
        async move {
            for module in &self.modules {
                module
                    .init(InitCtx {
                        ctx: &mut self.ctx,
                        ext: &mut self.extensions,
                        config: &self.config,
                        root: &self.root,
                    })
                    .await?;
            }

            for initializer in self.initializers.into_iter() {
                initializer
                    .call(InitCtx {
                        ctx: &mut self.ctx,
                        ext: &mut self.extensions,
                        config: &self.config,
                        root: &self.root,
                    })
                    .await?;
            }

            let mut app = Uhuh {
                ctx: self.ctx,
                extensions: self.extensions,
                config: self.config,
                mode: self.mode,
                root: self.root,
                name: self.name,
            };

            for module in self.modules {
                module.finish(&mut app).await?;
            }

            Ok(app)
        }
    }
}

pub struct InitCtx<'a, C> {
    ctx: &'a mut C,
    ext: &'a mut Extensions,
    config: &'a Config,
    root: &'a Path,
}

impl<'a, C> InitCtx<'a, C> {
    pub fn root(&self) -> &Path {
        self.root
    }

    pub fn register<T: Send + Sync + 'static>(&mut self, value: T) -> &mut Self {
        self.ext.insert(value);
        self
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.ext.get::<T>()
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.ext.get_mut::<T>()
    }

    pub fn config(&self) -> &Config {
        self.config
    }
}

impl<'a, C> core::ops::Deref for InitCtx<'a, C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a, C> core::ops::DerefMut for InitCtx<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}
