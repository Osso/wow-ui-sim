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

fn deprecated_action_bar_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_DeprecatedActionBar/Blizzard_DeprecatedActionBar.toc")
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
fn blizzard_deprecated_action_bar_toc_is_minimal_with_no_flags_or_deps() {
    let toc = TocFile::from_file(&deprecated_action_bar_toc())
        .expect("Blizzard_DeprecatedActionBar TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_DeprecatedActionBar declares `## LoadOnDemand: 0` so the legacy action-bar \
         globals (GetActionTexture / HasAction / IsUsableAction / etc.) install before any \
         legacy action-bar Lua executes"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_DeprecatedActionBar does not declare UseSecureEnvironment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_DeprecatedActionBar declares NO dependencies — every shim simply forwards \
         to the always-loaded C_ActionBar namespace (src/lua_api/globals/action_bar_api.rs)"
    );

    let toc_text = std::fs::read_to_string(deprecated_action_bar_toc())
        .expect("Blizzard_DeprecatedActionBar TOC should read");
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_DeprecatedActionBar omits `## AllowLoad:` — defaults to Game-screen-only \
         (src/toc.rs:311), matching its in-game-only API surface"
    );
    assert!(
        !toc_text.contains("## AllowLoadGameType:"),
        "Blizzard_DeprecatedActionBar omits `## AllowLoadGameType:` so the shims install on \
         every game type without restriction"
    );
}

#[test]
fn blizzard_deprecated_action_bar_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedActionBar");
    assert!(
        in_game,
        "Blizzard_DeprecatedActionBar (no AllowLoad flag, defaults to Game-only) should appear \
         in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedActionBar");
    assert!(
        !in_login,
        "Blizzard_DeprecatedActionBar should NOT appear on the Login / glue screens — \
         action bars are an in-game concept"
    );
}

prefork_full_ui_case! {
fn blizzard_deprecated_action_bar_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("DeprecatedActionBar") || message.contains("Deprecated_ActionBar")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DeprecatedActionBar emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_action_bar_installs_action_state_query_shims(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(GetActionAutocast) == 'function' \
                and type(GetActionText) == 'function' \
                and type(GetActionTexture) == 'function' \
                and type(GetActionCount) == 'function' \
                and type(GetActionCooldown) == 'function' \
                and type(GetActionCharges) == 'function' \
                and type(GetActionLossOfControlCooldown) == 'function' \
                and type(HasAction) == 'function' \
                and type(IsAttackAction) == 'function' \
                and type(IsCurrentAction) == 'function' \
                and type(IsAutoRepeatAction) == 'function' \
                and type(IsUsableAction) == 'function' \
                and type(IsConsumableAction) == 'function' \
                and type(IsStackableAction) == 'function' \
                and type(IsItemAction) == 'function' \
                and type(IsEquippedAction) == 'function' \
                and type(ActionHasRange) == 'function' \
                and type(IsActionInRange) == 'function' \
                and type(SetActionUIButton) == 'function'",
        )
        .expect("Per-action query shim installation query should succeed");
    assert!(
        installed,
        "Deprecated_ActionBar.lua should publish 19 per-action-slot query globals: \
         GetActionAutocast / GetActionText / GetActionTexture / GetActionCount (renamed to \
         C_ActionBar.GetActionUseCount) / GetActionCooldown (unpacks the new \
         {{startTime,duration,isEnabled,modRate}} struct to 4 positional returns) / \
         GetActionCharges (unpacks the new {{currentCharges,maxCharges,cooldownStartTime,\
         cooldownDuration,chargeModRate}} struct to 5 positional returns) / \
         GetActionLossOfControlCooldown / HasAction / IsAttackAction / IsCurrentAction / \
         IsAutoRepeatAction / IsUsableAction / IsConsumableAction / IsStackableAction / \
         IsItemAction / IsEquippedAction / ActionHasRange (renamed to \
         C_ActionBar.HasRangeRequirements) / IsActionInRange / SetActionUIButton (renamed to \
         C_ActionBar.RegisterActionUIButton)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_action_bar_installs_bar_index_shims(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(GetBonusBarIndex) == 'function' \
                and type(GetBonusBarOffset) == 'function' \
                and type(GetExtraBarIndex) == 'function' \
                and type(GetMultiCastBarIndex) == 'function' \
                and type(GetOverrideBarIndex) == 'function' \
                and type(GetOverrideBarSkin) == 'function' \
                and type(GetTempShapeshiftBarIndex) == 'function' \
                and type(GetVehicleBarIndex) == 'function' \
                and type(GetActionBarPage) == 'function' \
                and type(ChangeActionBarPage) == 'function'",
        )
        .expect("Bar-index shim installation query should succeed");
    assert!(
        installed,
        "Deprecated_ActionBar.lua should publish 10 bar-index globals: GetBonusBarIndex / \
         GetBonusBarOffset / GetExtraBarIndex / GetMultiCastBarIndex / GetOverrideBarIndex / \
         GetOverrideBarSkin / GetTempShapeshiftBarIndex / GetVehicleBarIndex / \
         GetActionBarPage / ChangeActionBarPage (renamed to C_ActionBar.SetActionBarPage)"
    );
}
}

