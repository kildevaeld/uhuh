use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc};

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use futures_core::future::BoxFuture;
use tower::{Layer, Service};

pub trait Modifier<C> {
    type Modify: Modify<C>;
    fn before<'a>(
        &'a self,
        request: &'a mut Request,
        state: &'a C,
    ) -> impl Future<Output = Self::Modify> + 'a + Send;
}

pub trait Modify<C> {
    fn modify<'a>(
        self,
        response: &'a mut Response,
        state: &'a C,
    ) -> impl Future<Output = ()> + 'a + Send;
}

pub trait DynModifier<C> {
    fn before<'a>(&'a self, request: &'a mut Request, state: &'a C) -> BoxFuture<'a, BoxModify<C>>;
}

pub trait DynModify<C> {
    fn modify<'a>(
        self: Box<Self>,
        response: &'a mut Response,
        state: &'a C,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

struct ModifyBox<T>(T);

impl<T, C: Send + Sync> DynModify<C> for ModifyBox<T>
where
    T: Modify<C> + Send + 'static,
{
    fn modify<'a>(
        self: Box<Self>,
        response: &'a mut Response,
        state: &'a C,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move { self.0.modify(response, state).await })
    }
}

struct ModifierBox<T>(T);

impl<T, C> DynModifier<C> for ModifierBox<T>
where
    T: Modifier<C> + Send + Sync,
    T::Modify: Send + 'static,
    C: Send + Sync,
{
    fn before<'a>(&'a self, request: &'a mut Request, state: &'a C) -> BoxFuture<'a, BoxModify<C>> {
        Box::pin(
            async move { Box::new(ModifyBox(self.0.before(request, state).await)) as BoxModify<C> },
        )
    }
}

pub fn modifier_box<T, C>(modifier: T) -> BoxModifier<C>
where
    T: Modifier<C> + Send + Sync + 'static,
    T::Modify: Send + 'static,
    C: Send + Sync,
{
    Box::new(ModifierBox(modifier))
}

pub type BoxModifier<C> = Box<dyn DynModifier<C> + Send + Sync>;
pub type BoxModify<C> = Box<dyn DynModify<C> + Send>;

#[derive(Clone)]
pub struct ModifierLayer<C> {
    modifiers: Arc<[BoxModifier<C>]>,
    state: C,
}

impl<C> ModifierLayer<C> {
    pub fn new(modifiers: Vec<BoxModifier<C>>, state: C) -> ModifierLayer<C> {
        ModifierLayer {
            modifiers: modifiers.into(),
            state,
        }
    }
}

impl<T, C: 'static + Send + Sync + Clone> Layer<T> for ModifierLayer<C> {
    type Service = ModifierLayerService<T, C>;
    fn layer(&self, inner: T) -> Self::Service {
        ModifierLayerService {
            service: inner,
            state: self.state.clone(),
            modifiers: self.modifiers.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ModifierLayerService<T, C> {
    service: T,
    state: C,
    modifiers: Arc<[BoxModifier<C>]>,
}

impl<T, C: 'static + Clone + Send + Sync> Service<Request> for ModifierLayerService<T, C>
where
    T: Service<Request, Error = Infallible> + Clone + Send + 'static,
    T::Response: IntoResponse,
    T::Future: Send,
{
    type Error = Infallible;
    type Response = Response;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let mut service = self.service.clone();
        let modifiers = self.modifiers.clone();
        let state = self.state.clone();

        Box::pin(async move {
            let mut mods = Vec::with_capacity(modifiers.len());

            req.extensions_mut().insert(state.clone());

            for modifier in modifiers.iter() {
                mods.push(modifier.before(&mut req, &state).await);
            }

            let mut res = service.call(req).await?.into_response();

            for modifier in mods {
                modifier.modify(&mut res, &state).await;
            }

            Ok(res)
        })
    }
}
