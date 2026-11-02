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

fn deprecated_specialization_toc() -> PathBuf {
    blizzard_ui_dir()
        .join("Blizzard_DeprecatedSpecialization/Blizzard_DeprecatedSpecialization.toc")
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
fn blizzard_deprecated_specialization_toc_declares_classic_and_standard_game_types() {
    let toc = TocFile::from_file(&deprecated_specialization_toc())
        .expect("Blizzard_DeprecatedSpecialization TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_DeprecatedSpecialization declares no `## LoadOnDemand: 1` — the legacy \
         specialization globals must install before any talent / class-spec addon Lua \
         executes that calls GetSpecialization / GetSpecializationInfo / GetTalentInfo / \
         GetSpecializationMasterySpells / MAX_TALENT_TIERS"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_DeprecatedSpecialization does not declare UseSecureEnvironment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_DeprecatedSpecialization declares NO dependencies — both Lua files forward \
         to C_SpecializationInfo (registered at src/c_api/c_spec.rs:32) and Constants (the \
         auto-creating-empty-table namespace at src/lua_api/env_init/enums.rs:32-43), both \
         of which are bootstrapped at env init"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_DeprecatedSpecialization declares `## AllowLoadGameType: classic, standard` \
         which INCLUDES `standard` (mainline retail) — the addon is NOT game-type restricted \
         on mainline (src/toc.rs:294 returns false when ANY listed type matches \
         `mainline`/`standard`)"
    );

    let toc_text = std::fs::read_to_string(deprecated_specialization_toc())
        .expect("Blizzard_DeprecatedSpecialization TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: classic, standard"),
        "Blizzard_DeprecatedSpecialization TOC must declare `## AllowLoadGameType: classic, \
         standard` — this is the FIRST deprecation shim that declares an explicit game-type \
         filter at the addon level. Both classic and standard listed allows the same shim \
         binary to load on classic and retail clients"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_DeprecatedSpecialization omits `## AllowLoad:` — defaults to Game-screen-only \
         (src/toc.rs:311), matching the legacy Specialization API in-game-only surface"
    );
}

#[test]
fn blizzard_deprecated_specialization_toc_lists_game_template_and_talent_consts() {
    let toc = TocFile::from_file(&deprecated_specialization_toc())
        .expect("Blizzard_DeprecatedSpecialization TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert_eq!(
        files.len(),
        2,
        "Blizzard_DeprecatedSpecialization should list exactly 2 files: the `[Game]`-template \
         specialization variant and Deprecated_TalentConsts.lua. Got: {files:?}"
    );
    assert!(
        files
            .iter()
            .any(|f| f == "Deprecated_Specialization_Standard.lua"),
        "First entry `Deprecated_Specialization_[Game].lua` should resolve to \
         `Deprecated_Specialization_Standard.lua` via the `[Game]` template substitution \
         (src/toc.rs:146 replaces `[Game]` with `Standard` for mainline retail). Got: \
         {files:?}"
    );
    assert!(
        files.iter().any(|f| f == "Deprecated_TalentConsts.lua"),
        "Second entry should be `Deprecated_TalentConsts.lua` (the talent constants module). \
         Got: {files:?}"
    );
}

#[test]
fn blizzard_deprecated_specialization_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedSpecialization");
    assert!(
        in_game,
        "Blizzard_DeprecatedSpecialization (no AllowLoad flag, defaults to Game-only; \
         AllowLoadGameType includes `standard`) should appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedSpecialization");
    assert!(
        !in_login,
        "Blizzard_DeprecatedSpecialization should NOT appear on the Login / glue screens — \
         specialization / talent management is an in-game concept"
    );
}

prefork_full_ui_case! {
fn blizzard_deprecated_specialization_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("DeprecatedSpecialization")
                || message.contains("Deprecated_Specialization")
                || message.contains("Deprecated_TalentConsts")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DeprecatedSpecialization emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_specialization_installs_four_direct_aliases_as_functions(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(GetNumSpecializationsForClassID) == 'function' \
                and type(GetSpecializationInfo) == 'function' \
                and type(GetSpecialization) == 'function' \
                and type(GetActiveSpecGroup) == 'function'",
        )
        .expect("4-direct-alias function-installation query should succeed");
    assert!(
        installed,
        "Deprecated_Specialization_Standard.lua lines 10/13/16/19 publish 4 direct global \
         aliases (no closure wrapping — plain right-hand-side reads of \
         C_SpecializationInfo.<Method>): GetNumSpecializationsForClassID, \
         GetSpecializationInfo, GetSpecialization, GetActiveSpecGroup. All 4 backing methods \
         are explicitly registered Rust impls (src/c_api/c_spec.rs:37/41/53/72), so each \
         global is a real function value — not a no-op closure"
    );
}
}

