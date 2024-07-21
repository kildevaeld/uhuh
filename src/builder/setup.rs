use crate::{
    configure::Configure,
    initializer::Initializer,
    module::{box_module, DynamicModule},
    Error, Mode, Module,
};
use extensions::concurrent::Extensions;
use johnfig::{Config, ConfigBuilder};
use std::{future::Future, path::PathBuf};
use tracing::debug;

use super::{Build, Builder, Phase};

#[cfg(feature = "cli")]
use super::cmd::*;

impl<C> Builder<Setup<C>> {
    pub fn new(ctx: C, name: &str, mode: Mode) -> Self {
        Self {
            phase: Setup {
                ctx,
                modules: Vec::default(),
                initializers: Vec::default(),
                configures: Vec::default(),
                mode,
                name: name.to_string(),
                skip_on_missing_config: false,
                root: None,
                config_builder: ConfigBuilder::new(),
            },
        }
    }

    pub fn root(mut self, path: impl Into<PathBuf>) -> Self {
        self.phase.root = Some(path.into());
        self
    }

    pub fn set_root(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.phase.root = Some(path.into());
        self
    }

    pub fn skip_missing_config(mut self, on: bool) -> Self {
        self.phase.skip_on_missing_config = on;
        self
    }

    pub fn configure<T>(mut self, func: T) -> Self
    where
        T: Configure + 'static,
    {
        self.phase.configures.push(Box::new(func));
        self
    }

    pub fn add_configure<T>(&mut self, func: T) -> &mut Self
    where
        T: Configure + 'static,
    {
        self.phase.configures.push(Box::new(func));
        self
    }

    pub fn initializer<T: Initializer<C> + 'static>(mut self, init: T) -> Self {
        self.phase.initializers.push(Box::new(init));
        self
    }

    pub fn add_initializer<T: Initializer<C> + 'static>(&mut self, init: T) -> &mut Self {
        self.phase.initializers.push(Box::new(init));
        self
    }

    pub fn module<T: Module<C> + 'static>(mut self) -> Self {
        self.phase.modules.push(box_module::<T, C>());
        self
    }

    pub fn add_module<T: Module<C> + 'static>(&mut self) -> &mut Self {
        self.phase.modules.push(box_module::<T, C>());
        self
    }

    pub async fn setup(self) -> Result<Builder<Build<C>>, Error> {
        Ok(Builder {
            phase: self.phase.next().await?,
        })
    }
}

pub struct Setup<C> {
    ctx: C,
    modules: Vec<Box<dyn DynamicModule<C>>>,
    initializers: Vec<Box<dyn Initializer<C>>>,
    configures: Vec<Box<dyn Configure>>,
    mode: Mode,
    name: String,
    skip_on_missing_config: bool,
    root: Option<PathBuf>,
    config_builder: ConfigBuilder,
}

impl<C> Phase for Setup<C> {
    type Next = Build<C>;
    fn next(mut self) -> impl Future<Output = Result<Self::Next, Error>> {
        async move {
            let mut config = Config::default();
            let mut extensions = Extensions::default();

            #[cfg(feature = "cli")]
            let mut cmds = Vec::default();
            for module in &self.modules {
                #[cfg(feature = "cli")]
                let mut cmd = None;
                debug!(module = ?module.config_section(), "Setup module");
                module.setup(SetupCtx {
                    ctx: &mut self.ctx,
                    module_name: module.config_section(),
                    #[cfg(feature = "cli")]
                    cmds: &mut cmd,
                    extensions: &mut extensions,
                })?;

                #[cfg(feature = "cli")]
                if let Some(cmd) = cmd {
                    debug!(module = ?module.config_section(), "Adding command");
                    cmds.push(cmd);
                }

                if let Some(cfg) = module.default_config() {
                    debug!(module = ?module.config_section(), cfg = ?cfg, "Setting default config");
                    config.set(module.config_section(), cfg);
                }
            }

            for cfg in self.configures {
                cfg.call(&mut config)?;
            }

            self.config_builder.add_default(move |cfg| {
                cfg.extend(config.clone());
            });

            Ok(Build {
                ctx: self.ctx,
                modules: self.modules,
                initializers: self.initializers,
                #[cfg(feature = "cli")]
                cmds,
                extensions,
                config: self.config_builder,
                mode: self.mode,
                name: self.name,
                skip_on_missing_config: self.skip_on_missing_config,
                root: self.root,
            })
        }
    }
}

pub struct SetupCtx<'a, C> {
    module_name: &'a str,
    #[allow(unused)]
    ctx: &'a mut C,
    #[cfg(feature = "cli")]
    cmds: &'a mut Option<Cmd<C>>,
    extensions: &'a mut Extensions,
}

impl<'a, C> SetupCtx<'a, C> {
    #[cfg(feature = "cli")]
    pub fn cmd<A>(&mut self, cmd: clap::Command, action: A) -> &mut Self
    where
        A: CmdAction<C> + 'static,
        C: 'static,
    {
        let name = self.module_name.to_string();
        *self.cmds = Some(Cmd {
            cmd: cmd.name(name),
            action: box_action(action),
        });
        self
    }

    pub fn name(&self) -> &str {
        self.module_name
    }

    pub fn register<T: Send + Sync + Clone + 'static>(&mut self, value: T) -> &mut Self {
        self.extensions.insert(value);
        self
    }
}
