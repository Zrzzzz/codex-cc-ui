// Aggregates all former standalone integration tests as modules.
#[cfg(unix)]
mod focus_palette;
#[cfg(unix)]
mod reconnect;
mod resize_reflow;
mod status_indicator;
#[cfg(unix)]
mod tool_input_progress;
mod vt100_history;
mod vt100_live_commit;
#[cfg(unix)]
mod worktree_stack;
