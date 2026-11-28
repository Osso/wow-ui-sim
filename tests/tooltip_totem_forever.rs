//! Forever 70707 publishes `C_TooltipInfo.GetTotem`; its line composition is
//! INFERRED shared-model reuse (Round 6c). This test proves publication and the
//! two-line contract on that profile only, through the original accessor.
#![cfg(feature = "client-wowforever")]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::globals::real::totems::Totem;

#[test]
fn tooltip_totem_forever_getter_publishes_two_lines_for_an_occupied_slot() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.state().borrow_mut().totem_slots = vec![
        Some(Totem {
            name: "Searing Totem".into(),
            start_time: 0.0,
            duration: 90.0,
            icon: 0,
            mod_rate: 1.0,
            spell_id: 19750,
        }),
        Some(Totem {
            name: "Healing Stream Totem".into(),
            start_time: 0.0,
            duration: 0.0,
            icon: 0,
            mod_rate: 1.0,
            spell_id: 19750,
        }),
    ];
    env.exec(
        r#"
        assert(type(C_TooltipInfo.GetTotem) == 'function', 'GetTotem must be published on Forever')
        local data = C_TooltipInfo.GetTotem(1)
        assert(data and data.type == 18 and data.id == nil, 'identity is type 18 with no id')
        assert(#data.lines == 2, 'exactly two text lines')
        assert(data.lines[1].type == 0 and data.lines[1].leftText == 'Searing Totem')
        assert(data.lines[2].type == 0 and #data.lines[2].leftText > 0)
        assert(C_TooltipInfo.GetTotem(2).lines[1].leftText == 'Healing Stream Totem')
        -- Round 6a: the one-based reader subtracts one and truncates.
        assert(C_TooltipInfo.GetTotem(1.9).lines[1].leftText == 'Searing Totem')
        assert(select('#', C_TooltipInfo.GetTotem(0)) == 0)
        assert(select('#', C_TooltipInfo.GetTotem(3)) == 0)
        assert(select('#', C_TooltipInfo.GetTotem(9)) == 0)
        assert(not pcall(C_TooltipInfo.GetTotem, -1))
        -- The original accessor reaches the same getter.
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetTotem(1)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        assert(info and info.getterName == 'GetTotem', 'totem accessor missing')
        assert(GameTooltip:GetPrimaryTooltipData().lines[1].leftText == 'Searing Totem')
        "#,
    )
    .unwrap();
}
