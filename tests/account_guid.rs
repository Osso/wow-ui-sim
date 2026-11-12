#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn fresh_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("fresh account GUID environment");
    {
        let mut state = env.state().borrow_mut();
        state.current_target = None;
        state.current_focus = None;
        state.party_members.clear();
        state.party_group_active = false;
    }
    env
}

#[test]
fn account_guid_siblings_project_id_and_current_player_identity() {
    let env = fresh_env();
    env.exec(
        r#"
        local getID = C_AccountInfo.GetIDFromBattleNetAccountGUID
        local related = C_AccountInfo.IsGUIDRelatedToLocalAccount
        assert(select('#', getID('Player-1-2')) == 1 and getID('Player-1-2') == nil)
        assert(getID('BNetAccount-0-0000002A') == 42)
        assert(getID('BNetAccount-7-000000010000002A') == 42)
        assert(getID('BNetAccount-') == 0 and getID('BNetAccount-foo') == 0)
        assert(getID('BNetAccount-1X2') == 2)
        assert(related(UnitGUID('player')) == true)
        assert(related('Player-999-00000001') == false)
        assert(related('BNetAccount-0-0000002A') == false)
        for _, case in ipairs({
            {getID, 'Usage: local battleNetAccountID = C_AccountInfo.GetIDFromBattleNetAccountGUID(battleNetAccountGUID)'},
            {related, 'Usage: local isLocalUser = C_AccountInfo.IsGUIDRelatedToLocalAccount(guid)'},
        }) do
            for _, value in ipairs({false, {}}) do
                local ok, err = pcall(case[1], value)
                assert(not ok and err:find(case[2], 1, true))
            end
            local ok, err = pcall(case[1])
            assert(not ok and err:find(case[2], 1, true))
        end
        "#,
    )
    .unwrap();
}

#[test]
fn account_guid_predicate_accepts_permissive_native_inputs() {
    let env = fresh_env();
    env.exec(
        r#"
        local predicate = C_AccountInfo.IsGUIDBattleNetAccountType
        for _, guid in ipairs({'BNetAccount-1-0000002A', 'BNetAccount-', 'BNetAccount-foo', 'BNetAccount-1X2', 'bnetaccount-1-2'}) do
            assert(predicate(guid, {}) == true, guid)
        end
        for _, guid in ipairs({'BNetAccount', 'BNetAccount-288230376151711744-1', 'Player-1-2', 'junk', 42}) do
            assert(predicate(guid) == false, tostring(guid))
        end
        for _, value in ipairs({false, {}}) do
            local ok, err = pcall(predicate, value)
            assert(not ok and err:find('Usage: local isBNet = C_AccountInfo.IsGUIDBattleNetAccountType(guid)', 1, true))
        end
        local ok, err = pcall(predicate)
        assert(not ok and err:find('Usage: local isBNet = C_AccountInfo.IsGUIDBattleNetAccountType(guid)', 1, true))
        debug.settaintmode(true)
        local secret = secretwrap('BNetAccount-1-2')
        assert(predicate(secret) == true)
        local function addon()
            local accepted, message = pcall(predicate, secret)
            assert(not accepted and message:find('Secret values are only allowed during untainted execution', 1, true))
        end
        debug.setobjecttaint(addon, 'AccountGuidProbe')
        addon()
        "#,
    )
    .unwrap();
}
