use uhuh_exp::{BuildContext, Builder, SetupPhase};

use crate::{configure::Configure, resolver::ConfigResolver};

pub trait ConfigBuildContext<C>: BuildContext
where
    C: ConfigResolver<Self::Config>,
{
    fn configure<F>(&mut self, func: F)
    where
        F: Configure<C, Self::Config> + 'static;
}

pub trait BuilderExt<C, T>: Sized
where
    C: ConfigBuildContext<T>,
    T: ConfigResolver<C::Config>,
{
    fn configure<F>(self, func: F) -> Self
    where
        F: Configure<T, C::Config> + 'static;
}

impl<C, T> BuilderExt<C, T> for Builder<SetupPhase<C>, C>
where
    C: ConfigBuildContext<T>,
    T: ConfigResolver<C::Config>,
{
    fn configure<F>(mut self, func: F) -> Self
    where
        F: Configure<T, <C>::Config> + 'static,
    {
        self.context().configure(func);
        self
    }
}
