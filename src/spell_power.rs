//! Profile-aware spell power lookup.
pub use sim_data::spell_power::{SpellPowerCost, power_type_name};

pub fn get_spell_power(id: u32) -> Option<&'static [SpellPowerCost]> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
        if let Some(costs) = crate::forever_spell_runtime::power(id) {
            return Some(costs);
        }
    }
    sim_data::spell_power::get_spell_power(id)
}
