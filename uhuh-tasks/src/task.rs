use std::any::Any;

use async_trait::async_trait;

use crate::{context::Context, error::Error, request::Request};

#[async_trait]
pub trait Task<C, R>
where
    R: Request<C>,
{
    type Error;
    async fn run(&self, ctx: Context<C>, value: R::Output) -> Result<(), Self::Error>;
}

pub struct UhuhTask<R, T> {
    req: R,
    task: T,
}

impl<R, T> UhuhTask<R, T> {
    pub fn new(req: R, task: T) -> UhuhTask<R, T> {
        UhuhTask { req, task }
    }
}

impl<R, T> UhuhTask<R, T> {
    pub async fn run<C: Clone + Send + Sync + 'static>(self, ctx: C) -> Result<(), Error>
    where
        R: Request<C> + Send + Sync + 'static,
        R::Error: std::error::Error + Send + Sync + 'static,
        R::Output: Send,
        T: Task<C, R> + Send + Sync + 'static,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        infinitask::InifiniTask::default().run(ctx, self).await;
        Ok(())
    }
}

#[async_trait]
impl<R, T, C: Send + Sync + 'static> infinitask::Task<C> for UhuhTask<R, T>
where
    R: Request<C> + Send + Sync + 'static,
    R::Error: std::error::Error + Send + Sync + 'static,
    R::Output: Send,
    T: Task<C, R> + Send + Sync + 'static,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    async fn run(
        self: Box<Self>,
        ctx: infinitask::TaskCtx<C>,
    ) -> Result<(), infinitask::TaskError> {
        let output = self
            .req
            .send(ctx.data())
            .await
            .map_err(infinitask::TaskError::new)?;

        self.task
            .run(Context { ctx }, output)
            .await
            .map_err(infinitask::TaskError::new)?;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
