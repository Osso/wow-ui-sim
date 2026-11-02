#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn deprecated_guild_script_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_DeprecatedGuildScript/Blizzard_DeprecatedGuildScript.toc")
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_deprecated_guild_script_toc_is_minimal_with_no_flags_or_deps() {
    let toc = TocFile::from_file(&deprecated_guild_script_toc())
        .expect("Blizzard_DeprecatedGuildScript TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_DeprecatedGuildScript declares `## LoadOnDemand: 0` — the GuildInvite / \
         GuildUninvite / GuildPromote / GuildDemote / GuildSetLeader / GuildSetMOTD / \
         GuildLeave / GuildDisband / GetGuildRosterMOTD / GetGuildInfoText / SetGuildInfoText \
         globals must install before any guild-frame addon Lua executes that calls them"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_DeprecatedGuildScript does not declare UseSecureEnvironment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_DeprecatedGuildScript declares NO dependencies — every shim simply \
         forwards to the always-loaded C_GuildInfo namespace (src/lua_api/env_init/\
         runtime_surface_bootstrap.lua:8818 + src/lua_api/globals/guild_info.rs)"
    );

    let toc_text = std::fs::read_to_string(deprecated_guild_script_toc())
        .expect("Blizzard_DeprecatedGuildScript TOC should read");
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_DeprecatedGuildScript omits `## AllowLoad:` — defaults to Game-screen-only \
         (src/toc.rs:311), matching the legacy guild-management in-game-only API surface"
    );
    assert!(
        !toc_text.contains("## AllowLoadGameType:"),
        "Blizzard_DeprecatedGuildScript omits `## AllowLoadGameType:` so the shims install \
         on every game type without restriction"
    );
}

#[test]
fn blizzard_deprecated_guild_script_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedGuildScript");
    assert!(
        in_game,
        "Blizzard_DeprecatedGuildScript (no AllowLoad flag, defaults to Game-only) should \
         appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedGuildScript");
    assert!(
        !in_login,
        "Blizzard_DeprecatedGuildScript should NOT appear on the Login / glue screens — \
         guild management is an in-game concept"
    );
}

prefork_full_ui_case! {
fn blizzard_deprecated_guild_script_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("DeprecatedGuildScript") || message.contains("Deprecated_GuildScript")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DeprecatedGuildScript emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_guild_script_installs_guild_management_function_shims(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(GuildInvite) == 'function' \
                and type(GuildUninvite) == 'function' \
                and type(GuildPromote) == 'function' \
                and type(GuildDemote) == 'function' \
                and type(GuildSetLeader) == 'function' \
                and type(GuildSetMOTD) == 'function' \
                and type(GuildLeave) == 'function' \
                and type(GuildDisband) == 'function'",
        )
        .expect("Guild management function-shim installation query should succeed");
    assert!(
        installed,
        "Deprecated_GuildScript.lua line 9-16 should publish 8 forwarding global functions: \
         GuildInvite → C_GuildInfo.Invite; GuildUninvite → C_GuildInfo.Uninvite; \
         GuildPromote → C_GuildInfo.Promote; GuildDemote → C_GuildInfo.Demote; \
         GuildSetLeader → C_GuildInfo.SetLeader; GuildSetMOTD → C_GuildInfo.SetMOTD; \
         GuildLeave → C_GuildInfo.Leave; GuildDisband → C_GuildInfo.Disband. These backing \
         C_GuildInfo methods are NOT explicitly stubbed; they resolve via the namespace \
         metatable's __index in runtime_surface_bootstrap.lua:2019-2028 which lazily \
         materializes a no-op `function() return nil end` on first read — so each global \
         alias becomes a callable no-op, not nil"
    );
}
}

