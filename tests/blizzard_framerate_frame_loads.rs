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

fn framerate_frame_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_FramerateFrame/Blizzard_FramerateFrame_Mainline.toc")
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
fn blizzard_framerate_frame_toc_is_eager_with_action_bar_dep_and_allow_load_game_only() {
    let toc = TocFile::from_file(&framerate_frame_toc())
        .expect("Blizzard_FramerateFrame TOC should parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_FramerateFrame has no `## LoadOnDemand` line — the FramerateFrame is \
         a permanent in-world FPS counter that needs to be ready when the user toggles \
         it via the Show Framerate option, so it must auto-load at startup"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_FramerateFrame does not declare `## UseSecureEnvironment` — the FPS \
         counter is a passive read-only display in the standard taint environment"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_ActionBar".to_string()],
        "Blizzard_FramerateFrame declares `## Dependencies: Blizzard_ActionBar` — the \
         single dependency provides MicroMenu / MicroMenuContainer / \
         MicroMenuPositionEnum which OnLoad and UpdatePosition reference for anchoring \
         the FPS counter relative to the micro-menu position"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_FramerateFrame declares `## AllowLoadGameType: mainline` — `mainline` \
         is one of the standard-retail tokens (src/toc.rs:299) so \
         is_game_type_restricted() returns false and the addon IS reachable from \
         standard-retail discovery"
    );

    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_FramerateFrame declares no `## SavedVariables` — visibility is \
         controlled by the Show-Framerate console toggle (saved separately) and the \
         frame's hidden=true XML default"
    );

    let toc_text = std::fs::read_to_string(framerate_frame_toc())
        .expect("Blizzard_FramerateFrame TOC should read");
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_FramerateFrame declares `## DefaultState: enabled` — the addon is \
         enabled by default; user can disable via the Addons UI to suppress the FPS \
         display entirely"
    );
    assert!(
        toc_text.contains("## AllowLoad: Game"),
        "Blizzard_FramerateFrame declares `## AllowLoad: Game` (NOT Both) — the FPS \
         counter only makes sense in-world, glue screens have their own framerate \
         display logic"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline"),
        "Blizzard_FramerateFrame declares `## AllowLoadGameType: mainline` — explicitly \
         opts into mainline retail (no classic/plunderstorm variant exists for this \
         addon)"
    );
}

#[test]
fn blizzard_framerate_frame_allows_only_game_screen() {
    let toc = TocFile::from_file(&framerate_frame_toc())
        .expect("Blizzard_FramerateFrame TOC should parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` must allow the Game screen (src/toc.rs:307)"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: Game` must reject the Login screen — the FPS counter is \
         in-world only"
    );
    assert!(
        !toc.allows_screen(ScreenKind::CharacterSelect),
        "`## AllowLoad: Game` must reject CharacterSelect — glue screens don't host \
         this counter"
    );
}

#[test]
fn blizzard_framerate_frame_auto_loads_on_game_screen_only() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FramerateFrame");
    assert!(
        in_game,
        "Blizzard_FramerateFrame has no `## LoadOnDemand` line and `## AllowLoad: Game` \
         with mainline game type, so it MUST appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FramerateFrame");
    assert!(
        !in_login,
        "`## AllowLoad: Game` means Blizzard_FramerateFrame MUST NOT appear in \
         Login-screen auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_framerate_frame_loads_via_full_game_ui_without_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("FramerateFrame") || message.contains("Blizzard_FramerateFrame")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_FramerateFrame emitted Lua errors during the full Game-screen load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_framerate_frame_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FramerateFrame') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        post_load,
        "After full Game-screen load, IsAddOnLoaded('Blizzard_FramerateFrame') must \
         return true — auto-discovery picks up the addon (no LoadOnDemand) and \
         `mark_addon_loaded` registers it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_framerate_frame_singleton_publishes_with_world_frame_parent(env: &WowLuaEnv) {

    let info: (String, String, bool) = env
        .eval(
            "return FramerateFrame:GetName(), \
                    FramerateFrame:GetParent():GetName(), \
                    FramerateFrame:IsShown()",
        )
        .expect("FramerateFrame singleton probe should succeed");
    assert_eq!(
        info,
        (
            "FramerateFrame".to_string(),
            "WorldFrame".to_string(),
            false,
        ),
        "FramerateFrame.xml:4 declares `<Frame name=\"FramerateFrame\" \
         mixin=\"FramerateFrameMixin\" inherits=\"ResizeLayoutFrame\" hidden=\"true\" \
         parent=\"WorldFrame\">` — the FPS counter is parented to WorldFrame (NOT \
         UIParent) so it is not affected by UI scale changes that affect normal \
         windows; hidden=true keeps it suppressed at startup until the Show-Framerate \
         toggle calls Toggle()"
    );
}
}

