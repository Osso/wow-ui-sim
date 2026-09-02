use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn spectate_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SpectateFrame")
}

fn spectate_toc() -> PathBuf {
    spectate_dir().join("Blizzard_SpectateFrame.toc")
}

const PUBLISHED_MIXINS: &[&str] = &[
    "SpectateFrameMixin",
    "SpectateLeaveMatchButtonMixin",
    "MatchDetailsButtonMixin",
    "SpectateCycleModeMixin",
];

const SPECTATE_FRAME_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnEvent",
    "OnUpdate",
    "UpdateArrowText",
    "ShouldBeInSpectateMode",
    "StartZoomingFOV",
    "IsZoomingInFOV",
    "IsZoomingOutFOV",
    "IsZoomingFOV",
    "InitializeSpectateMode",
    "UpdatePlayerName",
    "LeaveSpectatingMode",
];

const STATIC_POPUP_DIALOGS: &[&str] = &[
    "CONFIRM_LEAVE_MATCH_WHILE_RESSURECTABLE",
    "CONFIRM_LEAVE_MATCH_WITH_PLUNDER",
    "CONFIRM_LEAVE_MATCH_WITH_PLUNDER_SOLO",
];

fn load_spectate_frame(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &spectate_toc())
        .expect("Blizzard_SpectateFrame should load via explicit Rust loader call");
}

#[test]
fn find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&spectate_dir()).expect("SpectateFrame TOC should resolve");
    assert_eq!(
        resolved,
        spectate_toc(),
        "Blizzard_SpectateFrame ships exactly one bare TOC `Blizzard_SpectateFrame.toc` \
         (NO `_Mainline.toc` flavor variant). Plunderstorm flavor gating is expressed via \
         `## AllowLoadGameType: plunderstorm` rather than separate flavor TOCs"
    );
}

#[test]
fn toc_declares_six_directives_and_current_dependencies() {
    let toc = TocFile::from_file(&spectate_toc()).expect("SpectateFrame TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "SpectateFrame omits `## LoadOnDemand:` — under the Plunderstorm client it \
         auto-discovers on the Game screen with `## DefaultState: enabled`; under standard \
         retail the game-type filter excludes it entirely. SPECTATE_BEGIN must already \
         have a registered listener when the server transitions a Plunderstorm \
         spectator into the match"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_UIPanels_Game".to_string(),
            "Blizzard_UIModes".to_string(),
        ],
        "Retail 12.1.0.69497 declares Blizzard_UIPanels_Game and Blizzard_UIModes \
         in published dependency order"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "SpectateFrame declares zero saved variables — match state is server-driven \
         (C_SpectatingUI / C_Commentator), no client-side persistence"
    );
    assert!(toc.default_enabled());
}

#[test]
fn toc_is_plunderstorm_only_and_game_only() {
    let toc = TocFile::from_file(&spectate_toc()).expect("SpectateFrame TOC parses");

    assert!(
        toc.is_game_type_restricted(),
        "SpectateFrame declares `## AllowLoadGameType: plunderstorm` — \
         neither `mainline` nor `standard` is in the gametype list at \
         src/toc.rs:298-299, so is_game_type_restricted (src/toc.rs:294) \
         returns true. The auto-discovery sweep at src/loader/mod.rs \
         filters this addon out on standard retail; only Plunderstorm \
         clients pick it up automatically. The simulator's default \
         flavor target is mainline, so the addon stays excluded from \
         eager discovery in tests"
    );

    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: Game` MUST exclude Login glue — spectate mode is \
         reachable only after the player has joined a Plunderstorm match"
    );
    assert!(!toc.allows_screen(ScreenKind::CharacterSelect));
    assert!(!toc.allows_screen(ScreenKind::CharacterCreate));
}

#[test]
fn raw_bytes_pin_six_metadata_directives() {
    let raw = std::fs::read_to_string(spectate_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_SpectateFrame",
        "## Author: Blizzard Entertainment",
        "## DefaultState: enabled",
        "## Dependencies: Blizzard_UIPanels_Game, Blizzard_UIModes",
        "## AllowLoadGameType: plunderstorm",
        "## AllowLoad: Game",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw bytes MUST pin `{directive}` — SpectateFrame TOC is \
             small (6 metadata lines + 2 body entries)"
        );
    }

    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## Version"));
}

#[test]
fn body_lists_lua_before_xml() {
    let toc = TocFile::from_file(&spectate_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = ["SpectateFrame.lua", "SpectateFrame.xml"];

    assert_eq!(
        body.len(),
        expected.len(),
        "Body must contain exactly 2 entries — addon ships one .lua + \
         one .xml. Got: {body:?}"
    );

    for (i, want) in expected.iter().enumerate() {
        assert_eq!(
            &body[i], want,
            "Body entry {i}: expected {want}, got {}",
            body[i]
        );
    }

    assert!(
        body[0].ends_with(".lua") && body[1].ends_with(".xml"),
        "SpectateFrame.lua MUST load BEFORE SpectateFrame.xml — the XML \
         references SpectateFrameMixin / SpectateCycleModeMixin / \
         SpectateLeaveMatchButtonMixin / MatchDetailsButtonMixin via \
         `mixin=\"…\"` attributes on the named Frame and its child \
         Buttons. The XML loader resolves the mixin tables at \
         template-registration time, so they MUST already exist as _G \
         tables when the .xml chunk is processed"
    );
}

