//! Explicit DamageMeter inputs only; query publication remains unimplemented.
//! Plain Rust records carry no permission to disclose sensitive fields to Lua.

use std::collections::HashMap;

/// Empty/disabled default and explicit type bindings are simulator inferences.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageMeterInput {
    pub available: bool,
    pub failure_reason: String,
    pub available_sessions: Vec<DamageMeterAvailableCombatSession>,
    /// DamageMeterSessionType numeric value -> explicitly supplied session ID.
    pub session_ids_by_type: HashMap<i32, i64>,
    /// (session ID, DamageMeterType numeric value) -> aggregate snapshot input.
    pub sessions: HashMap<(i64, i32), DamageMeterCombatSession>,
    pub source_details: HashMap<(i64, i32, DamageMeterSourceKey), DamageMeterCombatSessionSource>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageMeterAvailableCombatSession {
    pub session_id: i64,
    pub name: String,
    pub duration_seconds: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageMeterCombatSession {
    pub combat_sources: Vec<DamageMeterCombatSource>,
    pub max_amount: f64,
    pub total_amount: f64,
    pub duration_seconds: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DamageMeterSourceKey {
    pub source_guid: Option<String>,
    pub source_creature_id: Option<i64>,
}

/// DamageMeterCombatSource: aggregate identity/metrics, never spell details.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageMeterCombatSource {
    pub source_guid: Option<String>,
    pub source_creature_id: Option<i64>,
    /// ConditionalSecret in cached documentation; not inherently readable.
    pub name: String,
    pub class_filename: String,
    pub spec_icon_id: i64,
    pub total_amount: f64,
    pub amount_per_second: f64,
    pub is_local_player: bool,
    pub death_recap_id: i64,
    pub death_time_seconds: f64,
    pub classification: String,
    pub source_display_type: i32,
    pub faction_group: Option<String>,
}

/// DamageMeterCombatSessionSource: direct-detail result, not an aggregate row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageMeterCombatSessionSource {
    pub combat_spells: Vec<DamageMeterCombatSpell>,
    pub max_amount: f64,
    pub total_amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageMeterCombatSpell {
    pub spell_id: i64,
    pub total_amount: f64,
    pub amount_per_second: f64,
    pub creature_name: String,
    pub overkill_amount: f64,
    pub is_avoidable: bool,
    pub is_deadly: bool,
    // Documentation declares one structure, not a sequence of target rows.
    pub combat_spell_details: DamageMeterCombatSpellUnitDetails,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageMeterCombatSpellUnitDetails {
    pub unit_name: String,
    pub unit_class_filename: String,
    pub classification: String,
    pub is_pet: bool,
    pub is_mob: bool,
    pub amount: f64,
    pub spec_icon_id: i64,
}
