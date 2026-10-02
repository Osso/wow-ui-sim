//! INFERRED explicit spell classifications; no native catalog or acquisition claim.

use std::collections::HashMap;

#[derive(Clone, Copy, Default)]
pub struct AuraSpellClassification {
    pub is_big_defensive: bool,
    pub is_private: bool,
}

#[derive(Default)]
pub struct AuraSpellClassifications {
    /// Host-declared resolved spell IDs only; empty by default.
    /// Generic buffs, private instances and cooldown associations supply no classifications.
    pub spells: HashMap<u32, AuraSpellClassification>,
}
