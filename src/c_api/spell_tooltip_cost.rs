//! Resource-cost payload shared by C_TooltipInfo spell producers.
//! Only Flash of Light has a cost fixture; unknown costs are not invented.

pub(crate) fn spell_cost_line(spell_id: u32, hyperlink: bool) -> Option<&'static str> {
    // 6.2.0 removed costs from spell hyperlinks, not direct spell tooltips.
    // Pre-Warlords profiles retain their existing direct/link behavior.
    if hyperlink && cfg!(feature = "profile-retail") {
        return None;
    }
    match spell_id {
        19750 => Some("10% of Base MANA"),
        _ => None,
    }
}