#[test]
fn excluded_from_every_screen_auto_discovery() {
    let screens = [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ];

    for screen in screens {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SpectateFrame");
        assert!(
            !found,
            "Blizzard_SpectateFrame must be filtered out of auto-discovery on \
             standard retail across every ScreenKind. The TOC declares \
             `## AllowLoadGameType: plunderstorm`, and \
             discover_blizzard_addons_for_screen skips game-type-restricted \
             addons unless the active game type matches. (Screen tested: \
             {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn explicit_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let cross_addon_gaps = ["EndOfMatchFrame"];

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            let mentions_addon = message.contains("Blizzard_SpectateFrame")
                || message.contains("SpectateFrameMixin")
                || message.contains("SpectateCycleModeMixin")
                || message.contains("SpectateLeaveMatchButtonMixin")
                || message.contains("MatchDetailsButtonMixin")
                || message.contains("LeaveMatchUtil_LeaveMatchPopup");
            let is_known_cross_addon_gap = cross_addon_gaps.iter().any(|gap| message.contains(gap));
            mentions_addon && !is_known_cross_addon_gap
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_SpectateFrame emitted addon-specific Lua errors during \
         load (excluding known cross-addon gaps from sibling plunderstorm \
         addons that are not loaded under the simulator's default mainline \
         flavor — EndOfMatchFrame in particular ships with a separate \
         plunderstorm-gated addon, and SpectateFrameMixin:OnShow at \
         SpectateFrame.lua:23 dereferences it via \
         `EndOfMatchFrame:HasMatchDetails()`. The `mentions_addon && \
         !is_known_cross_addon_gap` predicate keeps the test focused on \
         genuine SpectateFrame regressions while tolerating the documented \
         sibling-addon resolution gap):\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_explicit_load(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SpectateFrame')")
        .expect("IsAddOnLoaded probe");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_SpectateFrame') must return true \
         after explicit load_addon — confirms the loader registered the \
         addon name in the loaded-set even though auto-discovery skipped \
         it on the mainline simulator target"
    );
}
}

prefork_full_ui_case! {
fn publishes_four_mixin_tables(env: &WowLuaEnv) {
    load_spectate_frame(env);

    for mixin in PUBLISHED_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} type probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at `_G` as a table after \
             Blizzard_SpectateFrame loads. The 4 mixins map to: \
             SpectateFrameMixin (root frame controller — registers 6 \
             SPECTATE_*/PLAYER_* events, OnUpdate drives FOV zoom \
             throttled at 0.1s, OnShow rebinds the strafe arrow text \
             from GetBindingKey lookups), SpectateCycleModeMixin (the \
             two arrow buttons SpectateFrameArrowLeft / Right — OnClick \
             dispatches C_SpectatingUI.SpectateChange(self.spectateNext) \
             driven by the per-button KeyValue boolean), \
             SpectateLeaveMatchButtonMixin (Leave-match button — OnClick \
             routes through LeaveMatchUtil_LeaveMatchPopup gating), \
             MatchDetailsButtonMixin (Return-to-match-details button — \
             OnClick exits spectate mode then triggers \
             EventRegistry:TriggerEvent('EndOfMatchUI.TryShow', false))"
        );
    }
}
}

