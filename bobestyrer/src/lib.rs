#![no_std]

use core::future::Future;

extern crate alloc;

#[cfg(any(feature = "smol", feature = "tokio"))]
use core::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

pub trait JoinHandle<T> {
    type Future: Future<Output = Result<T, Self::Error>>;
    type Error;

    fn into_future(self) -> Self::Future;

    fn abort(self);

    fn is_finished(&self) -> bool;

    fn detach(self);
}

pub trait Executor {
    type JoinHandle<T>: JoinHandle<T>;

    fn spawn<T: Future + Send + 'static>(&self, future: T) -> Self::JoinHandle<T::Output>
    where
        T::Output: Send;
    fn spawn_blocking<F, O>(&self, func: F) -> Self::JoinHandle<O>
    where
        F: FnOnce() -> O + Send + 'static,
        O: Send + 'static;

    fn block_on<T>(&self, future: T) -> T::Output
    where
        T: Future;
}

#[cfg(feature = "tokio")]
#[derive(Debug, Clone)]
pub enum Tokio {
    Runtime(alloc::sync::Arc<tokio::runtime::Runtime>),
    Handle(tokio::runtime::Handle),
}

#[cfg(feature = "tokio")]
impl Tokio {
    pub fn from_global() -> Tokio {
        Tokio::Handle(tokio::runtime::Handle::current())
    }

    pub fn from_runtime(runtime: tokio::runtime::Runtime) -> Tokio {
        Tokio::Runtime(runtime.into())
    }

    pub fn from_handle(handle: tokio::runtime::Handle) -> Tokio {
        Tokio::Handle(handle)
    }

    #[inline]
    pub fn handle(&self) -> &tokio::runtime::Handle {
        match self {
            Self::Handle(handle) => handle,
            Self::Runtime(runtime) => runtime.handle(),
        }
    }
}

#[cfg(feature = "tokio")]
impl Executor for Tokio {
    type JoinHandle<T> = tokio::task::JoinHandle<T>;

    fn spawn<T: Future + Send + 'static>(&self, future: T) -> Self::JoinHandle<T::Output>
    where
        T::Output: Send,
    {
        self.handle().spawn(future)
    }

    fn spawn_blocking<F, O>(&self, func: F) -> Self::JoinHandle<O>
    where
        F: FnOnce() -> O + Send + 'static,
        O: Send + 'static,
    {
        self.handle().spawn_blocking(func)
    }

    fn block_on<T>(&self, future: T) -> T::Output
    where
        T: Future,
    {
        self.handle().block_on(future)
    }
}

#[cfg(feature = "tokio")]
impl<T> JoinHandle<T> for tokio::task::JoinHandle<T> {
    type Future = tokio::task::JoinHandle<T>;
    type Error = tokio::task::JoinError;

    fn abort(self) {
        self.abort_handle().abort()
    }

    fn into_future(self) -> Self::Future {
        self
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
    }

    fn detach(self) {}
}

#[cfg(feature = "smol")]
#[derive(Debug, Default, Clone)]
pub struct Smol;

#[cfg(feature = "smol")]
impl Executor for Smol {
    type JoinHandle<T> = smol::Task<T>;

    fn spawn<T: Future + Send + 'static>(&self, future: T) -> Self::JoinHandle<T::Output>
    where
        T::Output: Send,
    {
        smol::spawn(future)
    }

    fn spawn_blocking<F, O>(&self, func: F) -> Self::JoinHandle<O>
    where
        F: FnOnce() -> O + Send + 'static,
        O: Send + 'static,
    {
        smol::unblock(func)
    }

    fn block_on<T>(&self, future: T) -> T::Output
    where
        T: Future,
    {
        smol::block_on(future)
    }
}

#[cfg(feature = "smol")]
impl<T> JoinHandle<T> for smol::Task<T> {
    type Future = SmolJoinHandleFuture<T>;

    type Error = Infallible;

    fn into_future(self) -> Self::Future {
        SmolJoinHandleFuture { task: self }
    }

    fn abort(self) {
        smol::block_on(self.cancel());
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
    }

    fn detach(self) {
        self.detach()
    }
}

#[cfg(feature = "smol")]
pub struct SmolJoinHandleFuture<T> {
    task: smol::Task<T>,
}

#[cfg(feature = "smol")]
impl<T> core::future::Future for SmolJoinHandleFuture<T> {
    type Output = Result<T, Infallible>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        match poll(&mut self.as_mut().task, cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(ret) => Poll::Ready(Ok(ret)),
        }
    }
}

#[cfg(feature = "any")]
#[derive(Debug)]
pub enum AnyJoinError {
    #[cfg(feature = "tokio")]
    Tokio(tokio::task::JoinError),
    #[cfg(feature = "smol")]
    Smol(Infallible),
}

#[cfg(feature = "any")]
impl core::fmt::Display for AnyJoinError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "tokio")]
            Self::Tokio(e) => e.fmt(f),
            #[cfg(feature = "smol")]
            Self::Smol(e) => e.fmt(f),
        }
    }
}

#[cfg(feature = "any")]
impl core::error::Error for AnyJoinError {}

