use bobestyrer::AnyExecutor;
use extensions::concurrent::Extensions;
use futures_core::Future;
use std::path::{Path, PathBuf};
use tracing::debug;

use crate::{
    context::Context, module::DynamicModule, plugin::PluginsList, Error, Initializer, Mode, Plugin,
};

use super::{config::ConfigBuilder, Builder, Init, Phase};

#[cfg(feature = "cli")]
use super::cmd::*;

impl<C: Context> Builder<Build<C>> {
    pub async fn build(self) -> Result<Builder<Init<C>>, Error> {
        Ok(Builder {
            phase: self.phase.next().await?,
        })
    }

    pub async fn build_app(self) -> Result<C::Output, Error> {
        self.phase.next().await?.next().await
    }

    #[cfg(feature = "cli")]
    pub async fn cli<T>(self, run: T) -> Result<(), Error>
    where
        T: CmdAction<C>,
    {
        self.cli_with(clap::Command::new("wilbur"), run).await
    }

    #[cfg(feature = "cli")]
    pub async fn cli_with<T>(mut self, mut app: clap::Command, run: T) -> Result<(), Error>
    where
        T: CmdAction<C>,
    {
        let cmds = std::mem::take(&mut self.phase.cmds);

        app = app
            .arg(clap::Arg::new("config").long("config").short('c'))
            .arg(
                clap::Arg::new("mode")
                    .long("mode")
                    .default_value("development"),
            )
            .arg(clap::Arg::new("root").long("root").short('r'))
            .name(self.phase.name.clone());

        for cmd in &cmds {
            app = app.subcommand(&cmd.cmd);
        }

        let cli = app.get_matches();

        let mode = cli.get_one::<String>("mode");

        self.phase.mode = match mode.map(|m: &String| m.as_str()) {
            Some("prod" | "production") => Mode::Production,
            Some("dev" | "development") => Mode::Development,
            _ => {
                eprintln!("Unknown mode. using development");
                Mode::Development
            }
        };

        if let Some(config_path) = cli.get_one::<String>("config") {
            debug!(config_path = ?config_path, "Using config paths");

            let path = PathBuf::from(config_path);

            if !path.is_file() {
                self.phase
                    .config
                    .add_search_path(path)
                    .map_err(Error::new)?;
            } else {
                self.phase.config.add_file(path);
            }
        }

        if let Some(root) = cli.get_one::<String>("root") {
            self.phase.root = Some(PathBuf::from(root).canonicalize().map_err(Error::new)?);
        }

        debug!(mode = ?self.phase.mode, "Mode set");

        let app = self.phase.next().await?.next().await?;

        match cli.subcommand() {
            Some((name, args)) => {
                let Some(cmd) = cmds.into_iter().find(|m| m.cmd.get_name() == name) else {
                    eprintln!("unknown command");
                    return Ok(());
                };

                cmd.action.call(app, args.clone()).await?;

                Ok(())
            }
            None => {
                Box::new(run).call(app, cli).await?;
                Ok(())
            }
        }
    }
}

pub struct Build<C> {
    pub(super) ctx: C,
    pub(super) modules: Vec<Box<dyn DynamicModule<C>>>,
    pub(super) initializers: Vec<Box<dyn Initializer<C>>>,
    #[cfg(feature = "cli")]
    pub(super) cmds: Vec<Cmd<C>>,
    pub(super) extensions: Extensions,
    pub(super) config: ConfigBuilder,
    pub(super) mode: Mode,
    pub(super) name: String,
    pub(super) skip_on_missing_config: bool,
    pub(super) root: Option<PathBuf>,
    pub(super) executor: AnyExecutor,
    pub(super) plugins: PluginsList<C>,
}

impl<C: Context> Phase for Build<C> {
    type Next = Init<C>;
    fn next(mut self) -> impl Future<Output = Result<Self::Next, Error>> {
        async move {
            let root = match self.root {
                Some(path) => path,
                None => std::env::current_dir().map_err(Error::new)?,
            };

            debug!(path = ?root, "Root directory");

            let config = self.config.build(&self.executor, self.mode.clone()).await?;

            debug!(files = ?config.files(), "Using config files");

            for module in &self.modules {
                let Some(cfg) = config.get(module.config_section()) else {
                    if self.skip_on_missing_config {
                        debug!(module = ?module.config_section(), "Missing config. Skipping");
                        continue;
                    }

                    return Err(Error::new(format!(
                        "config not set for: {}",
                        module.config_section()
                    )));
                };

                debug!(module = ?module.config_section(), "Initializing");
                module
                    .build(
                        BuildCtx {
                            ctx: &mut self.ctx,
                            initializers: &mut self.initializers,
                            extensions: &mut self.extensions,
                            mode: &self.mode,
                            root: &*root,
                            plugins: &mut self.plugins,
                        },
                        cfg.clone(),
                    )
                    .await?;
            }

            self.plugins
                .build(&mut self.extensions, &root, &self.mode)
                .await?;

            Ok(Init {
                ctx: self.ctx,
                initializers: self.initializers,
                extensions: self.extensions,
                config,
                mode: self.mode,
                name: self.name,
                modules: self.modules,
                root,
            })
        }
    }
}

pub struct BuildCtx<'a, C> {
    ctx: &'a mut C,
    initializers: &'a mut Vec<Box<dyn Initializer<C>>>,
    extensions: &'a mut Extensions,
    mode: &'a Mode,
    root: &'a Path,
    plugins: &'a mut PluginsList<C>,
}

impl<'a, C> BuildCtx<'a, C> {
    pub fn mode(&self) -> &Mode {
        self.mode
    }

    pub fn root(&self) -> &Path {
        self.root
    }

    pub fn add_initializer<T: Initializer<C> + 'static>(&mut self, init: T) -> &mut Self {
        self.initializers.push(Box::new(init));
        self
    }

    pub fn register<T: Send + Sync + 'static>(&mut self, value: T) -> &mut Self {
        self.extensions.insert(value);
        self
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.extensions.get_mut::<T>()
    }

    pub fn plugin<T>(&mut self) -> Result<&mut T, Error>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        self.plugins.get_mut()
    }
}

impl<'a, C> core::ops::Deref for BuildCtx<'a, C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a, C> core::ops::DerefMut for BuildCtx<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}
