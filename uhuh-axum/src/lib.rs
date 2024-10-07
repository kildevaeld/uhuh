pub mod context;
mod modifier;
mod router;
mod routing;

use std::sync::Arc;

use uhuh_ext::Extensions;

pub use self::{
    modifier::{Modifier, Modify},
    router::Router,
    routing::Routing,
};

// pub trait AxumContext: uhuh_ext::ContextBuilder {
//     type State;
//     fn router(&mut self) -> &mut Router<Self::State>;
// }

// #[derive(Default)]
// pub struct DefaultCtxBuilder {
//     ext: Extensions,
//     router: Router<Ctx>,
// }

// impl uhuh_ext::ContextBuilder for DefaultCtxBuilder {
//     type Context = DefaultCtx;
//     type Error = ();

//     fn register<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T> {
//         self.ext.insert(value)
//     }

//     fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
//         self.ext.get()
//     }

//     fn build(
//         self,
//     ) -> impl futures_core::Future<Output = Result<Self::Context, Self::Error>> + Send {
//         async move {
//             let ctx = DefaultCtx {
//                 ext: Arc::new(self.ext),
//                 router: self.router,
//             };

//             Ok(ctx)
//         }
//     }
// }

// impl AxumContext for DefaultCtxBuilder {
//     type State = Ctx;
//     fn router(&mut self) -> &mut Router<Self::State> {
//         &mut self.router
//     }
// }

// #[derive(Clone)]
// pub struct Ctx {
//     ext: Arc<Extensions>,
// }

// impl uhuh_ext::Context for Ctx {
//     fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
//         self.ext.get()
//     }
// }

// pub struct DefaultCtx {
//     ext: Arc<Extensions>,
//     router: Router<Ctx>,
// }

// impl DefaultCtx {
//     pub fn into_router(self) -> axum::Router {
//         let ctx = Ctx { ext: self.ext };
//         let router = self.router.build(&ctx);
//         router.with_state(ctx)
//     }
// }

// impl From<DefaultCtx> for axum::Router {
//     fn from(value: DefaultCtx) -> Self {
//         value.into_router()
//     }
// }

// impl uhuh_ext::Context for DefaultCtx {
//     fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
//         self.ext.get()
//     }
// }
