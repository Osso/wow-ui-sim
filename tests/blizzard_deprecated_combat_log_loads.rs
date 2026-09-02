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

fn deprecated_combat_log_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_DeprecatedCombatLog/Blizzard_DeprecatedCombatLog.toc")
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
fn blizzard_deprecated_combat_log_toc_has_combat_log_base_dependency() {
    let toc = TocFile::from_file(&deprecated_combat_log_toc())
        .expect("Blizzard_DeprecatedCombatLog TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_DeprecatedCombatLog declares `## LoadOnDemand: 0` — the CombatLog* / \
         COMBATLOG_OBJECT_* / DeathRecap_* globals must install before any combat-tracking \
         addon Lua executes that calls them"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_DeprecatedCombatLog does not declare UseSecureEnvironment"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_CombatLogBase".to_string()],
        "Retail 12.1.0.69497 declares `## Dependencies: Blizzard_CombatLogBase`, which \
         provides CombatLogUtil before Deprecated_CombatLog.lua installs its aliases"
    );

    let toc_text = std::fs::read_to_string(deprecated_combat_log_toc())
        .expect("Blizzard_DeprecatedCombatLog TOC should read");
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_DeprecatedCombatLog omits `## AllowLoad:` — defaults to Game-screen-only \
         (src/toc.rs:311), matching the legacy combat-log API in-game-only surface"
    );
    assert!(
        !toc_text.contains("## AllowLoadGameType:"),
        "Blizzard_DeprecatedCombatLog omits `## AllowLoadGameType:` so the shims install on \
         every game type without restriction"
    );
}

