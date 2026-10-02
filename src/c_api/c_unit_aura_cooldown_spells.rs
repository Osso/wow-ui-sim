//! Explicit cooldown-spell associations; chosen simulator inputs, not native acquisition.

use std::collections::HashMap;

#[derive(Default)]
pub struct CooldownAuraAssociations {
    /// Keys are resolved query spell IDs; values are declared returned cooldown spell IDs.
    /// Empty by default. No generic aura lookup, implicit association, or native direction claim.
    pub cooldown_spell_ids: HashMap<u32, u32>,
}
