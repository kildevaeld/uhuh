use std::sync::Arc;

use uhuh_ext::Extensions;

use crate::{
    request::Request,
    task::{Task, UhuhTask},
};

#[derive(Debug, Default)]
pub struct ContextBuilder {
    ext: Extensions,
}

impl uhuh_ext::ContextBuilder for ContextBuilder {
    fn register<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T> {
        self.ext.insert(value)
    }
}

impl ContextBuilder {
    pub fn build(self) -> Ctx {
        Ctx {
            ext: Arc::new(self.ext),
        }
    }
}

#[derive(Debug, Default)]
pub struct Ctx {
    ext: Arc<Extensions>,
}

impl uhuh_ext::Context for Ctx {
    fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.ext.get()
    }
}

#[derive(Debug, Clone)]
pub struct Context<C> {
    pub(crate) ctx: infinitask::TaskCtx<C>,
}

impl<C: Send + Sync + 'static> Context<C> {
    pub async fn register<R, T>(&self, request: R, task: T)
    where
        R: Request<C> + serde::Serialize + Send + Sync + 'static,
        R::Error: std::error::Error + Send + Sync + 'static,
        R::Output: Send,
        T: Task<C, R> + Send + Sync + 'static,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        self.ctx.register(UhuhTask::new(request, task)).await
    }
}

impl<C: uhuh_ext::Context> uhuh_ext::Context for Context<C> {
    fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.ctx.data().get()
    }
}
