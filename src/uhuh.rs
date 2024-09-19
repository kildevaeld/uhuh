use std::path::{Path, PathBuf};

use extensions::concurrent::Extensions;
use johnfig::Config;

use crate::Mode;

pub struct Uhuh {
    pub extensions: Extensions,
    pub config: Config,
    pub mode: Mode,
    pub name: String,
    pub root: PathBuf,
}

impl Uhuh {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn register<T: Send + Sync + 'static>(&mut self, value: T) -> &mut Self {
        self.extensions.insert(value);
        self
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.extensions.get_mut::<T>()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
