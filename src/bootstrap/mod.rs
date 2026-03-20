use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use anyhow::Result;

use crate::infrastructure::config::default_config_dir;
use crate::infrastructure::fs_scan::FsDiffLoader;
use crate::infrastructure::syntax_assets::load_syntax_assets;
use crate::ports::DiffLoader;
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
        target_rx: spawn_target_build(cli.clone(), cfg),
    })
}

impl RuntimePlan {
    pub(crate) fn run(self) -> Result<()> {
        run_tui_with_loading(self.settings, self.painter, self.loader, self.target_rx)
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
