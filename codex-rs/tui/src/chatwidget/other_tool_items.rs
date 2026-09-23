//! Preserve dynamic, clock, and remaining collaboration calls in the tool transcript.

use super::*;

impl ChatWidget {
    pub(super) fn on_other_tool_item(&mut self, item: ThreadItem) {
        let in_progress = matches!(
            &item,
            ThreadItem::DynamicToolCall {
                status: codex_app_server_protocol::DynamicToolCallStatus::InProgress,
                ..
            }
        );
        let header = match &item {
            ThreadItem::DynamicToolCall {
                namespace,
                tool,
                status,
                ..
            } => {
                let name = namespace
                    .as_ref()
                    .map(|ns| format!("{ns}.{tool}"))
                    .unwrap_or_else(|| tool.clone());
                let status = match status {
                    codex_app_server_protocol::DynamicToolCallStatus::InProgress => "running",
                    codex_app_server_protocol::DynamicToolCallStatus::Completed => "completed",
                    codex_app_server_protocol::DynamicToolCallStatus::Failed => "failed",
                };
                format!("Tool {name} · {status}")
            }
            ThreadItem::CollabAgentToolCall { tool, status, .. } => {
                format!("Agent tool {tool:?} · {status:?}")
            }
            ThreadItem::Sleep(_) => "Waited".to_string(),
            _ => return,
        };
        let mut lines = vec![Line::from(vec!["● ".dim(), header.bold()])];
        if let Ok(details) = serde_json::to_string_pretty(&item) {
            lines.extend(details.lines().map(|line| Line::from(line.to_string())));
        }
        self.flush_answer_stream_with_separator();
        let mut cell = history_cell::ToolResultCell::new(PlainHistoryCell::new(lines));
        cell.call_id = Some(item.id().to_string());
        cell.failure_count = usize::from(matches!(
            &item,
            ThreadItem::DynamicToolCall {
                status: codex_app_server_protocol::DynamicToolCallStatus::Failed,
                ..
            } | ThreadItem::DynamicToolCall {
                success: Some(false),
                ..
            } | ThreadItem::CollabAgentToolCall {
                status: CollabAgentToolCallStatus::Failed | CollabAgentToolCallStatus::Interrupted,
                ..
            }
        ));
        let replaces_active = self
            .transcript
            .active_cell
            .as_ref()
            .and_then(|active| {
                active
                    .as_any()
                    .downcast_ref::<history_cell::ToolResultCell>()
            })
            .is_some_and(|active| active.call_id == cell.call_id);
        if in_progress || replaces_active {
            if !replaces_active {
                self.retain_active_tool_cell();
            }
            self.transcript.active_cell = Some(Box::new(cell));
            self.bump_active_cell_revision();
            if !in_progress {
                self.retain_active_tool_cell();
            }
        } else {
            self.add_to_history(cell);
        }
        self.request_redraw();
    }
}
