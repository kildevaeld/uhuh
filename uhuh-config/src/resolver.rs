use uhuh_exp::{BoxError, UhuhError};

#[derive(Debug, Default)]
pub struct SimpleResolver<T> {
    config: T,
}

impl<T> SimpleResolver<T> {
    pub fn new(config: T) -> SimpleResolver<T> {
        SimpleResolver { config }
    }

    pub fn set(&mut self, config: T) {
        self.config = config;
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.config
    }
}

impl<T> core::ops::Deref for SimpleResolver<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<T> core::ops::DerefMut for SimpleResolver<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl<T> ConfigResolver<T> for SimpleResolver<T> {
    type Error = UhuhError;
    fn build(self) -> Result<T, Self::Error> {
        Ok(self.config)
    }
}

pub trait ConfigResolver<T> {
    type Error: Into<BoxError<'static>>;
    fn build(self) -> Result<T, Self::Error>;
}

#[cfg(feature = "std")]
pub trait FsConfigResolver<T>: ConfigResolver<T> {
    fn add_search_path(&mut self, path: std::path::PathBuf);
}
