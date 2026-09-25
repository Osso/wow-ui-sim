use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn restricted_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RestrictedAddOnEnvironment")
}

fn restricted_toc() -> PathBuf {
    restricted_dir().join("Blizzard_RestrictedAddOnEnvironment.toc")
}

const TOC_FILES: &[&str] = &[
    "RestrictedInfrastructure.lua",
    "RestrictedEnvironment.lua",
    "RestrictedExecution.lua",
    "RestrictedFrames.lua",
    "SecureHandlers.lua",
    "SecureHandlerTemplates.xml",
    "SecureStateDriver.lua",
    "SecureHoverDriver.lua",
    "SecureGroupHeaders.lua",
    "SecureGroupHeaders.xml",
    "SecureAuraHeader.lua",
    "SecureAuraHeader.xml",
];

const RETAIL_TOC_FILE_COUNT: usize = 10;

const HARD_DEPENDENCIES: &[&str] = &["Blizzard_FrameXML"];

const RESTRICTED_INFRASTRUCTURE_GLOBALS: &[&str] = &[
    "IsFrameHandle",
    "GetFrameHandleFrame",
    "GetFrameHandle",
    "GetReadonlyRestrictedTable",
    "IsWritableRestrictedTable",
    "GetManagedEnvironment",
];

const RESTRICTED_EXECUTION_GLOBALS: &[&str] = &[
    "PropagateForbiddenToReferencedFrames",
    "AddReferencedFrame",
    "CallRestrictedClosure",
];

const SECURE_HANDLER_GLOBALS: &[&str] = &[
    "SecureHandler_OnLoad",
    "SecureHandler_OnSimpleEvent",
    "SecureHandler_OnClick",
    "SecureHandler_OnMouseUpDown",
    "SecureHandler_OnMouseWheel",
    "SecureHandler_StateOnAttributeChanged",
    "SecureHandler_AttributeOnAttributeChanged",
    "SecureHandler_OnDragEvent",
    "SecureHandlerWrapScript",
    "SecureHandlerUnwrapScript",
    "SecureHandlerExecute",
    "SecureHandlerSetFrameRef",
];

const SECURE_STATE_DRIVER_GLOBALS: &[&str] = &[
    "RegisterAttributeDriver",
    "UnregisterAttributeDriver",
    "RegisterStateDriver",
    "UnregisterStateDriver",
    "RegisterUnitWatch",
    "UnregisterUnitWatch",
    "UnitWatchRegistered",
];

const SECURE_HOVER_DRIVER_GLOBALS: &[&str] =
    &["RegisterAutoHide", "AddToAutoHide", "UnregisterAutoHide"];

const SECURE_GROUP_HEADER_GLOBALS: &[&str] = &[
    "SecureGroupHeader_OnLoad",
    "SecureGroupHeader_OnEvent",
    "SecureGroupHeader_OnAttributeChanged",
    "SecureGroupHeader_Update",
    "SecureGroupPetHeader_OnLoad",
    "SecureGroupPetHeader_OnEvent",
    "SecureGroupPetHeader_OnAttributeChanged",
    "SecureGroupPetHeader_Update",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "SecureHandlerBaseTemplate",
    "SecureHandlerStateTemplate",
    "SecureHandlerAttributeTemplate",
    "SecureHandlerClickTemplate",
    "SecureHandlerDoubleClickTemplate",
    "SecureHandlerDragTemplate",
    "SecureHandlerShowHideTemplate",
    "SecureHandlerMouseUpDownTemplate",
    "SecureHandlerMouseWheelTemplate",
    "SecureHandlerEnterLeaveTemplate",
    "SecureGroupHeaderTemplate",
    "SecurePartyHeaderTemplate",
    "SecureRaidGroupHeaderTemplate",
    "SecureGroupPetHeaderTemplate",
    "SecurePartyPetHeaderTemplate",
    "SecureRaidPetHeaderTemplate",
];

