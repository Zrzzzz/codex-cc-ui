//! Real SSE -> core -> app-server -> PTY checks while tool generation is paused.

use super::focus_palette::PtyCodex;
use super::focus_palette::write_test_config;
use anyhow::Result;
use anyhow::ensure;
use axum::Router;
use axum::response::Sse;
use axum::response::sse::Event;
use axum::routing::post;
use futures::stream;
use serde_json::json;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

enum ToolKind {
    Custom,
    Function,
}

async fn assert_live_tool_input_progress(kind: ToolKind) -> Result<()> {
    let root = codex_utils_cargo_bin::repo_root()?;
    let home = tempfile::tempdir_in("/tmp")?;
    write_test_config(home.path(), &root)?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let config_path = home.path().join("config.toml");
    let config = std::fs::read_to_string(&config_path)?;
    std::fs::write(
        &config_path,
        format!(
            "{config}\n[model_providers.stream_test]\nname = \"Stream test\"\nbase_url = \"http://{address}/v1\"\nwire_api = \"responses\"\nrequires_openai_auth = false\nsupports_websockets = false\n"
        ),
    )?;

    let (tx, rx) = mpsc::channel::<Event>(8);
    let receiver = Arc::new(Mutex::new(Some(rx)));
    let app = Router::new().route(
        "/v1/responses",
        post(move || {
            let receiver = Arc::clone(&receiver);
            async move {
                let rx = receiver.lock().await.take().expect("one model request");
                Sse::new(stream::unfold(rx, |mut rx| async move {
                    rx.recv()
                        .await
                        .map(|event| (Ok::<_, Infallible>(event), rx))
                }))
            }
        }),
    );
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
    });
    let mut terminal = PtyCodex::start(
        &root,
        home,
        &[
            "-c",
            "model_provider=\"stream_test\"",
            "Generate a tool call for this local streaming test.",
        ],
    )?;
    terminal.wait_for_startup()?;

    let (item, event_type, first, second, expected) = match kind {
        ToolKind::Custom => (
            json!({"type":"custom_tool_call", "id":"tool-stream", "call_id":"call-stream", "name":"exec", "input":""}),
            "response.custom_tool_call_input.delta",
            "text(\"ab",
            "cdef\");",
            "↓ ~4 tokens",
        ),
        ToolKind::Function => (
            json!({"type":"function_call", "id":"tool-stream", "call_id":"call-stream", "name":"shell_command", "arguments":""}),
            "response.function_call_arguments.delta",
            "{\"comman",
            "d\":\"pwd\"}",
            "↓ ~5 tokens",
        ),
    };
    for event in [
        json!({"type":"response.created", "response":{"id":"stream-response"}}),
        json!({"type":"response.output_item.added", "item":item}),
        json!({"type":event_type, "item_id":"tool-stream", "call_id":"call-stream", "delta":first}),
    ] {
        tx.send(Event::default().data(event.to_string())).await?;
    }
    terminal.wait_for_screen("↓ ~2 tokens")?;
    tx.send(Event::default().data(json!({"type":event_type, "item_id":"tool-stream", "call_id":"call-stream", "delta":second}).to_string())).await?;
    terminal.wait_for_screen(expected)?;
    terminal.ensure_running()?;
    ensure!(
        !terminal.screen_contains("abcdef"),
        "tool input leaked into the TUI"
    );

    // Deliberately never send item/done or response/completed: no tool can execute.
    drop(terminal);
    drop(tx);
    let _ = shutdown_tx.send(());
    server.await??;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn custom_tool_input_updates_tokens_before_generation_finishes() -> Result<()> {
    assert_live_tool_input_progress(ToolKind::Custom).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn function_arguments_update_tokens_before_generation_finishes() -> Result<()> {
    assert_live_tool_input_progress(ToolKind::Function).await
}
