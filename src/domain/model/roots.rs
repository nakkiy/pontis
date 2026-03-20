use std::path::PathBuf;

use super::Mode;

#[derive(Debug, Clone)]
pub struct Roots {
    pub left: PathBuf,
    pub right: PathBuf,
    pub mode: Mode,
    pub left_label: Option<String>,
    pub right_label: Option<String>,
}
