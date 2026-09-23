use crate::exec_cell::CommandOutput;
use crate::exec_cell::new_active_exec_command;
use crate::history_cell::HistoryCell;
use codex_app_server_protocol::CommandExecutionSource;
use codex_protocol::parse_command::ParsedCommand;
use pretty_assertions::assert_eq;
use std::time::Duration;

#[test]
fn mixed_activity_summary_tracks_categories_and_running_state() {
    let mut cell = crate::exec_cell::ExecCell::new_wait(/*animations_enabled*/ false);
    let calls = [
        vec![
            ParsedCommand::Search {
                cmd: "rg first".into(),
                query: Some("first".into()),
                path: None,
            },
            ParsedCommand::Search {
                cmd: "rg second".into(),
                query: Some("second".into()),
                path: Some("src".into()),
            },
        ],
        vec![ParsedCommand::Read {
            cmd: "cat README.md".into(),
            name: "README.md".into(),
            path: "README.md".into(),
        }],
        vec![
            ParsedCommand::ListFiles {
                cmd: "ls src".into(),
                path: Some("src".into()),
            },
            ParsedCommand::Unknown {
                cmd: "just check".into(),
            },
            ParsedCommand::Unknown {
                cmd: "just build".into(),
            },
        ],
        Vec::new(),
        vec![ParsedCommand::Read {
            cmd: "head README.md".into(),
            name: "README.md".into(),
            path: "README.md".into(),
        }],
        vec![
            ParsedCommand::ListFiles {
                cmd: "ls src".into(),
                path: Some("src".into()),
            },
            ParsedCommand::Search {
                cmd: "rg first".into(),
                query: Some("first".into()),
                path: None,
            },
            ParsedCommand::Read {
                cmd: "cat other".into(),
                name: "other".into(),
                path: "other".into(),
            },
        ],
    ];
    for (index, parsed) in calls.into_iter().enumerate() {
        cell.add_call(
            index.to_string(),
            vec![format!("command-{index}")],
            parsed,
            CommandExecutionSource::Agent,
            /*interaction_input*/ None,
        );
    }
    cell.waited_terminals
        .insert("terminal".into(), Some("just test".into()));
    let mut previews = Vec::new();
    previews.push(cell.display_lines(/*width*/ 120));
    for id in ["0", "1", "2", "4", "5"] {
        cell.complete_call(
            id,
            CommandOutput::new(/*exit_code*/ 0, "output".into()),
            Duration::from_secs(1),
        );
    }
    previews.push(cell.display_lines(/*width*/ 120));
    cell.complete_call(
        "3",
        CommandOutput::new(/*exit_code*/ 1, "failed".into()),
        Duration::from_secs(1),
    );
    let transcript = cell.transcript_lines(/*width*/ 120);
    let header = &cell.display_lines(/*width*/ 120)[0];
    assert_eq!(
        header.to_string(),
        "● Searched for 1 pattern, read 1 file, listed 1 directory, ran 2 shell commands, waiting 1 terminal (ctrl+t)"
    );
    let styled = header
        .spans
        .iter()
        .skip(1)
        .filter(|span| !span.content.is_empty())
        .map(|span| (span.content.to_string(), span.style.add_modifier))
        .collect::<Vec<_>>();
    assert!(
        styled
            .iter()
            .all(|(_, modifiers)| modifiers.contains(ratatui::style::Modifier::DIM))
    );
    assert_eq!(
        styled
            .iter()
            .filter(|(_, modifiers)| modifiers.contains(ratatui::style::Modifier::BOLD))
            .map(|(text, _)| text.as_str())
            .collect::<Vec<_>>(),
        vec!["1", "1", "1", "2", "1"]
    );
    for width in [0, 1, 20, 60, 120] {
        let lines = cell.display_lines(width);
        assert!(lines.iter().all(|line| line.width() <= usize::from(width)));
        previews.push(lines);
    }
    assert_eq!(cell.transcript_lines(/*width*/ 120), transcript);
    assert_eq!(
        cell.display_lines(/*width*/ 120)[0].spans[0].style.fg,
        Some(ratatui::style::Color::Red)
    );
    insta::assert_debug_snapshot!(
        previews
            .iter()
            .map(|lines| lines.iter().map(ToString::to_string).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
}

#[test]
fn loaded_file_tree_groups_retries_and_bounds_long_paths() {
    let mut cell = new_active_exec_command(
        "setup".into(),
        vec!["pwd".into()],
        Vec::new(),
        CommandExecutionSource::Agent,
        /*interaction_input*/ None,
        /*animations_enabled*/ false,
    );
    cell.complete_call(
        "setup",
        CommandOutput::new(/*exit_code*/ 0, String::new()),
        Duration::from_secs(1),
    );
    let paths = [
        "CLAUDE.md",
        "docs/TASKS.md",
        "src/arch_ppt_agent/CLAUDE.md",
        "docs/TASKS.md",
    ];
    for (index, path) in paths.iter().enumerate() {
        let id = index.to_string();
        let command = format!("cat {path}");
        cell.add_call(
            id.clone(),
            vec![command.clone()],
            vec![ParsedCommand::Read {
                cmd: command,
                name: path.to_string(),
                path: std::path::PathBuf::from(*path),
            }],
            CommandExecutionSource::Agent,
            /*interaction_input*/ None,
        );
        cell.complete_call(
            &id,
            CommandOutput::new(/*exit_code*/ 0, "contents".into()),
            Duration::from_secs(1),
        );
    }
    insta::assert_snapshot!(
        cell.display_lines(/*width*/ 80)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    for index in 0..10 {
        let id = format!("extra-{index}");
        let path = format!("{}/file-{index}.md", "long-directory/".repeat(20));
        let command = format!("cat {path}");
        cell.add_call(
            id.clone(),
            vec![command.clone()],
            vec![ParsedCommand::Read {
                cmd: command,
                name: path.clone(),
                path: path.into(),
            }],
            CommandExecutionSource::Agent,
            /*interaction_input*/ None,
        );
        let exit_code = if index == 0 { 1 } else { 0 };
        cell.complete_call(
            &id,
            CommandOutput::new(exit_code, "contents".into()),
            Duration::from_secs(1),
        );
        assert!(!cell.should_flush());
    }
    let transcript = cell.transcript_lines(/*width*/ 80);
    for width in [0, 1, 4, 10, 40, 80] {
        let preview = cell.display_lines(width);
        assert!(
            preview
                .iter()
                .all(|line| line.width() <= usize::from(width))
        );
        assert!(preview.len() <= 1);
    }
    assert_eq!(cell.transcript_lines(/*width*/ 80), transcript);
}