const NAMED_MANAGER_FRAMES: &[&str] = &[
    "SecureStateDriverManager",
    "SecureHoverDriverManager",
    "SecureHandlersUpdateFrame",
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
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&restricted_dir())
        .expect("Blizzard_RestrictedAddOnEnvironment TOC should resolve");
    assert_eq!(
        resolved,
        restricted_toc(),
        "Blizzard_RestrictedAddOnEnvironment ships exactly one TOC at the bare \
         `Blizzard_RestrictedAddOnEnvironment.toc` path — no `_Mainline` flavor split. The \
         single TOC carries cross-flavor metadata via `## AllowLoadGameType: classic, standard` \
         instead of fanning out into per-flavor files. find_toc_file at src/loader/mod.rs:65 \
         tries `_Mainline.toc` first (miss) then bare (hit)"
    );
}

#[test]
fn toc_title_carries_spaces_verbatim() {
    let toc = TocFile::from_file(&restricted_toc())
        .expect("Blizzard_RestrictedAddOnEnvironment TOC should parse");
    assert_eq!(
        toc.metadata.get("Title").map(String::as_str),
        Some("Blizzard Restricted AddOn Environment"),
        "## Title metadata stored verbatim with embedded spaces — TocFile::name uses the Title \
         when present (src/toc.rs:60-68), so the addon's display name diverges from the folder \
         name `Blizzard_RestrictedAddOnEnvironment`. The folder/TOC stem is what the loader \
         uses for IsAddOnLoaded lookups and dependency resolution; the Title is purely cosmetic"
    );
}

