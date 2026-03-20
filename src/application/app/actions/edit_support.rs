use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::model::DiffFile;
use crate::text::encode_utf8_for_write;

#[derive(Clone, Copy)]
pub(super) enum EditSide {
    Left,
    Right,
}

pub(super) fn can_edit_side(
    side: EditSide,
    allow_left_write: bool,
    allow_right_write: bool,
) -> bool {
    match side {
        EditSide::Left => allow_left_write,
        EditSide::Right => allow_right_write,
    }
}

pub(super) fn edit_side_binary(file: &DiffFile) -> bool {
    file.is_binary
}

pub(super) fn prepare_edit_buffer(side: EditSide, file: &DiffFile) -> Result<PathBuf> {
    let seed_text = match side {
        EditSide::Left => file.left_text.clone(),
        EditSide::Right => file.right_text.clone(),
    };
    let seed_has_utf8_bom = match side {
        EditSide::Left => file.left_has_utf8_bom,
        EditSide::Right => file.right_has_utf8_bom,
    };
    let temp_path = unique_edit_buffer_path(side, file);
    let bytes = encode_utf8_for_write(&seed_text, seed_has_utf8_bom);
    fs::write(&temp_path, bytes)
        .with_context(|| format!("failed to write edit buffer {}", temp_path.display()))?;
    Ok(temp_path)
}

pub(super) fn cleanup_edit_buffer(path: &Path) {
    let _ = fs::remove_file(path);
}

pub(super) fn apply_edited_text(
    side: EditSide,
    file: &mut DiffFile,
    text: String,
    has_utf8_bom: bool,
) {
    match side {
        EditSide::Left => {
            file.left_text = text;
            file.left_has_utf8_bom = has_utf8_bom;
            file.left_dirty = true;
        }
        EditSide::Right => {
            file.right_text = text;
            file.right_has_utf8_bom = has_utf8_bom;
            file.right_dirty = true;
        }
    }
}

fn unique_edit_buffer_path(side: EditSide, file: &DiffFile) -> PathBuf {
    let uniq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let side_name = match side {
        EditSide::Left => "left",
        EditSide::Right => "right",
    };
    let file_name = file
        .rel_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("pontis-edit.txt");
    std::env::temp_dir().join(format!("pontis-{side_name}-{uniq}-{file_name}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EditorCommand {
    pub(super) program: String,
    pub(super) args: Vec<String>,
}

impl EditorCommand {
    pub(super) fn display(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }

    pub(super) fn program(&self) -> &str {
        self.program.as_str()
    }

    pub(super) fn args(&self) -> &[String] {
        self.args.as_slice()
    }
}

pub(super) fn resolve_editor_command(editor: Option<OsString>) -> Result<EditorCommand> {
    let Some(raw) = editor else {
        anyhow::bail!("EDITOR is not set; set EDITOR to use external edit");
    };
    let s = raw.to_string_lossy().trim().to_string();
    if s.is_empty() {
        anyhow::bail!("EDITOR is empty; set EDITOR to use external edit");
    }
    let parts = split_editor_command(&s).context("failed to parse EDITOR command")?;
    let Some((program, args)) = parts.split_first() else {
        anyhow::bail!("EDITOR is empty; set EDITOR to use external edit");
    };
    Ok(EditorCommand {
        program: program.clone(),
        args: args.to_vec(),
    })
}

pub(super) fn split_editor_command(input: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(ch) = chars.next() {
        match quote {
            Some(active) if ch == active => quote = None,
            Some(_) if ch == '\\' => {
                let next = chars
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("unterminated escape in EDITOR"))?;
                current.push(next);
            }
            Some(_) => current.push(ch),
            None if ch == '\'' || ch == '"' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            None if ch == '\\' => {
                let next = chars
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("unterminated escape in EDITOR"))?;
                current.push(next);
            }
            None => current.push(ch),
        }
    }

    if quote.is_some() {
        anyhow::bail!("unterminated quote in EDITOR");
    }
    if !current.is_empty() {
        parts.push(current);
    }
    Ok(parts)
}
