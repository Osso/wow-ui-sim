#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn deprecated_spell_script_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_DeprecatedSpellScript/Blizzard_DeprecatedSpellScript.toc")
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
fn blizzard_deprecated_spell_script_toc_is_minimal_with_no_flags_or_deps() {
    let toc = TocFile::from_file(&deprecated_spell_script_toc())
        .expect("Blizzard_DeprecatedSpellScript TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_DeprecatedSpellScript declares `## LoadOnDemand: 0` — the 7 spell-script \
         globals (TargetSpellReplacesBonusTree / GetMaxSpellStartRecoveryOffset / \
         GetSpellQueueWindow / GetSchoolString / SpellIsPriorityAura / SpellIsSelfBuff / \
         SpellGetVisibilityInfo) plus the in-place C_Spell.GetSpellLossOfControlCooldown \
         method must install before any spell-frame addon Lua executes that calls them"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_DeprecatedSpellScript does not declare UseSecureEnvironment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_DeprecatedSpellScript declares NO dependencies — every shim forwards to \
         the C_Spell namespace (registered at src/c_api/c_spell.rs via SPELL_VALUE_METHODS / \
         SPELL_BOOLEAN_METHODS / SPELL_TARGET_METHODS) plus the Enum.SpellAuraVisibilityType \
         enum (populated at src/lua_api/globals/enum_data/combat_system.rs:212), all \
         bootstrapped at env init"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_DeprecatedSpellScript declares no `## AllowLoadGameType:` filter — loads \
         on every game type without restriction"
    );

    let toc_text = std::fs::read_to_string(deprecated_spell_script_toc())
        .expect("Blizzard_DeprecatedSpellScript TOC should read");
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_DeprecatedSpellScript omits `## AllowLoad:` — defaults to Game-screen-only \
         (src/toc.rs:311), matching the legacy spell-script API in-game-only surface"
    );
    assert!(
        !toc_text.contains("## AllowLoadGameType:"),
        "Blizzard_DeprecatedSpellScript omits `## AllowLoadGameType:` — single-file shim is \
         universal across game types"
    );
}