#[derive(Debug, Clone)]
#[cfg(feature = "any")]
pub enum AnyExecutor {
    #[cfg(feature = "tokio")]
    Tokio(Tokio),
    #[cfg(feature = "smol")]
    Smol(Smol),
}

#[cfg(feature = "any")]
impl Executor for AnyExecutor {
    type JoinHandle<T> = AnyJoinHandle<T>;

    fn spawn<T: Future + Send + 'static>(&self, future: T) -> Self::JoinHandle<T::Output>
    where
        T::Output: Send,
    {
        match self {
            #[cfg(feature = "tokio")]
            Self::Tokio(tokio) => AnyJoinHandle::Tokio(tokio.spawn(future)),
            #[cfg(feature = "smol")]
            Self::Smol(smol) => AnyJoinHandle::Smol(smol.spawn(future)),
        }
    }

    fn spawn_blocking<F, O>(&self, func: F) -> Self::JoinHandle<O>
    where
        F: FnOnce() -> O + Send + 'static,
        O: Send + 'static,
    {
        match self {
            #[cfg(feature = "tokio")]
            Self::Tokio(tokio) => AnyJoinHandle::Tokio(tokio.spawn_blocking(func)),
            #[cfg(feature = "smol")]
            Self::Smol(smol) => AnyJoinHandle::Smol(smol.spawn_blocking(func)),
        }
    }

    fn block_on<T>(&self, future: T) -> T::Output
    where
        T: Future,
    {
        match self {
            #[cfg(feature = "tokio")]
            Self::Tokio(tokio) => tokio.block_on(future),
            #[cfg(feature = "smol")]
            Self::Smol(smol) => smol.block_on(future),
        }
    }
}

#[cfg(feature = "any")]
pub enum AnyJoinHandle<T> {
    #[cfg(feature = "tokio")]
    Tokio(tokio::task::JoinHandle<T>),
    #[cfg(feature = "smol")]
    Smol(smol::Task<T>),
}

#[cfg(feature = "any")]
impl<T> JoinHandle<T> for AnyJoinHandle<T> {
    type Future = AnyJoinFuture<T>;
    type Error = AnyJoinError;

    fn into_future(self) -> Self::Future {
        match self {
            #[cfg(feature = "tokio")]
            Self::Tokio(tokio) => AnyJoinFuture::Tokio(tokio.into_future()),
            #[cfg(feature = "smol")]
            Self::Smol(smol) => AnyJoinFuture::Smol(smol),
        }
    }

    fn abort(self) {
        match self {
            #[cfg(feature = "tokio")]
            Self::Tokio(tokio) => tokio.abort(),
            #[cfg(feature = "smol")]
            Self::Smol(smol) => smol.abort(),
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            #[cfg(feature = "tokio")]
            Self::Tokio(tokio) => tokio.is_finished(),
            #[cfg(feature = "smol")]
            Self::Smol(smol) => smol.is_finished(),
        }
    }

    fn detach(self) {
        match self {
            #[cfg(feature = "tokio")]
            Self::Tokio(tokio) => tokio.detach(),
            #[cfg(feature = "smol")]
            Self::Smol(smol) => smol.detach(),
        }
    }
}

#[cfg(feature = "any")]
pub enum AnyJoinFuture<T> {
    #[cfg(feature = "tokio")]
    Tokio(tokio::task::JoinHandle<T>),
    #[cfg(feature = "smol")]
    Smol(smol::Task<T>),
}

#[cfg(feature = "any")]
impl<T> Future for AnyJoinFuture<T> {
    type Output = Result<T, AnyJoinError>;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let this = unsafe { Pin::get_unchecked_mut(self) };
        match this {
            #[cfg(feature = "tokio")]
            Self::Tokio(tokio) => match poll(tokio, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(ret) => Poll::Ready(ret.map_err(AnyJoinError::Tokio)),
            },
            #[cfg(feature = "smol")]
            Self::Smol(smol) => match poll(smol, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(ret) => Poll::Ready(Ok(ret)),
            },
        }
    }
}

#[cfg(any(feature = "smol", feature = "tokio"))]
fn poll<T: Future>(future: &mut T, cx: &mut Context<'_>) -> Poll<T::Output>
where
    T: Unpin,
{
    Future::poll(Pin::new(future), cx)
}

#[cfg(all(feature = "smol", feature = "any"))]
impl From<Smol> for AnyExecutor {
    fn from(smol: Smol) -> AnyExecutor {
        AnyExecutor::Smol(smol)
    }
}

#[cfg(all(feature = "tokio", feature = "any"))]
impl From<Tokio> for AnyExecutor {
    fn from(tokio: Tokio) -> AnyExecutor {
        AnyExecutor::Tokio(tokio)
    }
}

#[cfg(feature = "tokio")]
impl From<tokio::runtime::Runtime> for Tokio {
    fn from(value: tokio::runtime::Runtime) -> Self {
        Tokio::Runtime(value.into())
    }
}

#[cfg(all(feature = "tokio", feature = "any"))]
impl From<tokio::runtime::Runtime> for AnyExecutor {
    fn from(value: tokio::runtime::Runtime) -> Self {
        Tokio::Runtime(value.into()).into()
    }
}
