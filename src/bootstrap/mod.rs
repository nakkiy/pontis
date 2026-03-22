use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use anyhow::Result;

use crate::infrastructure::config::default_config_dir;
use crate::infrastructure::fs_scan::FsDiffLoader;
use crate::infrastructure::syntax_assets::load_syntax_assets;
use crate::ports::{ComparisonReloader, DiffLoader, ReloadedTargets};
use crate::presentation::driver::run_tui_with_loading;
use crate::settings::AppSettings;
use crate::syntax::SyntaxPainter;

mod cli;
mod config;
mod targets;
#[cfg(test)]
mod tests;

pub use cli::Cli;
pub(crate) use cli::GitCommand;

pub(crate) struct RuntimePlan {
    settings: AppSettings,
    painter: SyntaxPainter,
    loader: Arc<dyn DiffLoader>,
    reloader: Option<Arc<dyn ComparisonReloader>>,
    target_rx: Receiver<TargetBuildResult>,
}

type TargetBuildResult = Result<TargetBuildOutput, String>;
type TargetBuildOutput = (Vec<crate::model::DiffFile>, crate::model::Roots, bool, bool);

pub(crate) fn prepare_runtime(cli: &Cli) -> Result<RuntimePlan> {
    let cfg = config::resolve_runtime_config(cli);
    let config_dir = default_config_dir();
    let assets = load_syntax_assets(config_dir.as_deref());
    let painter = SyntaxPainter::new(&cfg, assets.ps, assets.ts);
    Ok(RuntimePlan {
        settings: cfg.clone(),
        painter,
        loader: Arc::new(FsDiffLoader),
        reloader: build_reloader(cli),
        target_rx: spawn_target_build(cli.clone(), cfg),
    })
}

impl RuntimePlan {
    pub(crate) fn run(self) -> Result<()> {
        run_tui_with_loading(
            self.settings,
            self.painter,
            self.loader,
            self.reloader,
            self.target_rx,
        )
    }
}

pub fn run(cli: &Cli) -> Result<()> {
    prepare_runtime(cli)?.run()
}

fn spawn_target_build(cli: Cli, cfg: AppSettings) -> Receiver<TargetBuildResult> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = targets::build_comparison_targets(&cli, &cfg).map_err(|err| err.to_string());
        let _ = tx.send(result);
    });
    rx
}

pub(super) fn reload_supported(cli: &Cli) -> bool {
    match &cli.command {
        None => true,
        Some(cli::Commands::Git(git)) => !git.staged && git.diff.is_none(),
    }
}

fn build_reloader(cli: &Cli) -> Option<Arc<dyn ComparisonReloader>> {
    if !reload_supported(cli) {
        return None;
    }
    Some(Arc::new(BootstrapReloader { cli: cli.clone() }))
}

#[derive(Debug, Clone)]
struct BootstrapReloader {
    cli: Cli,
}

impl ComparisonReloader for BootstrapReloader {
    fn reload_targets(&self, cfg: &AppSettings) -> Result<ReloadedTargets> {
        targets::build_comparison_targets(&self.cli, cfg)
    }
}
