//! Per-turn output estimates from received deltas, reconciled with reported usage.

#[derive(Default)]
pub(crate) struct LiveOutputTokens {
    started: bool,
    last_total: Option<i64>,
    confirmed: u64,
    pending_bytes: u64,
    reasoning_bytes: u64,
    summary_bytes: u64,
    reported: bool,
}

pub(crate) enum ReasoningStream {
    Raw,
    Summary,
}

impl LiveOutputTokens {
    pub(crate) fn start(&mut self, total: Option<i64>) {
        *self = Self {
            started: true,
            last_total: total,
            ..Self::default()
        };
    }

    pub(crate) fn push(&mut self, delta: &str) -> bool {
        self.push_bytes(delta.len() as u64)
    }

    pub(crate) fn push_bytes(&mut self, bytes: u64) -> bool {
        if !self.started || bytes == 0 {
            return false;
        }
        let previous = self.pending_estimate();
        self.pending_bytes = self.pending_bytes.saturating_add(bytes);
        previous != self.pending_estimate()
    }

    pub(crate) fn push_reasoning(&mut self, delta: &str, stream: ReasoningStream) -> bool {
        if !self.started || delta.is_empty() {
            return false;
        }
        let previous = self.pending_estimate();
        let bytes = match stream {
            ReasoningStream::Raw => &mut self.reasoning_bytes,
            ReasoningStream::Summary => &mut self.summary_bytes,
        };
        *bytes = bytes.saturating_add(delta.len() as u64);
        previous != self.pending_estimate()
    }

    fn pending_estimate(&self) -> u64 {
        // Reasoning and its summary describe the same generation; do not add both.
        self.pending_bytes
            .saturating_add(self.reasoning_bytes.max(self.summary_bytes))
            .div_ceil(4)
    }

    pub(crate) fn reconcile(&mut self, total: i64, last: i64) {
        if !self.started || self.last_total == Some(total) {
            return;
        }
        let added = self
            .last_total
            .map_or(last, |previous| total.saturating_sub(previous));
        self.last_total = Some(total);
        if added < 0 {
            return;
        }
        self.confirmed = self.confirmed.saturating_add(added as u64);
        self.pending_bytes = 0;
        self.reasoning_bytes = 0;
        self.summary_bytes = 0;
        self.reported = true;
    }

    pub(crate) fn label(&self) -> Option<String> {
        self.started.then(|| {
            let estimate = if self.pending_estimate() > 0 || !self.reported {
                "~"
            } else {
                ""
            };
            let tokens = self.confirmed.saturating_add(self.pending_estimate());
            format!("{estimate}{tokens} out")
        })
    }

    pub(crate) fn working_label(&self) -> Option<String> {
        let tokens = self.confirmed.saturating_add(self.pending_estimate());
        (self.started && tokens > 0).then(|| {
            let estimate = if self.pending_estimate() > 0 || !self.reported {
                "~"
            } else {
                ""
            };
            format!("↓ {estimate}{tokens} tokens")
        })
    }
}

#[cfg(test)]
#[path = "live_output_tokens_tests.rs"]
mod tests;