#[test]
fn blizzard_deprecated_spell_script_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedSpellScript");
    assert!(
        in_game,
        "Blizzard_DeprecatedSpellScript (no AllowLoad flag, defaults to Game-only) should \
         appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedSpellScript");
    assert!(
        !in_login,
        "Blizzard_DeprecatedSpellScript should NOT appear on the Login / glue screens — \
         spell-script management is an in-game concept"
    );
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("DeprecatedSpellScript") || message.contains("Deprecated_SpellScript")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DeprecatedSpellScript emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_installs_four_direct_aliases(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(TargetSpellReplacesBonusTree) == 'function' \
                and type(GetMaxSpellStartRecoveryOffset) == 'function' \
                and type(GetSpellQueueWindow) == 'function' \
                and type(GetSchoolString) == 'function'",
        )
        .expect("4-direct-alias function-installation query should succeed");
    assert!(
        installed,
        "Deprecated_SpellScript.lua lines 9-12 publish 4 direct global aliases (no closure \
         wrapping — plain right-hand-side reads of C_Spell.<Method>): \
         TargetSpellReplacesBonusTree (temporary target-spell metadata default, returns false); \
         GetMaxSpellStartRecoveryOffset → C_Spell.GetSpellQueueWindow; GetSpellQueueWindow → \
         C_Spell.GetSpellQueueWindow (both globals share the same registered CVar-backed \
         function); GetSchoolString (registered at c_spell.rs, returns localized school \
         name string)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_direct_aliases_are_identity_equal_to_c_spell(env: &WowLuaEnv) {

    let aliases_match: bool = env
        .eval(
            r#"
            local originalValue = GetCVar("SpellQueueWindow")
            local defaultValue = C_Spell.GetSpellQueueWindow()
            SetCVar("SpellQueueWindow", "250")
            local updatedValue = C_Spell.GetSpellQueueWindow()
            SetCVar("SpellQueueWindow", originalValue)

            return TargetSpellReplacesBonusTree == C_Spell.TargetSpellReplacesBonusTree
                and GetMaxSpellStartRecoveryOffset == C_Spell.GetSpellQueueWindow
                and GetSpellQueueWindow == C_Spell.GetSpellQueueWindow
                and GetSchoolString == C_Spell.GetSchoolString
                and type(defaultValue) == "number"
                and defaultValue == 400
                and updatedValue == 250
            "#,
        )
        .expect("identity and CVar-backed queue-window query should succeed");
    assert!(
        aliases_match,
        "Direct globals must be identity-equal to their backing C_Spell methods, and \
         C_Spell.GetSpellQueueWindow must return the current numeric SpellQueueWindow CVar"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_get_spell_queue_window_aliases_share_the_same_closure(env: &WowLuaEnv) {

    let same_closure: bool = env
        .eval("return GetMaxSpellStartRecoveryOffset == GetSpellQueueWindow")
        .expect("identity-equality query between two queue-window aliases should succeed");
    assert!(
        same_closure,
        "GetMaxSpellStartRecoveryOffset and GetSpellQueueWindow must reference the EXACT \
         SAME function value — both are aliased to the registered \
         C_Spell.GetSpellQueueWindow function on lines 10 and 11. Both globals therefore \
         reference the same CVar-backed closure"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_installs_three_wrapper_closures(env: &WowLuaEnv) {

    let wrappers_installed: bool = env
        .eval(
            "return type(SpellIsPriorityAura) == 'function' \
                and type(SpellIsSelfBuff) == 'function' \
                and type(SpellGetVisibilityInfo) == 'function'",
        )
        .expect("3-wrapper-closure installation query should succeed");
    assert!(
        wrappers_installed,
        "Deprecated_SpellScript.lua lines 16-33 install 3 LOCALLY-DEFINED wrapper closures \
         (NOT direct aliases): SpellIsPriorityAura (1-line delegation to \
         C_Spell.IsPriorityAura, registered at c_spell.rs:106 returning false); \
         SpellIsSelfBuff (1-line delegation to C_Spell.IsSelfBuff, registered at \
         c_spell.rs:107 returning bool based on implicit_target); SpellGetVisibilityInfo \
         (string→Enum lookup wrapper that maps `\"RAID_INCOMBAT\"` / `\"RAID_OUTOFCOMBAT\"` / \
         `\"ENEMY_TARGET\"` to Enum.SpellAuraVisibilityType variants \
         (RaidInCombat / RaidOutOfCombat / EnemyTarget) before delegating to \
         the temporary C_Spell.GetVisibilityInfo visibility shim)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_wrapper_closures_are_not_identity_equal_to_c_spell(env: &WowLuaEnv) {

    let not_identity_equal: bool = env
        .eval(
            "return SpellIsPriorityAura ~= C_Spell.IsPriorityAura \
                and SpellIsSelfBuff ~= C_Spell.IsSelfBuff \
                and SpellGetVisibilityInfo ~= C_Spell.GetVisibilityInfo",
        )
        .expect("non-identity-equality query for 3 wrappers should succeed");
    assert!(
        not_identity_equal,
        "All 3 wrapper closures must NOT be identity-equal to their backing C_Spell methods \
         — each is a fresh Lua closure created in Deprecated_SpellScript.lua via \
         `function GlobalName(...) ... end` syntax. Even SpellIsPriorityAura and \
         SpellIsSelfBuff (which look like trivial 1-call passthroughs) are still distinct \
         function values from the registered Rust impls"
    );
}
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_visibility_wrapper_translates_string_keys(env: &WowLuaEnv) {
    let translated: bool = env
        .eval(
            "local observedType \
             local original = C_Spell.GetVisibilityInfo \
             C_Spell.GetVisibilityInfo = function(spellID, visibilityType) \
                 observedType = visibilityType \
                 return original(spellID, visibilityType) \
             end \
             SpellGetVisibilityInfo(0, 'RAID_INCOMBAT') \
             return observedType == Enum.SpellAuraVisibilityType.RaidInCombat",
        )
        .expect("string-keyed visibility-info call should succeed");
    assert!(
        translated,
        "The selected Retail wrapper translates 'RAID_INCOMBAT' to \
         Enum.SpellAuraVisibilityType.RaidInCombat before calling C_Spell.GetVisibilityInfo"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_installs_get_spell_loss_of_control_cooldown_on_c_spell(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(C_Spell.GetSpellLossOfControlCooldown)")
        .expect("type(C_Spell.GetSpellLossOfControlCooldown) query should succeed");
    assert_eq!(
        kind, "function",
        "Deprecated_SpellScript.lua line 38 declares \
         `function C_Spell.GetSpellLossOfControlCooldown(spellIdentifier) ... end` — \
         installing the legacy method directly on the C_Spell namespace (an in-place \
         namespace mutation, unlike the 4 direct global aliases or 3 global wrappers \
         above). The replacement closure delegates to C_Spell.GetSpellLossOfControlCooldownInfo \
         (registered at c_spell.rs:98) and destructures the returned table into the legacy \
         (startTime, duration) 2-multireturn shape"
    );

    let no_global_leak: bool = env
        .eval("return _G.GetSpellLossOfControlCooldown == nil")
        .expect("global-namespace probe should succeed");
    assert!(
        no_global_leak,
        "The C_Spell.GetSpellLossOfControlCooldown method must NOT leak as a `_G` global. \
         Unlike the 4 aliases / 3 wrappers above which publish to `_G`, this entry is an \
         in-place mutation of the C_Spell namespace ONLY"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_loss_of_control_returns_nothing_for_unknown_spell_id(env: &WowLuaEnv) {

    let returned_nothing: bool = env
        .eval(
            "do \
                local startTime, duration = C_Spell.GetSpellLossOfControlCooldown(0) \
                return startTime == nil and duration == nil \
            end",
        )
        .expect("LoC cooldown invocation query should succeed");
    assert!(
        returned_nothing,
        "Calling C_Spell.GetSpellLossOfControlCooldown(0) must return all nils because the \
         underlying C_Spell.GetSpellLossOfControlCooldownInfo (c_spell.rs:628-654) returns \
         nil when SimState.spell_loss_of_control map has no entry for the spell_id (the \
         default — empty map). With `lossOfControlInfo == nil`, the `if (info) then` branch \
         at line 41 of the shim is skipped and the wrapper returns nothing — both \
         destructured locals are nil"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_script_load_deprecation_fallbacks_cvar_is_default_on(env: &WowLuaEnv) {

    let cvar_on: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("GetCVarBool query should succeed");
    assert!(
        cvar_on,
        "The `loadDeprecationFallbacks` CVar must default to true (src/cvars.yaml:899 sets \
         '1') so the early-return guard at Deprecated_SpellScript.lua:4 doesn't bail before \
         the 7 globals (4 direct aliases + 3 wrappers) and the in-place \
         C_Spell.GetSpellLossOfControlCooldown mutation are applied"
    );
}
}

#[test]
fn blizzard_deprecated_spell_script_has_no_xml_or_other_assets() {
    let dir = blizzard_ui_dir().join("Blizzard_DeprecatedSpellScript");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_DeprecatedSpellScript dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let has_xml = entries.iter().any(|n| n.ends_with(".xml"));
    assert!(
        !has_xml,
        "Blizzard_DeprecatedSpellScript has NO XML files — pure Lua single-file definitions \
         only. Got entries: {entries:?}"
    );

    let has_runtime_shims = entries.iter().any(|n| n == "Deprecated_SpellScript.lua");
    assert!(
        has_runtime_shims,
        "Blizzard_DeprecatedSpellScript should ship `Deprecated_SpellScript.lua` (the \
         runtime shim definitions for the 7 deprecated spell-script globals + the in-place \
         C_Spell.GetSpellLossOfControlCooldown method)"
    );
}