const LEGACY_BAR_GLOBALS_MATCH_STATE_LUA: &str = r#"
assert(GetCVarBool("loadDeprecationFallbacks"))
local globals = {
    "GetBonusBarIndex", "GetBonusBarOffset", "GetExtraBarIndex",
    "GetMultiCastBarIndex", "GetOverrideBarIndex", "GetOverrideBarSkin",
    "GetTempShapeshiftBarIndex", "GetVehicleBarIndex", "GetActionBarPage",
}
for _, name in ipairs(globals) do
    if _G[name]() ~= C_ActionBar[name]() then return name end
end
ChangeActionBarPage(4)
if GetActionBarPage() ~= C_ActionBar.GetActionBarPage() then return "page" end
if GetActionBarPage() ~= 4 then return "changed_page" end
return "ok"
"#;

fn seed_non_default_action_bar_state(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.action_bar_page = 2;
    state.bonus_bar_index = 9;
    state.has_bonus_action_bar = true;
    state.vehicle_bar_index = 21;
    state.has_vehicle_action_bar = true;
    state.override_bar_index = 24;
    state.override_bar_skin = Some(25);
    state.has_override_action_bar = true;
    state.temp_shapeshift_bar_index = 22;
    state.has_temp_shapeshift_action_bar = true;
    state.action_bar_state.possess_bar_visible = true;
}

fn assert_legacy_bar_globals_match_state(env: &WowLuaEnv) {
    let result: String = env
        .eval(LEGACY_BAR_GLOBALS_MATCH_STATE_LUA)
        .expect("deprecated action-bar globals should preserve observable behavior");
    assert_eq!(result, "ok");
}

prefork_full_ui_case! {
fn blizzard_deprecated_action_bar_legacy_globals_follow_original_provider(env: &WowLuaEnv) {
    seed_non_default_action_bar_state(env);
    assert_legacy_bar_globals_match_state(env);
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_action_bar_installs_bar_visibility_shims(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(HasBonusActionBar) == 'function' \
                and type(HasExtraActionBar) == 'function' \
                and type(HasOverrideActionBar) == 'function' \
                and type(HasTempShapeshiftActionBar) == 'function' \
                and type(HasVehicleActionBar) == 'function' \
                and type(IsPossessBarVisible) == 'function'",
        )
        .expect("Bar-visibility shim installation query should succeed");
    assert!(
        installed,
        "Deprecated_ActionBar.lua should publish 6 bar-visibility globals: HasBonusActionBar \
         / HasExtraActionBar / HasOverrideActionBar / HasTempShapeshiftActionBar / \
         HasVehicleActionBar / IsPossessBarVisible — all 1:1 forwarders to C_ActionBar.*"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_action_bar_overrides_loss_of_control_method_on_namespace(env: &WowLuaEnv) {

    let overridden: bool = env
        .eval(
            "return type(C_ActionBar) == 'table' \
                and type(C_ActionBar.GetActionLossOfControlCooldown) == 'function' \
                and type(C_ActionBar.GetActionLossOfControlCooldownInfo) == 'function'",
        )
        .expect("C_ActionBar.GetActionLossOfControlCooldown* query should succeed");
    assert!(
        overridden,
        "Deprecated_ActionBar.lua line 154 should re-define \
         C_ActionBar.GetActionLossOfControlCooldown to unpack the new \
         GetActionLossOfControlCooldownInfo struct ({{startTime, duration}}) into the legacy \
         two-return positional shape — the override stomps the original C-level method on the \
         C_ActionBar namespace itself, while keeping the new GetActionLossOfControlCooldownInfo \
         method available alongside it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_action_bar_load_deprecation_fallbacks_cvar_is_default_on(env: &WowLuaEnv) {

    let cvar_on: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("GetCVarBool query should succeed");
    assert!(
        cvar_on,
        "The `loadDeprecationFallbacks` CVar must default to true (src/cvars.yaml:899 sets \
         '1') so the early-return guard at Deprecated_ActionBar.lua:4 doesn't bail before any \
         shim is defined. If this CVar flips to false, ALL 35 action-bar deprecated globals \
         are skipped and any legacy addon calling them blows up with `attempt to call a nil \
         value`"
    );
}
}

#[test]
fn blizzard_deprecated_action_bar_has_no_xml_or_other_assets() {
    let dir = blizzard_ui_dir().join("Blizzard_DeprecatedActionBar");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_DeprecatedActionBar dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let has_xml = entries.iter().any(|n| n.ends_with(".xml"));
    assert!(
        !has_xml,
        "Blizzard_DeprecatedActionBar has NO XML files — pure Lua function shims only. \
         Got entries: {entries:?}"
    );

    let has_runtime_shims = entries.iter().any(|n| n == "Deprecated_ActionBar.lua");
    assert!(
        has_runtime_shims,
        "Blizzard_DeprecatedActionBar should ship `Deprecated_ActionBar.lua` (the runtime \
         shim definitions for the 35 deprecated action-bar globals)"
    );
}
