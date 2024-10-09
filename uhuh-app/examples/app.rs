use uhuh_app::BuildContext;

struct Context {}

impl BuildContext for Context {
    type Setup<'a> = ();

    type Build<'a> = ();

    type Init<'a> = ();

    type Config = ();

    type Output = ();

    fn run_setup<'a, T: uhuh_app::SetupAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl std::future::Future<Output = Result<(), uhuh_app::UhuhError>> + 'a {
        async move {
            //
            action.run(&mut ()).await
        }
    }

    fn run_build<'a, T: uhuh_app::BuildAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl std::future::Future<Output = Result<(), uhuh_app::UhuhError>> + 'a {
        async move {
            //
            action.run(&mut ()).await
        }
    }

    fn run_init<'a, T: uhuh_app::BuildAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl std::future::Future<Output = Result<(), uhuh_app::UhuhError>> + 'a {
        async move {
            //
            action.run(&mut ()).await
        }
    }

    fn build(self) -> impl std::future::Future<Output = Result<Self::Output, uhuh_app::UhuhError>> {
        async move { Ok(()) }
    }
}

fn main() {}
