//! Bounded conversation previews; history cells retain their expanded transcript.

use crate::wrapping::RtOptions;
use crate::wrapping::word_wrap_lines;
use ratatui::style::Stylize;
use ratatui::text::Line;

pub(crate) fn summary(mut line: Line<'static>, width: u16) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    line.push_span(" (ctrl+t)".dim());
    word_wrap_lines([line], RtOptions::new(usize::from(width)))
}

pub(crate) fn collapse(lines: Vec<Line<'static>>, width: u16, limit: usize) -> Vec<Line<'static>> {
    if width == 0 || limit == 0 {
        return Vec::new();
    }
    let wrapped = if lines.iter().all(|line| line.width() <= usize::from(width)) {
        lines
    } else {
        word_wrap_lines(&lines, RtOptions::new(usize::from(width)))
    };
    if wrapped.len() <= limit {
        return wrapped;
    }
    let hint = if width >= 28 {
        "  └ … (ctrl+t to expand)"
    } else if width >= 12 {
        "… (ctrl+t)"
    } else {
        "…"
    };
    let mut preview = wrapped;
    preview.truncate(limit.saturating_sub(1));
    preview.push(Line::from(hint.dim()));
    preview
}

#[cfg(test)]
#[path = "tool_preview_tests.rs"]
mod tests;
