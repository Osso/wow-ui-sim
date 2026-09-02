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

fn deprecated_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Deprecated/Blizzard_Deprecated.toc")
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
fn blizzard_deprecated_toc_is_minimal_with_no_flags_or_deps() {
    let toc = TocFile::from_file(&deprecated_toc()).expect("Blizzard_Deprecated TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_Deprecated is non-LOD — the deprecated-fallback shims must install before any \
         legacy addon Lua executes (otherwise GetBattlefieldScore / GetBattlefieldStatData / \
         UnitIsSpellTarget / C_SpellBook.GetSpellBookItemLossOfControlCooldown would be nil at \
         the call site)"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_Deprecated does not declare UseSecureEnvironment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_Deprecated declares NO dependencies — it only redirects to C_PvP.GetScoreInfo \
         / PlayerIsSpellTarget / C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo, all of \
         which live in the always-loaded FrameXML/SharedXML base"
    );

    let toc_text =
        std::fs::read_to_string(deprecated_toc()).expect("Blizzard_Deprecated TOC should read");
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_Deprecated does NOT declare an explicit `## AllowLoad:` flag — the loader \
         (src/toc.rs:311) treats a missing AllowLoad as Game-screen-only, which matches the \
         deprecation semantics: the legacy in-game globals don't need to exist on the \
         Login/Glue screens"
    );
    assert!(
        !toc_text.contains("## AllowLoadGameType:"),
        "Blizzard_Deprecated does NOT declare `## AllowLoadGameType:` so the deprecated shims \
         install on every game type (mainline + standard) without restriction"
    );
}

#[test]
fn blizzard_deprecated_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_Deprecated");
    assert!(
        in_game,
        "Blizzard_Deprecated (no AllowLoad flag, defaults to Game-only via src/toc.rs:311) \
         should appear in Game-screen auto-discovery so the deprecated globals are installed \
         before any addon Lua references them"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_Deprecated");
    assert!(
        !in_login,
        "Blizzard_Deprecated should NOT appear on the Login / glue screens — the deprecated \
         globals (GetBattlefieldScore / UnitIsSpellTarget / etc.) are in-game-only API"
    );
}

prefork_full_ui_case! {
fn blizzard_deprecated_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("Deprecated_12_0_1") || message.contains("Deprecated/"))
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_Deprecated emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_installs_battlefield_score_shim(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(GetBattlefieldScore) == 'function' \
                and type(GetBattlefieldStatData) == 'function'",
        )
        .expect("GetBattlefieldScore / GetBattlefieldStatData query should succeed");
    assert!(
        installed,
        "Blizzard_Deprecated/Deprecated_12_0_1.lua line 8+52 should publish global functions \
         `GetBattlefieldScore(playerIndex)` and `GetBattlefieldStatData(playerIndex, statIndex)` \
         — both unpack the new C_PvP.GetScoreInfo struct table back to the legacy 17-return / \
         single-stat positional return shape. The CVar `loadDeprecationFallbacks` defaults to \
         '1' (src/cvars.yaml:899) so the fallback file body executes"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_installs_unit_is_spell_target_shim(env: &WowLuaEnv) {

    let installed: bool = env
        .eval("return type(UnitIsSpellTarget) == 'function'")
        .expect("UnitIsSpellTarget query should succeed");
    assert!(
        installed,
        "Blizzard_Deprecated/Deprecated_12_0_1.lua line 67 should publish a global \
         `UnitIsSpellTarget(unit, target)` shim that delegates to PlayerIsSpellTarget(unit) \
         when target == 'player' and returns false otherwise. The new API only supports the \
         player-perspective query, so legacy callers that asked about other targets get a \
         silent false"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_installs_spellbook_loss_of_control_shim(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(C_SpellBook) == 'table' \
                and type(C_SpellBook.GetSpellBookItemLossOfControlCooldown) == 'function'",
        )
        .expect("C_SpellBook.GetSpellBookItemLossOfControlCooldown query should succeed");
    assert!(
        installed,
        "Blizzard_Deprecated/Deprecated_12_0_1.lua line 75 should publish \
         `C_SpellBook.GetSpellBookItemLossOfControlCooldown(spellBookItem)` that unpacks the \
         new `GetSpellBookItemLossOfControlCooldownInfo` struct (`startTime`, `duration`) back \
         to the legacy two-return positional shape"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_unit_is_spell_target_returns_false_for_non_player_target(env: &WowLuaEnv) {

    let result: bool = env
        .eval("return UnitIsSpellTarget('player', 'target')")
        .expect("UnitIsSpellTarget('player', 'target') should succeed");
    assert!(
        !result,
        "UnitIsSpellTarget shim should return false when target ~= 'player' — the new C-API \
         only supports the player-perspective query, so legacy callers that asked about other \
         targets get a silent false (Deprecated_12_0_1.lua line 67-72)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_load_deprecation_fallbacks_cvar_is_default_on(env: &WowLuaEnv) {

    let cvar_on: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("GetCVarBool query should succeed");
    assert!(
        cvar_on,
        "The `loadDeprecationFallbacks` CVar must default to true (src/cvars.yaml:899 sets \
         '1') so the early-return guard at Deprecated_12_0_1.lua:4 doesn't bail before the \
         shim definitions. If this CVar flips to false, ALL deprecated globals \
         (GetBattlefieldScore / GetBattlefieldStatData / UnitIsSpellTarget / \
         C_SpellBook.GetSpellBookItemLossOfControlCooldown) are skipped and any addon still \
         calling them blows up with `attempt to call a nil value`"
    );
}
}

#[test]
fn blizzard_deprecated_toc_lists_current_profile_scoped_shims() {
    let dir = blizzard_ui_dir().join("Blizzard_Deprecated");
    let mut entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_Deprecated dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    entries.sort();

    let has_xml = entries.iter().any(|n| n.ends_with(".xml"));
    assert!(
        !has_xml,
        "Blizzard_Deprecated has NO XML files — it only ships function-shim Lua. Got entries: \
         {entries:?}"
    );

    assert_eq!(
        entries,
        vec![
            "11_0_0_SpellBookAPITransitionGuide.lua",
            "Blizzard_Deprecated.toc",
            "Classic",
            "Mainline",
            "Shared",
        ],
        "Retail 12.1.0.69497 ships the transition guide, TOC, and three profile-scoped source \
         directories"
    );

    let toc = TocFile::from_file(&deprecated_toc()).expect("Blizzard_Deprecated TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Mainline/Deprecated_12_0_1.lua",
            "Mainline/Deprecated_12_0_5.lua",
            "Mainline/Deprecated_12_0_7.lua",
            "Mainline/Deprecated_12_1_0.lua",
            "Shared/Deprecated_12_1_0.lua",
        ],
        "Retail TOC parsing keeps active mainline and shared deprecated shims while filtering \
         classic-only entries"
    );
}
