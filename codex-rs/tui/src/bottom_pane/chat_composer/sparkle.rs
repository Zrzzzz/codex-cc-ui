//! One short Astra flourish for a confirmed new task with an untouched prompt.
//!
//! Ordinary draft content ends the opportunity permanently; slash commands can still select Astra
//! if the flourish has never started. The original field fills blank composer cells without drawing
//! over the placeholder or normal terminal cursor, and hidden frames do not schedule work. The
//! separate field renderer only paints its buffer and has no input or scheduling state. History
//! search hides an active field while its original deadline continues; its query and previews do
//! not count as input before acceptance. Shortcut help on an empty composer also hides an active
//! field without restarting its deadline and leaves an unused opportunity intact. The disconnected
//! editor shares draft tracking but treats composer-only shortcuts as ordinary editor input. A key
//! that finishes a paste burst reconciles the draft even if it removes a held slash in the same edit.

use std::cell::Cell;
use std::cell::RefCell;
use std::time::Duration;
use std::time::Instant;

use codex_config::types::Tui;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthStr;

use super::ChatComposer;
use super::FooterMode;
use super::InputResult;
use crate::bottom_pane::BottomPane;
use crate::terminal_palette::StdoutColorLevel;
use crate::terminal_palette::default_bg;
use crate::terminal_palette::default_fg;
use crate::terminal_palette::effective_stdout_color_level;

#[path = "sparkle_field.rs"]
mod field;

const FRAME_TICK: Duration = Duration::from_millis(/*millis*/ 150);
const IDLE_TIMEOUT: Duration = Duration::from_secs(/*secs*/ 15);
const IDLE_FADE: Duration = Duration::from_secs(/*secs*/ 1);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SparkleDraft {
    #[default]
    Untouched,
    Command,
    Dismissed,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Unarmed,
    Waiting,
    Visible(Instant),
    Finished,
}

pub(super) struct SparkleEditorState {
    text: String,
    paste_burst_active: bool,
}

pub(super) struct Sparkle {
    pub(super) draft: Cell<SparkleDraft>,
    /// Scoped to history's own replacements; other draft changes still count while search is open.
    pub(super) history_preview: bool,
    command_input: RefCell<String>,
    phase: Cell<Phase>,
    fresh: bool,
    terminal_focused: bool,
}

impl Default for Sparkle {
    fn default() -> Self {
        Self {
            draft: Cell::new(SparkleDraft::Untouched),
            history_preview: false,
            command_input: RefCell::new(String::new()),
            phase: Cell::new(Phase::Unarmed),
            fresh: false,
            terminal_focused: true,
        }
    }
}

impl BottomPane {
    pub(crate) fn is_sparkle_model(model: &str) -> bool {
        model
            .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
            .any(|part| part.eq_ignore_ascii_case("astra"))
    }

    pub(crate) fn mark_fresh_task_for_sparkle(&mut self, model: &str, settings: &Tui) {
        self.composer.sparkle.fresh = true;
        if matches!(
            self.composer.sparkle.draft.get(),
            SparkleDraft::Untouched | SparkleDraft::Command
        ) && self.composer.sparkle.phase.get() == Phase::Unarmed
            && settings.animations
            && settings.effects.starfield
            && Self::is_sparkle_model(model)
        {
            self.composer.sparkle.phase.set(Phase::Waiting);
            self.request_redraw();
        }
    }

    pub(crate) fn select_sparkle_model(&mut self, model: &str, settings: &Tui) {
        if self.composer.sparkle.fresh {
            self.mark_fresh_task_for_sparkle(model, settings);
        }
    }

    pub(crate) fn stop_ineligible_sparkle(&self, model: &str, settings: &Tui) {
        if !settings.animations || !settings.effects.starfield || !Self::is_sparkle_model(model) {
            self.composer.stop_visible_sparkle();
        }
    }

    pub(crate) fn dismiss_composer_sparkle(&self) {
        self.composer.dismiss_sparkle();
    }

    pub(crate) fn prepare_composer_sparkle_key(&self, key: KeyEvent) {
        if !self.has_active_view() {
            self.composer.prepare_sparkle_key(key);
        }
    }

    pub(crate) fn inherit_startup_sparkle(&self, draft: SparkleDraft) {
        if self.composer.sparkle.draft.get() == SparkleDraft::Untouched
            && draft != SparkleDraft::Untouched
        {
            self.composer.sparkle.draft.set(draft);
            if draft == SparkleDraft::Dismissed {
                self.composer.stop_visible_sparkle();
            }
        }
    }

    pub(crate) fn set_sparkle_terminal_focus(&mut self, focused: bool) {
        if self.composer.sparkle.terminal_focused != focused {
            self.composer.sparkle.terminal_focused = focused;
            self.request_redraw();
        }
    }
}

impl ChatComposer {
    fn prepare_sparkle_key(&self, key: KeyEvent) {
        if self.sparkle.draft.get() != SparkleDraft::Command
            && !(key.code == KeyCode::Char('/')
                && crate::key_hint::is_plain_text_key_event(key)
                && self.is_empty()
                && self.slash_commands_enabled())
        {
            self.stop_visible_sparkle();
        }
    }

