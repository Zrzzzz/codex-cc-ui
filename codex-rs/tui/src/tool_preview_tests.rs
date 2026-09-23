use super::*;
use crate::history_cell::HistoryCell;
use pretty_assertions::assert_eq;

#[test]
fn consecutive_commands_share_a_summary_and_keep_transcript_details() {
    use crate::exec_cell::CommandOutput;
    use codex_app_server_protocol::CommandExecutionSource;
    use std::time::Duration;

    let mut cell = crate::exec_cell::new_active_exec_command(
        "first".into(),
        vec!["echo first".into()],
        Vec::new(),
        CommandExecutionSource::Agent,
        /*interaction_input*/ None,
        /*animations_enabled*/ false,
    );
    let mut previews = Vec::new();
    previews.push(cell.display_lines(/*width*/ 80));
    assert!(cell.complete_call(
        "first",
        CommandOutput::new(/*exit_code*/ 0, "first output".into()),
        Duration::from_secs(1),
    ));
    assert!(!cell.should_flush());
    assert!(cell.add_call(
        "second".into(),
        vec!["echo second".into()],
        Vec::new(),
        CommandExecutionSource::Agent,
        /*interaction_input*/ None,
    ));
    previews.push(cell.display_lines(/*width*/ 80));
    assert!(cell.complete_call(
        "second",
        CommandOutput::new(/*exit_code*/ 1, "second output".into()),
        Duration::from_secs(1),
    ));
    assert!(!cell.should_flush());
    previews.push(cell.display_lines(/*width*/ 80));
    assert!(!cell.add_call(
        "manual".into(),
        vec!["echo manual".into()],
        Vec::new(),
        CommandExecutionSource::UserShell,
        /*interaction_input*/ None,
    ));
    let transcript = cell.transcript_lines(/*width*/ 80);
    let text = transcript
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("echo first"));
    assert!(text.contains("echo second"));
    assert!(text.contains("first output"));
    assert!(text.contains("second output"));
    insta::assert_debug_snapshot!(
        previews
            .iter()
            .map(|lines| lines.iter().map(ToString::to_string).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
}

#[test]
fn finished_commands_hide_details_without_losing_transcript() {
    let mut snapshots = Vec::new();
    for exit_code in [0, 1] {
        let mut cell = crate::exec_cell::new_active_exec_command(
            "call".into(),
            vec!["bash".into(), "-lc".into(), "echo private-command".into()],
            Vec::new(),
            codex_app_server_protocol::CommandExecutionSource::Agent,
            /*interaction_input*/ None,
            /*animations_enabled*/ false,
        );
        assert!(cell.complete_call(
            "call",
            crate::exec_cell::CommandOutput::new(exit_code, "private-output".into()),
            std::time::Duration::from_secs(1)
        ));
        let transcript = cell.transcript_lines(/*width*/ 80);
        for width in [10, 40, 80] {
            let preview = cell.display_lines(width);
            let text = preview
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(!text.contains("private-command"));
            assert!(!text.contains("private-output"));
            snapshots.push((exit_code, width, text));
        }
        assert_eq!(cell.transcript_lines(/*width*/ 80), transcript);
        let text = transcript
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("private-command"));
        assert!(text.contains("private-output"));
    }
    insta::assert_debug_snapshot!(snapshots);
}

#[test]
fn preview_is_bounded_at_narrow_and_wide_widths() {
    let source = Line::from("● Called tools.read ".to_owned() + &"文件\n".repeat(80));
    let mut snapshots = Vec::new();
    for width in [1, 10, 20, 80] {
        let preview = summary(source.clone(), width);
        assert!(preview.len() <= 1);
        assert!(
            preview
                .iter()
                .all(|line| line.width() <= usize::from(width))
        );
        snapshots.push((
            width,
            preview.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ));
    }
    insta::assert_debug_snapshot!(snapshots);
}

#[test]
fn short_preview_preserves_styles_and_empty_output() {
    let line = Line::from("● Error".red());
    assert_eq!(
        summary(line.clone(), /*width*/ 80),
        vec![Line::from(vec!["● Error".red(), " (ctrl+t)".dim()])]
    );
    assert_eq!(summary(line, /*width*/ 0), Vec::<Line>::new());
}

#[test]
fn long_urls_and_wide_characters_fit_the_preview() {
    let lines = vec![
        Line::from(format!("https://example.com/{}", "segment/".repeat(30))),
        Line::from("文件修改完成 ".repeat(30)),
    ];
    for width in [10, 20, 80] {
        for line in &lines {
            let preview = summary(line.clone(), width);
            assert!(preview.len() <= 1);
            assert!(
                preview
                    .iter()
                    .all(|line| line.width() <= usize::from(width))
            );
        }
    }
}

#[test]
fn collapsed_patch_keeps_the_complete_transcript() {
    let changes = std::collections::HashMap::from([(
        std::path::PathBuf::from("example.rs"),
        crate::diff_model::FileChange::Add {
            content: (0..50).map(|i| format!("// line {i}\n")).collect(),
        },
    )]);
    let cell = crate::history_cell::new_patch_event(changes.clone(), std::path::Path::new("."));
    let transcript = cell.transcript_lines(/*width*/ 80);
    assert_eq!(
        transcript,
        crate::diff_render::create_diff_summary(
            &changes,
            std::path::Path::new("."),
            /*wrap_cols*/ 80
        )
    );
    let preview = cell.display_lines(/*width*/ 80);
    assert_eq!(preview.len(), 1);
    assert!(transcript.len() > preview.len());
    insta::assert_snapshot!(
        preview
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}
