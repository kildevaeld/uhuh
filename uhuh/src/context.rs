use futures_core::Future;

use crate::{Error, Uhuh};

pub trait Context {
    type Output;

    fn build(self, uhuh: Uhuh) -> impl Future<Output = Result<Self::Output, Error>>;
}

impl Context for () {
    type Output = Uhuh;

    fn build(self, uhuh: Uhuh) -> impl Future<Output = Result<Self::Output, Error>> {
        async move { Ok(uhuh) }
    }
}
