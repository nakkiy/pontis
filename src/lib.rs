mod application;
pub mod bootstrap;
mod domain;
mod infrastructure;
mod presentation;

pub use application::settings;
pub use domain::{diff, model};
pub use infrastructure::{fs_scan, git_scan};

pub(crate) use application::{app, ports};
pub(crate) use infrastructure::text;
pub(crate) use presentation::{syntax, ui};
