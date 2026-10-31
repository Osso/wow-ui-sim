//! Profile-aware spell description lookup.
pub fn get_spell_description(id: u32) -> Option<&'static str> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
        if let Some(description) = crate::forever_spell_runtime::description(id) {
            return Some(description);
        }
    }
    sim_data::spell_descriptions::get_spell_description(id)
}
