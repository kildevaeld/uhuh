use uhuh_app::{
    builder::Builder, Actions, BuildAction, BuildContext, Factory, HookCtx, InitAction,
    SetupAction, UhuhError,
};

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

    fn build(self) -> impl std::future::Future<Output = Result<Self::Output, uhuh_app::UhuhError>> {
        async move { Ok(()) }
    }

    fn run_build<'a, T: uhuh_app::BuildAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl futures::Future<Output = Result<(), uhuh_app::UhuhError>> + 'a {
        async move { action.run(&mut (), &()).await }
    }

    fn run_init<'a, T: uhuh_app::InitAction<Self> + 'a>(
        &'a mut self,
        action: T,
    ) -> impl futures::Future<Output = Result<(), uhuh_app::UhuhError>> + 'a {
        async move { action.run(&mut ()).await }
    }
}

struct Test(&'static str);

impl<C: BuildContext> SetupAction<C> for Test {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Setup<'b>,
    ) -> impl futures::Future<Output = Result<(), UhuhError>> {
        async move {
            print!("{}: ", self.0);
            println!("Hello, World");
            Ok(())
        }
    }
}

impl<C: BuildContext> BuildAction<C> for Test {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Build<'b>,
        config: &'a C::Config,
    ) -> impl futures::Future<Output = Result<(), UhuhError>> {
        async move {
            print!("{}: ", self.0);
            println!("Hello, World");
            Ok(())
        }
    }
}

impl<C: BuildContext> InitAction<C> for Test {
    fn run<'a, 'b>(
        self,
        ctx: &'a mut <C as BuildContext>::Init<'b>,
    ) -> impl futures::Future<Output = Result<(), UhuhError>> {
        async move {
            print!("{}: ", self.0);
            println!("Hello, World");
            Ok(())
        }
    }
}

struct TestFature;

impl<C: BuildContext> Factory<C> for TestFature {
    type Error = UhuhError;

    fn on_setup<'a, 'b>(
        &'a mut self,
        mut ctx: HookCtx<'a, C, <C as BuildContext>::Setup<'b>>,
    ) -> impl futures::Future<Output = Result<(), Self::Error>> + 'a {
        async move {
            println!("Setup");

            ctx.register(Test2);

            //
            Ok(())
        }
    }

    fn on_build<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, <C as BuildContext>::Build<'b>>,
        config: &'a C::Config,
    ) -> impl futures::Future<Output = Result<(), Self::Error>> + 'a {
        async move {
            //
            println!("Build");

            Ok(())
        }
    }

    fn on_init<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, <C as BuildContext>::Init<'b>>,
    ) -> impl futures::Future<Output = Result<(), Self::Error>> + 'a {
        async move {
            //
            println!("Init");

            Ok(())
        }
    }
}

struct Test2;

impl<C: BuildContext> Factory<C> for Test2 {
    type Error = UhuhError;
    fn on_setup<'a, 'b>(
        &'a mut self,
        ctx: HookCtx<'a, C, <C as BuildContext>::Setup<'b>>,
    ) -> impl futures::Future<Output = Result<(), Self::Error>> + 'a {
        async move {
            println!("Set child");
            Ok(())
        }
    }
}

fn main() {
    futures::executor::block_on(async move {
        let builder = Builder::new(Context {})
            .with(TestFature)
            .on_setup(Test("Setup"))
            .on_build(Test("Build"))
            .on_init(Test("Init"))
            .setup()
            .await?
            .on_build(Test("Build2"))
            .on_init(Test("Init2"))
            .build()
            .await?
            .on_init(Test("Init3"))
            .init()
            .await?;

        Result::<_, UhuhError>::Ok(())
    })
    .unwrap();
}
