//! Resource-cost payload shared by C_TooltipInfo spell producers.
//! R81 Q2: opaque resource helpers remain host-supplied final display text.

/// Explicit host context for testing/feeding the native outer cost gates.
/// Native normal getters use mode 0 with content suppression clear (WM98).
#[derive(Clone, Copy, Debug, Default)]
pub struct SpellTooltipCostContext {
    pub mode: i32,
    pub suppress_content: bool,
    pub attributes_9: Option<u32>,
}

pub(crate) fn spell_cost_line(sim: &crate::lua_api::SimState, spell_id: u32) -> Option<&str> {
    let context = sim
        .spell_tooltip_cost_contexts
        .get(&spell_id)
        .copied()
        .unwrap_or_default();
    let attributes = context
        .attributes_9
        .or_else(|| crate::spells::get_spell(spell_id).and_then(|spell| spell.attributes_9))
        .unwrap_or_else(unknown_spell_attributes_9);
    // WM98: isLink and dontOverride are not cost-exclusion conditions.
    let input = sim.spell_tooltip_inputs.get(&spell_id)?;
    if context.mode == 1 || input.mode == 1 || context.suppress_content || attributes & 32768 != 0 {
        return None;
    }
    input.cost.as_deref().filter(|text| !text.is_empty())
}

/// UNKNOWN: no attribute row. Treat absent bits as clear, without claiming
/// native missing-data behavior. All profiles share this single named seam.
fn unknown_spell_attributes_9() -> u32 {
    0
}
