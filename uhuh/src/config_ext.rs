use futures_core::Future;
use johnfig::Config;

use crate::{Error, Mode};

mod internal {
    pub trait Sealed {}

    impl Sealed for johnfig::ConfigBuilder {}
}

pub trait ConfigBuilderExt: internal::Sealed + Sized {
    fn load(self, mode: Mode) -> impl Future<Output = Result<Config, Error>>;
}

impl ConfigBuilderExt for johnfig::ConfigBuilder {
    fn load(self, mode: Mode) -> impl Future<Output = Result<Config, Error>> {
        async move {
            let (sx, rx) = futures_channel::oneshot::channel();
            std::thread::spawn(move || {
                let ret = self
                    .build_with(move |ext| {
                        vaerdi::value!({
                            "ext": ext,
                            "mode": mode.clone()
                        })
                    })
                    .and_then(|m| m.config())
                    .map_err(Error::new);
                sx.send(ret)
            });
            rx.await.map_err(Error::new)?
        }
    }
}
