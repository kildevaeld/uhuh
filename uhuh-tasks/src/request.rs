use crate::{
    error::{Error, RequestError},
    Task, UhuhTask,
};
use async_trait::async_trait;

#[async_trait]
pub trait Request<C> {
    type Output;
    type Error;

    async fn send(self, ctx: &C) -> Result<Self::Output, Self::Error>;
}

pub trait RequestExt<C>: Request<C> {
    fn map<T>(self, transformer: T) -> Transformer<T, Self>
    where
        Self: Sized,
        T: Transform<C, Self>,
    {
        Transformer {
            req: self,
            transform: transformer,
        }
    }

    fn task<T: Task<C, Self>>(self, task: T) -> UhuhTask<Self, T>
    where
        Self: Sized,
    {
        UhuhTask::new(self, task)
    }
}

impl<T, C> RequestExt<C> for T where T: Request<C> {}

#[async_trait]
pub trait Transform<C, R: Request<C>> {
    type Output;
    type Error;

    async fn transform(&self, ctx: &C, req: R::Output) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Transformer<T, R> {
    req: R,
    transform: T,
}

#[async_trait]
impl<T, R, C: Send + Sync> Request<C> for Transformer<T, R>
where
    R: Request<C> + Send,
    R::Output: Send,
    R::Error: std::error::Error + Send + Sync + 'static,
    T: Transform<C, R> + Send + Sync,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    type Output = T::Output;
    type Error = Error;

    async fn send(self, ctx: &C) -> Result<Self::Output, Self::Error> {
        let resp = self.req.send(ctx).await.map_err(RequestError::new)?;
        let resp = self
            .transform
            .transform(ctx, resp)
            .await
            .map_err(RequestError::new)?;

        Ok(resp)
    }
}