#[test]
fn toc_declares_eager_game_only_with_classic_standard_allowlist() {
    let toc = TocFile::from_file(&restricted_toc())
        .expect("Blizzard_RestrictedAddOnEnvironment TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_RestrictedAddOnEnvironment must NOT declare `## LoadOnDemand: 1` — the \
         RegisterStateDriver / RegisterAttributeDriver / RegisterUnitWatch / SecureHandler_* \
         globals AND the SecureGroupHeader / SecureAuraHeader templates must publish before any \
         downstream Blizzard frame XML references them at OnLoad time. LOD would defer the \
         entire secure-handler subsystem until first protected click"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_RestrictedAddOnEnvironment declares `## AllowLoadGameType: classic, standard` \
         — `standard` is in the mainline allowlist at src/toc.rs:294-302 \
         (matches!(t.trim(), \"mainline\" | \"standard\")), so is_game_type_restricted returns \
         FALSE. The addon ships in BOTH classic and retail builds because every protected \
         action — clicks on action buttons, party/raid frame headers, aura buttons — relies on \
         this restricted-execution sandbox regardless of game flavor"
    );
    assert_eq!(
        toc.metadata.get("AllowLoadGameType").map(String::as_str),
        Some("classic, standard"),
        "AllowLoadGameType must round-trip the comma-separated list verbatim — both classic and \
         standard tokens are required because the secure-handler subsystem is shared infrastructure"
    );
    assert!(
        !toc.metadata.contains_key("DefaultState"),
        "Blizzard_RestrictedAddOnEnvironment intentionally omits `## DefaultState` — \
         default_enabled at src/toc.rs:251-256 returns TRUE when the metadata is absent. The \
         addon is foundational and cannot be disabled without breaking every protected-frame \
         lifecycle, so omitting the directive matches that semantic (always enabled, never user-toggleable)"
    );
    assert_eq!(
        toc.dependencies(),
        HARD_DEPENDENCIES,
        "## Dependencies: Blizzard_FrameXML — the lone hard dependency. \
         RestrictedFrames.lua wraps GetFrameMetatable / GetButtonMetatable (defined in \
         FrameXML), and the SecureHandlerTemplates.xml virtual frames inherit from \
         SecureFrameTemplate (defined in Blizzard_FrameXML/SecureTemplatesBase.xml). Without \
         FrameXML loaded first, every CopyTable(GetFrameMetatable().__index) call at module \
         load time would explode"
    );
}

#[test]
fn toc_lists_ten_retail_files_in_documented_order() {
    let toc = TocFile::from_file(&restricted_toc())
        .expect("Blizzard_RestrictedAddOnEnvironment TOC should parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    let expected: Vec<String> = TOC_FILES[..RETAIL_TOC_FILE_COUNT]
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    assert_eq!(
        listed, expected,
        "Retail parsing must retain the first ten ordered files and filter the classic-gated SecureAuraHeader pair"
    );
}

#[test]
fn restricted_environment_lua_carries_secure_env_per_file_override() {
    let toc = TocFile::from_file(&restricted_toc())
        .expect("Blizzard_RestrictedAddOnEnvironment TOC should parse");
    assert!(
        !toc.is_secure_env(),
        "is_secure_env at src/toc.rs:237-242 reads `## UseSecureEnvironment: 1` which this \
         addon does NOT declare — so the addon-level default is FALSE. Per-file overrides via \
         the `[LoadIntoEnvironment secure]` annotation override the default for individual \
         files only"
    );

    let restricted_env_index = TOC_FILES
        .iter()
        .position(|name| *name == "RestrictedEnvironment.lua")
        .expect("RestrictedEnvironment.lua must appear in TOC_FILES");
    assert_eq!(
        toc.file_use_secure_env(restricted_env_index),
        Some(true),
        "RestrictedEnvironment.lua [LoadIntoEnvironment secure] — the per-file annotation at \
         src/toc.rs:108-117 returns Some(true). load_addon_files at src/loader/addon.rs:626 \
         OVERRIDES the addon's use_secure_env with the per-file value, so this single file \
         loads under the secure environment while the rest of the addon stays in the default \
         scope. Why: RestrictedEnvironment.lua exports RESTRICTED_FUNCTIONS_SCOPE via the \
         addonTable second-vararg — that table is the closure environment for every \
         CallRestrictedClosure invocation. Loading it under the secure env stamps the closure \
         with secure taint so the restricted-execution sandbox stays insecure-sealed"
    );

    for (idx, file) in toc.files.iter().enumerate() {
        let name = file.to_string_lossy();
        if name == "RestrictedEnvironment.lua" {
            continue;
        }
        assert_eq!(
            toc.file_use_secure_env(idx),
            None,
            "{name} carries no `[LoadIntoEnvironment ...]` annotation — file_env_overrides at \
             that index must be None so load_addon_files inherits the addon-level use_secure_env \
             (false). Only RestrictedEnvironment.lua needs the secure stamping; the surrounding \
             infrastructure runs in the standard environment"
        );
    }
}

#[test]
fn allows_screen_returns_true_only_for_game() {
    let toc = TocFile::from_file(&restricted_toc())
        .expect("Blizzard_RestrictedAddOnEnvironment TOC should parse");
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad=Game must allow Game screen — src/toc.rs:308 maps `game` → \
         (screen == ScreenKind::Game). The secure-handler subsystem only exists in-game; glue \
         screens (login, character select, character create) have no protected frames"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "AllowLoad=Game must REJECT every glue screen ({screen:?}) — src/toc.rs:308 maps \
             `game` to Game-only. Glue addons never instantiate protected frames so the \
             restricted-execution sandbox is unreachable from there"
        );
    }
}

#[test]
fn included_in_eager_discovery_for_game_screen() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_RestrictedAddOnEnvironment");
    assert!(
        found,
        "Blizzard_RestrictedAddOnEnvironment must appear in eager auto-discovery on Game — the \
         is_game_type_restricted filter passes (standard is allowlisted), is_load_on_demand \
         is false, and AllowLoad=Game matches the requested screen. Required because the \
         secure-handler globals are referenced by countless Blizzard XML templates' OnLoad \
         attributes that resolve at load time"
    );
}

#[test]
fn excluded_from_eager_discovery_on_glue_screens() {
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_RestrictedAddOnEnvironment");
        assert!(
            !found,
            "Blizzard_RestrictedAddOnEnvironment must NOT appear on glue screen {screen:?} — \
             allows_screen returns false because AllowLoad=Game gates Game-only. The \
             discovery sweep at src/loader/mod.rs filters via toc.allows_screen(screen) before \
             enrolling"
        );
    }
}

