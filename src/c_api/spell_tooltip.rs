//! Host inputs for R81's unresolved spell-tooltip helpers.
//! These are final helper results, not DB2 fields or inferred formulas.

#[derive(Clone, Debug, Default)]
pub struct SpellTooltipInput {
    pub resolved_spell_id: Option<u32>,
    pub overrides_spell_id: u32,
    pub suppress_replacement: bool,
    pub azerite_annotation: Option<String>,
    pub cost: Option<String>,
    pub ranges: Vec<String>,
    pub cast_ms: Option<i32>,
    pub alternate_cast_text: Option<String>,
    pub recast_text: Option<String>,
    pub cooldown_text: Option<String>,
    pub mode: i32,
    pub alternate_header: bool,
    pub first_effect_kind: Option<u32>,
    pub attributes_0: Option<u32>,
    pub attributes_3: Option<u32>,
    pub attributes_13: Option<u32>,
}

/// Aura duration composition is UNKNOWN (R81 Q4). Keyed by aura instance ID.
#[derive(Clone, Debug, Default)]
pub struct AuraTooltipInput {
    pub remaining_text: Option<String>,
}
