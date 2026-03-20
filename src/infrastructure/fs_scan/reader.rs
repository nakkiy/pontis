use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::text::{DecodedKind, decode_bytes_for_diff};

#[derive(Debug, Clone)]
pub(super) struct FileContent {
    pub(super) text: String,
    pub(super) is_binary: bool,
    pub(super) has_unsupported_encoding: bool,
    pub(super) has_utf8_bom: bool,
    pub(super) bytes: usize,
    pub(super) raw: Vec<u8>,
}

impl FileContent {
    pub(super) fn text(text: String) -> Self {
        Self {
            text,
            is_binary: false,
            has_unsupported_encoding: false,
            has_utf8_bom: false,
            bytes: 0,
            raw: Vec::new(),
        }
    }
}

pub(super) fn read_file_content(path: &Path) -> Result<FileContent> {
    let bytes = fs::read(path).with_context(|| format!("cannot read file: {}", path.display()))?;
    let decoded = decode_bytes_for_diff(bytes);
    Ok(FileContent {
        text: decoded.text,
        is_binary: decoded.kind != DecodedKind::TextUtf8,
        has_unsupported_encoding: decoded.kind == DecodedKind::UnsupportedEncoding,
        has_utf8_bom: decoded.has_utf8_bom,
        bytes: decoded.bytes,
        raw: decoded.raw,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::read_file_content;

    #[test]
    fn binary_file_is_detected() {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("pontis-binary-{uniq}.bin"));

        fs::write(&path, [0, 1, 2, 3]).expect("write test file");
        let content = read_file_content(&path).expect("read");
        fs::remove_file(&path).expect("cleanup");

        assert!(content.is_binary);
        assert_eq!(content.text, "[binary file]");
    }

    #[test]
    fn utf8_bom_file_is_loaded_as_utf8_text() {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("pontis-bom-{uniq}.txt"));
        fs::write(&path, b"\xEF\xBB\xBFabc\n").expect("write test file");
        let content = read_file_content(&path).expect("read");
        fs::remove_file(&path).expect("cleanup");

        assert!(!content.is_binary);
        assert!(content.has_utf8_bom);
        assert_eq!(content.text, "abc\n");
    }

    #[test]
    fn non_utf8_text_is_marked_as_unsupported() {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("pontis-enc-{uniq}.txt"));
        fs::write(&path, [0xFF, 0xFE, 0x41]).expect("write test file");
        let content = read_file_content(&path).expect("read");
        fs::remove_file(&path).expect("cleanup");

        assert!(content.is_binary);
        assert!(content.has_unsupported_encoding);
        assert_eq!(content.text, "[unsupported text encoding]");
    }
}
