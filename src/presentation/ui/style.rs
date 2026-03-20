use ratatui::style::Color;

pub(super) const HEADER_FG: Color = Color::White;
pub(super) const STATUS_FG: Color = Color::Gray;
pub(super) const FOCUS_BORDER_FG: Color = Color::Rgb(230, 196, 64);
pub(super) const SELECTED_BG: Color = Color::DarkGray;
pub(super) const SELECTED_FG: Color = Color::White;
pub(super) const LINE_NO_FG: Color = Color::DarkGray;
pub(super) const LINE_ENDING_CR_FG: Color = Color::DarkGray;
pub(super) const LINE_ENDING_LF_FG: Color = Color::DarkGray;
pub(super) const LINE_ENDING_CRLF_FG: Color = Color::DarkGray;
pub(super) const DIFF_BG_CHANGED: Color = Color::Rgb(55, 38, 10);
pub(super) const DIFF_BG_CURRENT: Color = Color::Rgb(18, 74, 42);
pub(super) const INLINE_DIFF_BG_CHANGED: Color = Color::Rgb(95, 58, 14);
pub(super) const INLINE_DIFF_BG_CURRENT: Color = Color::Rgb(30, 112, 64);
