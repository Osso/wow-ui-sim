//! INFERRED selector-to-column mapping for R88n.2 Azerite spell requests.
//!
//! R88n.2 verifies the native selector roles and the AzeriteEssencePower table,
//! but the four role-to-column bindings use the older exact-build 70391 CSV
//! export. Keep those bindings isolated from the VERIFIED rank and slot rules.

use crate::c_api::tooltip_azerite_catalog_data::AzeriteEssencePowerRecord;

#[derive(Clone, Copy, Debug)]
pub(crate) enum AzeriteEssenceSpellSelector {
    MajorDescription,
    MinorDescription,
    MajorActual,
    MinorActual,
}

pub(crate) fn spell_for_selector(
    power: &AzeriteEssencePowerRecord,
    selector: AzeriteEssenceSpellSelector,
) -> u32 {
    match selector {
        AzeriteEssenceSpellSelector::MajorDescription => power.major_power_description,
        AzeriteEssenceSpellSelector::MinorDescription => power.minor_power_description,
        AzeriteEssenceSpellSelector::MajorActual => power.major_power_actual,
        AzeriteEssenceSpellSelector::MinorActual => power.minor_power_actual,
    }
}
