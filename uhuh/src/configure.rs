use johnfig::Config;

use crate::Error;

pub trait Configure {
    fn call(self: Box<Self>, config: &mut Config) -> Result<(), Error>;
}

impl<T> Configure for T
where
    T: FnOnce(&mut Config) -> Result<(), Error>,
{
    fn call(self: Box<Self>, config: &mut Config) -> Result<(), Error> {
        (self)(config)
    }
}
