//! Resource-cost payload shared by C_TooltipInfo spell producers.
//! Only Flash of Light has a cost fixture; unknown costs are not invented.

/// Explicit host context for testing/feeding the native outer cost gates.
/// Native normal getters use mode 0 with content suppression clear (WM98).
#[derive(Clone, Copy, Debug, Default)]
pub struct SpellTooltipCostContext {
    pub mode: i32,
    pub suppress_content: bool,
    pub attributes_9: Option<u32>,
}

pub(crate) fn spell_cost_line(spell_id: u32, hyperlink: bool) -> Option<&'static str> {
    // 6.2.0 removed costs from spell hyperlinks, not direct spell tooltips.
    // Pre-Warlords profiles retain their existing direct/link behavior.
    // PTR 70493 TooltipInfoDocumentation.lua:337–353 declares GetHyperlink.
    // The shared link policy is documented in the pinned 6.2.0 extract,
    // data/patch-api/sources/6.2.0-api-changes.txt:12; PTR is also post-Warlords.
    if hyperlink
        && matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
        )
    {
        return None;
    }
    match spell_id {
        19750 => Some("10% of Base MANA"),
        _ => None,
    }
}
