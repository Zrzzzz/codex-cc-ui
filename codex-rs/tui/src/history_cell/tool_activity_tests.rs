use super::*;
use pretty_assertions::assert_eq;

#[test]
fn mixed_tools_share_one_row_and_keep_all_transcript_details() {
    let mut group = ToolActivityCell::default();
    let mut failed = ToolResultCell::new(PlainHistoryCell::new(vec![
        "● Failed tool".into(),
        "failure detail".into(),
    ]));
    failed.failure_count = 1;
    let cells: Vec<Arc<dyn HistoryCell>> = vec![
        Arc::new(failed),
        Arc::new(new_unified_exec_interaction(
            Some("long command".repeat(8)),
            "input\nsecond line".into(),
        )),
        Arc::new(new_unified_exec_interaction(
            Some("background task".into()),
            String::new(),
        )),
        Arc::new(new_web_search_call(
            "search".into(),
            "a long search query".repeat(10),
            WebSearchAction::Other,
        )),
        Arc::new(ToolResultCell::new(PlainHistoryCell::new(vec![
            "● Dynamic tool completed".into(),
            "full result".into(),
        ]))),
    ];
    let mut expected = Vec::new();
    for cell in cells {
        assert!(is_tool_activity(cell.as_ref()));
        if !expected.is_empty() {
            expected.push(Line::from(""));
        }
        expected.extend(cell.transcript_lines(/*width*/ 80));
        for width in [0, 1, 10, 40, 80] {
            let lines = cell.display_lines(width);
            assert!(lines.len() <= 1);
            assert!(lines.iter().all(|line| line.width() <= usize::from(width)));
        }
        group.push(cell);
    }
    assert_eq!(group.transcript_lines(/*width*/ 80), expected);
    assert_eq!(group.tool_failure_count(), 1);
    assert!(
        group.display_lines(/*width*/ 80)[0]
            .to_string()
            .contains("1 failed")
    );
    let mut previews = Vec::new();
    for width in [0, 1, 10, 40, 80] {
        let lines = group.display_lines(width);
        assert!(lines.len() <= 1);
        assert!(lines.iter().all(|line| line.width() <= usize::from(width)));
        previews.push((
            width,
            lines.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ));
    }
    insta::assert_debug_snapshot!(previews);
}
