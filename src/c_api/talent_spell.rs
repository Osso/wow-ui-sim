//! Shared talent-ID lookup for tooltip queries and cursor icons.

use crate::traits::{TRAIT_DEFINITION_DB, TRAIT_ENTRY_DB, TRAIT_NODE_DB};

pub(crate) fn spell_id_for_talent(talent_id: u32, selected_entry_id: Option<u32>) -> Option<u32> {
    if let Some(node) = TRAIT_NODE_DB.get(&talent_id) {
        let entry_id = selected_entry_id.or_else(|| node.entry_ids.first().copied())?;
        return spell_id_for_trait_entry(entry_id);
    }
    spell_id_for_trait_entry(talent_id).or_else(|| preferred_trait_spell_id(talent_id))
}

fn spell_id_for_trait_entry(entry_id: u32) -> Option<u32> {
    const MAX_DEFINITION_LINKS: usize = 8;
    let mut current_id = entry_id;
    for _ in 0..MAX_DEFINITION_LINKS {
        if let Some(spell_id) = preferred_trait_spell_id(current_id) {
            return Some(spell_id);
        }
        current_id = TRAIT_ENTRY_DB.get(&current_id)?.definition_id;
    }
    None
}

fn preferred_trait_spell_id(definition_id: u32) -> Option<u32> {
    let definition = TRAIT_DEFINITION_DB.get(&definition_id)?;
    [
        definition.visible_spell_id,
        definition.overrides_spell_id,
        definition.spell_id,
    ]
    .into_iter()
    .find(|spell_id| *spell_id != 0)
}
