use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn ping_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PingUI")
}

fn ping_ui_toc() -> PathBuf {
    ping_ui_dir().join("Blizzard_PingUI.toc")
}

const PING_UI_TOC_FILES: &[&str] = &[
    "Blizzard_PingManager.lua",
    "Blizzard_PingUI.lua",
    "Blizzard_PingUI.xml",
    "Blizzard_PingUtil.lua",
];

const REQUIRED_DEPS: &[&str] = &["Blizzard_SharedXML", "Blizzard_FrameXMLUtil"];

const SECURE_ENV_MIXINS: &[&str] = &[
    "PingFrameMixin",
    "PingListenerFrameMixin",
    "PingPinFrameMixin",
    "PingPinFlipBookAnimMixin",
];

const SECURE_ENV_NAMESPACES: &[&str] = &["PingManager"];

const SECURE_ENV_FREE_FUNCTIONS: &[&str] = &["GetScaledCursorPosition_Insecure"];

const PUBLIC_NAMED_FRAMES: &[&str] = &["PingFrame", "PingListenerFrame"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] =
    &["PingPinFrameTemplate", "PingSpotFrameTemplate"];

const C_PING_SECURE_CALLBACKS: &[&str] = &[
    "ClearPendingPingInfo",
    "CreateFrame",
    "DisplayError",
    "GetTargetPingReceiver",
    "GetTargetWorldPing",
    "GetTargetWorldPingAndSend",
    "SendPing",
    "SetPendingPingOffScreenCallback",
    "SetPingCooldownStartedCallback",
    "SetPingPinFrameAddedCallback",
    "SetPingPinFrameRemovedCallback",
    "SetPingPinFrameScreenClampStateUpdatedCallback",
    "SetPingRadialWheelCreatedCallback",
    "SetSendMacroPingCallback",
    "SetTogglePingListenerCallback",
];

