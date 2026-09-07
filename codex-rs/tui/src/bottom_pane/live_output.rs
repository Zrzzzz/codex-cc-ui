//! Keeps output counts on the running status row across streaming reflow.

use super::BottomPane;

impl BottomPane {
    pub(crate) fn set_live_output_tokens(&mut self, label: Option<String>) {
        let label = self.is_task_running.then_some(label).flatten();
        if self.live_output_tokens == label {
            return;
        }
        self.live_output_tokens = label;
        if self.live_output_tokens.is_some() {
            self.ensure_status_indicator();
        }
        self.sync_status_inline_message();
        self.request_redraw();
    }
}
