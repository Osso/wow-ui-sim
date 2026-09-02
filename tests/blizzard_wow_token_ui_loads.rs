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

fn token_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_WowTokenUI")
}

fn token_ui_toc() -> PathBuf {
    token_ui_dir().join("Blizzard_WowTokenUI.toc")
}

const REQUIRED_DEPS: &[&str] = &["Blizzard_SharedXML"];

const BODY_FILES: &[&str] = &["Blizzard_WowTokenUI.xml", "Blizzard_WowTokenUIInsecure.xml"];

const INBOUND_BRIDGE_FUNCTIONS: &[&str] = &[
    "WowToken_IsWowTokenAuctionDialogShown",
    "WowTokenRedemptionFrame_EscapePressed",
    "WowTokenRedemptionFrame_GetBalanceString",
    "WowTokenRedemptionFrame_ShowDialog",
];

const REDEMPTION_FRAME_HANDLERS: &[&str] = &[
    "WowTokenRedemptionFrame_OnLoad",
    "WowTokenRedemptionFrame_OnShow",
    "WowTokenRedemptionFrame_OnHide",
    "WowTokenRedemptionFrame_OnEvent",
    "WowTokenRedemptionFrame_OnAttributeChanged",
    "WowTokenRedemptionFrame_Update",
];

const DIALOG_HANDLERS: &[&str] = &[
    "WowTokenDialog_OnLoad",
    "WowTokenDialog_OnShow",
    "WowTokenDialog_OnHide",
    "WowTokenDialog_OnEvent",
    "WowTokenDialog_SetDialog",
    "WowTokenDialog_HideDialog",
    "WowTokenDialogButton_OnClick",
];

const NAMED_FRAMES: &[&str] = &["WowTokenRedemptionFrame", "WowTokenDialog"];

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
fn find_toc_file_resolves_bare_variant() {
    let resolved = find_toc_file(&token_ui_dir()).expect("Blizzard_WowTokenUI TOC should resolve");
    assert_eq!(
        resolved,
        token_ui_toc(),
        "Blizzard_WowTokenUI ships exactly one bare TOC — find_toc_file probes the \
         `_Mainline.toc` variant first (miss) and falls through to the bare TOC name (hit). \
         The token UI is a universal addon shared across every retail flavor — Blizzard does \
         not ship the WoW Token feature on classic-flavor servers, so no `_Classic.toc` / \
         `_Mists.toc` variant exists"
    );
}

#[test]
fn toc_declares_secure_environment_with_two_required_deps() {
    let toc = TocFile::from_file(&token_ui_toc()).expect("Blizzard_WowTokenUI TOC should parse");

    assert!(
        toc.is_secure_env(),
        "Blizzard_WowTokenUI declares `## UseSecureEnvironment: 1` — FIRST campaign addon \
         analyzed exercising this directive. Secure-env addons are loaded BEFORE non-secure \
         addons in src/loader/mod.rs:626 (the first eager pass collects \
         `is_load_first() || is_secure_env()` addons), ensuring the token UI's secure code \
         exists before any tainted addon code can run. The WoW Token redeems real-money \
         purchases and must run with stack-taint nil so its C_WowTokenSecure / C_StoreSecure \
         calls aren't blocked by `issecure()` checks"
    );

    let deps: Vec<String> = toc.dependencies().to_vec();
    assert_eq!(
        deps,
        REQUIRED_DEPS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "Blizzard_WowTokenUI declares exactly `## RequiredDep: Blizzard_SharedXML`. The \
         secure-environment addon has one explicit loader dependency: Blizzard_SharedXML, \
         which supplies shared UI templates before the token XML files load"
    );

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_WowTokenUI is NOT LoD — secure-env addons are eagerly loaded. The token \
         dialog and redemption frame must be ready BEFORE the player navigates to the \
         AuctionHouse / Recruit-A-Friend / character-services flows that pop them"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_WowTokenUI does NOT declare LoadFirst — UseSecureEnvironment alone qualifies \
         it for the first eager pass; LoadFirst is reserved for addons that must run before \
         even the secure-env addons (currently zero addons in this tree do)"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
}

#[test]
fn toc_omits_allow_load_directives() {
    let toc = TocFile::from_file(&token_ui_toc()).expect("Blizzard_WowTokenUI TOC should parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Without `## AllowLoad`, allows_screen falls through to the default Game-only branch"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Without `## AllowLoad`, allows_screen rejects glue screen {screen:?}"
        );
    }

    assert!(
        !toc.is_game_type_restricted(),
        "Without `## AllowLoadGameType`, is_game_type_restricted returns false — the token \
         UI ships on every retail flavor"
    );
}

