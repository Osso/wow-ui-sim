//! Burst bookkeeping and the cooldown pair (WM47 Q1/Q2, WM51-F1), on the cached
//! millisecond clock that `GetTime` reads.

const IDLE_RESET_MS: u32 = 3000;
const RESET_LIMIT: u32 = 2;
const RESET_WINDOW_MS: u32 = 3000;
const COOLDOWN_MS: u32 = 3000;

/// WM51-F1: the stored cooldown timing pair that `C_Ping.GetCooldownInfo`
/// reads, in cached milliseconds; (0, 0) when cleared.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PingCooldownInterval {
    pub start_time_ms: u32,
    pub end_time_ms: u32,
}

/// WM47: the burst count, limit and window, the last submission, and whether
/// the stored cooldown pair is active.
#[derive(Debug, Clone, Copy)]
pub struct PingBurst {
    count: u32,
    limit: u32,
    window_ms: u32,
    window_start: u32,
    last_submit: u32,
    cooldown_active: bool,
    pub interval: PingCooldownInterval,
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
            cooldown_active: false,
            interval: PingCooldownInterval::default(),
        }
    }
}

impl PingBurst {
    /// WM47 Q2: an active cooldown refuses while `now` precedes its deadline;
    /// expiry clears the flag and pair without an idle reset. Otherwise 3000 ms
    /// since the last submission resets the count, limit and window, starting
    /// the window now.
    pub(super) fn admit(&mut self, now: u32) -> bool {
        if self.cooldown_active {
            if (now.wrapping_sub(self.interval.end_time_ms) as i32) < 0 {
                return false;
            }
            self.cooldown_active = false;
            self.interval = PingCooldownInterval::default();
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
    /// increment. A count above the limit stores the pair {now, now + 3000} and
    /// returns it for the cooldown-started callback (WM51-F1).
    pub(super) fn count_submission(&mut self, now: u32) -> Option<PingCooldownInterval> {
        if self.count >= self.limit && now.wrapping_sub(self.window_start) >= self.window_ms {
            self.count = 0;
        }
        self.count = self.count.wrapping_add(1);
        if self.count <= self.limit {
            return None;
        }
        self.cooldown_active = true;
        self.interval = PingCooldownInterval {
            start_time_ms: now,
            end_time_ms: now.wrapping_add(COOLDOWN_MS),
        };
        Some(self.interval)
    }

    /// WM51-F1: the last submission is recorded after the callback.
    pub(super) fn finish_submission(&mut self, now: u32) {
        self.last_submit = now;
    }
}
