//! Compact tool records and consecutive activity groups retain full transcript cells.

use super::*;
use crate::exec_cell::ExecCell;
use std::sync::Arc;

/// A tool result whose detailed rendering already exists as an ordinary history cell.
#[derive(Debug)]
pub(crate) struct ToolResultCell {
    inner: PlainHistoryCell,
    pub(crate) call_id: Option<String>,
    pub(crate) failure_count: usize,
}

impl ToolResultCell {
    pub(crate) fn new(inner: PlainHistoryCell) -> Self {
        Self {
            inner,
            call_id: None,
            failure_count: 0,
        }
    }
}

impl HistoryCell for ToolResultCell {
    fn tool_failure_count(&self) -> usize {
        self.failure_count
    }

    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let line = self.inner.lines.first().cloned().unwrap_or_default();
        crate::tool_preview::summary(
            if self.failure_count > 0 {
                line.red()
            } else {
                line
            },
            width,
        )
    }

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        self.inner.transcript_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        self.inner.raw_lines()
    }
}

/// Identify tool output by type rather than matching translated display text.
pub(crate) fn is_tool_activity(cell: &dyn HistoryCell) -> bool {
    if let Some(exec) = cell.as_any().downcast_ref::<ExecCell>() {
        return !exec.has_user_shell_command();
    }
    cell.as_any().is::<McpToolCallCell>()
        || cell.as_any().is::<WebSearchCell>()
        || cell.as_any().is::<PatchHistoryCell>()
        || cell.as_any().is::<PlanUpdateCell>()
        || cell.as_any().is::<RequestUserInputResultCell>()
        || cell.as_any().is::<UnifiedExecInteractionCell>()
        || cell.as_any().is::<ToolResultCell>()
        || cell.as_any().is::<super::mcp::McpImageOutputCell>()
}

/// Adjacent completed tool records share one viewport row, with details in original order.
#[derive(Debug, Default)]
pub(crate) struct ToolActivityCell {
    cells: Vec<Arc<dyn HistoryCell>>,
    failures: usize,
    counts: [usize; 8],
}

impl ToolActivityCell {
    pub(crate) fn push(&mut self, cell: Arc<dyn HistoryCell>) {
        self.record(cell.as_ref());
        self.cells.push(cell);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub(crate) fn preview_lines(
        &self,
        active: Option<&dyn HistoryCell>,
        width: u16,
    ) -> Vec<Line<'static>> {
        if let Some(active) = active {
            let mut summary = Self {
                counts: self.counts,
                failures: self.failures,
                ..Self::default()
            };
            summary.record(active);
            summary.summary_lines(width)
        } else {
            self.display_lines(width)
        }
    }

    fn record(&mut self, cell: &dyn HistoryCell) {
        self.failures += cell.tool_failure_count();
        if let Some(exec) = cell.as_any().downcast_ref::<ExecCell>() {
            self.counts[0] += exec.calls.len();
            self.counts[1] += exec.waited_terminals.len();
        } else {
            let index = if cell.as_any().is::<UnifiedExecInteractionCell>() {
                1
            } else if cell.as_any().is::<McpToolCallCell>() {
                2
            } else if cell.as_any().is::<WebSearchCell>() {
                3
            } else if cell.as_any().is::<PatchHistoryCell>() {
                4
            } else if cell.as_any().is::<PlanUpdateCell>() {
                5
            } else if cell.as_any().is::<RequestUserInputResultCell>() {
                6
            } else {
                7
            };
            self.counts[index] += 1;
        }
    }
}

impl ToolActivityCell {
    fn summary_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut summary = vec![
            if self.failures > 0 {
                "● ".red()
            } else {
                "● ".dim()
            },
            "Tool activity".bold(),
        ];
        if self.failures > 0 {
            summary.push(format!(" · {} failed", self.failures).red());
        }
        for (count, (singular, plural)) in self.counts.iter().zip([
            ("command", "commands"),
            ("terminal interaction", "terminal interactions"),
            ("MCP call", "MCP calls"),
            ("web call", "web calls"),
            ("patch", "patches"),
            ("plan update", "plan updates"),
            ("question", "questions"),
            ("other event", "other events"),
        ]) {
            if *count > 0 {
                let label = if *count == 1 { singular } else { plural };
                summary.extend([
                    " · ".dim(),
                    count.to_string().bold(),
                    format!(" {label}").dim(),
                ]);
            }
        }
        crate::tool_preview::summary(summary.into(), width)
    }
}

impl HistoryCell for ToolActivityCell {
    fn tool_failure_count(&self) -> usize {
        self.failures
    }

    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        if self.cells.len() == 1 {
            return self.cells[0].display_lines(width);
        }
        self.summary_lines(width)
    }

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        visible_lines(self.transcript_hyperlink_lines(width))
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut lines = Vec::new();
        for cell in &self.cells {
            if !lines.is_empty() {
                lines.push(HyperlinkLine::from(""));
            }
            lines.extend(cell.transcript_hyperlink_lines(width));
        }
        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        self.cells
            .iter()
            .flat_map(|cell| cell.raw_lines())
            .collect()
    }
}

#[cfg(test)]
#[path = "tool_activity_tests.rs"]
mod tests;
