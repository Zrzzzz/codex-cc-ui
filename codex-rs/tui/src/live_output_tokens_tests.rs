use super::*;
use pretty_assertions::assert_eq;

#[test]
fn tool_input_bytes_update_the_same_live_counter() {
    let mut counter = LiveOutputTokens::default();
    counter.start(Some(100));
    assert!(counter.push_bytes(8));
    assert_eq!(counter.working_label().as_deref(), Some("↓ ~2 tokens"));
    assert!(counter.push_bytes(8));
    assert_eq!(counter.working_label().as_deref(), Some("↓ ~4 tokens"));
    counter.reconcile(106, 6);
    assert_eq!(counter.working_label().as_deref(), Some("↓ 6 tokens"));
}

#[test]
fn estimates_are_chunk_independent_and_do_not_tick_without_output() {
    let mut whole = LiveOutputTokens::default();
    let mut chunks = LiveOutputTokens::default();
    whole.start(Some(100));
    chunks.start(Some(100));
    whole.push("hello 世界");
    for chunk in ["h", "ello ", "世", "界"] {
        chunks.push(chunk);
    }
    assert_eq!(whole.label(), chunks.label());
    let previous = chunks.label();
    assert!(!chunks.push(""));
    assert_eq!(chunks.label(), previous);
}

#[test]
fn usage_checkpoints_reconcile_and_next_turn_resets() {
    let mut counter = LiveOutputTokens::default();
    assert_eq!(counter.label(), None);
    assert_eq!(counter.working_label(), None);
    assert!(!counter.push("history replay"));
    counter.start(Some(100));
    counter.push("abcdefgh");
    assert_eq!(counter.label().as_deref(), Some("~2 out"));
    assert_eq!(counter.working_label().as_deref(), Some("↓ ~2 tokens"));
    counter.reconcile(110, 10);
    assert_eq!(counter.label().as_deref(), Some("10 out"));
    assert_eq!(counter.working_label().as_deref(), Some("↓ 10 tokens"));
    counter.push("abcd");
    counter.reconcile(110, 10);
    assert_eq!(counter.label().as_deref(), Some("~11 out"));
    counter.reconcile(120, 10);
    assert_eq!(counter.label().as_deref(), Some("20 out"));
    counter.start(Some(120));
    assert_eq!(counter.label().as_deref(), Some("~0 out"));
    assert_eq!(counter.working_label(), None);
}

#[test]
fn unknown_baseline_and_counter_reset_do_not_include_past_usage() {
    let mut counter = LiveOutputTokens::default();
    counter.start(None);
    counter.reconcile(1000, 5);
    assert_eq!(counter.label().as_deref(), Some("5 out"));
    counter.reconcile(0, 0);
    counter.reconcile(3, 3);
    assert_eq!(counter.label().as_deref(), Some("8 out"));
}

#[test]
fn reasoning_summary_is_not_counted_twice() {
    let mut counter = LiveOutputTokens::default();
    counter.start(Some(0));
    counter.push_reasoning("abcdefgh", ReasoningStream::Summary);
    counter.push_reasoning("abcdefgh", ReasoningStream::Raw);
    assert_eq!(counter.label().as_deref(), Some("~2 out"));
    counter.push("abcd");
    assert_eq!(counter.label().as_deref(), Some("~3 out"));
}
