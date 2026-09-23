//! Retain consecutive completed tools beside the live tool, then commit at a conversation boundary.

use super::*;

impl ChatWidget {
    pub(super) fn retain_active_tool_cell(&mut self) {
        if self
            .transcript
            .active_cell
            .as_ref()
            .is_some_and(|cell| history_cell::is_tool_activity(cell.as_ref()))
        {
            if let Some(cell) = self.transcript.take_active_cell() {
                self.transcript.pending_tools.push(cell.into());
                self.transcript.needs_final_message_separator = true;
                self.bump_active_cell_revision();
            }
        } else if self.transcript.active_cell.is_some() {
            self.flush_active_cell();
        }
    }

    pub(super) fn flush_tool_activity(&mut self) {
        if !self.transcript.pending_tools.is_empty() {
            let group = std::mem::take(&mut self.transcript.pending_tools);
            self.app_event_tx
                .send(AppEvent::InsertHistoryCell(Box::new(group)));
            self.bump_active_cell_revision();
            self.request_pending_usage_output_insertion();
        }
    }

    pub(super) fn flush_active_cell(&mut self) {
        if self.transcript.active_cell.is_none() && self.transcript.pending_tools.is_empty() {
            return;
        }
        if let Some(active) = self.transcript.take_active_cell() {
            self.transcript.needs_final_message_separator = true;
            if history_cell::is_tool_activity(active.as_ref()) {
                self.transcript.pending_tools.push(active.into());
            } else {
                self.flush_tool_activity();
                self.app_event_tx.send(AppEvent::InsertHistoryCell(active));
            }
        }
        self.flush_tool_activity();
        self.request_pending_usage_output_insertion();
    }
}

pub(super) struct ToolActivityPreview<'a> {
    pub(super) pending: &'a history_cell::ToolActivityCell,
    pub(super) active: Option<&'a dyn HistoryCell>,
    pub(super) right: u16,
}

impl Renderable for ToolActivityPreview<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let area = Rect::new(
            area.x,
            area.y.saturating_add(/*rhs*/ 1),
            area.width.saturating_sub(self.right),
            area.height.saturating_sub(/*rhs*/ 1),
        );
        let lines = self.pending.preview_lines(self.active, area.width);
        ratatui::widgets::Clear.render(area, buf);
        Widget::render(Paragraph::new(lines), area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        u16::from(width > self.right) + 1
    }
}
