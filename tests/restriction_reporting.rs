#![cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
//! API-level: shared reason-0 restriction reporter (WM38F) through a guild request.

use wow_ui_sim::lua_api::WowLuaEnv;

fn reporter_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.world.guild_id = 42;
        sim.world.guild_player_rank = 0;
        sim.guild_request_inputs.transport_available = true;
    }
    env.exec(
        r#"
        Reports = {}
        Listener = CreateFrame('Frame')
        Listener:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        Listener:SetScript('OnEvent', function(_, _, addon, functionName, extra)
            assert(extra == nil)
            table.insert(Reports, addon .. '|' .. functionName)
        end)
        "#,
    )
    .expect("forbidden-action listener installs");
    env
}

#[test]
fn forbidden_report_uses_the_live_debug_name() {
    let env = reporter_env();
    env.exec(
        r#"
        local longName = string.rep('a', 1030)
        _G[longName] = C_GuildInfo.Demote
        local function addon()
            C_GuildInfo.Demote('Alice')
            local demote = C_GuildInfo.Demote
            demote('Alice')
            assert(pcall(C_GuildInfo.Demote, 'Alice'))
            loadstring(longName .. "('Alice')")()
        end
        debug.setobjecttaint(addon, 'ReporterFixture')
        addon()
        C_GuildInfo.Demote('Alice')
        assert(#Reports == 4)
        assert(Reports[1] == 'ReporterFixture|Demote()')
        assert(Reports[2] == 'ReporterFixture|demote()')
        assert(Reports[3] == 'ReporterFixture|UNKNOWN()')
        -- The diagnostic buffer holds at most 1023 bytes, cutting the suffix.
        assert(Reports[4] == 'ReporterFixture|' .. string.rep('a', 1023))
        "#,
    )
    .expect("reports carry the source and live debug name");
}

#[test]
fn forbidden_report_is_suppressed_during_reporter_reentry() {
    let env = reporter_env();
    env.exec(
        r#"
        local nested = function() C_GuildInfo.SetLeader('Bob') end
        debug.setobjecttaint(nested, 'NestedFixture')
        Listener:SetScript('OnEvent', function(_, _, addon, functionName)
            table.insert(Reports, addon .. '|' .. functionName)
            nested()
        end)
        local function addon() C_GuildInfo.Demote('Alice') end
        debug.setobjecttaint(addon, 'ReporterFixture')
        addon()
        assert(table.concat(Reports, ';') == 'ReporterFixture|Demote()')
        addon()
        assert(#Reports == 2)
        "#,
    )
    .expect("a refusal inside the report handler is not reported again");
    assert!(
        env.state()
            .borrow()
            .guild_request_inputs
            .submitted
            .is_empty()
    );
}