#[test]
fn blizzard_deprecated_combat_log_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedCombatLog");
    assert!(
        in_game,
        "Blizzard_DeprecatedCombatLog (no AllowLoad flag, defaults to Game-only) should \
         appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedCombatLog");
    assert!(
        !in_login,
        "Blizzard_DeprecatedCombatLog should NOT appear on the Login / glue screens — combat \
         log is an in-game concept"
    );
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("DeprecatedCombatLog") || message.contains("Deprecated_CombatLog")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DeprecatedCombatLog emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_installs_c_combat_log_function_aliases(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return CombatLog_Object_IsA == C_CombatLog.DoesObjectMatchFilter \
                and CombatLogAddFilter == C_CombatLog.AddEventFilter \
                and CombatLogClearEntries == C_CombatLog.ClearEntries \
                and CombatLogGetCurrentEntry == C_CombatLog.GetCurrentEntryInfo \
                and CombatLogGetCurrentEventInfo == C_CombatLog.GetCurrentEventInfo \
                and CombatLogGetNumEntries == C_CombatLog.GetEntryCount \
                and CombatLogGetRetentionTime == C_CombatLog.GetEntryRetentionTime \
                and CombatLogResetFilter == C_CombatLog.ClearEventFilters \
                and CombatLogSetRetentionTime == C_CombatLog.SetEntryRetentionTime \
                and CombatLogShowCurrentEntry == C_CombatLog.ShouldShowCurrentEntry",
        )
        .expect("C_CombatLog alias query should succeed");
    assert!(
        installed,
        "Deprecated_CombatLog.lua line 14-23 should publish 10 legacy C_CombatLog aliases as \
         direct table reference assignments: CombatLog_Object_IsA → DoesObjectMatchFilter, \
         CombatLogAddFilter → AddEventFilter, CombatLogClearEntries → ClearEntries, \
         CombatLogGetCurrentEntry → GetCurrentEntryInfo, CombatLogGetCurrentEventInfo → \
         GetCurrentEventInfo, CombatLogGetNumEntries → GetEntryCount, \
         CombatLogGetRetentionTime → GetEntryRetentionTime, CombatLogResetFilter → \
         ClearEventFilters, CombatLogSetRetentionTime → SetEntryRetentionTime, \
         CombatLogShowCurrentEntry → ShouldShowCurrentEntry"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_installs_combat_text_and_death_recap_aliases(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return CombatTextSetActiveUnit == C_CombatText.SetActiveUnit \
                and GetCurrentCombatTextEventInfo == C_CombatText.GetCurrentEventInfo \
                and DeathRecap_GetEvents == C_DeathRecap.GetRecapEvents \
                and DeathRecap_HasEvents == C_DeathRecap.HasRecapEvents \
                and GetDeathRecapLink == C_DeathRecap.GetRecapLink",
        )
        .expect("C_CombatText / C_DeathRecap alias query should succeed");
    assert!(
        installed,
        "Deprecated_CombatLog.lua line 25-30 should publish 5 legacy aliases: \
         CombatTextSetActiveUnit → C_CombatText.SetActiveUnit, \
         GetCurrentCombatTextEventInfo → C_CombatText.GetCurrentEventInfo, \
         DeathRecap_GetEvents → C_DeathRecap.GetRecapEvents, DeathRecap_HasEvents → \
         C_DeathRecap.HasRecapEvents, GetDeathRecapLink → C_DeathRecap.GetRecapLink"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_installs_combat_log_object_constants(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return COMBATLOG_OBJECT_AFFILIATION_MINE == Enum.CombatLogObject.AffiliationMine \
                and COMBATLOG_OBJECT_AFFILIATION_PARTY == Enum.CombatLogObject.AffiliationParty \
                and COMBATLOG_OBJECT_AFFILIATION_RAID == Enum.CombatLogObject.AffiliationRaid \
                and COMBATLOG_OBJECT_AFFILIATION_OUTSIDER \
                    == Enum.CombatLogObject.AffiliationOutsider \
                and COMBATLOG_OBJECT_REACTION_FRIENDLY == Enum.CombatLogObject.ReactionFriendly \
                and COMBATLOG_OBJECT_REACTION_NEUTRAL == Enum.CombatLogObject.ReactionNeutral \
                and COMBATLOG_OBJECT_REACTION_HOSTILE == Enum.CombatLogObject.ReactionHostile \
                and COMBATLOG_OBJECT_CONTROL_PLAYER == Enum.CombatLogObject.ControlPlayer \
                and COMBATLOG_OBJECT_CONTROL_NPC == Enum.CombatLogObject.ControlNpc \
                and COMBATLOG_OBJECT_TYPE_PLAYER == Enum.CombatLogObject.TypePlayer \
                and COMBATLOG_OBJECT_TYPE_NPC == Enum.CombatLogObject.TypeNpc \
                and COMBATLOG_OBJECT_TYPE_PET == Enum.CombatLogObject.TypePet \
                and COMBATLOG_OBJECT_TYPE_GUARDIAN == Enum.CombatLogObject.TypeGuardian \
                and COMBATLOG_OBJECT_TYPE_OBJECT == Enum.CombatLogObject.TypeObject \
                and COMBATLOG_OBJECT_TARGET == Enum.CombatLogObject.Target \
                and COMBATLOG_OBJECT_FOCUS == Enum.CombatLogObject.Focus \
                and COMBATLOG_OBJECT_MAINTANK == Enum.CombatLogObject.Maintank \
                and COMBATLOG_OBJECT_MAINASSIST == Enum.CombatLogObject.Mainassist \
                and COMBATLOG_OBJECT_NONE == Enum.CombatLogObject.None",
        )
        .expect("COMBATLOG_OBJECT_* enum-mirror query should succeed");
    assert!(
        installed,
        "Deprecated_CombatLog.lua line 39-57 should mirror 19 legacy COMBATLOG_OBJECT_* \
         globals from Enum.CombatLogObject (affiliation: MINE/PARTY/RAID/OUTSIDER, reaction: \
         FRIENDLY/NEUTRAL/HOSTILE, control: PLAYER/NPC, type: PLAYER/NPC/PET/GUARDIAN/OBJECT, \
         special: TARGET/FOCUS/MAINTANK/MAINASSIST/NONE) — backed by the EnumDef in \
         src/lua_api/globals/enum_data/combat_system.rs:61"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_installs_combat_log_object_masks(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return COMBATLOG_OBJECT_AFFILIATION_MASK \
                    == Constants.CombatLogObjectMasks.COMBATLOG_OBJECT_AFFILIATION_MASK \
                and COMBATLOG_OBJECT_REACTION_MASK \
                    == Constants.CombatLogObjectMasks.COMBATLOG_OBJECT_REACTION_MASK \
                and COMBATLOG_OBJECT_CONTROL_MASK \
                    == Constants.CombatLogObjectMasks.COMBATLOG_OBJECT_CONTROL_MASK \
                and COMBATLOG_OBJECT_TYPE_MASK \
                    == Constants.CombatLogObjectMasks.COMBATLOG_OBJECT_TYPE_MASK \
                and COMBATLOG_OBJECT_SPECIAL_MASK \
                    == Constants.CombatLogObjectMasks.COMBATLOG_OBJECT_SPECIAL_MASK \
                and COMBATLOG_OBJECT_RAIDTARGET_MASK \
                    == Constants.CombatLogObjectTargetMasks.COMBATLOG_OBJECT_RAID_TARGET_MASK \
                and COMBATLOG_OBJECT_RAID_MASK \
                    == Constants.CombatLogObjectTargetMasks.COMBATLOG_OBJECT_RAID_MASK",
        )
        .expect("COMBATLOG_OBJECT_*_MASK query should succeed");
    assert!(
        installed,
        "Deprecated_CombatLog.lua line 59-63 + 75-76 should mirror 7 bitmask globals from \
         Constants.CombatLogObjectMasks / Constants.CombatLogObjectTargetMasks: \
         COMBATLOG_OBJECT_AFFILIATION_MASK (15), COMBATLOG_OBJECT_REACTION_MASK (240), \
         COMBATLOG_OBJECT_CONTROL_MASK, COMBATLOG_OBJECT_TYPE_MASK, \
         COMBATLOG_OBJECT_SPECIAL_MASK, COMBATLOG_OBJECT_RAIDTARGET_MASK, \
         COMBATLOG_OBJECT_RAID_MASK — backed by src/lua_api/globals/enum_data/\
         constants_values.lua:70-77"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_installs_combat_log_raid_target_constants(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return COMBATLOG_OBJECT_RAIDTARGET1 == Enum.CombatLogObjectTarget.Raidtarget1 \
                and COMBATLOG_OBJECT_RAIDTARGET2 == Enum.CombatLogObjectTarget.Raidtarget2 \
                and COMBATLOG_OBJECT_RAIDTARGET3 == Enum.CombatLogObjectTarget.Raidtarget3 \
                and COMBATLOG_OBJECT_RAIDTARGET4 == Enum.CombatLogObjectTarget.Raidtarget4 \
                and COMBATLOG_OBJECT_RAIDTARGET5 == Enum.CombatLogObjectTarget.Raidtarget5 \
                and COMBATLOG_OBJECT_RAIDTARGET6 == Enum.CombatLogObjectTarget.Raidtarget6 \
                and COMBATLOG_OBJECT_RAIDTARGET7 == Enum.CombatLogObjectTarget.Raidtarget7 \
                and COMBATLOG_OBJECT_RAIDTARGET8 == Enum.CombatLogObjectTarget.Raidtarget8 \
                and COMBATLOG_OBJECT_RAID_NONE == Enum.CombatLogObjectTarget.RaidNone",
        )
        .expect("COMBATLOG_OBJECT_RAIDTARGET* query should succeed");
    assert!(
        installed,
        "Deprecated_CombatLog.lua line 65-73 should mirror 9 raid-target globals from \
         Enum.CombatLogObjectTarget: COMBATLOG_OBJECT_RAIDTARGET1..8 (powers-of-two flags 1, \
         2, 4, ..., 128) plus COMBATLOG_OBJECT_RAID_NONE (sentinel -2147483648). Backed by \
         the EnumDef in src/lua_api/globals/enum_data/combat_system.rs:87"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_installs_aura_type_strings_and_highlight_multiplier(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return AURA_TYPE_BUFF == 'BUFF' \
                and AURA_TYPE_DEBUFF == 'DEBUFF' \
                and COMBATLOG_HIGHLIGHT_MULTIPLIER == 1.5",
        )
        .expect("AURA_TYPE_* / COMBATLOG_HIGHLIGHT_MULTIPLIER query should succeed");
    assert!(
        installed,
        "Deprecated_CombatLog.lua line 82-85 should publish 3 inline string/number constants: \
         AURA_TYPE_BUFF = 'BUFF', AURA_TYPE_DEBUFF = 'DEBUFF', COMBATLOG_HIGHLIGHT_MULTIPLIER \
         = 1.5"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_installs_combat_log_util_function_aliases(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return Blizzard_CombatLog_BitToBraceCode == CombatLogUtil.GetRaidTargetBraceCode \
                and CombatLog_Color_HighlightColorArray == CombatLogUtil.HighlightColor \
                and CombatLog_String_DamageResultString \
                    == CombatLogUtil.GenerateDamageResultString \
                and CombatLog_String_GetIcon == CombatLogUtil.GetUnitIcon \
                and CombatLog_String_PowerType == CombatLogUtil.GetPowerTypeString \
                and CombatLog_String_SchoolString == CombatLogUtil.GetSpellSchoolString",
        )
        .expect("CombatLogUtil alias query should succeed");
    assert!(
        installed,
        "Deprecated_CombatLog.lua line 100-108 should publish 6 CombatLogUtil aliases that \
         remain as direct function references: Blizzard_CombatLog_BitToBraceCode → \
         GetRaidTargetBraceCode, CombatLog_Color_HighlightColorArray → HighlightColor, \
         CombatLog_String_DamageResultString → GenerateDamageResultString, \
         CombatLog_String_GetIcon → GetUnitIcon, CombatLog_String_PowerType → \
         GetPowerTypeString, CombatLog_String_SchoolString → GetSpellSchoolString. The \
         CombatLog_Color_ColorArray* aliases listed at line 101-103 are immediately \
         overwritten by full closure definitions at line 123-133 that thread \
         Blizzard_CombatLog_CurrentSettings as a default fallback"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_installs_color_string_closures(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(CombatLog_Color_FloatToText) == 'function' \
                and type(CombatLog_Color_ColorArrayByEventType) == 'function' \
                and type(CombatLog_Color_ColorArrayByUnitType) == 'function' \
                and type(CombatLog_Color_ColorArrayBySchool) == 'function' \
                and type(CombatLog_Color_ColorStringByEventType) == 'function' \
                and type(CombatLog_Color_ColorStringBySchool) == 'function' \
                and type(CombatLog_Color_ColorStringByUnitType) == 'function'",
        )
        .expect("Color-helper closure query should succeed");
    assert!(
        installed,
        "Deprecated_CombatLog.lua line 110-145 should publish 7 closure-defined color \
         helpers (NOT plain table aliases): CombatLog_Color_FloatToText (line 110 — accepts \
         either a {{r,g,b,a}} table or 4 scalars and produces an 8-char hex `aarrggbb` string \
         via clamping each channel to [0,1] then scaling to 255); the three \
         CombatLog_Color_ColorArray* re-definitions at line 123-133 that thread \
         Blizzard_CombatLog_CurrentSettings as a fallback for filterSettings; and the three \
         CombatLog_Color_ColorString* helpers at line 135-145 that compose FloatToText with \
         the corresponding ColorArray getter"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_color_float_to_text_packs_argb_hex(env: &WowLuaEnv) {

    let scalar_form: String = env
        .eval("return CombatLog_Color_FloatToText(1.0, 0.5, 0.25, 0.75)")
        .expect("FloatToText scalar form should succeed");
    let table_form: String = env
        .eval("return CombatLog_Color_FloatToText({r=1.0, g=0.5, b=0.25, a=0.75})")
        .expect("FloatToText table form should succeed");
    assert_eq!(
        scalar_form, table_form,
        "CombatLog_Color_FloatToText should accept either 4 scalars or a single \
         {{r,g,b,a}} table and produce identical output. Scalar={scalar_form}, \
         Table={table_form}"
    );
    assert_eq!(
        scalar_form, "bfff7f3f",
        "CombatLog_Color_FloatToText(1.0, 0.5, 0.25, 0.75) should pack into hex `aarrggbb` \
         by clamping each channel to [0,1] and scaling by 255 before flooring: a=floor(0.75 \
         * 255)=191=0xbf, r=floor(1.0 * 255)=255=0xff, g=floor(0.5 * 255)=127=0x7f, \
         b=floor(0.25 * 255)=63=0x3f → `bfff7f3f`"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_combat_log_load_deprecation_fallbacks_cvar_is_default_on(env: &WowLuaEnv) {

    let cvar_on: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("GetCVarBool query should succeed");
    assert!(
        cvar_on,
        "The `loadDeprecationFallbacks` CVar must default to true (src/cvars.yaml:899 sets \
         '1') so the early-return guard at Deprecated_CombatLog.lua:4 doesn't bail before \
         any shim is defined. If this CVar flips to false, ALL ~50 deprecated combat-log \
         globals are skipped"
    );
}
}

#[test]
fn blizzard_deprecated_combat_log_has_no_xml_or_other_assets() {
    let dir = blizzard_ui_dir().join("Blizzard_DeprecatedCombatLog");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_DeprecatedCombatLog dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let has_xml = entries.iter().any(|n| n.ends_with(".xml"));
    assert!(
        !has_xml,
        "Blizzard_DeprecatedCombatLog has NO XML files — pure Lua function-shim definitions \
         only. Got entries: {entries:?}"
    );

    let has_runtime_shims = entries.iter().any(|n| n == "Deprecated_CombatLog.lua");
    assert!(
        has_runtime_shims,
        "Blizzard_DeprecatedCombatLog should ship `Deprecated_CombatLog.lua` (the runtime \
         shim definitions for ~50 deprecated combat-log/text/death-recap globals)"
    );
}
