//! One-line shell activity, including background terminal waits.

use super::model::ExecCell;
use codex_protocol::parse_command::ParsedCommand;
use ratatui::prelude::*;
use std::collections::HashSet;

pub(super) fn summary_lines(cell: &ExecCell, width: u16) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    // Count each tool call once, with listing taking precedence over searching and
    // reading. A call containing general shell work is not an exploration call.
    let mut counts = [0usize; 4];
    let mut read_paths = HashSet::new();
    for call in &cell.calls {
        if call.parsed.is_empty()
            || call
                .parsed
                .iter()
                .any(|parsed| matches!(parsed, ParsedCommand::Unknown { .. }))
        {
            counts[3] += 1;
        } else if call
            .parsed
            .iter()
            .any(|parsed| matches!(parsed, ParsedCommand::ListFiles { .. }))
        {
            counts[2] += 1;
        } else if call
            .parsed
            .iter()
            .any(|parsed| matches!(parsed, ParsedCommand::Search { .. }))
        {
            counts[0] += 1;
        } else {
            for parsed in &call.parsed {
                if let ParsedCommand::Read { path, .. } = parsed {
                    read_paths.insert(path);
                }
            }
        }
    }
    counts[1] = read_paths.len();
    let active = cell.is_active();
    let mut spans = Vec::new();
    for (index, (completed, running, singular, plural)) in [
        ("searched for", "searching for", "pattern", "patterns"),
        ("read", "reading", "file", "files"),
        ("listed", "listing", "directory", "directories"),
        ("ran", "running", "shell command", "shell commands"),
    ]
    .into_iter()
    .enumerate()
    {
        let count = counts[index];
        if count == 0 {
            continue;
        }
        let mut verb = if active { running } else { completed }.to_string();
        if spans.is_empty() {
            verb[..1].make_ascii_uppercase();
        } else {
            spans.push(", ".into());
        }
        let noun = if count == 1 { singular } else { plural };
        spans.extend([
            format!("{verb} ").into(),
            count.to_string().bold(),
            format!(" {noun}").into(),
        ]);
    }
    let waiting = cell.waited_terminals.len();
    if waiting > 0 {
        let noun = if waiting == 1 {
            "terminal"
        } else {
            "terminals"
        };
        spans.extend([
            if spans.is_empty() {
                "Waiting "
            } else {
                ", waiting "
            }
            .into(),
            waiting.to_string().bold(),
            format!(" {noun}").into(),
        ]);
    }
    if active {
        spans.push("…".into());
    } else {
        for span in &mut spans {
            span.style = span.style.dim();
        }
    }
    spans.insert(
        0,
        if active {
            "● ".dim()
        } else if cell.calls.iter().any(|call| {
            call.output
                .as_ref()
                .is_some_and(|output| output.exit_code != 0)
        }) {
            "● ".red()
        } else {
            "● ".green()
        },
    );
    crate::tool_preview::summary(spans.into(), width)
}

#[cfg(test)]
#[path = "summary_tests.rs"]
mod tests;
