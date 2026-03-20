#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: usize,
    pub old_end: usize,
    pub new_start: usize,
    pub new_end: usize,
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
}
