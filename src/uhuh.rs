use std::path::{Path, PathBuf};

use extensions::concurrent::Extensions;
use johnfig::Config;

use crate::Mode;

pub struct Uhuh<C> {
    pub ctx: C,
    pub extensions: Extensions,
    pub config: Config,
    pub mode: Mode,
    pub name: String,
    pub root: PathBuf,
}

impl<C> Uhuh<C> {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn register<T: Send + Sync + Clone + 'static>(&mut self, value: T) -> &mut Self {
        self.extensions.insert(value);
        self
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
