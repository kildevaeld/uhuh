use crate::{BuildContext, Extensions, Factory, UhuhError};
use core::future::Future;

pub trait BuildStateContext {
    fn state(&self) -> &Extensions;
    fn state_mut(&mut self) -> &mut Extensions;
}

pub struct Constant<T>(Option<T>);

impl<C: BuildContext, T: Send + Sync + 'static> Factory<C> for Constant<T>
where
    for<'a> C::Setup<'a>: BuildStateContext,
    for<'a> C::Init<'a>: BuildStateContext,
{
    type Error = UhuhError;

    fn on_setup<'a, 'b>(
        &'a mut self,
        mut ctx: crate::HookCtx<'a, C, <C as BuildContext>::Setup<'b>>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async move {
            ctx.state_mut().insert(self.0.take().unwrap());
            Ok(())
        }
    }

    fn on_init<'a, 'b>(
        &'a mut self,
        mut ctx: crate::HookCtx<'a, C, <C as BuildContext>::Init<'b>>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a {
        async move {
            let c = ctx.state_mut().remove::<T>().unwrap();
            Ok(())
        }
    }
}
