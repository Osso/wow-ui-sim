#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn create_named_unit_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create UnitName test environment");
    env.exec(
        r#"
        A_Admin.SetPlayerName('UnitName Fixture Player')
        A_Admin.SetTarget('UnitName Fixture Target', 63, 1, true)
        assert(UnitExists('target'), 'fixture must create an actual target')
        "#,
    )
    .expect("populate test-local player and target");
    env
}

#[test]
fn unit_name_secret_tokens_public_names_and_provider_arity() {
    let env = create_named_unit_env();
    env.exec(
        r#"
        assert(UnitName('player') == 'UnitName Fixture Player')
        assert(UnitName('target') == 'UnitName Fixture Target')
        assert(select('#', UnitName('player')) == 1)
        assert(select('#', UnitName('target')) == 1)
        local playerName, playerServer = UnitName('player')
        local targetName, targetServer = UnitName('target')
        assert(playerName == 'UnitName Fixture Player' and playerServer == nil)
        assert(targetName == 'UnitName Fixture Target' and targetServer == nil)
        "#,
    )
    .expect("ordinary UnitName queries retain names and current provider arity");
}

#[test]
fn unit_name_secret_tokens_rejected_from_secure_top_level() {
    let env = create_named_unit_env();
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil, 'fixture must start secure')
        local playerToken = secretwrap('player')
        local targetToken = secretwrap('target')
        assert(issecretvalue(playerToken) and issecretvalue(targetToken))
        assert(not pcall(UnitName, playerToken), 'secret player token must be rejected')
        assert(debug.getstacktaint() == nil, 'player rejection must preserve secure caller')
        assert(not pcall(UnitName, targetToken), 'secret target token must be rejected')
        assert(debug.getstacktaint() == nil, 'target rejection must preserve secure caller')
        "#,
    )
    .expect("secure UnitName caller rejects opaque secret token fixtures");
}

#[test]
fn unit_name_secret_tokens_rejected_from_tainted_caller_with_public_control() {
    let env = create_named_unit_env();
    env.exec(
        r#"
        local playerToken = secretwrap('player')
        local targetToken = secretwrap('target')
        assert(issecretvalue(playerToken) and issecretvalue(targetToken))
        local function addonCaller()
            assert(debug.getstacktaint() == 'UnitNameTokenFixtureAddon')
            assert(not pcall(UnitName, playerToken), 'secret player token must be rejected')
            assert(debug.getstacktaint() == 'UnitNameTokenFixtureAddon')
            assert(not pcall(UnitName, targetToken), 'secret target token must be rejected')
            assert(debug.getstacktaint() == 'UnitNameTokenFixtureAddon')

            local publicPlayerToken = 'player'
            local publicTargetToken = 'target'
            assert(not issecretvalue(publicPlayerToken) and not issecretvalue(publicTargetToken))
            local playerOk, playerName = pcall(UnitName, publicPlayerToken)
            local targetOk, targetName = pcall(UnitName, publicTargetToken)
            assert(playerOk and playerName == 'UnitName Fixture Player')
            assert(targetOk and targetName == 'UnitName Fixture Target')
            assert(debug.getstacktaint() == 'UnitNameTokenFixtureAddon')
        end
        debug.setobjecttaint(addonCaller, 'UnitNameTokenFixtureAddon')
        addonCaller()
        "#,
    )
    .expect("tainted UnitName caller retains taint and accepts public token controls");
}