#[test]
fn root_directory_holds_ten_lua_and_two_xml_files() {
    let mut entries: Vec<String> = std::fs::read_dir(restricted_dir())
        .expect("Blizzard_RestrictedAddOnEnvironment directory should read")
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name != "Blizzard_RestrictedAddOnEnvironment.toc")
        .collect();
    entries.sort();

    let mut expected: Vec<String> = TOC_FILES.iter().map(|name| (*name).to_string()).collect();
    expected.sort();

    assert_eq!(
        entries, expected,
        "Retail 12.1 root must contain exactly its ten Lua and two XML TOC body files"
    );
}

prefork_full_ui_case! {
fn loads_without_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("RestrictedInfrastructure")
                || message.contains("RestrictedEnvironment")
                || message.contains("RestrictedExecution")
                || message.contains("RestrictedFrames")
                || message.contains("SecureHandlers")
                || message.contains("SecureStateDriver")
                || message.contains("SecureHoverDriver")
                || message.contains("SecureGroupHeaders")
                || message.contains("SecureAuraHeader")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_RestrictedAddOnEnvironment emitted Lua errors during eager Game-screen load. \
         This addon is the foundation for every protected-frame interaction: action buttons, \
         party/raid headers, aura buttons, click-cast bindings, state-driver visibility, \
         override bindings. ANY error here cascades into broken click-handling everywhere:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_RestrictedAddOnEnvironment')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_RestrictedAddOnEnvironment') must return true after \
         the eager Game sweep — confirms the loader registers the non-LOD addon with the \
         loaded-set. Required because numerous downstream Blizzard frames branch on this \
         addon's IsAddOnLoaded status before assuming SecureHandler_OnLoad is available as a \
         global"
    );
}
}

prefork_full_ui_case! {
fn restricted_infrastructure_publishes_frame_handle_namespace_globals(env: &WowLuaEnv) {

    for fname in RESTRICTED_INFRASTRUCTURE_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G['{fname}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{fname}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{fname} must publish at `_G` as a function — RestrictedInfrastructure.lua exports \
             the frame-handle ↔ frame mapping (IsFrameHandle, GetFrameHandle, \
             GetFrameHandleFrame), the readonly/writable restricted-table predicates \
             (GetReadonlyRestrictedTable, IsWritableRestrictedTable), and the per-frame \
             managed-environment factory (GetManagedEnvironment). These are consumed by \
             RestrictedExecution's CallRestrictedClosure to translate insecure frame \
             references into secure handles before invoking restricted snippets. \
             RestrictedTable_unpack_ro stays file-local at line 383 (`local \
             RestrictedTable_unpack_ro;` forward-declares before `function \
             RestrictedTable_unpack_ro(...)` binds locally) and is intentionally not exported"
        );
    }
}
}

prefork_full_ui_case! {
fn init_frame_handle_namespace_is_explicitly_nilled_after_use(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(_G['InitFrameHandleNamespace'])")
        .expect("type(_G.InitFrameHandleNamespace) probe should succeed");
    assert_eq!(
        kind, "nil",
        "InitFrameHandleNamespace is intentionally cleared at the END of RestrictedFrames.lua:841 \
         (`InitFrameHandleNamespace = nil;`). The function exists during load to register the \
         FrameHandle method namespace (108 lifecycle methods on the HANDLE table — GetName, \
         IsShown, GetWidth, GetAttribute, GetFrameRef, etc.), then the function is removed \
         from `_G` so addon code can never re-invoke the namespace registration. This is a \
         deliberate one-shot pattern matching real WoW behaviour"
    );
}
}

prefork_full_ui_case! {
fn restricted_execution_publishes_call_restricted_closure(env: &WowLuaEnv) {
    for fname in RESTRICTED_EXECUTION_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G['{fname}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{fname}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{fname} must publish at `_G` as a function — RestrictedExecution.lua wires the \
             closure-cache and reference-frame-tracking globals. CallRestrictedClosure is the \
             single entry point that runs body source under a working environment with the \
             control handle, used by every SecureHandler_* dispatch path. \
             AddReferencedFrame / PropagateForbiddenToReferencedFrames are taint-propagation \
             helpers that mark frames touched by tainted execution"
        );
    }
}
}

