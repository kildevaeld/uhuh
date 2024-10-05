use core::{future::Future, pin::Pin};

use alloc::{boxed::Box, vec::Vec};

pub trait Context {
    fn get<T: 'static + Send + Sync>(&self) -> Option<&T>;
}

pub trait ContextBuilder {
    type Error;
    type Context;
    fn register<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T>;
    fn get<T: 'static + Send + Sync>(&self) -> Option<&T>;

    fn build(self) -> impl Future<Output = Result<Self::Context, Self::Error>> + Send;
}

pub trait Factory<C> {
    fn build<'a>(&self, ctx: &'a mut C) -> impl Future<Output = ()> + Send + 'a;
}

pub struct Builder<C> {
    factories: Vec<Box<dyn for<'a> FnOnce(&'a mut C) -> Pin<Box<dyn Future<Output = ()> + 'a>>>>,
}

impl<C: ContextBuilder> Builder<C> {
    pub fn new() -> Builder<C> {
        Builder {
            factories: Default::default(),
        }
    }

    pub fn push<T>(&mut self, factory: T)
    where
        T: Factory<C> + 'static,
    {
        self.factories.push(Box::new(move |ctx| {
            Box::pin(async move {
                factory.build(ctx).await;
            })
        }));
    }

    pub fn with<T>(mut self, factory: T) -> Self
    where
        T: Factory<C> + 'static,
    {
        self.push(factory);
        self
    }

    pub async fn build(
        self,
        mut ctx: C,
    ) -> Result<<C as ContextBuilder>::Context, <C as ContextBuilder>::Error> {
        for factory in self.factories {
            factory(&mut ctx).await;
        }

        ctx.build().await
    }
}
