//! Minimal unit helpers still referenced outside the deleted mlua module.

use crate::lua_api::state::SimState;

pub fn parse_party_index(unit: &str) -> Option<usize> {
    let suffix = unit.strip_prefix("party")?;
    let index: usize = suffix.parse().ok()?;
    index.checked_sub(1)
}

pub(crate) fn parse_raid_index(unit: &str) -> Option<usize> {
    let index: usize = unit.strip_prefix("raid")?.parse().ok()?;
    index.checked_sub(1)
}

pub(crate) fn is_raid_group(sim: &SimState) -> bool {
    // The simulator represents a raid with six or more other group members.
    sim.party_group_active && sim.party_members.len() >= 6
}

pub(crate) fn active_group_member_index(sim: &SimState, unit: &str) -> Option<usize> {
    if !sim.party_group_active {
        return None;
    }
    let index = match parse_party_index(unit) {
        Some(index) => index,
        None if is_raid_group(sim) => parse_raid_index(unit)?,
        None => return None,
    };
    (index < sim.party_members.len()).then_some(index)
}