#[test]
fn toc_body_lists_two_xml_files_only() {
    let toc = TocFile::from_file(&token_ui_toc()).expect("Blizzard_WowTokenUI TOC should parse");

    let body_files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    assert_eq!(
        body_files,
        BODY_FILES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "TOC body lists ONLY the 2 XML files (Blizzard_WowTokenUI.xml + \
         Blizzard_WowTokenUIInsecure.xml). The 4 lua files are NOT in the TOC body — they \
         load via `<Script file>` directives. Blizzard_WowTokenUI.xml has 3 `<Script file>` \
         lines at the top (Inbound.lua, WowTokenUI.lua, Outbound.lua) and \
         Blizzard_WowTokenUIInsecure.xml has 1 (Insecure.lua). The XML wrapper is the \
         mechanism by which the secure/insecure boundary is enforced — the inbound and main \
         lua files run inside the `Blizzard_WowTokenUI.xml` ScopedModifier forbidden=\"true\" \
         scope (i.e. the secure env after UseSecureEnvironment lifts the addon there), \
         while Blizzard_WowTokenUIInsecure.lua runs from a SEPARATE XML file deliberately \
         placed at the END of the TOC body so the `SwapToGlobalEnvironment()` calls inside \
         each lua file establish the correct env BEFORE any function in that file is defined"
    );
}

#[test]
fn toc_raw_bytes_pin_directives() {
    let raw = std::fs::read_to_string(token_ui_toc()).expect("Blizzard_WowTokenUI TOC should read");

    for directive in [
        "## Title: Blizzard WoW Token UI",
        "## RequiredDep: Blizzard_SharedXML",
        "## UseSecureEnvironment: 1",
        "Blizzard_WowTokenUI.xml",
        "Blizzard_WowTokenUIInsecure.xml",
    ] {
        assert!(
            raw.contains(directive),
            "TOC raw bytes must contain `{directive}` — secure-env token UI"
        );
    }

    for absent_directive in [
        "## Author:",
        "## Version:",
        "## Notes:",
        "## DefaultState:",
        "## Dependencies:",
        "## RequiredDeps:",
        "## OptionalDeps:",
        "## LoadOnDemand:",
        "## LoadFirst:",
        "## LoadWith:",
        "## SavedVariables:",
        "## AllowLoad:",
        "## AllowLoadGameType:",
    ] {
        assert!(
            !raw.contains(absent_directive),
            "TOC raw bytes must NOT contain `{absent_directive}`"
        );
    }
}

#[test]
fn directory_holds_seven_entries() {
    let entries: Vec<String> = std::fs::read_dir(token_ui_dir())
        .expect("Blizzard_WowTokenUI directory should exist")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        7,
        "Blizzard_WowTokenUI directory must hold exactly 7 entries: 1 toc + 4 lua \
         (WowTokenUI / Inbound / Outbound / Insecure) + 2 xml (WowTokenUI / Insecure). \
         The 4-lua / 2-xml split implements the secure↔insecure boundary architecture. \
         Got: {entries:?}"
    );
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in REQUIRED_DEPS {
        let dep_dir = blizzard_ui_dir().join(dep);
        assert!(
            dep_dir.is_dir(),
            "Required dep `{dep}` directory must exist at \
             `Interface/BlizzardUI/{dep}/` — the token UI's sole declared dependency"
        );
    }
}

#[test]
fn appears_in_game_eager_discovery() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let names: Vec<&str> = addons.iter().map(|(name, _)| name.as_str()).collect();
    assert!(
        names.contains(&"Blizzard_WowTokenUI"),
        "Blizzard_WowTokenUI must appear in Game-screen eager discovery — no LoD, no \
         AllowLoadGameType restriction. Furthermore the addon's UseSecureEnvironment:1 puts \
         it in the FIRST eager pass at src/loader/mod.rs:626 alongside any LoadFirst-marked \
         addons, so it loads before non-secure addons"
    );
}

#[test]
fn absent_from_glue_screen_auto_discovery() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let names: Vec<&str> = addons.iter().map(|(name, _)| name.as_str()).collect();
        assert!(
            !names.contains(&"Blizzard_WowTokenUI"),
            "Blizzard_WowTokenUI must NOT appear in {screen:?} eager discovery — token \
             redemption only makes sense in-game (the player must be logged into a character \
             to redeem game-time or claim recruit-a-friend rewards)"
        );
    }
}

prefork_full_ui_case! {
fn loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_WowTokenUI")
                || message.contains("WowTokenRedemptionFrame")
                || message.contains("WowTokenDialog")
                || message.contains("WowTokenOutbound")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_WowTokenUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_pass(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_WowTokenUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_WowTokenUI') must return true after the eager Game \
         pass — secure-env addons are eagerly loaded in the first pass"
    );

    for dep in REQUIRED_DEPS {
        let dep_loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{dep}')"))
            .unwrap_or_else(|err| panic!("dep {dep} probe failed: {err}"));
        assert!(
            dep_loaded,
            "Required dep `{dep}` must also be loaded — both core addons are eager-loaded"
        );
    }
}
}