prefork_full_ui_case! {
fn rtable_namespace_publishes_with_restricted_table_helpers(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(rtable)")
        .expect("type(rtable) probe should succeed");
    assert_eq!(
        kind, "table",
        "RestrictedInfrastructure.lua:563 exports `rtable` as a global table — the \
         restricted-aware analogue of the standard `table` library. Provides next/pairs/ipairs/ \
         unpack/newtable/copytable/maxn/insert/remove/sort/concat/wipe/type/rtgsub. The \
         restricted environment exposes `rtable` instead of `table` because raw tables crossing \
         the secure boundary would leak object identity to insecure code"
    );
    let copytable_kind: String = env
        .eval("return type(rtable.copytable)")
        .expect("type(rtable.copytable) probe should succeed");
    assert_eq!(
        copytable_kind, "function",
        "rtable.copytable must be a function — used by RestrictedEnvironment.lua's \
         ScrubInboundValue to deep-copy table arguments crossing the secure→restricted \
         boundary. Also installed under string.rtgsub as `string.rtgsub = RestrictedTable_rtgsub` \
         immediately after rtable construction"
    );
}
}

prefork_full_ui_case! {
fn secure_handler_execute_persists_restricted_tables_for_show_handlers(env: &WowLuaEnv) {
    let count: String = env
        .eval(r#"local frame = CreateFrame("Button", "SecureHandlerExecutePersistenceProbe", UIParent, "SecureHandlerShowHideTemplate"); frame:Hide(); frame:Execute([[ keybinds = table.new("ALT-X") ]]); frame:SetAttribute("_onshow", [[ self:SetAttribute("observedCount", tostring(table.maxn(keybinds))) ]]); frame:Show(); return frame:GetAttribute("observedCount")"#)
        .unwrap();

    assert_eq!(count, "1");
}
}

prefork_full_ui_case! {
fn tainted_addon_secure_handler_execute_runs_restricted_snippet_without_clearing_caller(env: &WowLuaEnv) {
    let (protected, caller_insecure_before, snippet_ran, caller_insecure_after): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local header = CreateFrame("Frame", nil, UIParent, "SecureHandlerBaseTemplate")
            local function addon()
                local before = not issecure()
                SecureHandlerExecute(header, [[ self:SetAttribute("addon-executed", true) ]])
                return before, header:GetAttribute("addon-executed") == true, not issecure()
            end
            debug.setobjecttaint(addon, "RestrictedAddonProbe")
            local before, ran, after = addon()
            return header:IsProtected(), before, ran, after
            "#,
        )
        .expect("tainted addon SecureHandlerExecute probe should evaluate");

    assert!(protected, "vendor SecureHandlerBaseTemplate must protect the header");
    assert!(caller_insecure_before, "addon caller must enter tainted");
    assert!(snippet_ran, "vendor restricted snippet must set the header attribute");
    assert!(caller_insecure_after, "secure dispatch must not clear addon caller taint");
}
}

prefork_full_ui_case! {
fn tainted_addon_secure_handler_wrap_script_runs_restricted_prebody(env: &WowLuaEnv) {
    let (caller_insecure_before, wrapped, prebody_ran, caller_insecure_after): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local header = CreateFrame("Frame", nil, UIParent, "SecureHandlerBaseTemplate")
            local button = CreateFrame("Button", nil, UIParent)
            button:SetScript("OnClick", function() end)
            local original = button:GetScript("OnClick")
            local function addon()
                local before = not issecure()
                SecureHandlerWrapScript(button, "OnClick", header,
                    [[ self:SetAttribute("addon-wrapped", true) ]])
                local wrapped = button:GetScript("OnClick") ~= original
                button:GetScript("OnClick")(button, "LeftButton", false)
                return before, wrapped, header:GetAttribute("addon-wrapped") == true, not issecure()
            end
            debug.setobjecttaint(addon, "RestrictedAddonProbe")
            return addon()
            "#,
        )
        .expect("tainted addon SecureHandlerWrapScript probe should evaluate");

    assert!(caller_insecure_before, "addon caller must enter tainted");
    assert!(wrapped, "vendor must install the secure OnClick wrapper");
    assert!(prebody_ran, "wrapped click must execute its restricted prebody");
    assert!(caller_insecure_after, "wrapper dispatch must not clear addon caller taint");
}
}