#[test]
fn specialization_info_for_class_id_returns_nothing_past_class_specs() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    let returns_nothing_past_end: bool = env
        .eval(
            r##"
            local count = C_SpecializationInfo.GetNumSpecializationsForClassID(2)
            local lastSpecID = GetSpecializationInfoForClassID(2, count)
            local pastEndReturns = select("#", GetSpecializationInfoForClassID(2, count + 1))
            return count > 0 and lastSpecID ~= nil and pastEndReturns == 0
            "##,
        )
        .expect("specialization class lookup contract should be queryable");

    assert!(
        returns_nothing_past_end,
        "GetSpecializationInfoForClassID is documented mayreturnnothing and addons such as \
         Syndicator iterate until it returns nil/no values"
    );
}

prefork_full_ui_case! {
fn blizzard_deprecated_specialization_direct_aliases_are_identity_equal_to_c_spec_info(env: &WowLuaEnv) {

    let aliases_match: bool = env
        .eval(
            "return GetNumSpecializationsForClassID == \
                    C_SpecializationInfo.GetNumSpecializationsForClassID \
                and GetSpecializationInfo == C_SpecializationInfo.GetSpecializationInfo \
                and GetSpecialization == C_SpecializationInfo.GetSpecialization \
                and GetActiveSpecGroup == C_SpecializationInfo.GetActiveSpecGroup",
        )
        .expect("identity-equality query for 4 direct aliases should succeed");
    assert!(
        aliases_match,
        "Each of the 4 direct globals must be IDENTITY-EQUAL to its backing \
         C_SpecializationInfo method. The shim performs plain table-read assignment \
         (`GetSpecialization = C_SpecializationInfo.GetSpecialization`) so both sides \
         literally reference the SAME registered Rust closure value"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_specialization_installs_two_wrapper_closures(env: &WowLuaEnv) {

    let wrappers_installed: bool = env
        .eval(
            "return type(GetSpecializationMasterySpells) == 'function' \
                and type(GetTalentInfo) == 'function'",
        )
        .expect("2-wrapper-closure installation query should succeed");
    assert!(
        wrappers_installed,
        "Deprecated_Specialization_Standard.lua lines 22-33 / 36-51 install 2 LOCALLY-DEFINED \
         wrapper closures (NOT direct aliases): GetSpecializationMasterySpells \
         (table-to-2-multireturn) and GetTalentInfo (query-table-to-11-multireturn). Both \
         are plain Lua function values written by the shim itself"
    );

    let not_identity_equal: bool = env
        .eval(
            "return GetSpecializationMasterySpells ~= \
                    C_SpecializationInfo.GetSpecializationMasterySpells \
                and GetTalentInfo ~= C_SpecializationInfo.GetTalentInfo",
        )
        .expect("non-identity-equality query for wrappers should succeed");
    assert!(
        not_identity_equal,
        "Both wrapper closures must NOT be identity-equal to their backing \
         C_SpecializationInfo methods. Unlike the 4 direct aliases, these are fresh Lua \
         closures created in Deprecated_Specialization_Standard.lua — distinct function \
         values from whatever the namespace `__index` materializes for \
         C_SpecializationInfo.GetSpecializationMasterySpells / .GetTalentInfo"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_specialization_get_specialization_mastery_spells_returns_spell_ids(env: &WowLuaEnv) {

    let (first, second_is_nil): (i32, bool) = env
        .eval(
            "local first, second = GetSpecializationMasterySpells(1, false, false) \
             return first, second == nil",
        )
        .expect("deprecated wrapper should run without error");
    assert_eq!(
        first, 183997,
        "spec index 1 for the default Paladin is Holy, whose mastery is Lightbringer (183997), \
         served by the modeled C_SpecializationInfo.GetSpecializationMasterySpells"
    );
    assert!(second_is_nil, "Holy Paladin has a single mastery spell");
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_specialization_get_talent_info_returns_nil_when_backing_returns_nil(env: &WowLuaEnv) {

    let returned_nil: bool = env
        .eval(
            "do \
                local ok, result = pcall(GetTalentInfo, 1, 1, 1, false, nil) \
                return ok and result == nil \
            end",
        )
        .expect("pcall probe of GetTalentInfo should succeed");
    assert!(
        returned_nil,
        "Calling GetTalentInfo must NOT throw and must return nil. The wrapper at lines \
         36-51 builds a query table, calls C_SpecializationInfo.GetTalentInfo(query) \
         (UNSTUBBED — resolves via namespace `__index` to a no-op closure returning nil), \
         then hits the `if not talentInfo then return nil end` early-return at lines 44-46. \
         pcall returns ok=true, result=nil — unlike GetSpecializationMasterySpells, this \
         wrapper has a guard that handles the unstubbed case gracefully"
    );
}
}

prefork_full_ui_case! {
fn deprecated_talent_query_validates_arguments_and_empty_grid(env: &WowLuaEnv) {
    env.exec(r#"
        assert(GetTalentInfo(1, 1, 1, false, nil) == nil)
        assert(GetTalentInfo(99, 9, 1, false, nil) == nil)
        assert(not pcall(C_SpecializationInfo.GetTalentInfo, nil))
        assert(not pcall(C_SpecializationInfo.GetTalentInfo, {}))
        assert(not pcall(C_SpecializationInfo.GetTalentInfo, { tier = 1 }))
        assert(not pcall(C_SpecializationInfo.GetTalentInfo, { tier = -1, column = 1 }))
        assert(C_SpecializationInfo.GetTalentInfo({ tier = 0, column = 1 }) == nil)
    "#).expect("original deprecated wrapper and query validation should execute");
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_specialization_talent_constants_are_populated(env: &WowLuaEnv) {

    let (max_talent_tiers, num_talent_columns): (i32, i32) = env
        .eval("return MAX_TALENT_TIERS, NUM_TALENT_COLUMNS")
        .expect("talent-constants query should succeed");
    assert_eq!(max_talent_tiers, 7);
    assert_eq!(num_talent_columns, 3);
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_specialization_load_deprecation_fallbacks_cvar_is_default_on(env: &WowLuaEnv) {

    let cvar_on: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("GetCVarBool query should succeed");
    assert!(
        cvar_on,
        "The `loadDeprecationFallbacks` CVar must default to true (src/cvars.yaml:899 sets \
         '1') so the early-return guards at Deprecated_Specialization_Standard.lua:4 and \
         Deprecated_TalentConsts.lua:4 don't bail before the 6 globals (4 direct aliases + \
         2 wrappers) and 2 talent constants are defined"
    );
}
}

#[test]
fn blizzard_deprecated_specialization_ships_six_game_variants_plus_talent_consts() {
    let dir = blizzard_ui_dir().join("Blizzard_DeprecatedSpecialization");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_DeprecatedSpecialization dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let has_xml = entries.iter().any(|n| n.ends_with(".xml"));
    assert!(
        !has_xml,
        "Blizzard_DeprecatedSpecialization has NO XML files — pure Lua function-shim and \
         constants definitions only. Got entries: {entries:?}"
    );

    for variant in [
        "Deprecated_Specialization_Standard.lua",
        "Deprecated_Specialization_Cata.lua",
        "Deprecated_Specialization_Mists.lua",
        "Deprecated_Specialization_TBC.lua",
        "Deprecated_Specialization_Vanilla.lua",
        "Deprecated_Specialization_Wrath.lua",
        "Deprecated_TalentConsts.lua",
    ] {
        assert!(
            entries.iter().any(|n| n == variant),
            "Blizzard_DeprecatedSpecialization should ship `{variant}` — the addon ships \
             6 game-variant specialization files (Standard, Cata, Mists, TBC, Vanilla, \
             Wrath) plus the shared TalentConsts module. The TOC's `[Game]` template \
             selects exactly one variant at load time per client. Got entries: {entries:?}"
        );
    }
}
