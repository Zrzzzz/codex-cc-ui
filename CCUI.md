# codex-claude-code-ui

Repository name: `codex-cc-ui`.

Native Codex TUI styling inspired by
[pi-claude-code-ui](https://github.com/FammasMaz/pi-cc-tools).

Base: `openai/codex` at `5ecb3afd1bf405149e2159bfda50093b0c1b5fab`.
Reference: `FammasMaz/pi-cc-tools` at `0d873aa94bcfe77b0aa18344a9f8665a9f274d29`.

## Appearance

- Transparent composer and user messages with thin horizontal rules.
- Solid status dots for shell and MCP calls, green for success and red for failure.
- Bash labels and grouped read/search history with the existing tree gutter.
- Compact MCP headers with arguments available in the transcript.
- Completed agent shell and MCP calls show only status summaries; command text
  and results are available in the transcript. Running calls retain live previews.
- Patch history shows file/change statistics without inline diff bodies.
- Manually entered shell commands keep their output visible.
- Live output appears beside `Working`, using Pi's `↓ 123 tokens · elapsed` layout,
  and stays visible while the reply streams. It updates as text and reasoning deltas
  arrive, including chunks without newlines. `~` marks a UTF-8 byte-count estimate
  (four bytes per token); reported output usage replaces the estimate. Tool stdout
  is excluded, and reasoning summaries are not added on top of raw reasoning.
  The count resets each turn and the working row disappears when the task ends.
  `turn-output-tokens` is an optional footer item, disabled in the local default
  to avoid duplication; `total-output-tokens` retains its session-total meaning.
- Tool-input fragments also feed the live counter, including Code Mode scripts
  and JSON function arguments, before tool execution begins. Only byte counts
  cross the UI progress channel; tool output remains excluded. Encrypted reasoning
  without a text stream cannot be counted live and is reconciled from final usage.

Live means received deltas, not a timer animation. A provider that buffers a custom
tool's arguments until `done` still produces a single counter update. The local
sub2api adapters in `responses_client_tools.go` and
`chatcompletions_responses_bridge.go` currently have this buffering behavior;
continuous Code Mode progress also requires incremental forwarding at that layer.
- Ctrl+T opens the transcript, including full command output, MCP text and patches.

This fork changes native rendering. It does not install a Pi extension or change
model requests, permissions, session storage, or provider configuration. Pi's
extension commands, split diffs and Ctrl+O key binding are not implemented here.

## Build And Run

From `codex-rs`, with Rust 1.95.0 and the native build dependencies installed:

```sh
CARGO_PROFILE_DEV_DEBUG=0 cargo build -p codex-cli -p codex-code-mode-host
./target/debug/codex
```

Keep `codex-code-mode-host` beside `codex` in the build directory. Models using
Code Mode require this companion executable to execute JavaScript and dispatch
tool calls. Restart `codex-cc` after installing a previously missing host.

The current local host is copied from the installed Codex 0.153.4 package. Its
protocol handshake, JavaScript execution, persistent values, tool callbacks and
shutdown were smoke-tested against the wire schema in this checkout. The source
host build currently fails because the V8 150.4.0 Linux prebuilt archive with
pointer compression and sandbox support is not published; do not disable the V8
sandbox to work around that missing artifact.

The build uses existing Codex configuration. It does not replace the installed
`codex` command.

## Validate

```sh
env -u NO_COLOR -u TERM_PROGRAM -u TMUX -u TMUX_PANE \
  TERM=xterm-256color CARGO_PROFILE_DEV_DEBUG=0 just test -p codex-tui
just fmt
```

The test command clears terminal-specific environment overrides so color and
keyboard-hint snapshots use consistent inputs. Padded terminal snapshots retain
intentional trailing spaces.
