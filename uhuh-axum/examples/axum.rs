use axum::routing::get;
use uhuh_axum::{
    context::{AxumBuildContext, AxumInitContext, AxumState, DefaultContext, InitCtx},
    Routing,
};
use uhuh_exp::{BuildContext, Builder, Module, UhuhError};

pub struct TestModule;

impl<C> Module<C> for TestModule
where
    C: AxumBuildContext,
    C::Output: AxumState + 'static,
    for<'a> <C as BuildContext>::Init<'a>: AxumInitContext<C>,
{
    const CONFIG_SECTION: &'static str = "";

    type Config = ();

    type Error = UhuhError;

    fn build(
        ctx: <C as uhuh_exp::BuildContext>::Build<'_>,
        config: Option<Self::Config>,
    ) -> impl futures_core::Future<Output = Result<(), Self::Error>> {
        async move { Ok(()) }
    }

    fn init(
        mut ctx: <C as BuildContext>::Init<'_>,
    ) -> impl futures_core::Future<Output = Result<(), Self::Error>> {
        async move {
            ctx.router()
                .add_route("/page", get(|| async move { "Hello, page!" }));
            Ok(())
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), UhuhError> {
    let builder = Builder::new(DefaultContext::default())
        .module::<TestModule>()
        .initializer(|ctx: &mut InitCtx| {
            ctx.router()
                .add_route("/", get(|| async move { "Hello, World!" }));
            Ok(())
        })
        .build()
        .await?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, builder.into_router()).await.unwrap();

    Ok(())
}
