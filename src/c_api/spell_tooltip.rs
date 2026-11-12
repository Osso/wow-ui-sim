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
    /// Final link-context helper output, when it differs from the direct context.
    pub link_cast_ms: Option<i32>,
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

impl SpellTooltipInput {
    pub(crate) fn range_is_visible(&self) -> bool {
        self.mode != 1
            && self.attributes_0.unwrap_or(0) & (4 | 1024) == 0
            && self.attributes_3.unwrap_or(0) & 1073741824 == 0
    }

    pub(crate) fn cast_is_visible(&self) -> bool {
        self.mode != 1
            && !self.alternate_header
            && !matches!(self.first_effect_kind, Some(47 | 78))
            && self.attributes_13.unwrap_or(0) & 33554432 == 0
            && self.attributes_0.unwrap_or(0) & 64 == 0
    }
}

/// Aura duration composition is UNKNOWN (R81 Q4). Keyed by aura instance ID.
#[derive(Clone, Debug, Default)]
pub struct AuraTooltipInput {
    pub remaining_text: Option<String>,
}
