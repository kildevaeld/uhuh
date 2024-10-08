use alloc::{boxed::Box, vec::Vec};
use core::marker::PhantomData;
use uhuh_exp::{BuildContext, UhuhError};

use crate::{configure::Configure, resolver::ConfigResolver};

pub struct ConfigBuilder<C, T>
where
    C: BuildContext,
    T: ConfigResolver<C::Config>,
{
    resolver: T,
    funcs: Vec<Box<dyn Configure<T, C::Config>>>,
    _c: PhantomData<C>,
}

impl<C, T: Default> Default for ConfigBuilder<C, T>
where
    C: BuildContext,
    T: ConfigResolver<C::Config>,
{
    fn default() -> Self {
        ConfigBuilder {
            resolver: T::default(),
            funcs: Default::default(),
            _c: PhantomData,
        }
    }
}

impl<C, T> ConfigBuilder<C, T>
where
    C: BuildContext,
    T: ConfigResolver<C::Config>,
{
    pub fn new(resolver: T) -> ConfigBuilder<C, T> {
        ConfigBuilder {
            resolver,
            funcs: Default::default(),
            _c: PhantomData,
        }
    }
}

impl<C, T> ConfigBuilder<C, T>
where
    C: BuildContext,
    T: ConfigResolver<C::Config>,
{
    pub fn configure<F>(&mut self, func: F)
    where
        F: Configure<T, C::Config> + 'static,
    {
        self.funcs.push(Box::new(func));
    }

    pub fn build(mut self) -> Result<C::Config, UhuhError> {
        for func in self.funcs {
            func.configure(&mut self.resolver)?;
        }

        self.resolver.build().map_err(UhuhError::new)
    }
}
