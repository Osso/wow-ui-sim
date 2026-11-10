//! Explicit engraving inputs; absence is not a native inactive value. Mists
//! seeds the native default state at startup (see `SimState::seed_default_game_state`).
use std::collections::HashMap;

/// Pinned EngravingInfoDocumentation.lua:329-340 (Mists).
#[derive(Clone, Debug, PartialEq)]
pub struct EngravingData {
    pub skill_line_ability_id: i32,
    pub item_enchantment_id: i32,
    pub name: String,
    pub icon_texture: i32,
    pub equipment_slot: i32,
    pub level: i32,
    pub learned_ability_spell_ids: Vec<i32>,
}

/// Simulator-supplied query inputs, local to one environment.
#[derive(Default)]
pub struct EngravingInputs {
    pub enabled: Option<bool>,
    /// Mists engraving mode; native client default is false.
    pub mode_enabled: bool,
    pub current_rune: Option<Option<EngravingData>>,
    pub inventory_eligibility: HashMap<(i32, i32), bool>,
    pub inventory_runes: HashMap<(i32, i32), Option<EngravingData>>,
    pub current_cast_eligibility: HashMap<(i32, i32), bool>,
    /// Request count only; no inferred native catalog refresh policy.
    pub refresh_requests: usize,
}

impl EngravingInputs {
    /// Mists startup defaults. Availability false and no current rune are
    /// inferred; the native engraving-mode default is false.
    pub fn mists_native_defaults() -> Self {
        Self {
            enabled: Some(false),
            mode_enabled: false,
            current_rune: Some(None),
            ..Self::default()
        }
    }
}
