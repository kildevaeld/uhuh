use std::convert::Infallible;

use axum::{extract::Request, response::IntoResponse, routing::MethodRouter};
use tower::{Layer, Service};

use tracing::debug;

use crate::{
    modifier::{modifier_box, BoxModifier, Modifier, ModifierLayer},
    routing::Routing,
};

pub struct Router<C> {
    inner: Option<axum::Router<C>>,
    modifiers: Vec<BoxModifier<C>>,
}

impl<C: 'static + Clone + Send + Sync> Default for Router<C> {
    fn default() -> Self {
        Router {
            inner: Some(axum::Router::<C>::new()),
            modifiers: Vec::default(),
        }
    }
}

impl<C: 'static + Clone + Send + Sync> Router<C> {
    fn mutate<T>(&mut self, func: T)
    where
        T: FnOnce(axum::Router<C>) -> axum::Router<C>,
    {
        let router = self.inner.take().unwrap();
        self.inner = Some(func(router));
    }

    pub fn build(self, state: &C) -> axum::Router<C> {
        self.inner
            .unwrap()
            .layer(ModifierLayer::new(self.modifiers, state.clone()))
    }
}

impl<C: 'static + Clone + Send + Sync> Routing<C> for Router<C> {
    fn add_route(&mut self, path: &str, route: MethodRouter<C>) -> &mut Self {
        debug!(path = %path, "register route");
        self.mutate(|router| router.route(path, route));
        self
    }

    fn add_route_service<T>(&mut self, path: &str, route: T) -> &mut Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        debug!(path = %path, "register service");
        self.mutate(|router| router.route_service(path, route));
        self
    }

    fn add_nest_service<T>(&mut self, path: &str, service: T) -> &mut Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        debug!(path = %path, "nest service");
        self.mutate(|router| router.nest_service(path, service));
        self
    }

    fn add_nest(&mut self, path: &str, nested: Router<C>) -> &mut Self {
        debug!(path = %path, "nest router");
        self.mutate(move |router| router.nest(path, nested.inner.unwrap()));
        self
    }

    fn add_fallback(&mut self, route: MethodRouter<C>) {
        self.mutate(|router| router.fallback(route))
    }

    fn add_layer<L>(&mut self, layer: L)
    where
        L: Layer<axum::routing::Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.mutate(|router| router.layer(layer))
    }

    fn add_modifier<M>(&mut self, modifier: M) -> &mut Self
    where
        M: Modifier<C> + Send + Sync + 'static,
        M::Modify: Send + 'static,
        C: Send + Sync,
    {
        self.modifiers.push(modifier_box(modifier));
        self
    }
}
