use std::sync::Arc;

use uhuh_exp::{
    extensions::{InitContext, InitList},
    BuildContext,
};
use uhuh_ext::Extensions;

use crate::Router;

pub trait AxumState {
    type State: Send + Sync + Clone;
}

pub trait AxumBuildContext: BuildContext
where
    Self::Output: AxumState,
    for<'a> Self::Init<'a>: AxumInitContext<Self>,
{
}

impl<T> AxumBuildContext for T
where
    T: BuildContext,
    T::Output: AxumState,
    for<'a> Self::Init<'a>: AxumInitContext<Self>,
{
}

pub trait AxumInitContext<C: BuildContext>
where
    C::Output: AxumState,
{
    fn router(&mut self) -> &mut Router<<C::Output as AxumState>::State>;
}

// Default Context
#[derive(Default)]
pub struct DefaultContext {
    ext: Extensions,
    router: Router<State>,
    init: InitList<Self>,
}

impl BuildContext for DefaultContext {
    type Setup<'a> = ();

    type Build<'a> = BuildCtx<'a>;

    type Init<'a> = InitCtx<'a>;

    type Config = ();

    type Output = Axum;

    fn run_setup<'a>(
        &'a mut self,
        modules: &'a [Box<dyn uhuh_exp::DynamicModule<Self>>],
    ) -> impl futures_core::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move {
            for module in modules {
                module.setup(()).await?;
            }

            Ok(())
        }
    }

    fn run_build<'a>(
        &'a mut self,
        modules: &'a [Box<dyn uhuh_exp::DynamicModule<Self>>],
        // config: &'a Config,
    ) -> impl futures_core::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move {
            for module in modules {
                module.build(BuildCtx { ext: &mut self.ext }, &()).await?;
            }

            Ok(())
        }
    }

    fn run_init<'a>(
        &'a mut self,
        modules: &'a [Box<dyn uhuh_exp::DynamicModule<Self>>],
    ) -> impl futures_core::Future<Output = Result<(), uhuh_exp::UhuhError>> + 'a {
        async move {
            for module in modules {
                module
                    .init(InitCtx {
                        router: &mut self.router,
                    })
                    .await?;
            }

            self.init
                .run(&mut InitCtx {
                    router: &mut self.router,
                })
                .await?;

            Ok(())
        }
    }

    fn build(
        self,
    ) -> impl futures_core::Future<Output = Result<Self::Output, uhuh_exp::UhuhError>> {
        async move {
            Ok(Axum {
                exts: self.ext,
                router: self.router,
            })
        }
    }
}

impl InitContext<DefaultContext> for DefaultContext {
    fn initializer<T>(&mut self, init: T)
    where
        T: uhuh_exp::extensions::Initializer<DefaultContext> + 'static,
        DefaultContext: 'static,
    {
        self.init.register(init);
    }
}

pub struct BuildCtx<'a> {
    ext: &'a mut Extensions,
}

impl<'a> uhuh_ext::Context for BuildCtx<'a> {
    fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.ext.get()
    }

    fn register<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T> {
        self.ext.insert(value)
    }
}

pub struct InitCtx<'a> {
    router: &'a mut Router<State>,
}

impl<'a> AxumInitContext<DefaultContext> for InitCtx<'a> {
    fn router(
        &mut self,
    ) -> &mut Router<<<DefaultContext as BuildContext>::Output as AxumState>::State> {
        self.router
    }
}

pub struct Axum {
    exts: Extensions,
    router: Router<State>,
}

impl Axum {
    pub fn into_router(self) -> axum::Router {
        self.into()
    }
}

impl From<Axum> for axum::Router {
    fn from(value: Axum) -> Self {
        let state = State {
            exts: value.exts.into(),
        };

        value.router.build(&state).with_state(state)
    }
}

impl AxumState for Axum {
    type State = State;
}

#[derive(Debug, Clone)]
pub struct State {
    exts: Arc<Extensions>,
}
