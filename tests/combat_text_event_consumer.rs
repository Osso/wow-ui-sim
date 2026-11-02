//! Original CombatText consumer observes supplied event payload during dispatch.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn combat_text_payload_is_scoped_to_original_consumer_dispatch(env: &WowLuaEnv) {
    env.exec(r#"
        assert(C_AddOns.LoadAddOn('Blizzard_CombatText'))
        CombatTextTypeInfo.DAMAGE.show = 1
        CombatTextTypeInfo.SPELL_CAST.show = 1
        local messages, payloads = {}, {}
        local consumer = {
            IsVisible = function() return true end,
            AddMessage = function(_, message) messages[#messages + 1] = message end,
        }
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('COMBAT_TEXT_UPDATE')
        listener:SetScript('OnEvent', function(_, event, name)
            CombatTextMixin.OnEvent(consumer, event, name)
            payloads[#payloads + 1] = { n = select('#', C_CombatText.GetCurrentEventInfo()), C_CombatText.GetCurrentEventInfo() }
        end)
        A_Admin.FireEvent('COMBAT_TEXT_UPDATE', 'DAMAGE', 123)
        assert(select('#', C_CombatText.GetCurrentEventInfo()) == 0)
        A_Admin.FireEvent('COMBAT_TEXT_UPDATE', 'SPELL_CAST', 'Fireball')
        assert(select('#', C_CombatText.GetCurrentEventInfo()) == 0)
        assert(payloads[1].n == 1 and payloads[1][1] == 123)
        assert(payloads[2].n == 1 and payloads[2][1] == 'Fireball')
        assert(messages[1] == '-123' and messages[2] == '<Fireball>')
        assert(not pcall(A_Admin.FireEvent, 'COMBAT_TEXT_UPDATE', 'MISFIRE', 4))
        assert(not pcall(A_Admin.FireEvent, 'COMBAT_TEXT_UPDATE', 'DAMAGE', 'bad'))
    "#).unwrap();
}
}
