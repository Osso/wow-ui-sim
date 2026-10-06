//! 11.2.7 removals are baseline for all supported retail epochs.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_2_7_console_catalog_publishes_neighborhood_commands() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    let count: i64 = env
        .eval(
            r#"
            local expected = {
                NeighborhoodAddManager = true, NeighborhoodCancelInvitation = true,
                NeighborhoodGetInvites = true, NeighborhoodInviteResident = true,
                NeighborhoodRemoveManager = true, NeighborhoodSetName = true,
                NeighborhoodSetPublic = true, OfferNeighborhoodOwnership = true,
                PlayerDeclineHousingInvitation = true, PlayerGetHousingInvitation = true,
            }
            local count = 0
            for _, record in ipairs(C_Console.GetAllCommands()) do
                if expected[record.command] then
                    assert(record.commandType == Enum.ConsoleCommandType.Command)
                    expected[record.command] = nil
                    count = count + 1
                end
            end
            return count
            "#,
        )
        .expect("enumerate command records");
    assert_eq!(count, 10);
}

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
