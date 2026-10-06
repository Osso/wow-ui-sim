//! 11.2.7 removals are baseline for all supported retail epochs.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_2_7_retirement_preserves_pvp_successor() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.exec(
        r#"
        for attempt = 1, 2 do
            assert(rawget(_G, 'JoinBattlefield') == nil)
            assert(JoinBattlefield == nil)
            assert(rawget(C_CharacterServices, 'RPEResetCharacter') == nil)
            assert(C_CharacterServices.RPEResetCharacter == nil)
            for _, name in ipairs({'AcceptPrompt', 'DeclinePrompt'}) do
                assert(rawget(C_ReturningPlayerUI, name) == nil)
                assert(C_ReturningPlayerUI[name] == nil)
            end
        end
        C_PvP.JoinBattlefield(3)
        local status, name = GetBattlefieldStatus(3)
        assert(status == 'queued')
        assert(name == 'Battleground 3')
        "#,
    )
    .expect("retired globals/members stay absent; live successor queues normally");
}
