use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn dynamic_tools_replace_the_live_row_and_keep_arguments_and_results() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    let mut item = ThreadItem::DynamicToolCall {
        id: "dynamic".into(),
        namespace: Some("plugin".into()),
        tool: "lookup".into(),
        arguments: serde_json::json!({"query": "original arguments"}),
        status: codex_app_server_protocol::DynamicToolCallStatus::InProgress,
        content_items: None,
        success: None,
        duration_ms: None,
    };
    chat.on_other_tool_item(item.clone());
    let running = chat
        .transcript
        .active_cell
        .as_ref()
        .unwrap()
        .display_lines(/*width*/ 80);
    assert_eq!(running.len(), 1);
    if let ThreadItem::DynamicToolCall {
        status,
        success,
        content_items,
        ..
    } = &mut item
    {
        *status = codex_app_server_protocol::DynamicToolCallStatus::Failed;
        *success = Some(false);
        *content_items = Some(vec![
            codex_app_server_protocol::DynamicToolCallOutputContentItem::InputText {
                text: "full failure detail".into(),
            },
        ]);
    }
    chat.on_other_tool_item(item);
    assert!(chat.transcript.active_cell.is_none());
    assert!(drain_insert_history(&mut rx).is_empty());
    chat.flush_active_cell();
    let cells = drain_insert_history_transcript(&mut rx);
    assert_eq!(cells.len(), 1);
    let completed = lines_to_single_string(&cells[0]);
    assert!(completed.contains("original arguments"));
    assert!(completed.contains("full failure detail"));
    insta::assert_snapshot!(lines_to_single_string(&running));
}

#[tokio::test]
async fn mixed_completed_and_live_tools_share_one_row_until_assistant_text() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.on_task_started();
    let command = begin_exec(&mut chat, "command", "echo full-command");
    end_exec(
        &mut chat,
        command,
        "full-output\n",
        "",
        /*exit_code*/ 0,
    );
    chat.add_to_history(history_cell::new_unified_exec_interaction(
        Some("background task".into()),
        String::new(),
    ));
    chat.on_web_search_begin("web".into());
    assert!(drain_insert_history(&mut rx).is_empty());
    let preview = chat
        .transcript
        .pending_tools
        .preview_lines(chat.transcript.active_cell.as_deref(), /*width*/ 100);
    assert_eq!(preview.len(), 1);
    let transcript = chat.active_cell_transcript_lines(/*width*/ 100).unwrap();
    let text = lines_to_single_string(&transcript);
    assert!(text.contains("full-command"));
    assert!(text.contains("full-output"));
    assert!(text.contains("Waited for background terminal"));
    assert!(text.contains("Searching the web"));
    insta::assert_snapshot!(render_bottom_popup(&chat, /*width*/ 100));
    chat.on_web_search_end(
        "web".into(),
        "full query".into(),
        codex_app_server_protocol::WebSearchAction::Other,
    );
    assert!(drain_insert_history(&mut rx).is_empty());
    chat.prepare_assistant_message();
    let history = drain_insert_history_transcript(&mut rx);
    assert!(!history.is_empty());
    assert!(lines_to_single_string(&history[0]).contains("full query"));
    assert!(chat.transcript.pending_tools.is_empty());
}
