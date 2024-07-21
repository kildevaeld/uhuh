use futures_core::Future;
use johnfig::Config;

use crate::Error;

mod internal {
    pub trait Sealed {}

    impl Sealed for johnfig::ConfigBuilder {}
}

pub trait ConfigBuilderExt: internal::Sealed + Sized {
    fn load(self) -> impl Future<Output = Result<Config, Error>>;
}

impl ConfigBuilderExt for johnfig::ConfigBuilder {
    fn load(self) -> impl Future<Output = Result<Config, Error>> {
        async move {
            let (sx, rx) = futures_channel::oneshot::channel();
            std::thread::spawn(|| {
                let ret = self.build_config().map_err(Error::new);
                sx.send(ret)
            });
            rx.await.map_err(Error::new)?
        }
    }
}
