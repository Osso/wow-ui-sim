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

fn deprecated_spell_book_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_DeprecatedSpellBook/Blizzard_DeprecatedSpellBook.toc")
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
fn blizzard_deprecated_spell_book_toc_is_minimal_with_no_flags_or_deps() {
    let toc = TocFile::from_file(&deprecated_spell_book_toc())
        .expect("Blizzard_DeprecatedSpellBook TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_DeprecatedSpellBook declares `## LoadOnDemand: 0` — the IsPlayerSpell / \
         IsSpellKnown / IsSpellKnownOrOverridesKnown / FindFlyoutSlotBySpellID / \
         FindSpellOverrideByID / FindBaseSpellByID globals (and HUNTER_DISMISS_PET constant) \
         must install before any spellbook addon Lua executes that calls them"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_DeprecatedSpellBook does not declare UseSecureEnvironment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_DeprecatedSpellBook declares NO dependencies — every shim simply forwards \
         to the C_SpellBook namespace (registered at src/c_api/c_spell_book.rs:19) plus the \
         Constants.SpellBookSpellIDs sub-table (populated at \
         src/lua_api/globals/enum_data/constants_values.lua:237) and \
         Enum.SpellBookSpellBank (populated at \
         src/lua_api/globals/enum_data/addon_system.rs:643), all bootstrapped at env init"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_DeprecatedSpellBook declares no `## AllowLoadGameType:` filter — loads on \
         every game type without restriction"
    );

    let toc_text = std::fs::read_to_string(deprecated_spell_book_toc())
        .expect("Blizzard_DeprecatedSpellBook TOC should read");
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_DeprecatedSpellBook omits `## AllowLoad:` — defaults to Game-screen-only \
         (src/toc.rs:311), matching the legacy spellbook API in-game-only surface"
    );
    assert!(
        !toc_text.contains("## AllowLoadGameType:"),
        "Blizzard_DeprecatedSpellBook omits `## AllowLoadGameType:` — single-file shim is \
         universal across game types (unlike Blizzard_DeprecatedSpecialization which has \
         the `[Game]` template fork)"
    );
}