prefork_full_ui_case! {
fn inbound_bridge_functions_publish_only_into_global_environment(env: &WowLuaEnv) {

    for fn_name in INBOUND_BRIDGE_FUNCTIONS {
        let (global_kind, secure_kind): (String, String) = env
            .eval(&format!(
                "return type(_G.{fn_name}), type(__secureenv.{fn_name})"
            ))
            .unwrap_or_else(|err| panic!("{fn_name} publication probe failed: {err}"));
        assert_eq!(
            global_kind, "function",
            "{fn_name} must publish in `_G` after Blizzard_WowTokenUIInbound.lua calls \
             `SwapToGlobalEnvironment()`"
        );
        assert_eq!(
            secure_kind, "nil",
            "{fn_name} must not remain in `__secureenv` after the environment swap"
        );
    }
}
}

prefork_full_ui_case! {
fn outbound_table_publishes_only_into_secure_environment(env: &WowLuaEnv) {

    let (global_kind, secure_kind): (String, String) = env
        .eval("return type(_G.WowTokenOutbound), type(__secureenv.WowTokenOutbound)")
        .expect("WowTokenOutbound publication probe should succeed");
    assert_eq!(
        global_kind, "nil",
        "WowTokenOutbound must not publish as a global table"
    );
    assert_eq!(
        secure_kind, "table",
        "Blizzard_WowTokenUIOutbound.lua must publish WowTokenOutbound into the captured \
         secure environment"
    );

    for outbound_fn in [
        "RedeemFailed",
        "AuctionWowTokenUpdate",
        "RecruitAFriendTryPlayClaimRewardFanfare",
        "RecruitAFriendTryCancelAutoClaim",
    ] {
        let fn_kind: String = env
            .eval(&format!(
                "return type(__secureenv.WowTokenOutbound.{outbound_fn})"
            ))
            .unwrap_or_else(|err| {
                panic!("__secureenv.WowTokenOutbound.{outbound_fn} probe failed: {err}")
            });
        assert_eq!(
            fn_kind, "function",
            "__secureenv.WowTokenOutbound.{outbound_fn} must be a function — each one wraps a \
             `securecall(\"...\", ...)` to dispatch into the insecure side. The 4 outbound \
             functions cover token-redeem-failed (calls RedeemFailed in Insecure.lua), \
             auction-house token market price update, and 2 RecruitAFriend reward fanfare \
             triggers"
        );
    }
}
}

prefork_full_ui_case! {
fn insecure_redeem_failed_publishes_only_into_global_environment(env: &WowLuaEnv) {

    let (global_kind, secure_kind): (String, String) = env
        .eval("return type(_G.RedeemFailed), type(__secureenv.RedeemFailed)")
        .expect("RedeemFailed publication probe should succeed");
    assert_eq!(
        global_kind, "function",
        "RedeemFailed must publish in `_G` after Blizzard_WowTokenUIInsecure.lua calls \
         `SwapToGlobalEnvironment()`"
    );
    assert_eq!(
        secure_kind, "nil",
        "RedeemFailed must not remain in `__secureenv` after the environment swap"
    );
}
}

