//! WM94: original SetTotem accessor must show stored name, not spell composition.
#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::globals::real::totems::Totem;

#[test]
fn tooltip_totem_original_accessor_uses_stored_name_and_two_lines() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec("A_Admin.SetManualClock(100)").unwrap();
    env.state().borrow_mut().totem_slots = vec![
        Some(Totem {
            name: "Searing Totem".into(),
            start_time: 90.0,
            duration: 60.0,
            icon: 0,
            mod_rate: 1.0,
            spell_id: 19750,
        }),
        Some(Totem {
            name: "Healing Stream Totem".into(),
            start_time: 95.0,
            duration: 30.0,
            icon: 0,
            mod_rate: 2.0,
            spell_id: 19750,
        }),
    ];
    env.exec(
        r#"
        for slot, name in ipairs({'Searing Totem', 'Healing Stream Totem'}) do
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetTotem(slot)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetTotem', 'totem accessor missing')
            assert(data and data.type == 18 and data.id == nil)
            assert(#data.lines == 2, 'totem must not include a spell description block')
            assert(data.lines[1].leftText == name, 'must use the stored name, not spell ID')
            assert(GameTooltip:GetLeftLine(1):GetText() == name)
            assert(type(data.lines[2].leftText) == 'string' and #data.lines[2].leftText > 0)
            assert(GameTooltip:GetLeftLine(2):GetText() == data.lines[2].leftText)
            assert(data.lines[2].leftColor and data.lines[2].leftColor.a == nil)
        end
        assert(select('#', C_TooltipInfo.GetTotem(3)) == 0)
        assert(select('#', C_TooltipInfo.GetTotem(9)) == 0)
        "#,
    )
    .unwrap();
}
