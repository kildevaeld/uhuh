use std::convert::Infallible;

use axum::{extract::Request, response::IntoResponse, routing::MethodRouter};
use tower::{Layer, Service};

use crate::{modifier::Modifier, router::Router};

pub trait Routing<C> {
    fn add_route(&mut self, path: &str, route: MethodRouter<C>) -> &mut Self;

    fn add_route_service<T>(&mut self, path: &str, route: T) -> &mut Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static;

    fn add_nest_service<T>(&mut self, path: &str, service: T) -> &mut Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static;

    fn add_nest(&mut self, path: &str, nested: Router<C>) -> &mut Self;

    fn add_fallback(&mut self, route: MethodRouter<C>);

    fn add_layer<L>(&mut self, layer: L)
    where
        L: Layer<axum::routing::Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static;

    fn add_modifier<M>(&mut self, modifier: M) -> &mut Self
    where
        M: Modifier<C> + Send + Sync + 'static,
        M::Modify: Send + 'static,
        C: Send + Sync;
}
