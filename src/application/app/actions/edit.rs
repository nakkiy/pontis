use std::fs;
use std::process::Command;

use anyhow::{Context, Result};

use super::edit_support::{
    EditSide, apply_edited_text, can_edit_side, cleanup_edit_buffer, edit_side_binary,
    prepare_edit_buffer, resolve_editor_command,
};
use crate::app::App;
use crate::text::{DecodedKind, decode_bytes_for_diff};

impl App {
    pub(crate) fn edit_current_side_with_editor(&mut self, edit_left: bool) -> Result<()> {
        let side = if edit_left {
            EditSide::Left
        } else {
            EditSide::Right
        };

        if !can_edit_side(side, self.allow_left_write(), self.allow_right_write()) {
            self.set_temporary_status("target side is read-only: edit is disabled");
            return Ok(());
        }
        if self.files.is_empty() || self.current_file().is_none() {
            self.set_temporary_status("no visible file selected");
            return Ok(());
        }
        let editor = resolve_editor_command(std::env::var_os("EDITOR"))?;
        self.ensure_current_loaded();

        let idx = self.current_file;
        if edit_side_binary(&self.files[idx]) {
            self.set_temporary_status("binary file cannot be edited in inline flow");
            return Ok(());
        }

        let target_path = prepare_edit_buffer(side, &self.files[idx])?;

        let status = Command::new(editor.program())
            .args(editor.args())
            .arg(&target_path)
            .status()
            .with_context(|| format!("failed to launch editor `{}`", editor.display()))?;
        if !status.success() {
            cleanup_edit_buffer(&target_path);
            anyhow::bail!("editor exited with non-zero status: {status}");
        }

        let bytes = fs::read(&target_path)
            .with_context(|| format!("failed to read edited file {}", target_path.display()))?;
        cleanup_edit_buffer(&target_path);
        let decoded = decode_bytes_for_diff(bytes);
        if decoded.kind != DecodedKind::TextUtf8 {
            anyhow::bail!("edited file must be UTF-8 text");
        }
        apply_edited_text(
            side,
            &mut self.files[idx],
            decoded.text,
            decoded.has_utf8_bom,
        );
        self.clear_merge_history();
        self.refresh_current_file_after_text_change();
        self.set_temporary_status(match side {
            EditSide::Left => "edited left side with external editor",
            EditSide::Right => "edited right side with external editor",
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::super::edit_support::{EditorCommand, resolve_editor_command, split_editor_command};

    #[test]
    fn resolve_editor_rejects_missing_or_empty() {
        assert!(resolve_editor_command(None).is_err());
        assert!(resolve_editor_command(Some(OsString::from("   "))).is_err());
    }

    #[test]
    fn resolve_editor_accepts_value() {
        let editor = resolve_editor_command(Some(OsString::from("nvim"))).expect("editor");
        assert_eq!(
            editor,
            EditorCommand {
                program: "nvim".to_string(),
                args: Vec::new(),
            }
        );
    }

    #[test]
    fn resolve_editor_splits_arguments() {
        let editor =
            resolve_editor_command(Some(OsString::from("code -w --reuse-window"))).expect("editor");
        assert_eq!(
            editor,
            EditorCommand {
                program: "code".to_string(),
                args: vec!["-w".to_string(), "--reuse-window".to_string()],
            }
        );
    }

    #[test]
    fn resolve_editor_supports_quoted_arguments() {
        let editor = resolve_editor_command(Some(OsString::from(
            "env FOO='bar baz' nvim --cmd \"set number\"",
        )))
        .expect("editor");
        assert_eq!(
            editor,
            EditorCommand {
                program: "env".to_string(),
                args: vec![
                    "FOO=bar baz".to_string(),
                    "nvim".to_string(),
                    "--cmd".to_string(),
                    "set number".to_string(),
                ],
            }
        );
    }

    #[test]
    fn split_editor_command_rejects_unterminated_quote() {
        assert!(split_editor_command("nvim \"broken").is_err());
    }
}
