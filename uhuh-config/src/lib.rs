use std::marker::PhantomData;

use uhuh_exp::{BoxError, BuildContext, UhuhError};

pub trait ConfigResolver<T> {
    type Error: Into<BoxError<'static>>;
    fn build(self) -> Result<T, Self::Error>;
}

pub trait Configure<C, T>
where
    C: ConfigResolver<T>,
{
    fn configure(self: Box<Self>, resolver: &mut C) -> Result<(), UhuhError>;
}

pub trait ConfigBuildContext<C>: BuildContext
where
    C: ConfigResolver<Self::Config>,
{
    fn configure<F>(&mut self, func: F)
    where
        F: Configure<C, Self::Config> + 'static;
}

pub struct ConfigBuilder<C, T>
where
    C: BuildContext,
    T: ConfigResolver<C::Config>,
{
    resolver: T,
    funcs: Vec<Box<dyn Configure<T, C::Config>>>,
    _c: PhantomData<C>,
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
