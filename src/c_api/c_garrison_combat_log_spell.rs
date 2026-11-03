//! Garrison combat spell presentation inputs.

/// Host-supplied fields not present in the pinned GarrAutoSpell export.
#[derive(Clone, Debug)]
pub struct SpellPresentation {
    pub preview_mask: u32,
    pub spell_tutorial_flag: u32,
    pub has_thorns_effect: bool,
}
