#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn secure_util_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_MapCanvasSecureUtil")
}

fn secure_util_toc() -> PathBuf {
    secure_util_dir().join("Blizzard_MapCanvasSecureUtil.toc")
}

const SECURE_UTIL_TOC_FILES: &[&str] = &["Blizzard_MapCanvasSecureUtil.lua"];

const SECURE_UTIL_REGISTRY_METHODS: &[&str] = &[
    "AddHandler",
    "RemoveHandler",
    "FindHandler",
    "InvokeHandlers",
];

const SECURE_UTIL_FILE_LOCAL_HELPERS: &[&str] = &[
    "PrioritySorter",
    "EnumerateTaintedArray",
    "IsEquivalentHandler",
    "InvokeTaintedHandler",
];

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
fn blizzard_map_canvas_secure_util_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&secure_util_dir()).expect("Blizzard_MapCanvasSecureUtil TOC should resolve");
    assert_eq!(
        resolved,
        secure_util_toc(),
        "Blizzard_MapCanvasSecureUtil ships exactly one bare TOC. The secure-call utility is \
         a cross-flavor primitive that every map UI consumes via Blizzard_MapCanvas's \
         `## RequiredDep:` edge — there are no flavor-suffixed variants. `find_toc_file` \
         resolves the bare file after the `_Mainline.toc` lookup misses"
    );
}

#[test]
fn blizzard_map_canvas_secure_util_toc_declares_load_first_with_no_deps() {
    let toc =
        TocFile::from_file(&secure_util_toc()).expect("Blizzard_MapCanvasSecureUtil TOC parses");
    assert!(
        toc.is_load_first(),
        "Blizzard_MapCanvasSecureUtil declares `## LoadFirst: 1` — promoted into the eager \
         LoadFirst pass at src/loader/mod.rs (the loader's first pass eagerly emits addons \
         marked `LoadFirst` or `UseSecureEnvironment` before walking the dependency graph). \
         The MapCanvasSecureUtil global must exist BEFORE Blizzard_MapCanvas runs because \
         the Blizzard_MapCanvas data-provider / pin / area-trigger surfaces invoke \
         `MapCanvasSecureUtil.CreateHandlerRegistry()` during their OnLoad pipeline to \
         build per-canvas handler registries"
    );
    assert!(
        !toc.is_load_on_demand(),
        "TOC omits `## LoadOnDemand:` — eager-load. The factory function must publish \
         immediately at addon-load time, not deferred"
    );
    assert!(
        !toc.is_secure_env(),
        "TOC omits `## UseSecureEnvironment:` — the addon does NOT run inside Blizzard's \
         secure environment. The `Secure` in the addon name refers to its USE of \
         `securecallfunction` to invoke tainted handler bodies without taint leakage, NOT \
         to the addon itself running in the secure env"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Zero `## Dependencies:` / `## RequiredDep:` — the secure-call utility is \
         dependency-free at the addon level (it relies only on Lua stdlib + the global \
         `securecallfunction` / `SafePack` / `SafeUnpack` runtime helpers, all available \
         from the simulator's bootstrap surface)"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft dependencies. The factory output is consumed by \
         `Blizzard_MapCanvas` via its `## RequiredDep:` edge, so the relationship is hard"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — pure runtime scaffolding (one global table + one factory \
         function). Handler registries are per-canvas-instance state held only as long as \
         the canvas frame lives"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC omits `## AllowLoadGameType:` — universal contract. Every shipping flavor \
         (Mainline, Classic Era, Cataclysm, MoP Classic, etc) carries the secure-call \
         primitive because the map-canvas scaffolding is itself cross-flavor"
    );
}

#[test]
fn blizzard_map_canvas_secure_util_toc_routes_to_game_screen_only() {
    let toc =
        TocFile::from_file(&secure_util_toc()).expect("Blizzard_MapCanvasSecureUtil TOC parses");
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_MapCanvasSecureUtil omits `## AllowLoad:` — the default branch at \
         src/toc.rs:311 routes to Game-only. The secure-call utility is consumed by \
         Blizzard_MapCanvas which is itself Game-only; loading on glue screens would burn \
         memory for a global no consumer addon there ever pulls"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_MapCanvasSecureUtil must NOT auto-discover on glue screen \
             {screen:?} — missing `## AllowLoad:` falls through to the default-Game branch \
             at src/toc.rs:311. No glue-screen addon depends on MapCanvasSecureUtil"
        );
    }
}

#[test]
fn blizzard_map_canvas_secure_util_toc_lists_single_lua_file() {
    let toc =
        TocFile::from_file(&secure_util_toc()).expect("Blizzard_MapCanvasSecureUtil TOC parses");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        SECURE_UTIL_TOC_FILES,
        "TOC body lists exactly 1 file — Blizzard_MapCanvasSecureUtil.lua. The 80-line \
         addon publishes ONE global table + ONE factory function + 4 file-local closures; \
         no XML, no other Lua files, no Localization.lua (zero user-facing strings)"
    );
}

