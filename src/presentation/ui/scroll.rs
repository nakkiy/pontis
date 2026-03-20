pub(super) const HORIZONTAL_SCROLL_MARGIN: usize = 3;

pub(super) fn scroll_limit_with_margin(
    content_len: usize,
    viewport_len: usize,
    margin: usize,
) -> u16 {
    let overflow = content_len.saturating_sub(viewport_len);
    if overflow == 0 {
        0
    } else {
        overflow.saturating_add(margin).min(u16::MAX as usize) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::{HORIZONTAL_SCROLL_MARGIN, scroll_limit_with_margin};

    #[test]
    fn returns_zero_when_content_fits() {
        assert_eq!(scroll_limit_with_margin(5, 5, HORIZONTAL_SCROLL_MARGIN), 0);
        assert_eq!(scroll_limit_with_margin(4, 10, HORIZONTAL_SCROLL_MARGIN), 0);
    }

    #[test]
    fn adds_margin_when_content_overflows() {
        assert_eq!(scroll_limit_with_margin(14, 9, HORIZONTAL_SCROLL_MARGIN), 8);
        assert_eq!(scroll_limit_with_margin(5, 2, 1), 4);
    }
}