prefork_full_ui_case! {
fn spectate_frame_mixin_publishes_thirteen_canonical_methods(env: &WowLuaEnv) {
    load_spectate_frame(env);

    for method in SPECTATE_FRAME_METHODS {
        let kind: String = env
            .eval(&format!("return type(SpectateFrameMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("SpectateFrameMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "SpectateFrameMixin.{method} must publish as a function. The \
             13-method surface decomposes into: 4 XML-wired script \
             handlers (OnLoad / OnShow / OnEvent / OnUpdate); 1 input \
             helper UpdateArrowText (resolves strafe vs turn binding key \
             with preferred-key fallback); 4 zoom probes \
             (StartZoomingFOV / IsZoomingInFOV / IsZoomingOutFOV / \
             IsZoomingFOV — IsKeyDown polling on MOVEFORWARD/BACKWARD); \
             4 mode-state methods (ShouldBeInSpectateMode / \
             InitializeSpectateMode / UpdatePlayerName / \
             LeaveSpectatingMode — the last one calls SetFrameLock + \
             EditModeManagerFrame:ClearOverrideLayout + \
             C_EditMode.SetActiveLayout(1) to restore the default \
             modern preset layout)"
        );
    }
}
}

prefork_full_ui_case! {
fn cycle_mode_mixin_carries_on_click_and_set_arrow_text_methods(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let probe = "return type(SpectateCycleModeMixin.OnClick) == 'function' and \
                 type(SpectateCycleModeMixin.SetArrowText) == 'function'";
    let result: bool = env.eval(probe).expect("CycleMode methods probe");
    assert!(
        result,
        "SpectateCycleModeMixin must own exactly 2 methods: OnClick \
         (dispatches C_SpectatingUI.SpectateChange with the per-button \
         spectateNext KeyValue boolean) and SetArrowText (writes both \
         the OVERLAY-layer Text and HIGHLIGHT-layer HighlightText \
         FontStrings so the arrow caption reads consistently in both \
         hover states)"
    );
}
}

prefork_full_ui_case! {
fn leave_match_button_mixin_owns_on_click_only(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let kind: String = env
        .eval("return type(SpectateLeaveMatchButtonMixin.OnClick)")
        .expect("LeaveMatch.OnClick probe");
    assert_eq!(
        kind, "function",
        "SpectateLeaveMatchButtonMixin.OnClick must publish as a \
         function — single-method mixin that delegates to the global \
         LeaveMatchUtil_LeaveMatchPopup gate (which branches on \
         UnitIsDeadOrGhost('player') / GetNumGroupMembers / \
         C_CurrencyInfo.GetCurrencyInfo(PLUNDER_CURRENCY_ID).quantity \
         to pick the right CONFIRM_LEAVE_MATCH_* dialog before calling \
         ForceLogout)"
    );
}
}

prefork_full_ui_case! {
fn match_details_button_mixin_owns_on_click_only(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let kind: String = env
        .eval("return type(MatchDetailsButtonMixin.OnClick)")
        .expect("MatchDetails.OnClick probe");
    assert_eq!(
        kind, "function",
        "MatchDetailsButtonMixin.OnClick must publish as a function — \
         exits spectator mode (parent:LeaveSpectatingMode), then \
         triggers EventRegistry:TriggerEvent('EndOfMatchUI.TryShow', \
         false) to surface the post-match details panel"
    );
}
}

prefork_full_ui_case! {
fn leave_match_util_global_function_publishes(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let kind: String = env
        .eval("return type(LeaveMatchUtil_LeaveMatchPopup)")
        .expect("LeaveMatchUtil probe");
    assert_eq!(
        kind, "function",
        "LeaveMatchUtil_LeaveMatchPopup must publish at `_G` as a global \
         function — gates the leave-match flow by branching on \
         UnitIsDeadOrGhost, group composition, and PLUNDER_CURRENCY_ID \
         (3011) quantity to pick the right CONFIRM_LEAVE_MATCH_* \
         StaticPopup. The free-standing global is intentional: \
         SpectateLeaveMatchButtonMixin.OnClick at SpectateFrame.lua:206 \
         AND the in-match GameMenu both call it"
    );
}
}

prefork_full_ui_case! {
fn three_static_popup_dialogs_register(env: &WowLuaEnv) {
    load_spectate_frame(env);

    for dialog in STATIC_POPUP_DIALOGS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupDialogs['{dialog}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupDialogs[{dialog}] probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "StaticPopupDialogs['{dialog}'] must publish as a table — \
             SpectateFrame.lua registers 3 dialogs at file scope: \
             CONFIRM_LEAVE_MATCH_WHILE_RESSURECTABLE (fires when player \
             is dead but a teammate could resurrect — uses \
             customAlertIcon=Interface\\\\RaidFrame\\\\Raid-Icon-Rez), \
             CONFIRM_LEAVE_MATCH_WITH_PLUNDER (fires when player has \
             plunder + group context), CONFIRM_LEAVE_MATCH_WITH_PLUNDER_SOLO \
             (fires when player has plunder + solo context). All three \
             share button1=WOW_LABS_REMATCH / button2=WOW_LABS_STAY / \
             whileDead=1 / hideOnEscape=1 / exclusive=1"
        );
    }
}
}

