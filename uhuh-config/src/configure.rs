use alloc::boxed::Box;
use uhuh_exp::UhuhError;

use crate::resolver::ConfigResolver;

pub trait Configure<C, T>
where
    C: ConfigResolver<T>,
{
    fn configure(self: Box<Self>, resolver: &mut C) -> Result<(), UhuhError>;
}

impl<C, T, F> Configure<C, T> for F
where
    C: ConfigResolver<T>,

    for<'a> F: FnOnce(&'a mut C) -> Result<(), UhuhError>,
{
    fn configure(self: Box<Self>, resolver: &mut C) -> Result<(), UhuhError> {
        (self)(resolver)
    }
}