prefork_full_ui_case! {
fn blizzard_framerate_frame_publishes_label_and_framerate_text_children(env: &WowLuaEnv) {

    let children_present: (bool, bool) = env
        .eval(
            "return type(FramerateFrame.Label) == 'table', \
                    type(FramerateFrame.FramerateText) == 'table'",
        )
        .expect("FramerateFrame children probe should succeed");
    assert_eq!(
        children_present,
        (true, true),
        "FramerateFrame has two ARTWORK-layer FontString children: parentKey=\"Label\" \
         (inherits SystemFont_Shadow_Med1, text=FRAMERATE_LABEL, anchored TOPLEFT — \
         the static \"FPS:\" prefix) and parentKey=\"FramerateText\" (inherits \
         SystemFont_Shadow_Med1, anchored LEFT relative to Label.RIGHT — the live \
         numeric value updated by OnUpdate). Both must publish via the parentKey \
         pipeline so the OnUpdate handler can call \
         `self.FramerateText:SetFormattedText(...)` and so Layout() can recompute the \
         frame size from ResizeLayoutFrame"
    );
}
}

prefork_full_ui_case! {
fn blizzard_framerate_frame_publishes_mixin_with_lifecycle_and_benchmark_methods(env: &WowLuaEnv) {

    let methods_present: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(FramerateFrameMixin) == 'table', \
                    type(FramerateFrameMixin.OnLoad) == 'function', \
                    type(FramerateFrameMixin.OnShow) == 'function', \
                    type(FramerateFrameMixin.OnUpdate) == 'function', \
                    type(FramerateFrameMixin.Toggle) == 'function', \
                    type(FramerateFrameMixin.BeginBenchmark) == 'function'",
        )
        .expect("FramerateFrameMixin lifecycle probe should succeed");
    assert_eq!(
        methods_present,
        (true, true, true, true, true, true),
        "FramerateFrame.lua:1 publishes `FramerateFrameMixin` with the lifecycle + \
         benchmark API: OnLoad (lua:3 — calls MicroMenuContainer:GetPosition() and \
         MicroMenu:UpdateFramerateFrameAnchor to position relative to micro-menu), \
         OnShow (lua:26 — resets fpsTime=0 so OnUpdate fires immediately on next \
         tick), OnUpdate (lua:9 — fpsTime countdown driven by FRAMERATE_FREQUENCY, \
         calls GetFramerate / IsCpuBound, falls back to plain \"%.1f\" format if \
         FPS_COUNTER_CPU_BOUND/GPU_BOUND localization globals are nil, otherwise uses \
         them with framerate substitution; calls self:Layout() to resize), Toggle \
         (lua:30 — `self:SetShown(not self:IsShown())`), BeginBenchmark (lua:34 — \
         marks self.benchmark=true + Show)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_framerate_frame_publishes_micro_menu_anchoring_and_position_methods(env: &WowLuaEnv) {

    let methods_present: (bool, bool, bool) = env
        .eval(
            "return type(FramerateFrameMixin.EndBenchmark) == 'function', \
                    type(FramerateFrameMixin.GetMicroMenuRelativeAnchoring) == 'function', \
                    type(FramerateFrameMixin.UpdatePosition) == 'function'",
        )
        .expect("FramerateFrameMixin anchoring probe should succeed");
    assert_eq!(
        methods_present,
        (true, true, true),
        "FramerateFrameMixin publishes the micro-menu-aware anchoring API: \
         EndBenchmark (lua:39 — clears benchmark + Hide), \
         GetMicroMenuRelativeAnchoring (lua:44 — branches on \
         MicroMenuPositionEnum.{{BottomLeft,BottomRight,TopLeft,TopRight}} and \
         isMenuHorizontal to return point/relativePoint/offsetX/offsetY tuple — \
         horizontal layouts anchor LEFT/RIGHT side-by-side with ±5px gap; vertical \
         layouts stack TOP/BOTTOM with ±5px gap), UpdatePosition (lua:68 — calls \
         GetMicroMenuRelativeAnchoring then ClearAllPoints + SetPoint relative to \
         MicroMenuContainer). The split lets MicroMenu's UpdateFramerateFrameAnchor \
         drive position changes in response to user-dragged micro-menu repositioning"
    );
}
}

prefork_full_ui_case! {
fn blizzard_framerate_frame_inherits_resize_layout_frame_layout_method(env: &WowLuaEnv) {

    let layout_method_present: bool = env
        .eval("return type(FramerateFrame.Layout) == 'function'")
        .expect("FramerateFrame Layout method probe should succeed");
    assert!(
        layout_method_present,
        "FramerateFrame inherits ResizeLayoutFrame so the OnUpdate handler can call \
         `self:Layout()` (lua:20) to recompute the frame's size from its child \
         FontStrings after each FPS update — the FramerateText label width changes \
         every tick because numeric values have variable digit count. ResizeLayoutFrame \
         must publish a Layout method on the instance for the OnUpdate handler not to \
         error"
    );
}
}

prefork_full_ui_case! {
fn blizzard_framerate_frame_label_inherits_system_font_shadow_med1(env: &WowLuaEnv) {

    let label_text: String = env
        .eval(
            "local txt = FramerateFrame.Label:GetText(); \
             return txt or ''",
        )
        .expect("FramerateFrame.Label text probe should succeed");
    assert!(
        !label_text.is_empty(),
        "FramerateFrame.Label has `text=\"FRAMERATE_LABEL\"` in XML — the simulator's \
         FontString text resolution should substitute the FRAMERATE_LABEL global \
         localization string, yielding a non-empty user-visible label like `Framerate:` \
         (en-US). Empty text would mean the localization global was nil at \
         FontString creation time"
    );
}
}
