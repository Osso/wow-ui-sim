//! Cooldown DTO metadata derived from the same interval snapshot as duration.
use crate::lua_api::SimState;

pub(crate) struct SpellCooldownSnapshot {
    pub start: f64,
    pub duration: f64,
    pub restricted: bool,
    pub recovery_remaining: Option<f64>,
    pub is_on_gcd: Option<bool>,
}

pub(crate) fn snapshot(sim: &SimState, spell_id: u32, now: f64) -> SpellCooldownSnapshot {
    let (start, duration) =
        super::cooldown_duration::select_cooldown_duration_times(sim, spell_id, now, false);
    let recovery_remaining = sim.gcd.and_then(|(gcd_start, gcd_duration)| {
        let remaining = gcd_start + gcd_duration - now;
        (remaining > 0.0).then_some(remaining)
    });
    // INFERRED: the simulator's start-recovery interval is its shared GCD.
    // isOnGCD means GCD determined the returned interval (not a longer spell
    // cooldown). Unknown GCD state omits both optional fields; known expiry
    // reports false. Native per-spell GCD eligibility is not modeled.
    let is_on_gcd = sim.gcd.map(|(gcd_start, gcd_duration)| {
        let gcd_end = gcd_start + gcd_duration;
        let spell_ends_later = sim
            .spell_cooldowns
            .get(&spell_id)
            .is_some_and(|cooldown| cooldown.start + cooldown.duration > gcd_end);
        recovery_remaining.is_some() && !spell_ends_later
    });
    SpellCooldownSnapshot {
        start,
        duration,
        restricted: super::charge_state::cooldowns_are_restricted(sim),
        recovery_remaining,
        is_on_gcd,
    }
}