const REQUIRED_LOCALIZED_STRINGS: &[&str] = &[
    "PING_TYPE_ASSIST",
    "PING_TYPE_ATTACK",
    "PING_TYPE_ON_MY_WAY",
    "PING_TYPE_WARNING",
    "PING_FAILED_GENERIC",
    "PING_FAILED_SPAMMING",
    "PING_FAILED_DISABLED_BY_LEADER",
    "PING_FAILED_DISABLED_BY_SETTINGS",
    "PING_FAILED_OUT_OF_PING_AREA",
    "PING_FAILED_SQUELCHED",
    "PING_FAILED_UNSPECIFIED",
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
fn blizzard_ping_ui_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&ping_ui_dir()).expect("Blizzard_PingUI TOC resolves");
    assert_eq!(
        resolved,
        ping_ui_toc(),
        "Blizzard_PingUI ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The Mainline-only restriction is enforced via `## AllowLoadGameType: mainline` \
         instead of a flavor-split TOC. The hot ping ring (Assist/Attack/OnMyWay/Warning) \
         is a Mainline-only social feature; Mists Classic and earlier flavors have no \
         radial-ping system at all"
    );

    let mainline = ping_ui_dir().join("Blizzard_PingUI_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_ping_ui_toc_declares_eager_secure_mainline_only_with_sharedxml_dep() {
    let toc = TocFile::from_file(&ping_ui_toc()).expect("Blizzard_PingUI TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_PingUI is non-LOD — the radial ping wheel must be wired before the \
         player ever taps the ping keybind, and the C_PingSecure callbacks \
         (SetPingRadialWheelCreatedCallback, SetPendingPingOffScreenCallback, \
         SetTogglePingListenerCallback, SetPingPinFrameAddedCallback, etc.) must be \
         installed before any ping subsystem fires; deferring would lose the very first \
         ping the player triggers in a fresh session"
    );
    assert!(!toc.is_load_first());
    assert!(
        toc.is_secure_env(),
        "TOC declares `## UseSecureEnvironment: 1` — UNUSUAL flag. The PingUI runs inside \
         the `secureenv` fenv (src/lua_api/globals/security/secure_env.rs:82-86 swaps the \
         compiled function's fenv to the registry-stored `__secureenv` table), so all \
         globals assigned by the addon code (PingFrameMixin, PingListenerFrameMixin, \
         PingPinFrameMixin, PingPinFlipBookAnimMixin, PingManager, \
         GetScaledCursorPosition_Insecure) land in `secureenv`, NOT in `_G`. The flag \
         protects the secure ping callback dispatch from addon-side stack-taint \
         poisoning; only an untainted call site can talk to C_PingSecure (per the \
         `SecretArguments = \"AllowedWhenUntainted\"` annotations in \
         Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua)"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Default-Game-only `allows_screen` at src/toc.rs:311 returns true for \
         ScreenKind::Game when AllowLoad is omitted — pings are an in-world social \
         action, not a glue-screen feature"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Omitted `## AllowLoad:` must NOT enable {screen:?} — radial ping is an \
             in-world player-facing UI; glue screens have no ping subsystem"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline` — `is_game_type_restricted` at \
         src/toc.rs:294-302 returns FALSE (allowed) when the value contains the \
         `mainline` or `standard` token. PingUI is a Mainline-only addon; the radial \
         ping wheel and the C_PingSecure callback surface do not exist on Mists Classic \
         or earlier flavors"
    );

    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS,
        "Retail 12.1.0.69497 declares Blizzard_SharedXML and Blizzard_FrameXMLUtil through \
         its singular RequiredDep line"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — PingUI is a stateless mirror of server-authoritative \
         ping state pulled via C_Ping.GetCooldownInfo / C_PingSecure each ping action. \
         pingMode is a CVar (read via `tonumber(GetCVar('pingMode'))` at line 207 of \
         Blizzard_PingUI.lua), not a saved variable"
    );
}

#[test]
fn blizzard_ping_ui_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(ping_ui_toc()).expect("Blizzard_PingUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_PingUI"),
        "TOC must declare `## Title: Blizzard_PingUI` exactly — UNUSUAL: the title uses \
         the underscore-namespace spelling (matching Blizzard_PerksProgram pattern), not \
         the space-and-prose form `Blizzard Ping UI`"
    );
    assert!(
        raw.contains("## RequiredDep: Blizzard_SharedXML, Blizzard_FrameXMLUtil"),
        "Retail 12.1.0.69497 declares both dependencies on its singular RequiredDep line"
    );
    assert!(
        raw.contains("## UseSecureEnvironment: 1"),
        "TOC must declare `## UseSecureEnvironment: 1` exactly — flips `is_secure_env()` \
         to true so the loader pipes every Lua/XML chunk through `mark_secure_state` \
         which sets the chunk's fenv to the `__secureenv` registry table"
    );
    assert!(
        raw.contains("## AllowLoadGameType: mainline"),
        "TOC must declare `## AllowLoadGameType: mainline` exactly — UNUSUAL token \
         choice (`mainline` rather than the more frequent `standard`); both tokens are \
         recognized by `is_game_type_restricted` as Mainline-allowed (src/toc.rs:299), \
         but `mainline` is the older retail-product spelling"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` — Game-only is the default behavior when \
         the key is omitted"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless mirror; \
         pingMode is a CVar"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — UNUSUAL omission compared to most \
         Blizzard-shipped addons. Together with the missing `## Version:`, the metadata \
         profile is bare-bones: only Title / RequiredDep / UseSecureEnvironment / \
         AllowLoadGameType. Matches the Blizzard_PetBattleUI minimal-metadata pattern"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — UNUSUAL omission; the addon is unversioned"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand:` — eager load is required so the \
         C_PingSecure callbacks are wired before the player's first ping keystroke"
    );
}

#[test]
fn blizzard_ping_ui_toc_lists_four_files_in_canonical_order() {
    let toc = TocFile::from_file(&ping_ui_toc()).expect("Blizzard_PingUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PING_UI_TOC_FILES,
        "Retail 12.1.0.69497 lists PingManager.lua, PingUI.lua, PingUI.xml, then \
         PingUtil.lua with LoadIntoEnvironment global. The first two Lua files still publish \
         symbols before PingUI.xml parses"
    );
}

#[test]
fn blizzard_ping_ui_appears_in_game_screen_eager_discovery_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_PingUI");
    assert!(
        in_game,
        "Blizzard_PingUI must appear in Game-screen eager discovery — it is non-LOD and \
         AllowLoad is omitted (defaults to Game-only via src/toc.rs:311). The eager \
         sweep at src/loader/mod.rs prioritizes UseSecureEnvironment addons in the \
         secure-pass first, so PingUI loads before any insecure addon Lua runs"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_PingUI");
        assert!(
            !found,
            "Blizzard_PingUI must NOT appear in {screen:?} eager discovery — radial \
             ping is an in-world social feature, glue screens have no ping subsystem"
        );
    }
}

#[test]
fn blizzard_ping_ui_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory.iter().any(|(name, _)| name == "Blizzard_PingUI");
    assert!(
        found,
        "Blizzard_PingUI must appear in `discover_all_blizzard_addons` — the full \
         inventory is a structural listing of every parseable TOC and includes \
         glue-restricted addons too"
    );
}

prefork_full_ui_case! {
fn blizzard_ping_ui_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PingUI")
                || message.contains("PingFrame")
                || message.contains("PingListener")
                || message.contains("PingPin")
                || message.contains("PingManager")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PingUI emitted addon-specific Lua errors during Game-screen load:\n  \
         {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PingUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PingUI') must return true after the eager \
         Game-screen sweep — the addon is non-LOD with `## UseSecureEnvironment: 1` so \
         it loads in the secure-pass first phase"
    );
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_secure_env_redirects_mixins_out_of_globals(env: &WowLuaEnv) {

    for mixin in SECURE_ENV_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{mixin} must be nil — Blizzard_PingUI declares `## UseSecureEnvironment: \
             1` so the addon's compiled chunks have their fenv swapped to the \
             `__secureenv` registry table by `mark_secure_state` \
             (src/lua_api/globals/security/secure_env.rs:82-86). Top-level assignments \
             like `PingFrameMixin = {{}}` write into that secureenv table, NOT into the \
             global `_G` table that addons see. This is the intended security behavior: \
             insecure addon Lua cannot read or mutate the ping mixin tables, which \
             blocks taint propagation through the secure ping dispatch path"
        );
    }

    assert_eq!(
        SECURE_ENV_MIXINS.len(),
        4,
        "SECURE_ENV_MIXINS must contain exactly 4 entries — pinned so XML/Lua mixin \
         additions surface here on a vendor TAG bump"
    );
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_secure_env_redirects_namespace_globals(env: &WowLuaEnv) {

    for namespace in SECURE_ENV_NAMESPACES {
        let kind: String = env
            .eval(&format!("return type(_G.{namespace})"))
            .unwrap_or_else(|err| panic!("type(_G.{namespace}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{namespace} must be nil — same `secureenv` redirect as the mixin tables. \
             Blizzard_PingManager.lua's top-level `PingManager = {{}}` writes to \
             secureenv. PingListenerFrameMixin:OnLoad calls PingManager:Initialize() \
             from inside secureenv where the lookup succeeds"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_secure_env_redirects_free_function_globals(env: &WowLuaEnv) {

    for func in SECURE_ENV_FREE_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G.{func})"))
            .unwrap_or_else(|err| panic!("type(_G.{func}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{func} must be nil — Blizzard_PingUI.lua line 2-5 declares the free \
             top-level function `GetScaledCursorPosition_Insecure(...)` which writes to \
             secureenv. PingListenerFrameMixin:SetCursorPositions invokes it via \
             `securecallfunction(GetScaledCursorPosition_Insecure)` (line 211) where \
             the lookup happens inside secureenv. Two more secureenv-local helpers \
             (GetWorldFrameCenter_Insecure, GetUIParentScale_Insecure) at lines 7-16 \
             are file-local because they're declared with `local function`, so they \
             never touch any globals table"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_creates_named_non_virtual_frames(env: &WowLuaEnv) {

    for frame in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a frame userdata (FrameRef reports as `'table'` \
             via the custom __type metamethod). Blizzard_PingUI.xml declares 2 named \
             non-virtual top-level Frames inside `<ScopedModifier forbidden=\"true\">`: \
             PingFrame (inherits=RadialWheelFrameTemplate, mixin=PingFrameMixin, \
             toplevel=true, parent=UIParent, frameStrata=HIGH, hidden=true) and \
             PingListenerFrame (mixin=PingListenerFrameMixin, parent=UIParent, \
             setAllPoints=true, registerForDrag=LeftButton, passThroughButtons=\
             RightButton, retainClickThroughOverride=true, frameStrata=FULLSCREEN, \
             hidden=true). XML-created frames register in `_G` regardless of the \
             ScopedModifier; `forbidden=true` only sets `frame.forbidden=true` for \
             IsForbidden() probes, it does NOT remove the frame from globals"
        );

        let name: String = env
            .eval(&format!("return _G.{frame}:GetName()"))
            .unwrap_or_else(|err| panic!("_G.{frame}:GetName() probe failed: {err}"));
        assert_eq!(
            name, *frame,
            "_G.{frame}:GetName() must round-trip the same name — the XML-driven name \
             registration writes `frame.name = \"{frame}\"` and registers under the \
             same key in `_G`"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_marks_named_frames_as_forbidden(env: &WowLuaEnv) {

    for frame in PUBLIC_NAMED_FRAMES {
        let forbidden: bool = env
            .eval(&format!("return _G.{frame}:IsForbidden()"))
            .unwrap_or_else(|err| panic!("_G.{frame}:IsForbidden() probe failed: {err}"));
        assert!(
            forbidden,
            "_G.{frame}:IsForbidden() must return true — Blizzard_PingUI.xml wraps every \
             frame in `<ScopedModifier forbidden=\"true\">`. The simulator's XML loader \
             at src/loader/xml_file.rs:82-104 (`process_scoped_modifier`) flips the \
             loader state's `forbidden` flag before processing the scoped children, so \
             every frame created inside the block is stamped \
             `widget::Frame::forbidden=true` (src/widget/frame.rs:498-500). The \
             IsForbidden Lua method (registered at \
             src/lua_api/frame/methods/text_attribute_event/mod.rs:244) reads that flag. \
             Forbidden frames cannot be reparented or modified by tainted code in real \
             WoW; the simulator stores the flag for inspection but does not yet enforce \
             the restriction at the API surface"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_does_not_leak_virtual_templates_to_globals(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the template \
             registry, NOT in `_G`. Leaking would let consumer addons mutate the \
             template definition and break every existing instance. \
             PingPinFrameTemplate is the per-pin frame for each in-world ping marker \
             (Icon + IconFlipBook + GroundPin + UnitPin + ClampedPin children, with \
             5 nested AnimationGroups for the intro/outro animations); \
             PingSpotFrameTemplate is the brief glow-pulse frame shown at the spot \
             where a ping was cast (GlowOut + GlowIn textures with a 4-track \
             PulseAnim AnimationGroup)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_publishes_c_ping_secure_callback_setters_in_globals(env: &WowLuaEnv) {

    let ns_kind: String = env
        .eval("return type(C_PingSecure)")
        .expect("type(C_PingSecure) probe succeeds");
    assert_eq!(
        ns_kind, "table",
        "C_PingSecure must publish as a global namespace table — the documentation \
         table at \
         Interface/BlizzardUI/Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua \
         names it `C_PingSecure` with `Environment = \"SecureOnly\"`. The simulator \
         publishes the namespace eagerly so PingFrameMixin:OnLoad / \
         PingListenerFrameMixin:OnLoad / PingManager:Initialize can wire all 14 \
         callback setters during the eager-load pass without needing the LoD \
         `Blizzard_APIDocumentationGenerated` addon to be loaded first"
    );

    for func in C_PING_SECURE_CALLBACKS {
        let kind: String = env
            .eval(&format!("return type(C_PingSecure.{func})"))
            .unwrap_or_else(|err| panic!("type(C_PingSecure.{func}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "C_PingSecure.{func} must be a function — required by the load-time \
             OnLoad / Initialize chain. Missing entries here would surface as \
             `attempt to call method '{func}' (a nil value)` errors during the secure \
             pass"
        );
    }

    assert_eq!(
        C_PING_SECURE_CALLBACKS.len(),
        15,
        "C_PING_SECURE_CALLBACKS must contain exactly 15 entries — the count is \
         pinned to the PingManagerSecureDocumentation.lua surface (14 callback \
         setters / probes + ClearPendingPingInfo). Adding/removing a function on \
         vendor TAG bumps surfaces here"
    );
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_publishes_enum_ping_mode_result_subject_type(env: &WowLuaEnv) {

    let ping_mode_kind: String = env
        .eval("return type(Enum.PingMode)")
        .expect("type(Enum.PingMode) probe succeeds");
    assert_eq!(
        ping_mode_kind, "table",
        "Enum.PingMode must publish as a table — referenced by \
         PingListenerFrameMixin:GetPingMode at line 207-208 of Blizzard_PingUI.lua \
         (returns `tonumber(GetCVar('pingMode'))`) and compared to `Enum.PingMode.\
         KeyDown` in OnMouseDown / OnMouseUp / OnDragStart / OnDragStop / \
         TogglePingListener (lines 88, 96, 106, 119, 154, 164). Backed by \
         missing_enums.lua:10659-10665"
    );

    let ping_mode_keydown: f64 = env
        .eval("return Enum.PingMode.KeyDown")
        .expect("Enum.PingMode.KeyDown probe succeeds");
    assert!(
        (0.0..=10.0).contains(&ping_mode_keydown),
        "Enum.PingMode.KeyDown must be a numeric variant in range [0, 10]; got \
         {ping_mode_keydown}"
    );

    let ping_result_kind: String = env
        .eval("return type(Enum.PingResult)")
        .expect("type(Enum.PingResult) probe succeeds");
    assert_eq!(
        ping_result_kind, "table",
        "Enum.PingResult must publish as a table — used as the return type of \
         C_PingSecure.SendPing per PingManagerSecureDocumentation.lua:88-89, and \
         compared to Enum.PingResult.{{Success, FailedSpamming, FailedGeneric, \
         FailedDisabledByLeader, FailedDisabledBySettings, FailedOutOfPingArea, \
         FailedSquelched, FailedUnspecified}} in PingManager:HandleSendPingError"
    );

    let ping_subject_kind: String = env
        .eval("return type(Enum.PingSubjectType)")
        .expect("type(Enum.PingSubjectType) probe succeeds");
    assert_eq!(
        ping_subject_kind, "table",
        "Enum.PingSubjectType must publish as a table — used as the input type of \
         C_PingSecure.SendPing per PingManagerSecureDocumentation.lua:82-84. \
         PingManager:DeterminePingTarget compares against \
         Enum.PingSubjectType.{{Assist, Attack, OnMyWay, Warning}} when building the \
         wedge-info payload"
    );
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_publishes_required_localized_strings(env: &WowLuaEnv) {

    for key in REQUIRED_LOCALIZED_STRINGS {
        let kind: String = env
            .eval(&format!("return type(_G.{key})"))
            .unwrap_or_else(|err| panic!("type(_G.{key}) probe failed: {err}"));
        assert_eq!(
            kind, "string",
            "_G.{key} must publish as a string — `data/global_strings.rs` ships the \
             enUS localization for all 11 PING_TYPE_* / PING_FAILED_* keys consumed by \
             Blizzard_PingManager.lua's wedge-info table and \
             Blizzard_PingUI.lua's `C_PingSecure.DisplayError(PING_FAILED_GENERIC)` \
             call site (line 114, 230). Missing entries would surface as \
             `attempt to concatenate global '{key}' (a nil value)` during \
             string formatting in the manager's pingType builder"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_ping_ui_radial_wheel_frame_mixin_is_provided_by_shared_xml(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(_G.RadialWheelFrameMixin)")
        .expect("type(_G.RadialWheelFrameMixin) probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.RadialWheelFrameMixin must publish as a table from Blizzard_SharedXML \
         (the addon's RequiredDep). PingFrameMixin:OnLoad calls \
         `RadialWheelFrameMixin.OnLoad(self)` at line 22 of Blizzard_PingUI.lua, so \
         the mixin must be visible at the point PingFrame's OnLoad fires. \
         Blizzard_SharedXML is NOT a UseSecureEnvironment addon \
         (Interface/BlizzardUI/Blizzard_SharedXML/Blizzard_SharedXML_Mainline.toc), so \
         RadialWheelFrameMixin lives in `_G` and is visible from inside Blizzard_PingUI's \
         secureenv via the shared globals lookup chain"
    );
}
}
