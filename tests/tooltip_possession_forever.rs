//! Forever's documented GetPossession consumer and the shared Retail producer mapping.

#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn possess_button_hover_uses_supplied_spell_on_retail_and_forever() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    for (spell_id, expected_name) in [(116, "Frostbolt"), (19750, "Flash of Light")] {
        env.state().borrow_mut().possession_spell_id = Some(spell_id);
        env.exec(&format!(
            r#"
            GameTooltip:Hide()
            GameTooltip:ClearHandlerInfo()
            GameTooltip:ClearLines()
            PossessButtonMixin:OnEnter(PossessButton1)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            local spell = C_TooltipInfo.GetSpellByID({spell_id})
            assert(info and info.getterName == 'GetPossession', 'original possession accessor ran')
            assert(data and data.id == {spell_id}, 'supplied possession spell sets tooltip identity')
            assert(data.lines[1].leftText == spell.lines[1].leftText, 'spell line follows supplied state')
            assert(data.lines[1].leftText == '{expected_name}', 'distinct supplied spell is published')
            "#
        ))
        .unwrap();
    }
}