    fn stop_visible_sparkle(&self) {
        if matches!(self.sparkle.phase.get(), Phase::Waiting | Phase::Visible(_)) {
            self.sparkle.phase.set(Phase::Finished);
            if let Some(requester) = &self.frame_requester {
                requester.schedule_frame();
            }
        }
    }

    pub(super) fn dismiss_sparkle(&self) {
        self.sparkle.draft.set(SparkleDraft::Dismissed);
        self.sparkle.command_input.borrow_mut().clear();
        self.stop_visible_sparkle();
    }

    pub(super) fn before_sparkle_key(&self, key: KeyEvent) -> Option<SparkleEditorState> {
        if self.history_search.is_some()
            || (Self::is_history_search_key(&key, &self.history_search_previous_keys)
                && (self.popups.active() || !self.draft.textarea.wants_vim_search_key(key))
                && !self.wants_vim_history_key(key))
            || self.empty_prompt_shortcut_toggle(&key).is_some()
        {
            return None;
        }
        self.before_sparkle_editor_key(key)
    }

    pub(super) fn before_sparkle_editor_key(&self, key: KeyEvent) -> Option<SparkleEditorState> {
        self.prepare_sparkle_key(key);
        if self.draft.textarea.vim_query().is_some() {
            return None;
        }
        if let KeyCode::Char(ch) = key.code
            && crate::key_hint::is_plain_text_key_event(key)
            && !key
                .modifiers
                .intersects(KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META)
            && !self.draft.textarea.is_vim_normal_mode()
        {
            match self.sparkle.draft.get() {
                SparkleDraft::Untouched
                    if ch == '/' && self.is_empty() && self.slash_commands_enabled() =>
                {
                    self.sparkle.draft.set(SparkleDraft::Command);
                    self.sparkle.command_input.replace(ch.to_string());
                }
                SparkleDraft::Command => {
                    let actual = self.current_text();
                    let mut projected = self.sparkle.command_input.borrow().clone();
                    let cursor = if actual == projected {
                        self.current_cursor()
                    } else {
                        projected.len()
                    };
                    projected.insert(cursor, ch);
                    if self.is_sparkle_command(&projected) {
                        self.sparkle.command_input.replace(projected);
                    } else {
                        self.dismiss_sparkle();
                    }
                }
                SparkleDraft::Untouched => self.dismiss_sparkle(),
                SparkleDraft::Dismissed => {}
            }
            None
        } else {
            Some(SparkleEditorState {
                text: self.current_text(),
                paste_burst_active: self.is_in_paste_burst(),
            })
        }
    }

    pub(super) fn after_sparkle_key(
        &self,
        before: Option<SparkleEditorState>,
        result: &InputResult,
    ) {
        match result {
            InputResult::Command(_) | InputResult::ServiceTierCommand(_) => {
                if self.sparkle.draft.get() == SparkleDraft::Command {
                    self.sparkle.draft.set(SparkleDraft::Untouched);
                    self.sparkle.command_input.borrow_mut().clear();
                }
            }
            InputResult::Submitted { .. }
            | InputResult::Queued { .. }
            | InputResult::ParentOwnedInputBlocked => self.dismiss_sparkle(),
            InputResult::CommandWithArgs(_, _, _) | InputResult::None => {
                if self.history_search.is_none() && self.draft.textarea.vim_query().is_none() {
                    let draft = self.current_text();
                    if before.is_some_and(|before| {
                        before.text != draft
                            || (before.paste_burst_active && !self.is_in_paste_burst())
                    }) {
                        self.note_sparkle_replaced_text(&draft);
                    }
                }
            }
            let background = match cell.bg {
                Color::Rgb(r, g, b) => Some((r, g, b)),
                Color::Reset => default_bg(),
                _ => None,
            };
            let Some(background) = background else {
                continue;
            };
            // A stable coordinate hash gives each star its own dot, period, and phase.
            let mut hash = u64::from(y - area.y) * 65537 + u64::from(x - area.x);
            hash = (hash ^ (hash >> 16)).wrapping_mul(0x45d9f3b);
            hash = (hash ^ (hash >> 16)).wrapping_mul(0x45d9f3b);
            hash ^= hash >> 16;
            if hash % 5 != 0 {
                continue;
            }
        }
    }

    pub(super) fn note_sparkle_replaced_text(&self, text: &str) {
        match self.sparkle.draft.get() {
            SparkleDraft::Command if text.is_empty() => {
                self.sparkle.draft.set(SparkleDraft::Untouched);
                self.sparkle.command_input.borrow_mut().clear();
            }
            buf[(x, y)]
                .set_symbol(DOTS[(hash / 161 % 8) as usize])
                .set_fg(rgb_color(blend(foreground, background, brightness)));
        }
    }
}

#[cfg(test)]
#[path = "sparkle_tests.rs"]
mod tests;