prefork_full_ui_case! {
fn tainted_replacement_attribute_handler_stays_insecure_in_combat(env: &WowLuaEnv) {
    let (handler_ran, handler_insecure, protected_write_blocked, caller_insecure): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local delegate = CreateFrame("Frame")
            delegate:SetForbidden()
            local target = CreateFrame("Frame", nil, UIParent, "SecureFrameTemplate")
            local ran, insecure = false, false
            local function replacement()
                ran, insecure = true, not issecure()
                target:SetAttribute("addon-combat-write", true)
            end
            debug.setobjecttaint(replacement, "RestrictedAddonProbe")
            delegate:SetScript("OnAttributeChanged", replacement)
            local function addon()
                A_Admin.SetInCombat(true)
                delegate:SetAttribute("addon-trigger", true)
                local still_insecure = not issecure()
                A_Admin.SetInCombat(false)
                return still_insecure
            end
            debug.setobjecttaint(addon, "RestrictedAddonProbe")
            local still_insecure = addon()
            return ran, insecure, target:GetAttribute("addon-combat-write") == nil, still_insecure
            "#,
        )
        .expect("tainted replacement handler combat probe should evaluate");

    assert!(handler_ran, "replacement must run on forbidden delegate dispatch");
    assert!(handler_insecure, "addon-tainted replacement must not be promoted to secure");
    assert!(protected_write_blocked, "tainted handler must not write protected attributes in combat");
    assert!(caller_insecure, "delegate dispatch must preserve addon caller taint");
}
}

prefork_full_ui_case! {
fn secure_handlers_publish_full_global_surface(env: &WowLuaEnv) {
    for fname in SECURE_HANDLER_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G['{fname}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{fname}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{fname} must publish at `_G` as a function — SecureHandlers.lua wires 12 globals \
             driving every secure-handler dispatch path: SecureHandler_OnLoad sets up the \
             managed environment table, SecureHandler_OnSimpleEvent / OnClick / OnMouseUpDown / \
             OnMouseWheel are the XML-script entry points wired by SecureHandlerTemplates.xml, \
             SecureHandler_StateOnAttributeChanged / AttributeOnAttributeChanged dispatch on \
             attribute writes, SecureHandlerWrapScript / UnwrapScript / Execute / SetFrameRef \
             form the public addon API for binding restricted closures to frame scripts. \
             CheckForbidden / MakeForbidden are forward-declared on line 425 \
             (`local CheckForbidden, MakeForbidden;`) so the `function CheckForbidden(...)` / \
             `function MakeForbidden(...)` definitions at lines 591/595 bind locally — they \
             stay file-scoped despite the no-`local` syntax"
        );
    }
}
}

prefork_full_ui_case! {
fn secure_state_driver_publishes_attribute_state_unit_drivers(env: &WowLuaEnv) {
    for fname in SECURE_STATE_DRIVER_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G['{fname}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{fname}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{fname} must publish at `_G` as a function — SecureStateDriver.lua exports the 7 \
             driver-registration functions. RegisterStateDriver (state=values) and \
             RegisterAttributeDriver (attribute=values) drive frames via the macro-conditional \
             expression evaluator — used for things like `[combat]` to flip visibility on \
             combat enter/exit. RegisterUnitWatch hooks UNIT_VISIBILITY events so the frame's \
             visibility tracks UnitExists. UnitWatchRegistered queries the watch state"
        );
    }
}
}

prefork_full_ui_case! {
fn secure_hover_driver_publishes_auto_hide_globals(env: &WowLuaEnv) {
    for fname in SECURE_HOVER_DRIVER_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G['{fname}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{fname}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{fname} must publish at `_G` as a function — SecureHoverDriver.lua exports the \
             auto-hide registration triplet. RegisterAutoHide(frame, ttl) installs the \
             frame on a hover-tracked list so it auto-hides after `ttl` seconds when the mouse \
             leaves both the frame and any registered child. AddToAutoHide(frame, child) \
             extends an existing registration's bounding box. UnregisterAutoHide reverses. \
             Used by Blizzard tooltip-style transient UIs that should fade when hover lapses"
        );
    }
}
}