#[test]
fn blizzard_deprecated_spell_book_appears_in_game_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedSpellBook");
    assert!(
        in_game,
        "Blizzard_DeprecatedSpellBook (no AllowLoad flag, defaults to Game-only) should \
         appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeprecatedSpellBook");
    assert!(
        !in_login,
        "Blizzard_DeprecatedSpellBook should NOT appear on the Login / glue screens — \
         spellbook management is an in-game concept"
    );
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_book_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("DeprecatedSpellBook") || message.contains("Deprecated_SpellBook")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DeprecatedSpellBook emitted Lua errors during Game-screen load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_book_installs_six_function_shims(env: &WowLuaEnv) {

    let installed: bool = env
        .eval(
            "return type(IsPlayerSpell) == 'function' \
                and type(IsSpellKnown) == 'function' \
                and type(IsSpellKnownOrOverridesKnown) == 'function' \
                and type(FindFlyoutSlotBySpellID) == 'function' \
                and type(FindSpellOverrideByID) == 'function' \
                and type(FindBaseSpellByID) == 'function'",
        )
        .expect("6-shim function-installation query should succeed");
    assert!(
        installed,
        "Deprecated_SpellBook.lua lines 11-38 publish 6 forwarding global functions — all \
         locally-defined wrappers (NOT direct table-read aliases): IsPlayerSpell (calls \
         C_SpellBook.IsSpellKnown with Player bank); IsSpellKnown (re-DEFINES the existing \
         IsSpellKnown global from src/lua_api/globals/spell_state_probes.rs:183, replacing \
         it with a wrapper that calls C_SpellBook.IsSpellInSpellBook with Player/Pet bank); \
         IsSpellKnownOrOverridesKnown (re-DEFINES the existing global, calls IsSpellInSpellBook \
         with includeOverrides=true); FindFlyoutSlotBySpellID (calls \
         C_SpellBook.FindFlyoutSlotBySpellID — UNSTUBBED, resolves to no-op); \
         FindSpellOverrideByID (calls C_SpellBook.FindSpellOverrideByID — registered at \
         c_spell_book.rs:180 backed by the GetOverrideSpell impl); FindBaseSpellByID (calls \
         C_SpellBook.FindBaseSpellByID — UNSTUBBED, resolves to no-op). All install as \
         function values"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_book_global_wrappers_are_not_identity_equal_to_c_spell_book(env: &WowLuaEnv) {

    let not_identity_equal: bool = env
        .eval(
            "return IsPlayerSpell ~= C_SpellBook.IsSpellKnown \
                and IsSpellKnown ~= C_SpellBook.IsSpellInSpellBook \
                and IsSpellKnownOrOverridesKnown ~= C_SpellBook.IsSpellInSpellBook \
                and FindFlyoutSlotBySpellID ~= C_SpellBook.FindFlyoutSlotBySpellID \
                and FindSpellOverrideByID ~= C_SpellBook.FindSpellOverrideByID \
                and FindBaseSpellByID ~= C_SpellBook.FindBaseSpellByID",
        )
        .expect("non-identity-equality query for 6 wrappers should succeed");
    assert!(
        not_identity_equal,
        "Unlike Blizzard_DeprecatedSpecialization's 4 direct aliases (which use \
         `Global = C_*.Method;` plain assignment), every shim in Deprecated_SpellBook.lua \
         is `function GlobalName(args) ... return C_SpellBook.Method(...) end` — a fresh \
         Lua closure. None are identity-equal to the C_SpellBook backing methods, even when \
         the wrapper just returns the result of a single delegated call"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_book_shim_overrides_existing_is_spell_known_global(env: &WowLuaEnv) {

    let calls_through_namespace: bool = env
        .eval(
            "do \
                local ok, result = pcall(IsSpellKnown, 12345, false) \
                return ok and type(result) == 'boolean' \
            end",
        )
        .expect("pcall(IsSpellKnown) probe should succeed");
    assert!(
        calls_through_namespace,
        "The shim's `function IsSpellKnown(spellID, isPet)` at line 16 OVERWRITES the \
         simulator's pre-existing IsSpellKnown global registered at \
         src/lua_api/globals/spell_state_probes.rs:183 (the load-order-dependent overwrite \
         is intentional — Blizzard's deprecation shim wants the new spellBank-aware \
         signature exposed to legacy code). The replacement wrapper calls \
         C_SpellBook.IsSpellInSpellBook(spellID, Enum.SpellBookSpellBank.Player, false) \
         which is a registered Rust impl (c_spell_book.rs:484) returning a single bool. \
         pcall returns ok=true with a boolean result"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_book_hunter_dismiss_pet_constant_pulls_from_constants_table(env: &WowLuaEnv) {

    let constant_value: f64 = env
        .eval("return HUNTER_DISMISS_PET")
        .expect("HUNTER_DISMISS_PET query should succeed");
    assert_eq!(
        constant_value, 2641.0,
        "Deprecated_SpellBook.lua line 9 reads `HUNTER_DISMISS_PET = \
         Constants.SpellBookSpellIDs.SPELL_ID_DISMISS_PET;`. The simulator pre-populates \
         this constant at src/lua_api/globals/enum_data/constants_values.lua:237 with the \
         value 2641 (the hunter Dismiss Pet spellID). Even though Constants has an `__index` \
         that auto-creates empty tables, the load order is: enums.rs:45 runs \
         CONSTANTS_VALUES_LUA at env init, populating Constants.SpellBookSpellIDs.SPELL_ID_DISMISS_PET=2641 \
         BEFORE any addon loads — so the addon reads the populated value rather than nil"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_book_unstubbed_shims_call_without_error(env: &WowLuaEnv) {

    let no_error: bool = env
        .eval(
            "do \
                local ok1 = pcall(FindFlyoutSlotBySpellID, 0) \
                local ok2 = pcall(FindBaseSpellByID, 0) \
                local ok3 = pcall(IsPlayerSpell, 0) \
                local ok4 = pcall(IsSpellKnownOrOverridesKnown, 0, false) \
                local ok5 = pcall(FindSpellOverrideByID, 0) \
                return ok1 and ok2 and ok3 and ok4 and ok5 \
            end",
        )
        .expect("pcall probe of 5 shim invocations should succeed");
    assert!(
        no_error,
        "All 5 callable shims must NOT throw. FindFlyoutSlotBySpellID and FindBaseSpellByID \
         resolve through `__wow_namespace_mt.__index` (runtime_surface_bootstrap.lua:2019-2028) \
         to no-op `function() return nil end` closures — wrapper returns nil. IsPlayerSpell \
         and IsSpellKnownOrOverridesKnown delegate to registered C_SpellBook methods that \
         return bools. FindSpellOverrideByID is registered at c_spell_book.rs:180. All five \
         pcalls return ok=true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_deprecated_spell_book_load_deprecation_fallbacks_cvar_is_default_on(env: &WowLuaEnv) {

    let cvar_on: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("GetCVarBool query should succeed");
    assert!(
        cvar_on,
        "The `loadDeprecationFallbacks` CVar must default to true (src/cvars.yaml:899 sets \
         '1') so the early-return guard at Deprecated_SpellBook.lua:4 doesn't bail before \
         the 6 spellbook globals and HUNTER_DISMISS_PET constant are defined. If this CVar \
         flips to false, the entire shim is skipped — including the IsSpellKnown override, \
         meaning the simulator's pre-existing 1-arg IsSpellKnown would survive instead of \
         the 2-arg (spellID, isPet) wrapper"
    );
}
}

#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
#[test]
fn deprecated_spell_book_globals_follow_the_original_cvar_gate() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_mode(ScreenKind::Game);

    let ui = blizzard_ui_dir();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    env.exec("SetCVar('loadDeprecationFallbacks', '0')")
        .expect("fixture should disable Blizzard's deprecated spell wrappers");

    let toc = find_toc_file(&ui.join("Blizzard_DeprecatedSpellBook"))
        .expect("Retail game screen should select Blizzard_DeprecatedSpellBook");
    load_addon(&env.loader_env(), &toc)
        .expect("selected Blizzard deprecated spellbook addon should load");

    let (loaded, cvar_disabled, wrappers_absent): (bool, bool, bool) = env
        .eval(
            r#"
            return C_AddOns.IsAddOnLoaded("Blizzard_DeprecatedSpellBook"),
                   not GetCVarBool("loadDeprecationFallbacks"),
                   IsPlayerSpell == nil
                       and IsSpellKnownOrOverridesKnown == nil
                       and FindFlyoutSlotBySpellID == nil
                       and FindSpellOverrideByID == nil
                       and FindBaseSpellByID == nil
            "#,
        )
        .expect("deprecated spell wrapper state should be readable");

    assert!(loaded, "the selected original addon ran");
    assert!(
        cvar_disabled,
        "the original deprecation-fallback CVar stays off"
    );
    assert!(
        wrappers_absent,
        "the original Retail addon returns before publishing these globals when the CVar is off"
    );
}

#[test]
fn blizzard_deprecated_spell_book_has_no_xml_or_other_assets() {
    let dir = blizzard_ui_dir().join("Blizzard_DeprecatedSpellBook");
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_DeprecatedSpellBook dir should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    let has_xml = entries.iter().any(|n| n.ends_with(".xml"));
    assert!(
        !has_xml,
        "Blizzard_DeprecatedSpellBook has NO XML files — pure Lua single-file function-shim \
         definitions only. Got entries: {entries:?}"
    );

    let has_runtime_shims = entries.iter().any(|n| n == "Deprecated_SpellBook.lua");
    assert!(
        has_runtime_shims,
        "Blizzard_DeprecatedSpellBook should ship `Deprecated_SpellBook.lua` (the runtime \
         shim definitions for the 6 deprecated spellbook globals + HUNTER_DISMISS_PET \
         constant)"
    );
}
