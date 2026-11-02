//! Shared send eligibility and burst bookkeeping (WM38, WM47 Q1/Q2), on the
//! cached millisecond clock that `GetTime` reads.

const IDLE_RESET_MS: u32 = 3000;
const RESET_LIMIT: u32 = 2;
const RESET_WINDOW_MS: u32 = 3000;
const COOLDOWN_MS: u32 = 3000;

/// WM47: the burst count, limit and window, the last submission, and the
/// deadline of an active cooldown.
#[derive(Debug, Clone, Copy)]
pub struct PingBurst {
    count: u32,
    limit: u32,
    window_ms: u32,
    window_start: u32,
    last_submit: u32,
    cooldown_deadline: Option<u32>,
}

impl Default for PingBurst {
    /// INFERRED: the manager starts with the idle-reset values.
    fn default() -> Self {
        Self {
            count: 0,
            limit: RESET_LIMIT,
            window_ms: RESET_WINDOW_MS,
            window_start: 0,
            last_submit: 0,
            cooldown_deadline: None,
        }
    }
}

impl PingBurst {
    /// WM47 Q2: an active cooldown refuses while `now` precedes its deadline, and
    /// expiry clears it without an idle reset. Otherwise 3000 ms since the last
    /// submission resets the count, limit and window, starting the window now.
    pub(super) fn admit(&mut self, now: u32) -> bool {
        if let Some(deadline) = self.cooldown_deadline {
            if (now.wrapping_sub(deadline) as i32) < 0 {
                return false;
            }
            self.cooldown_deadline = None;
        } else if now
            .wrapping_sub(self.last_submit)
            .wrapping_sub(IDLE_RESET_MS) as i32
            >= 0
        {
            self.count = 0;
            self.limit = RESET_LIMIT;
            self.window_ms = RESET_WINDOW_MS;
            self.window_start = now;
        }
        true
    }

    /// WM47 Q1: a full count whose window has elapsed rolls over before the
    /// increment; a count above the limit starts a 3000 ms cooldown.
    pub(super) fn record_submission(&mut self, now: u32) {
        if self.count >= self.limit && now.wrapping_sub(self.window_start) >= self.window_ms {
            self.count = 0;
        }
        self.count = self.count.wrapping_add(1);
        self.last_submit = now;
        if self.count > self.limit {
            self.cooldown_deadline = Some(now.wrapping_add(COOLDOWN_MS));
        }
    }
}
