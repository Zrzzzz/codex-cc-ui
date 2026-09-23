//! Bounded conversation previews; history cells retain their expanded transcript.

use ratatui::style::Stylize;
use ratatui::text::Line;

pub(crate) fn summary(mut line: Line<'static>, width: u16) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    for span in &mut line.spans {
        if span.content.contains(['\n', '\r']) {
            span.content = span.content.replace(['\n', '\r'], " ").into();
        }
    }
    let hint = " (ctrl+t)";
    let width = usize::from(width);
    let hint_width = if width > hint.len() { hint.len() } else { 0 };
    let mut line =
        crate::line_truncation::truncate_line_with_ellipsis_if_overflow(line, width - hint_width);
    if hint_width > 0 {
        line.push_span(hint.dim());
    }
    vec![line]
}

#[cfg(test)]
#[path = "tool_preview_tests.rs"]
mod tests;
