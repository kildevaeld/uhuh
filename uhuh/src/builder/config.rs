use std::path::PathBuf;

use bobestyrer::{AnyExecutor, Executor, JoinHandle};
use johnfig::Config;
use toback::Toback;
use tracing::{debug, warn};

use crate::{Configure, Error, Mode};

#[derive(Default)]
pub struct ConfigBuilder {
    files: Vec<PathBuf>,
    builder: Option<johnfig::ConfigBuilder>,
    configures: Vec<Box<dyn Configure + Send>>,
}

impl ConfigBuilder {
    pub fn add_file(&mut self, path: PathBuf) -> &mut Self {
        self.files.push(path);
        self
    }

    pub fn add_search_path(&mut self, path: PathBuf) -> Result<&mut Self, Error> {
        self.builder
            .get_or_insert_with(|| johnfig::ConfigBuilder::new())
            .add_search_path(path)?;

        Ok(self)
    }

    pub fn add_configure(&mut self, configure: Box<dyn Configure + Send>) -> &mut Self {
        self.configures.push(configure);
        self
    }

    pub fn add_filename_pattern(&mut self, pattern: String) -> &mut Self {
        self.builder
            .get_or_insert_with(johnfig::ConfigBuilder::new)
            .add_name_pattern(pattern);
        self
    }

    pub async fn build(self, executor: &AnyExecutor, mode: Mode) -> Result<Config, Error> {
        executor
            .spawn_blocking(move || {
                let mut config = Config::default();

                for cfg in self.configures {
                    cfg.call(&mut config)?;
                }

                if let Some(builder) = self.builder {
                    let cfg = builder
                        .build_with(move |ext| {
                            vaerdi::value!({
                                "ext": ext,
                                "mode": mode.clone()
                            })
                        })
                        .and_then(|m| m.config())
                        .map_err(Error::new)?;

                    config.extend(cfg);
                }

                let encoder = Toback::<Config>::new();

                for path in self.files {
                    if !path.is_file() {
                        debug!(path = ?path, "Path not a file. Skipping");
                        continue;
                    }

                    let Some(encoder) = encoder.encoder_from_path(&path) else {
                        warn!(path = ?path, "Could not find a decoder for path. Skipping");
                        continue;
                    };

                    let ret = std::fs::read(path).map_err(Error::new)?;

                    let cfg = encoder.load(&ret).map_err(Error::new)?;

                    config.extend(cfg);
                }

                Result::<_, Error>::Ok(config)
            })
            .into_future()
            .await
            .map_err(Error::new)?
    }
}
