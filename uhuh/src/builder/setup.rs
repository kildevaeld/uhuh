use crate::{
    configure::Configure,
    context::Context,
    initializer::Initializer,
    module::{box_module, DynamicModule},
    plugin::PluginsList,
    Error, Mode, Module, Plugin,
};
use bobestyrer::AnyExecutor;
use extensions::concurrent::Extensions;
use johnfig::Config;
use std::{any::TypeId, collections::VecDeque, future::Future, path::PathBuf};
use tracing::debug;
use vaerdi::hashbrown::HashSet;

use super::{config::ConfigBuilder, Build, Builder, Phase};

#[cfg(feature = "cli")]
use super::cmd::*;

impl<C> Builder<Setup<C>>
where
    C: Context,
{
    pub fn new<E: Into<AnyExecutor>>(ctx: C, name: &str, mode: Mode, executor: E) -> Self {
        Self {
            phase: Setup {
                ctx,
                modules: Vec::default(),
                initializers: Vec::default(),
                mode,
                name: name.to_string(),
                skip_on_missing_config: false,
                root: None,
                config_builder: ConfigBuilder::default(),
                module_map: Default::default(),
                executor: executor.into(),
                plugins: Default::default(),
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

    pub fn config_pattern(mut self, pattern: impl ToString) -> Self {
        self.phase
            .config_builder
            .add_filename_pattern(pattern.to_string());
        self
    }

    pub fn add_config_pattern(&mut self, pattern: impl ToString) -> &mut Self {
        self.phase
            .config_builder
            .add_filename_pattern(pattern.to_string());
        self
    }

    pub fn config_search_path(mut self, path: impl Into<PathBuf>) -> Result<Self, Error> {
        self.phase.config_builder.add_search_path(path.into())?;
        Ok(self)
    }

    pub fn add_config_search_path(&mut self, path: impl Into<PathBuf>) -> Result<&mut Self, Error> {
        self.phase.config_builder.add_search_path(path.into())?;
        Ok(self)
    }

    pub fn config_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.phase.config_builder.add_file(path.into());
        self
    }

    pub fn add_config_file(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.phase.config_builder.add_file(path.into());
        self
    }

    pub fn skip_missing_config(mut self, on: bool) -> Self {
        self.phase.skip_on_missing_config = on;
        self
    }

    pub fn configure<T>(mut self, func: T) -> Self
    where
        T: Configure + Send + 'static,
    {
        self.phase.config_builder.add_configure(Box::new(func));
        self
    }

    pub fn add_configure<T>(&mut self, func: T) -> &mut Self
    where
        T: Configure + Send + 'static,
    {
        self.phase.config_builder.add_configure(Box::new(func));
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
        self.add_module::<T>();
        self
    }

    pub fn add_module<T: Module<C> + 'static>(&mut self) -> &mut Self {
        if !self.phase.module_map.contains(&TypeId::of::<T>()) {
            self.phase.modules.push(box_module::<T, C>());
            self.phase.module_map.insert(TypeId::of::<T>());
        }
        self
    }

    pub async fn setup(self) -> Result<Builder<Build<C>>, Error> {
        Ok(Builder {
            phase: self.phase.next().await?,
        })
    }

    pub async fn build(self) -> Result<C::Output, Error> {
        self.setup().await?.build_app().await
    }

    #[cfg(feature = "cli")]
    pub async fn cli<T>(self, run: T) -> Result<(), Error>
    where
        T: CmdAction<C>,
    {
        self.setup().await?.cli(run).await
    }
}

pub struct Setup<C> {
    ctx: C,
    modules: Vec<Box<dyn DynamicModule<C>>>,
    initializers: Vec<Box<dyn Initializer<C>>>,
    mode: Mode,
    name: String,
    skip_on_missing_config: bool,
    root: Option<PathBuf>,
    config_builder: ConfigBuilder,
    module_map: HashSet<TypeId>,
    executor: AnyExecutor,
    plugins: PluginsList<C>,
}

impl<C: Context> Phase for Setup<C> {
    type Next = Build<C>;
    fn next(mut self) -> impl Future<Output = Result<Self::Next, Error>> {
        async move {
            let mut config = Config::default();
            let mut extensions = Extensions::default();
            let mut modules = Vec::default();

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
                    extra_modules: &mut modules,
                    module_map: &mut self.module_map,
                    plugins: &mut self.plugins,
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

            let mut extra_modules: VecDeque<Box<dyn DynamicModule<C>>> = VecDeque::default();

            loop {
                let Some(module) = modules.pop() else {
                    break;
                };

                #[cfg(feature = "cli")]
                let mut cmd = None;
                debug!(module = ?module.config_section(), "Setup module");
                module.setup(SetupCtx {
                    ctx: &mut self.ctx,
                    module_name: module.config_section(),
                    #[cfg(feature = "cli")]
                    cmds: &mut cmd,
                    extensions: &mut extensions,
                    extra_modules: &mut modules,
                    module_map: &mut self.module_map,
                    plugins: &mut self.plugins,
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

                extra_modules.push_front(module);
            }

            extra_modules.extend(self.modules);

            self.config_builder
                .add_configure(Box::new(move |cfg: &mut Config| {
                    cfg.extend(config);
                    Ok(())
                }));

            Ok(Build {
                ctx: self.ctx,
                modules: Vec::from_iter(extra_modules),
                initializers: self.initializers,
                #[cfg(feature = "cli")]
                cmds,
                extensions,
                config: self.config_builder,
                mode: self.mode,
                name: self.name,
                skip_on_missing_config: self.skip_on_missing_config,
                root: self.root,
                executor: self.executor,
                plugins: self.plugins,
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
    module_map: &'a mut HashSet<TypeId>,
    plugins: &'a mut PluginsList<C>,
    extra_modules: &'a mut Vec<Box<dyn DynamicModule<C>>>,
}

impl<'a, C: Context> SetupCtx<'a, C> {
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

    pub fn register<T: Send + Sync + 'static>(&mut self, value: T) -> &mut Self {
        self.extensions.insert(value);
        self
    }

    pub fn add_module<T: Module<C> + 'static>(&mut self) -> &mut Self {
        if !self.module_map.contains(&TypeId::of::<T>()) {
            self.extra_modules.push(box_module::<T, C>());
            self.module_map.insert(TypeId::of::<T>());
        }
        self
    }

    pub fn register_plugin<T>(&mut self, plugin: T) -> Result<&mut Self, Error>
    where
        T: 'static + Plugin<C> + Send + Sync,
        T::Output: Send + Sync + 'static,
        T::Error: 'static,
    {
        self.plugins.insert(plugin)?;

        Ok(self)
    }
}