prefork_full_ui_case! {
fn deprecated_guild_requests_validate_and_preserve_roster(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_guild_info::GuildRequest;
    use wow_ui_sim::lua_api::state::GuildMember;

    {
        let mut sim = env.state().borrow_mut();
        sim.world.guild_id = 42;
        sim.world.guild_player_rank = 0;
        sim.world.guild_members = vec![GuildMember {
            name: "Alice".into(), rank_index: 2, online: true,
        }];
        sim.guild_request_inputs.transport_available = true;
    }
    env.exec(r#"
        assert(not pcall(GuildDemote))
        assert(not pcall(GuildSetLeader))
        GuildDemote('aLiCe')
        GuildDemote('Nobody')
        GuildSetLeader('Bob')
        GuildDisband()
        assert(not pcall(GuildDemote, string.rep('x', 49)))
    "#).expect("original deprecated guild aliases should validate and submit");
    {
        let sim = env.state().borrow();
        assert_eq!(sim.world.guild_members.len(), 1);
        assert_eq!(sim.world.guild_members[0].rank_index, 2);
        assert_eq!(sim.world.guild_id, 42);
        assert_eq!(sim.guild_request_inputs.submitted, vec![
            GuildRequest::Demote("Alice".into()),
            GuildRequest::SetLeader("Bob".into()),
            GuildRequest::Disband,
        ]);
    }
    env.state().borrow_mut().world.guild_player_rank = 1;
    env.exec("assert(pcall(GuildSetLeader, string.rep('x', 306)))")
        .expect("leader prerequisite must precede length validation");
    env.state().borrow_mut().guild_request_inputs.action_disabled = true;
    env.exec("assert(pcall(GuildDemote, string.rep('x', 49))); GuildDisband()")
        .expect("disabled gate must precede length validation and submission");
    assert_eq!(env.state().borrow().guild_request_inputs.submitted.len(), 3);
    {
        let mut sim = env.state().borrow_mut();
        sim.guild_request_inputs.action_disabled = false;
        sim.guild_request_inputs.transport_available = false;
    }
    env.exec("GuildDisband()")
        .expect("request without transport should not mutate guild state");
    assert_eq!(env.state().borrow().guild_request_inputs.submitted.len(), 3);
}
}

prefork_full_ui_case! {
fn deprecated_guild_requests_refuse_tainted_callers_after_parsing(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.world.guild_id = 42;
        sim.world.guild_player_rank = 0;
        sim.guild_request_inputs.transport_available = true;
    }
    let (missing_demote, missing_leader, long_demote, long_leader, disband_ok, refused):
        (bool, bool, bool, bool, bool, i32) = env.eval(r#"
        local count = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        listener:SetScript('OnEvent', function() count = count + 1 end)
        local function missingDemote() return GuildDemote() end
        local function missingLeader() return GuildSetLeader() end
        local function longDemote() return GuildDemote(string.rep('x', 49)) end
        local function longLeader() return GuildSetLeader(string.rep('x', 306)) end
        local function disband() return GuildDisband() end
        for _, fn in ipairs({ missingDemote, missingLeader, longDemote, longLeader, disband }) do
            debug.setobjecttaint(fn, 'GuildProbe')
        end
        return pcall(missingDemote), pcall(missingLeader), pcall(longDemote),
            pcall(longLeader), pcall(disband), count
    "#).expect("tainted guild request probes should execute");
    assert_eq!((missing_demote, missing_leader, long_demote, long_leader, disband_ok, refused),
        (false, false, true, true, true, 3));
    assert!(env.state().borrow().guild_request_inputs.submitted.is_empty());
}
}

#[cfg(feature = "retail-12-1-5")]
prefork_full_ui_case! {
fn deprecated_guild_request_refusals_report_alias_names(env: &WowLuaEnv) {
    // Deprecated_GuildScript.lua:12-16 aliases the members directly, so the
    // live debug name is the global alias (WM38F).
    env.state().borrow_mut().guild_request_inputs.transport_available = true;
    let reports: String = env.eval(r#"
        local reports = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        listener:SetScript('OnEvent', function(_, _, addon, functionName)
            table.insert(reports, addon .. '|' .. functionName)
        end)
        local function addon()
            GuildDemote('Alice')
            GuildSetLeader('Bob')
            GuildDisband()
        end
        debug.setobjecttaint(addon, 'GuildAliasProbe')
        addon()
        return table.concat(reports, ';')
    "#).expect("tainted alias probes should execute");
    assert_eq!(
        reports,
        "GuildAliasProbe|GuildDemote();GuildAliasProbe|GuildSetLeader();GuildAliasProbe|GuildDisband()"
    );
    assert!(env.state().borrow().guild_request_inputs.submitted.is_empty());
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_guild_script_installs_guild_text_function_shims(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(GetGuildRosterMOTD) == 'function' \
                and type(GetGuildInfoText) == 'function' \
                and type(SetGuildInfoText) == 'function'",
        )
        .expect("Guild text function-shim installation query should succeed");
    assert!(
        installed,
        "Deprecated_GuildScript.lua line 19-21 (the `Deprecated in 12.0.1 onwards.` block) \
         should publish 3 guild-text global functions: GetGuildRosterMOTD → \
         C_GuildInfo.GetMOTD; GetGuildInfoText → C_GuildInfo.GetInfoText; SetGuildInfoText → \
         C_GuildInfo.SetInfoText. These three backing methods ARE explicitly stubbed in \
         src/lua_api/env_init/runtime_surface_bootstrap.lua:8825-8839 against the \
         C_GuildInfo._textState seed table (motd / infoText) so they return real strings, \
         not nil"
    );

    let motd: String = env
        .eval("return GetGuildRosterMOTD()")
        .expect("GetGuildRosterMOTD() should return a string");
    assert!(
        !motd.is_empty(),
        "GetGuildRosterMOTD() must return the seeded MOTD string from \
         C_GuildInfo._textState.motd (runtime_surface_bootstrap.lua:8821), not an empty \
         string"
    );

    let info_text: String = env
        .eval("return GetGuildInfoText()")
        .expect("GetGuildInfoText() should return a string");
    assert!(
        !info_text.is_empty(),
        "GetGuildInfoText() must return the seeded info-text string from \
         C_GuildInfo._textState.infoText (runtime_surface_bootstrap.lua:8822), not empty"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_guild_script_globals_alias_c_guild_info_methods(env: &WowLuaEnv) {
    let aliases_match: bool = env
        .eval(
            "return GuildInvite == C_GuildInfo.Invite \
                and GuildUninvite == C_GuildInfo.Uninvite \
                and GuildPromote == C_GuildInfo.Promote \
                and GuildDemote == C_GuildInfo.Demote \
                and GuildSetLeader == C_GuildInfo.SetLeader \
                and GuildSetMOTD == C_GuildInfo.SetMOTD \
                and GuildLeave == C_GuildInfo.Leave \
                and GuildDisband == C_GuildInfo.Disband \
                and GetGuildRosterMOTD == C_GuildInfo.GetMOTD \
                and GetGuildInfoText == C_GuildInfo.GetInfoText \
                and SetGuildInfoText == C_GuildInfo.SetInfoText",
        )
        .expect("alias-equality query should succeed");
    assert!(
        aliases_match,
        "All 11 deprecated guild globals must reference identity-equal values to their \
         backing C_GuildInfo methods after Blizzard_DeprecatedGuildScript loads"
    );

    {
        let mut state = env.state().borrow_mut();
        state.world.guild_name = Some("State Backed Guild".into());
        state.world.guild_rank = Some("Member".into());
        state.world.guild_members.clear();
        state.world.guild_num_members = 0;
    }

    env.exec(r#"C_GuildInfo.Invite("StateBackedRecruit")"#)
        .expect("C_GuildInfo.Invite should execute");
    let invited_rank = env
        .state()
        .borrow()
        .world
        .guild_members
        .iter()
        .find(|member| member.name == "StateBackedRecruit")
        .map(|member| member.rank_index)
        .expect("C_GuildInfo.Invite should append the named member");
    assert!(invited_rank > 1, "seeded guild should have a promotable rank");

    env.exec(r#"C_GuildInfo.Promote("StateBackedRecruit")"#)
        .expect("C_GuildInfo.Promote should execute");
    let promoted_rank = env
        .state()
        .borrow()
        .world
        .guild_members
        .iter()
        .find(|member| member.name == "StateBackedRecruit")
        .map(|member| member.rank_index)
        .expect("promoted member should remain in the roster");
    assert_eq!(promoted_rank, invited_rank - 1);

    env.exec(r#"C_GuildInfo.Uninvite("StateBackedRecruit")"#)
        .expect("C_GuildInfo.Uninvite should execute");
    assert!(
        env.state()
            .borrow()
            .world
            .guild_members
            .iter()
            .all(|member| member.name != "StateBackedRecruit"),
        "C_GuildInfo.Uninvite should remove the named member"
    );

    env.exec(
        r#"C_GuildInfo.Invite("LastMember")
           C_GuildInfo.Leave()"#,
    )
    .expect("C_GuildInfo.Leave should execute after an invite");
    let state = env.state().borrow();
    assert!(state.world.guild_name.is_none());
    assert!(state.world.guild_rank.is_none());
    assert_eq!(state.world.guild_num_members, 0);
    assert!(state.world.guild_members.is_empty());
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_guild_script_load_deprecation_fallbacks_cvar_is_default_on(env: &WowLuaEnv) {

    let cvar_on: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("GetCVarBool query should succeed");
    assert!(
        cvar_on,
        "The `loadDeprecationFallbacks` CVar must default to true (src/cvars.yaml:899 sets \
         '1') so the early-return guard at Deprecated_GuildScript.lua:4 doesn't bail before \
         the 11 guild globals are defined. If this CVar flips to false, all 11 are skipped \
         and any legacy guild-management addon calling GuildInvite / GuildPromote / \
         GetGuildRosterMOTD / etc. blows up with `attempt to call a nil value`"
    );
}
}

#[test]
fn blizzard_deprecated_guild_script_has_no_xml_or_other_assets() {
    let dir = blizzard_ui_dir().join("Blizzard_DeprecatedGuildScript");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_DeprecatedGuildScript dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let has_xml = entries.iter().any(|n| n.ends_with(".xml"));
    assert!(
        !has_xml,
        "Blizzard_DeprecatedGuildScript has NO XML files — pure Lua function-shim \
         definitions only. Got entries: {entries:?}"
    );

    let has_runtime_shims = entries.iter().any(|n| n == "Deprecated_GuildScript.lua");
    assert!(
        has_runtime_shims,
        "Blizzard_DeprecatedGuildScript should ship `Deprecated_GuildScript.lua` (the \
         runtime shim definitions for the 11 deprecated guild globals)"
    );
}