prefork_full_ui_case! {
fn ressurectable_dialog_carries_custom_alert_icon(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let icon: String = env
        .eval(
            "return tostring(StaticPopupDialogs['CONFIRM_LEAVE_MATCH_WHILE_RESSURECTABLE']\
             .customAlertIcon)",
        )
        .expect("ressurectable icon probe");
    assert!(
        icon.contains("Raid-Icon-Rez"),
        "CONFIRM_LEAVE_MATCH_WHILE_RESSURECTABLE.customAlertIcon must \
         include the resurrection icon path \
         `Interface\\\\RaidFrame\\\\Raid-Icon-Rez` — visually \
         distinguishes the resurrection-pending dialog from the \
         plunder-loss dialogs. Got: {icon:?}"
    );
}
}

prefork_full_ui_case! {
fn named_spectate_frame_publishes_with_arrow_buttons_and_setallpoints(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let probe = "local f = SpectateFrame \
                 if not f then return 'nil' end \
                 if f:GetName() ~= 'SpectateFrame' then return 'name='..f:GetName() end \
                 if not f:GetParent() then return 'no parent' end \
                 if type(SpectateFrameArrowLeft) ~= 'table' then return 'no ArrowLeft global' end \
                 if type(SpectateFrameArrowRight) ~= 'table' then return 'no ArrowRight global' end \
                 if type(f.PlayerName) ~= 'table' then return 'no PlayerName' end \
                 if type(f.Spectating) ~= 'table' then return 'no Spectating' end \
                 if type(f.Shadow) ~= 'table' then return 'no Shadow' end \
                 return 'OK'";
    let report: String = env.eval(probe).expect("SpectateFrame probe");
    assert_eq!(
        report, "OK",
        "After explicit load, the named global SpectateFrame MUST \
         resolve to a Frame with: GetName() == 'SpectateFrame', \
         non-nil parent (UIParent via `parent=\"UIParent\"` + \
         `setAllPoints=\"true\"`), parentKey-published OVERLAY-layer \
         FontStrings (PlayerName / Spectating using GameFontHighlightHuge \
         / Game16Font with NORMAL_FONT_COLOR), BACKGROUND-layer Shadow \
         Texture (atlas plunderstorm-spectate-background, useAtlasSize), \
         AND globally-named child Buttons SpectateFrameArrowLeft / \
         SpectateFrameArrowRight (the two arrow buttons get explicit \
         names instead of just parentKey because their bindings text \
         labels need stable globals for /run debug access)"
    );
}
}

prefork_full_ui_case! {
fn arrow_buttons_carry_spectate_next_key_value_booleans(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let probe = "return tostring(SpectateFrameArrowLeft.spectateNext) \
                 .. ',' .. tostring(SpectateFrameArrowRight.spectateNext)";
    let report: String = env.eval(probe).expect("spectateNext probe");
    assert_eq!(
        report, "false,true",
        "SpectateFrameArrowLeft.spectateNext MUST be false and \
         SpectateFrameArrowRight.spectateNext MUST be true — declared \
         via `<KeyValue key=\"spectateNext\" value=\"false|true\" \
         type=\"boolean\"/>` so each arrow's OnClick dispatches \
         C_SpectatingUI.SpectateChange with the right cycle direction. \
         The parentKey + named-button duplication on the Right button \
         is intentional: parentKey=ArrowRight gives \
         SpectateFrame.ArrowRight access for SpectateFrameMixin.OnShow \
         (UpdateArrowText fan-out), and name=SpectateFrameArrowRight \
         gives global access for /run inspection. Got: {report:?}"
    );
}
}

prefork_full_ui_case! {
fn match_details_and_leave_match_buttons_publish_via_parent_keys(env: &WowLuaEnv) {
    load_spectate_frame(env);

    let probe = "return type(SpectateFrame.LeaveMatchButton) == 'table' and \
                 type(SpectateFrame.MatchDetailsButton) == 'table'";
    let result: bool = env.eval(probe).expect("button parentKey probe");
    assert!(
        result,
        "SpectateFrame.LeaveMatchButton and SpectateFrame.MatchDetailsButton \
         must publish via parentKey assignment — both inherit \
         `UIPanelButtonNoTooltipResizeToFitTemplate`, both carry \
         `fixedHeight=32` + `widthPadding=80` KeyValues for the resize-fit \
         template, and the MatchDetails button anchors LEFT of the \
         LeaveMatch button (-12 px gap)"
    );
}
}