prefork_full_ui_case! {
fn secure_group_headers_publish_full_lifecycle_surface(env: &WowLuaEnv) {
    for fname in SECURE_GROUP_HEADER_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G['{fname}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{fname}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{fname} must publish at `_G` as a function — retail SecureGroupHeaders.lua wires \
             eight lifecycle globals across group and pet headers. SecureGroupHeader_* \
             (party/raid frames) sort group members and create child unit frames matching the \
             showParty/showRaid attributes; SecureGroupPetHeader_* mirrors the structure for \
             group pets. SecureAuraHeader.lua is classic-only in the current TOC and is not part \
             of the retail publication contract"
        );
    }
}
}

prefork_full_ui_case! {
fn restricted_environment_module_locals_stay_file_scoped(env: &WowLuaEnv) {
    let no_module_local_leak: bool = env
        .eval(
            "return _G['RESTRICTED_FUNCTIONS_SCOPE'] == nil \
                and _G['DIRECT_MACRO_CONDITIONAL_NAMES'] == nil \
                and _G['ENV'] == nil \
                and _G['ScrubInboundValue'] == nil \
                and _G['ScrubOutboundValue'] == nil \
                and _G['ImportOutboundFunctions'] == nil",
        )
        .expect("Module-local probes should succeed");
    assert!(
        no_module_local_leak,
        "RestrictedEnvironment.lua's module-locals must NOT leak into `_G`: \
         RESTRICTED_FUNCTIONS_SCOPE (the closure environment exported via addonTable instead \
         of `_G`), DIRECT_MACRO_CONDITIONAL_NAMES (the SecureCmdOptionParse / IsStealthed / \
         UnitExists / etc. allowlist), ENV (the staging table for ScrubInboundValue-wrapped \
         functions before they migrate into RESTRICTED_FUNCTIONS_SCOPE), \
         ScrubInboundValue/ScrubOutboundValue (the boundary scrubbers that turn tables into \
         RestrictedTable_copytable copies and FrameHandles into themselves), \
         ImportOutboundFunctions (the recursive copy-with-wrapping helper). All scoped via \
         file-locals and the addonTable export"
    );
}
}

prefork_full_ui_case! {
fn virtual_templates_stay_off_global_scope(env: &WowLuaEnv) {
    for template in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G['{template}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "{template} must NOT publish at `_G` — declared as `virtual=\"true\"` in either \
             SecureHandlerTemplates.xml or SecureGroupHeaders.xml so the loader keeps them in \
             the template registry only, never as globals. The 10 SecureHandler*Templates \
             provide the OnLoad/OnClick/OnDoubleClick/OnDragStart/OnReceiveDrag/OnShow/OnHide/ \
             OnMouseUp/OnMouseDown/OnMouseWheel/OnEnter/OnLeave script-routing wiring; the 8 \
             SecureGroup/Aura templates host the OnLoad/OnEvent/OnShow/OnUpdate/ \
             OnAttributeChanged dispatch for the multi-frame layout managers"
        );
    }
}
}