prefork_full_ui_case! {
fn redemption_frame_handlers_publish_into_secure_environment(env: &WowLuaEnv) {

    for handler in REDEMPTION_FRAME_HANDLERS {
        let kind: String = env
            .eval(&format!("return type(__secureenv.{handler})"))
            .unwrap_or_else(|err| panic!("__secureenv.{handler} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{handler} must publish in `__secureenv` as a function — declared at file \
             scope in Blizzard_WowTokenUI.lua which has NO SwapToGlobalEnvironment call and \
             therefore explicitly stays in the secure env. The XML script-handler lookup \
             resolves via the function's compiled fenv, so referencing the handler by name \
             from the WowTokenRedemptionFrame frame's script attribute lands the function \
             in the per-frame script slot regardless of whether it's in `_G` or \
             `__secureenv`"
        );
    }
}
}

prefork_full_ui_case! {
fn dialog_handlers_publish_into_secure_environment(env: &WowLuaEnv) {

    for handler in DIALOG_HANDLERS {
        let kind: String = env
            .eval(&format!("return type(__secureenv.{handler})"))
            .unwrap_or_else(|err| panic!("__secureenv.{handler} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{handler} must publish in `__secureenv` as a function — same reasoning as the \
             redemption-frame handlers. The dialog driver functions are intentionally kept \
             in the secure env so tainted addon code cannot replace WowTokenDialog_OnEvent \
             with a handler that reads dialog data and exfiltrates it to a third-party UI"
        );
    }
}
}

prefork_full_ui_case! {
fn wow_token_button_template_registers_as_virtual(env: &WowLuaEnv) {
    let _env = env;
    assert!(
        wow_ui_sim::xml::get_template("WowTokenButtonTemplate").is_some(),
        "WowTokenButtonTemplate (`<Button virtual=\"true\">` from Blizzard_WowTokenUI.xml \
         line 7) must register in the template registry. The template is wrapped in a \
         `<ScopedModifier forbidden=\"true\">` parent element, so frames instantiated from \
         it inherit the forbidden flag — the dialog button row INSIDE the secure dialog uses \
         this template for both Confirm and Cancel buttons. The template implements 3-slice \
         button textures (Left/Middle/Right with explicit TexCoords on UI-Panel-Button-Up) \
         and swaps to UI-Panel-Button-Down/Disabled atlases on OnMouseDown/OnDisable"
    );
}
}

prefork_full_ui_case! {
fn named_frames_publish_with_dialog_strata_and_uiparent_parent(env: &WowLuaEnv) {

    for frame_name in NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type({frame_name})"))
            .unwrap_or_else(|err| panic!("{frame_name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{frame_name} must publish at `_G` as a table after the eager Game pass"
        );

        let actual_name: String = env
            .eval(&format!("return {frame_name}:GetName()"))
            .unwrap_or_else(|err| panic!("{frame_name}:GetName probe failed: {err}"));
        assert_eq!(actual_name, *frame_name);

        let strata: String = env
            .eval(&format!("return {frame_name}:GetFrameStrata()"))
            .unwrap_or_else(|err| panic!("{frame_name}:GetFrameStrata probe failed: {err}"));
        assert_eq!(
            strata, "DIALOG",
            "{frame_name} must declare frameStrata=\"DIALOG\" — both the redemption frame \
             and the wow-token dialog are modal-style fullscreen overlays that must sit \
             above HIGH-strata UIPanels but below TOOLTIP-strata tooltips"
        );
    }
}
}

prefork_full_ui_case! {
fn redemption_frame_inherits_default_panel_template(env: &WowLuaEnv) {

    let object_type: String = env
        .eval("return WowTokenRedemptionFrame:GetObjectType()")
        .expect("GetObjectType probe should succeed");
    assert_eq!(
        object_type, "Frame",
        "WowTokenRedemptionFrame must be a Frame (NOT a Button or Dialog) — the XML element \
         is `<Frame name=\"WowTokenRedemptionFrame\" inherits=\"DefaultPanelTemplate\" ...>`. \
         The DefaultPanelTemplate inheritance contributes the standard portrait-frame chrome \
         (close button + title bar + nine-slice border)"
    );
}
}

prefork_full_ui_case! {
fn wow_token_dialog_publishes_with_toplevel_flag(env: &WowLuaEnv) {

    let object_type: String = env
        .eval("return WowTokenDialog:GetObjectType()")
        .expect("GetObjectType probe should succeed");
    assert_eq!(object_type, "Frame");

    let is_toplevel: bool = env
        .eval("return WowTokenDialog:IsToplevel()")
        .expect("IsToplevel probe should succeed");
    assert!(
        is_toplevel,
        "WowTokenDialog must declare `toplevel=\"true\"` in XML — the toplevel flag means \
         the frame raises itself to the top of its strata when clicked, so the dialog stays \
         on top even if other DIALOG-strata frames are shown after it. The redemption frame \
         does NOT have toplevel=true — only the wow-token dialog (which can stack on top of \
         the redemption frame) needs the click-raise behavior"
    );
}
}

prefork_full_ui_case! {
fn forbidden_scoped_modifier_marks_named_frames_forbidden(env: &WowLuaEnv) {

    for frame_name in NAMED_FRAMES {
        let is_forbidden: bool = env
            .eval(&format!("return {frame_name}:IsForbidden()"))
            .unwrap_or_else(|err| panic!("{frame_name}:IsForbidden probe failed: {err}"));
        assert!(
            is_forbidden,
            "{frame_name} must report IsForbidden=true — Blizzard_WowTokenUI.xml wraps both \
             named frames + the WowTokenButtonTemplate inside a single \
             `<ScopedModifier forbidden=\"true\">` element at line 6. The XML element \
             handler propagates the forbidden flag to every direct + transitive child, so \
             every frame and template defined inside the scope is marked forbidden. \
             Forbidden frames are subject to extra security guards in the C API surface — \
             `IsForbidden()` returning true here proves the ScopedModifier propagation \
             reached both named frames"
        );
    }
}
}
