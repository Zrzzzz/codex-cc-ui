//! Current thread name centered in the right half of the composer's top rule.

use super::ChatComposer;
use crate::bottom_pane::BottomPane;
use crate::line_truncation::truncate_line_with_ellipsis_if_overflow;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Widget;

impl BottomPane {
    pub(crate) fn set_composer_thread_name(&mut self, name: Option<String>) {
        if self.composer.thread_name != name {
            self.composer.thread_name = name;
            self.request_redraw();
        }
    }
}

pub(super) fn render(composer: &ChatComposer, area: Rect, buf: &mut Buffer) {
    let Some(name) = &composer.thread_name else {
        return;
    };
    let half_width = area.width / 2;
    if area.height == 0 || half_width < 5 {
        return;
    }
    let title = truncate_line_with_ellipsis_if_overflow(
        Line::from(name.split_whitespace().collect::<Vec<_>>().join(" ")),
        usize::from(half_width - 4),
    );
    let mut spans = vec![" ".into()];
    spans.extend(title.spans);
    spans.push(" ".into());
    let line = Line::from(spans).dim();
    let width = line.width() as u16;
    let x = area.right() - half_width + (half_width - width) / 2;
    line.render(Rect::new(x, area.y, width, 1), buf);
}

#[cfg(test)]
#[path = "chat_composer_thread_title_tests.rs"]
mod tests;