prefork_full_ui_case! {
fn named_manager_frames_publish_globally_under_secure_template(env: &WowLuaEnv) {
    for fname in NAMED_MANAGER_FRAMES {
        let (exists, is_protected, blocked_insecure_attribute): (bool, bool, bool) = env
            .eval(&format!(
                "return type({fname}) == 'table' and \
                    type({fname}.IsObjectType) == 'function' and \
                    {fname}:IsObjectType('Frame'), \
                    {fname}:IsProtected(), \
                    (function() \
                        A_Admin.SetInCombat(true); \
                        forceinsecure(); \
                        {fname}:SetAttribute('codex-protected-probe', 'blocked'); \
                        debug.setstacktaint(nil); \
                        A_Admin.SetInCombat(false); \
                        return {fname}:GetAttribute('codex-protected-probe') == nil; \
                    end)()"
            ))
            .unwrap_or_else(|err| panic!("{fname} frame probe failed: {err}"));
        assert!(
            exists,
            "{fname} must be a real Frame — created via \
             CreateFrame(\"Frame\", \"{fname}\", nil, \"SecureFrameTemplate\"). The \
             SecureFrameTemplate inheritance is critical because it stamps the frame as \
             `protected=true` in Blizzard_FrameXML/SecureTemplatesBase.xml — without that \
             ancestry the secure-handler subsystem would treat them as insecure and refuse to \
             register their OnUpdate drivers. SecureStateDriverManager (line 178) binds via \
             direct global assignment and owns the 0.2s STATE_DRIVER_UPDATE_THROTTLE OnUpdate. \
             SecureHoverDriverManager (line 181) and SecureHandlersUpdateFrame (line 589) bind \
             the Lua side to file-local `LOCAL_UpdateFrame` / `LOCAL_API_Frame` — only the \
             CreateFrame name argument exposes them globally, so the OnUpdate handlers are \
             scoped to their respective files while debug tooling can still locate the frames"
        );
        assert!(
            is_protected,
            "{fname} must report IsProtected=true — Blizzard creates it with \
             CreateFrame(\"Frame\", \"{fname}\", nil, \"SecureFrameTemplate\"), and \
             SecureFrameTemplate is declared protected=\"true\" in \
             Blizzard_FrameXML/SecureTemplatesBase.xml. This checks the real Blizzard frame \
             behavior, not just the simulator's synthetic A_Admin.SetFrameProtected helper."
        );
        assert!(
            blocked_insecure_attribute,
            "{fname} must reject insecure in-combat SetAttribute writes because its \
             protection comes from the real Blizzard SecureFrameTemplate inheritance. \
             A false result means IsProtected() is only cosmetic and the protected-frame \
             enforcement path is not using the XML/template-derived protection flag."
        );
    }
}
}

#[test]
fn xml_files_declare_only_virtual_templates() {
    let handler_xml = std::fs::read_to_string(restricted_dir().join("SecureHandlerTemplates.xml"))
        .expect("SecureHandlerTemplates.xml should read");
    assert!(
        handler_xml.contains("virtual=\"true\""),
        "SecureHandlerTemplates.xml must mark every <Frame>/<Button> with virtual=\"true\" — \
         the entire file is a template registry, no concrete frames. Each of the 10 templates \
         exists to be inherited by addon-defined or Blizzard-defined frames that need a \
         secure-handler script wiring shortcut"
    );
    assert!(
        !handler_xml.contains("inherits=\"SecureFrameTemplate\" virtual=\"false\""),
        "SecureHandlerTemplates.xml must NEVER declare a non-virtual frame — that would attempt \
         to instantiate SecureFrameTemplate without a name and trigger an immediate load error \
         because the template requires either a name or a parent context"
    );

    let header_xml = std::fs::read_to_string(restricted_dir().join("SecureGroupHeaders.xml"))
        .expect("SecureGroupHeaders.xml should read");
    assert!(
        header_xml.contains("name=\"SecureGroupHeaderTemplate\"")
            && header_xml.contains("virtual=\"true\""),
        "SecureGroupHeaders.xml must declare SecureGroupHeaderTemplate as virtual — the base \
         party/raid header all four PartyHeader / RaidGroupHeader / PartyPetHeader / \
         RaidPetHeader templates inherit from"
    );
    let aura_xml = std::fs::read_to_string(restricted_dir().join("SecureAuraHeader.xml"))
        .expect("SecureAuraHeader.xml should read");
    assert!(
        aura_xml.contains("name=\"SecureAuraHeaderTemplate\"")
            && aura_xml.contains("name=\"SecureAuraButtonTemplate\"")
            && aura_xml.contains("virtual=\"true\""),
        "Retail 12.1 keeps the virtual aura header and button templates in SecureAuraHeader.xml, separate from group-header templates"
    );
}
