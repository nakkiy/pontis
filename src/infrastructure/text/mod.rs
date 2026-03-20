pub(crate) const BINARY_PLACEHOLDER: &str = "[binary file]";
pub(crate) const UNSUPPORTED_ENCODING_PLACEHOLDER: &str = "[unsupported text encoding]";
const UTF8_BOM: &[u8; 3] = b"\xEF\xBB\xBF";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecodedKind {
    TextUtf8,
    Binary,
    UnsupportedEncoding,
}

#[derive(Debug, Clone)]
pub(crate) struct DecodedText {
    pub(crate) text: String,
    pub(crate) kind: DecodedKind,
    pub(crate) bytes: usize,
    pub(crate) raw: Vec<u8>,
    pub(crate) has_utf8_bom: bool,
}

pub(crate) fn decode_bytes_for_diff(bytes: Vec<u8>) -> DecodedText {
    if bytes.contains(&0) {
        return DecodedText {
            text: BINARY_PLACEHOLDER.to_string(),
            kind: DecodedKind::Binary,
            bytes: bytes.len(),
            raw: bytes,
            has_utf8_bom: false,
        };
    }

    let (has_utf8_bom, body) = if bytes.starts_with(UTF8_BOM) {
        (true, &bytes[UTF8_BOM.len()..])
    } else {
        (false, bytes.as_slice())
    };

    match std::str::from_utf8(body) {
        Ok(text) => DecodedText {
            text: text.to_string(),
            kind: DecodedKind::TextUtf8,
            bytes: bytes.len(),
            raw: bytes,
            has_utf8_bom,
        },
        Err(_) => DecodedText {
            text: UNSUPPORTED_ENCODING_PLACEHOLDER.to_string(),
            kind: DecodedKind::UnsupportedEncoding,
            bytes: bytes.len(),
            raw: bytes,
            has_utf8_bom: false,
        },
    }
}

pub(crate) fn encode_utf8_for_write(content: &str, with_bom: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(content.len() + if with_bom { UTF8_BOM.len() } else { 0 });
    if with_bom {
        out.extend_from_slice(UTF8_BOM);
    }
    out.extend_from_slice(content.as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::{DecodedKind, decode_bytes_for_diff, encode_utf8_for_write};

    #[test]
    fn decode_strips_utf8_bom_and_marks_flag() {
        let decoded = decode_bytes_for_diff(b"\xEF\xBB\xBFhello\n".to_vec());
        assert_eq!(decoded.kind, DecodedKind::TextUtf8);
        assert_eq!(decoded.text, "hello\n");
        assert!(decoded.has_utf8_bom);
    }

    #[test]
    fn decode_marks_invalid_utf8_as_unsupported_encoding() {
        let decoded = decode_bytes_for_diff(vec![0xFF, 0xFE, 0x41]);
        assert_eq!(decoded.kind, DecodedKind::UnsupportedEncoding);
        assert!(!decoded.has_utf8_bom);
    }

    #[test]
    fn encode_utf8_for_write_respects_bom_flag() {
        assert_eq!(encode_utf8_for_write("abc", false), b"abc".to_vec());
        assert_eq!(
            encode_utf8_for_write("abc", true),
            b"\xEF\xBB\xBFabc".to_vec()
        );
    }
}