#[test]
fn blizzard_map_canvas_secure_util_directory_holds_two_entries() {
    let entries = std::fs::read_dir(secure_util_dir())
        .expect("Blizzard_MapCanvasSecureUtil directory reads")
        .count();
    assert_eq!(
        entries, 2,
        "Directory holds exactly 2 entries — Blizzard_MapCanvasSecureUtil.toc + \
         Blizzard_MapCanvasSecureUtil.lua. One of the smallest addons in the Blizzard tree, \
         alongside Blizzard_LoadLocale (also 2 entries)"
    );
}

#[test]
fn blizzard_map_canvas_secure_util_auto_discovered_on_game_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_MapCanvasSecureUtil");
    assert!(
        game_found,
        "Blizzard_MapCanvasSecureUtil must surface on the Game screen — `## LoadFirst: 1` + \
         missing `## AllowLoad:` (defaults to Game) routes it into the eager LoadFirst pass"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_MapCanvasSecureUtil");
        assert!(
            !found,
            "Blizzard_MapCanvasSecureUtil must NOT surface on glue screen {screen:?} — the \
             allows_screen filter at src/loader/mod.rs:527 strips Game-only addons from \
             the discovery results before the LoadFirst routing decision runs"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_MapCanvasSecureUtil")
                || message.contains("MapCanvasSecureUtil")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_MapCanvasSecureUtil emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_is_addon_loaded_after_auto_discovery(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MapCanvasSecureUtil')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MapCanvasSecureUtil') must return true after the \
         eager LoadFirst pass — proves the secure-call utility registers with the loaded \
         set during the standard Game-screen boot pipeline, no explicit load_addon call \
         required"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_publishes_global_table_with_factory(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MapCanvasSecureUtil)")
        .expect("MapCanvasSecureUtil type probe succeeds");
    assert_eq!(
        kind, "table",
        "MapCanvasSecureUtil must publish at `_G` as a table — declared at \
         Blizzard_MapCanvasSecureUtil.lua:9 as `MapCanvasSecureUtil = {{}}`. The single \
         user-facing surface this addon exposes; consumers hang `CreateHandlerRegistry` \
         off it as a static factory call"
    );

    let factory_kind: String = env
        .eval("return type(MapCanvasSecureUtil.CreateHandlerRegistry)")
        .expect("CreateHandlerRegistry type probe succeeds");
    assert_eq!(
        factory_kind, "function",
        "MapCanvasSecureUtil.CreateHandlerRegistry must be a function — declared at \
         Blizzard_MapCanvasSecureUtil.lua:37. The sole public method on the global; \
         returns a fresh registry table each call (closure over a private `handlers` array \
         so each canvas instance gets its own isolated handler list)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_create_registry_returns_table_with_four_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(MapCanvasSecureUtil.CreateHandlerRegistry())")
        .expect("CreateHandlerRegistry return-type probe succeeds");
    assert_eq!(
        kind, "table",
        "CreateHandlerRegistry() must return a table — the registry holds the 4 closure \
         methods bound to a private `handlers` array via lexical scope"
    );

    for method in SECURE_UTIL_REGISTRY_METHODS {
        let probe = format!(
            "local r = MapCanvasSecureUtil.CreateHandlerRegistry() return type(r.{method})"
        );
        let method_kind: String = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("registry.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "registry.{method} must be a function — covers the 4-method registry surface \
             (AddHandler inserts a {{handler, priority}} entry and tsorts by priority desc, \
             RemoveHandler finds-then-removes by handler+priority match, FindHandler \
             linear-scans via securecallfunction-wrapped rawget for taint isolation, \
             InvokeHandlers iterates via the same securecallfunction wrap and short-circuits \
             on the first truthy first-return-value)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_file_local_helpers_not_exposed_globally(env: &WowLuaEnv) {

    for helper in SECURE_UTIL_FILE_LOCAL_HELPERS {
        let kind: String = env
            .eval(&format!("return type({helper})"))
            .unwrap_or_else(|err| panic!("{helper} type probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "{helper} must remain unexposed at `_G` — declared as `local function` at \
             Blizzard_MapCanvasSecureUtil.lua. The 4 helpers (PrioritySorter / \
             EnumerateTaintedArray / IsEquivalentHandler / InvokeTaintedHandler) are \
             implementation details of the registry closure factory; exposing them \
             globally would let addon code bypass the securecallfunction taint isolation \
             that the registry guarantees"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_handlers_invoke_in_priority_descending_order(env: &WowLuaEnv) {

    let order: String = env
        .eval(
            "local r = MapCanvasSecureUtil.CreateHandlerRegistry() \
             local trail = {} \
             r:AddHandler(function() table.insert(trail, 'low') end, 1) \
             r:AddHandler(function() table.insert(trail, 'high') end, 10) \
             r:AddHandler(function() table.insert(trail, 'mid') end, 5) \
             r:InvokeHandlers() \
             return table.concat(trail, ',')",
        )
        .expect("priority-order invocation probe succeeds");
    assert_eq!(
        order, "high,mid,low",
        "InvokeHandlers must dispatch in priority-descending order — `PrioritySorter` at \
         Blizzard_MapCanvasSecureUtil.lua:11 returns `left.priority > right.priority`, and \
         AddHandler tsorts the array after every insert via securecallfunction. This makes \
         priority a numeric weight where higher values fire first, regardless of insert \
         order. The trail proves all 3 handlers fired and they fired in descending priority"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_invoke_handlers_short_circuits_on_truthy_first_return(env: &WowLuaEnv) {

    let trail: String = env
        .eval(
            "local r = MapCanvasSecureUtil.CreateHandlerRegistry() \
             local seen = {} \
             r:AddHandler(function() table.insert(seen, 'first') return true, 'payload' end, 10) \
             r:AddHandler(function() table.insert(seen, 'second') end, 1) \
             local consumed, stop_value, payload = r:InvokeHandlers() \
             return table.concat(seen, ',') .. '|consumed=' .. tostring(consumed) \
                    .. '|stop=' .. tostring(stop_value) .. '|payload=' .. tostring(payload)",
        )
        .expect("short-circuit probe succeeds");
    assert_eq!(
        trail, "first|consumed=true|stop=true|payload=payload",
        "InvokeHandlers must short-circuit on the first handler whose first return value \
         is truthy — InvokeHandlers at Blizzard_MapCanvasSecureUtil.lua:66 reads \
         `stopChecking = results[1]` and `if stopChecking then return true, \
         SafeUnpack(results) end`. The lower-priority `second` handler must NOT fire (only \
         `first` lands in `seen`), the function returns `true` as the first return value \
         (the `consumed` flag), and the trailing returns carry the SafeUnpacked results \
         from the consumer (`true, 'payload'` here). This is the canonical \
         input-event-consumed protocol: the first handler that wants the event signals it \
         consumed by returning truthy, and downstream handlers don't run"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_invoke_handlers_returns_false_when_no_handler_consumes(env: &WowLuaEnv) {

    let result: String = env
        .eval(
            "local r = MapCanvasSecureUtil.CreateHandlerRegistry() \
             r:AddHandler(function() return false end, 5) \
             r:AddHandler(function() return nil end, 1) \
             local consumed = r:InvokeHandlers() \
             return tostring(consumed)",
        )
        .expect("no-consumer probe succeeds");
    assert_eq!(
        result, "false",
        "InvokeHandlers must return `false` when no handler returns a truthy first value \
         — Blizzard_MapCanvasSecureUtil.lua:75 falls through to `return false` after the \
         iteration completes without short-circuiting. Both `false` and `nil` first \
         returns count as not-consumed, so the unanimous-pass case yields `false` to the \
         caller"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_remove_handler_drops_matching_entry(env: &WowLuaEnv) {

    let trail: String = env
        .eval(
            "local r = MapCanvasSecureUtil.CreateHandlerRegistry() \
             local seen = {} \
             local doomed = function() table.insert(seen, 'doomed') end \
             local survivor = function() table.insert(seen, 'survivor') end \
             r:AddHandler(doomed, 5) \
             r:AddHandler(survivor, 5) \
             r:RemoveHandler(doomed, 5) \
             r:InvokeHandlers() \
             return table.concat(seen, ',')",
        )
        .expect("RemoveHandler probe succeeds");
    assert_eq!(
        trail, "survivor",
        "RemoveHandler must drop the matched {{handler, priority}} entry — RemoveHandler \
         at Blizzard_MapCanvasSecureUtil.lua:46 calls FindHandler then tremove. After \
         removal, the doomed handler must NOT fire, and the survivor (same priority but \
         distinct closure identity) must remain registered. Proves IsEquivalentHandler \
         compares by reference identity, not by structural equality of the closure"
    );
}
}

prefork_full_ui_case! {
fn blizzard_map_canvas_secure_util_find_handler_returns_index_or_nil(env: &WowLuaEnv) {

    let trail: String = env
        .eval(
            "local r = MapCanvasSecureUtil.CreateHandlerRegistry() \
             local target = function() end \
             r:AddHandler(function() end, 10) \
             r:AddHandler(target, 5) \
             r:AddHandler(function() end, 1) \
             local hit = r:FindHandler(target, 5) \
             local miss = r:FindHandler(function() end, 5) \
             local wrong_priority = r:FindHandler(target, 999) \
             local priority_omitted = r:FindHandler(target) \
             return tostring(hit) .. '|' .. tostring(miss) .. '|' .. \
                    tostring(wrong_priority) .. '|' .. tostring(priority_omitted)",
        )
        .expect("FindHandler probe succeeds");
    assert_eq!(
        trail, "2|nil|nil|2",
        "FindHandler must return the 1-based index of the matching {{handler, priority}} \
         entry, or nil. After the priority-desc tsort, the entries sit at indices 1 (pri \
         10), 2 (pri 5 = target), 3 (pri 1) — so `target` matches at index 2. A distinct \
         anonymous-function literal at the same priority misses (handler reference doesn't \
         match). A correct handler at the wrong priority misses. Omitting the priority \
         falls back to handler-only matching (the `not priority` branch in \
         IsEquivalentHandler at lua:30): same target hits at 2"
    );
}
}
