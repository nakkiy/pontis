use std::path::Path;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::settings::AppSettings;

use super::theme::pick_theme_name;

pub(crate) struct SyntaxPainter {
    ps: SyntaxSet,
    ts: ThemeSet,
    theme: String,
}

impl SyntaxPainter {
    pub(crate) fn new(cfg: &AppSettings, ps: SyntaxSet, ts: ThemeSet) -> Self {
        let theme = resolve_theme_name(cfg, &ts);
        Self { ps, ts, theme }
    }

    pub(crate) fn highlight(&self, path: &Path, lines: &[String]) -> Vec<Line<'static>> {
        let syntax = self
            .ps
            .find_syntax_for_file(path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());

        let theme = match self.ts.themes.get(&self.theme) {
            Some(t) => t,
            None => return lines.iter().map(|l| Line::from(l.clone())).collect(),
        };

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut out = Vec::with_capacity(lines.len());

        for line in lines {
            let src = format!("{line}\n");
            let regions = match highlighter.highlight_line(&src, &self.ps) {
                Ok(v) => v,
                Err(_) => {
                    out.push(Line::from(line.clone()));
                    continue;
                }
            };

            let spans = regions
                .into_iter()
                .map(|(style, text)| Span::styled(text.to_string(), to_ratatui_style(style)))
                .collect::<Vec<_>>();
            out.push(Line::from(spans));
        }

        out
    }
}

fn resolve_theme_name(cfg: &AppSettings, ts: &ThemeSet) -> String {
    if !cfg.theme.is_empty() && ts.themes.contains_key(&cfg.theme) {
        return cfg.theme.clone();
    }
    pick_theme_name(ts)
}

fn to_ratatui_style(style: SynStyle) -> Style {
    Style::default().fg(Color::Rgb(
        style.foreground.r,
        style.foreground.g,
        style.foreground.b,
    ))
}
