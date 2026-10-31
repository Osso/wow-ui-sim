//! Profile-aware spell lookup.
pub use sim_data::spells::{SPELL_DB, SpellInfo};

pub fn get_spell(id: u32) -> Option<&'static SpellInfo> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
        if let Some(spell) = crate::forever_spell_runtime::spell(id) {
            return Some(spell);
        }
    }
    sim_data::spells::get_spell(id)
}
