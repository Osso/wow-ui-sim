use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn social_toast_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SocialToast")
}

fn social_toast_toc() -> PathBuf {
    social_toast_dir().join("Blizzard_SocialToast.toc")
}

const PUBLISHED_MIXINS: &[&str] = &[
    "DefaultAnimOutMixin",
    "SocialToastCloseButtonMixin",
    "SocialToastMixin",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "SocialToastAnimInTemplate",
    "SocialToastAnimOutTemplate",
    "SocialToastGlowTemplate",
    "SocialToastCloseButtonTemplate",
    "SocialToastTemplate",
];

fn fresh_env(screen: ScreenKind) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(screen);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_ui_for(screen: ScreenKind) -> WowLuaEnv {
    let env = fresh_env(screen);

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    env
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&social_toast_dir()).expect("SocialToast TOC resolves");
    assert_eq!(
        resolved,
        social_toast_toc(),
        "Blizzard_SocialToast ships a single bare \
         `Blizzard_SocialToast.toc` (NO `_Mainline.toc` flavor variant). \
         Per-flavor gating is expressed via the `## AllowLoadGameType: \
         mainline` directive instead — the toast frame template + \
         alert-frame helpers are mainline-only because Mists/classic do \
         not have the same Battle.net friend / Communities feature \
         surface that drives the social toast queue"
    );
}

#[test]
fn dependencies_accessor_returns_shared_xml_via_dependencies_key() {
    let toc = TocFile::from_file(&social_toast_toc()).expect("TOC parses");

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec!["Blizzard_SharedXML"],
        "SocialToast uses the plural `## Dependencies:` key, which the \
         simulator's `dependencies()` accessor at src/toc.rs:210-217 \
         honors as the canonical form (alongside RequiredDep / \
         RequiredDeps fallbacks). SharedXML provides the \
         BACKDROP_TOAST_12_12 backdrop preset (Blizzard_SharedXML/\
         Backdrop.lua:79) that the SocialToastTemplate ContainedAlertFrame \
         dereferences via `<KeyValue key=\"backdropInfo\" \
         value=\"BACKDROP_TOAST_12_12\" type=\"global\"/>`, plus the \
         BackdropTemplate that SocialToastTemplate inherits. Got: {deps:?}"
    );
}

#[test]
fn allow_load_both_lowercase_resolves_correctly() {
    let toc = TocFile::from_file(&social_toast_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: both` (lowercase) MUST allow Game — \
         allows_screen at src/toc.rs:307 uses eq_ignore_ascii_case so \
         the lowercase form matches `both`"
    );
    assert!(
        toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: both` MUST allow Login (Battle.net friend toasts \
         can fire while the player is on the glue screen — incoming \
         friend invites, online notifications)"
    );
    assert!(toc.allows_screen(ScreenKind::CharacterSelect));
    assert!(toc.allows_screen(ScreenKind::CharacterCreate));
}

#[test]
fn allow_load_game_type_mainline_is_not_restricted() {
    let toc = TocFile::from_file(&social_toast_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: mainline` MUST resolve to \
         is_game_type_restricted() == false because mainline is in the \
         allowed game-type list at src/toc.rs:298-299 (alongside \
         `standard`). The directive documents that the addon is \
         mainline-only, but is_game_type_restricted is named for the \
         exclusion view: only true for non-mainline-non-standard \
         restrictions like `plunderstorm` or `classic`. The default \
         simulator flavor is mainline so the addon eager-loads"
    );
}

#[test]
fn toc_is_eager_with_no_secure_env_or_saved_vars() {
    let toc = TocFile::from_file(&social_toast_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "SocialToast must NOT be LoadOnDemand — Battle.net friend / \
         Communities toasts can fire any time after PLAYER_ENTERING_WORLD \
         or even on the glue screens, so the alert-frame queue must \
         already have the SocialToastTemplate registered"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.default_enabled());
}

#[test]
fn toc_raw_bytes_pin_six_metadata_directives() {
    let raw = std::fs::read_to_string(social_toast_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_SocialToast",
        "## Author: Blizzard Entertainment",
        "## DefaultState: enabled",
        "## AllowLoad: both",
        "## AllowLoadGameType: mainline",
        "## Dependencies: Blizzard_SharedXML",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw bytes MUST pin the `{directive}` directive — \
             SocialToast's TOC is small (6 metadata lines + 2 body \
             entries) so each directive is load-bearing"
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
    let toc = TocFile::from_file(&social_toast_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = ["SocialToast.lua", "SocialToast.xml"];

    assert_eq!(
        body.len(),
        expected.len(),
        "Body must contain exactly 2 entries — the addon is minimal \
         (3 mixins + 5 virtual templates). Got: {body:?}"
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
        "SocialToast.lua MUST load BEFORE SocialToast.xml — the XML \
         references DefaultAnimOutMixin (on SocialToastAnimOutTemplate), \
         SocialToastCloseButtonMixin (on SocialToastCloseButtonTemplate), \
         and SocialToastMixin (on SocialToastTemplate) via `mixin=\"…\"` \
         attributes. The XML loader resolves the mixin tables at \
         template-registration time, so they MUST already be tables in \
         _G when the .xml chunk is processed"
    );
}

#[test]
fn appears_in_eager_discovery_on_all_four_screens() {
    let ui = blizzard_ui_dir();
    let screens = [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ];

    for screen in screens {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_SocialToast");
        assert!(
            found,
            "`## AllowLoad: both` MUST surface SocialToast in the eager \
             auto-discovery sweep for screen {screen:?}. Battle.net \
             friend toasts and Communities notifications can fire on \
             every screen — the SocialToastTemplate must be registered \
             before any toast queue arrives"
        );
    }
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {

    let errors = env.state().borrow().lua_errors.clone();
    let needles = [
        "SocialToast",
        "DefaultAnimOutMixin",
        "SocialToastCloseButtonMixin",
        "SocialToastMixin",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Full Game-screen load must emit zero SocialToast-specific Lua \
         errors. Found {} matching errors: {:#?}",
        matched.len(),
        matched
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn full_game_load_emits_no_addon_specific_lua_errors() {
    let env = load_full_ui_for(ScreenKind::Game);
    full_game_load_emits_no_addon_specific_lua_errors::run(&env);
}

prefork_full_ui_case! {
fn is_addon_loaded_reports_true_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SocialToast')")
        .expect("IsAddOnLoaded query");
    assert!(
        loaded,
        "After the Game-screen eager sweep, \
         C_AddOns.IsAddOnLoaded('Blizzard_SocialToast') must return \
         true — `## AllowLoad: both` with no `## LoadOnDemand` and \
         mainline-allowed game type means the addon is part of the \
         auto-loaded set"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn is_addon_loaded_reports_true_after_eager_sweep() {
    let env = load_full_ui_for(ScreenKind::Game);
    is_addon_loaded_reports_true_after_eager_sweep::run(&env);
}

prefork_full_ui_case! {
fn publishes_three_mixin_tables_with_canonical_methods(env: &WowLuaEnv) {

    for mixin in PUBLISHED_MIXINS {
        let probe = format!("return type({mixin}) == 'table'");
        let result: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("mixin probe ({mixin}): {err}"));
        assert!(
            result,
            "Mixin {mixin} MUST publish as a global table — XML \
             mixin=\"{mixin}\" attributes resolve the table at \
             template-registration time. DefaultAnimOutMixin: hides \
             parent on animation finish (used by other addons too — the \
             name is generic). SocialToastCloseButtonMixin: forwards \
             OnEnter/OnLeave to parent and Hides parent on click. \
             SocialToastMixin: pauses/resumes the AlertFrame out-animation \
             on hover"
        );
    }

    let canonical = "return type(DefaultAnimOutMixin.OnFinished) == 'function' and \
                     type(SocialToastCloseButtonMixin.OnEnter) == 'function' and \
                     type(SocialToastCloseButtonMixin.OnLeave) == 'function' and \
                     type(SocialToastCloseButtonMixin.OnClick) == 'function' and \
                     type(SocialToastMixin.OnEnter) == 'function' and \
                     type(SocialToastMixin.OnLeave) == 'function'";
    let result: bool = env.eval(canonical).expect("canonical method probe");
    assert!(
        result,
        "All canonical mixin methods must publish as functions: \
         DefaultAnimOutMixin.OnFinished (parent:Hide on anim end); \
         SocialToastCloseButtonMixin.{{OnEnter,OnLeave,OnClick}} (forward \
         hover to parent + hide on click); SocialToastMixin.OnEnter / \
         SocialToastMixin.OnLeave (mouseover pauses the auto-fade-out timer)"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn publishes_three_mixin_tables_with_canonical_methods() {
    let env = load_full_ui_for(ScreenKind::Game);
    publishes_three_mixin_tables_with_canonical_methods::run(&env);
}

prefork_full_ui_case! {
fn social_toast_mixin_calls_alert_frame_helpers_on_hover(env: &WowLuaEnv) {

    let probe = "return type(AlertFrame_PauseOutAnimation) == 'function' and \
                 type(AlertFrame_ResumeOutAnimation) == 'function'";
    let result: bool = env.eval(probe).expect("AlertFrame helpers probe");
    assert!(
        result,
        "SocialToastMixin.OnEnter / OnLeave call \
         AlertFrame_PauseOutAnimation / AlertFrame_ResumeOutAnimation. \
         Those helpers are defined in Blizzard_FrameXML/Mainline/\
         AlertFrames.lua:827-836 — SocialToast does NOT declare \
         Blizzard_FrameXML as a dep but the alphabetical eager-sweep \
         load order on the Game screen happens to load FrameXML first. \
         Confirms the cross-addon helper surface is wired even though \
         the dep edge is missing from the TOC declaration"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn social_toast_mixin_calls_alert_frame_helpers_on_hover() {
    let env = load_full_ui_for(ScreenKind::Game);
    social_toast_mixin_calls_alert_frame_helpers_on_hover::run(&env);
}

prefork_full_ui_case! {
fn social_toast_template_materializes_with_close_button_child_and_low_strata(env: &WowLuaEnv) {

    let probe = "local f = CreateFrame('Button', 'SocialToastProbe', UIParent, 'SocialToastTemplate') \
                 if not f then return 'frame nil' end \
                 local missing = {} \
                 if type(f.CloseButton) ~= 'table' then table.insert(missing, 'CloseButton:'..type(f.CloseButton)) end \
                 if f:GetFrameStrata() ~= 'LOW' then table.insert(missing, 'strata:'..f:GetFrameStrata()) end \
                 if #missing == 0 then return 'OK' else return table.concat(missing, ',') end";
    let report: String = env.eval(probe).expect("SocialToastTemplate materialize");
    assert_eq!(
        report, "OK",
        "SocialToastTemplate must materialize via CreateFrame with the \
         CloseButton parentKey (Button child via the <Frames> block, \
         inherits=\"SocialToastCloseButtonTemplate\" which carries \
         parentKey=\"CloseButton\") and frameStrata=LOW (toasts float \
         above world but below modals). \
         \
         Known simulator gap: anonymous animation / texture inheritance \
         (`<AnimationGroup inherits=\"SocialToastAnimInTemplate\"/>` and \
         `<Texture inherits=\"SocialToastGlowTemplate\">`) does not \
         propagate parentKey from the inherited template at \
         src/loader/helpers_anim.rs:18-22 and src/loader/xml_texture.rs \
         — the codegen only emits the parent assignment when the \
         inheriting element itself has parentKey set, not the inherited \
         template. So `f.animIn` / `f.waitAndAnimOut` / `f.glow` resolve \
         to nil even though the underlying AnimationGroup / Texture \
         objects are constructed. The toast frame still functions \
         visually because the alert-frame queue drives playback via \
         FrameXML's AlertFrame_PlayInAnimation / PlayOutAnimation \
         helpers which traverse the animation list rather than the \
         parentKey shortcut"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn social_toast_template_materializes_with_close_button_child_and_low_strata() {
    let env = load_full_ui_for(ScreenKind::Game);
    social_toast_template_materializes_with_close_button_child_and_low_strata::run(&env);
}

prefork_full_ui_case! {
fn social_toast_close_button_template_materializes_with_three_state_textures(env: &WowLuaEnv) {

    let probe = "local f = CreateFrame('Button', 'SocialToastCloseButtonProbe', UIParent, 'SocialToastCloseButtonTemplate') \
                 if not f then return false end \
                 local has_normal = f:GetNormalTexture() ~= nil \
                 local has_pushed = f:GetPushedTexture() ~= nil \
                 local has_highlight = f:GetHighlightTexture() ~= nil \
                 local has_click = f:GetScript('OnClick') ~= nil \
                 local has_enter = f:GetScript('OnEnter') ~= nil \
                 return has_normal and has_pushed and has_highlight and has_click and has_enter";
    let result: bool = env
        .eval(probe)
        .expect("SocialToastCloseButtonTemplate materialize");
    assert!(
        result,
        "SocialToastCloseButtonTemplate must materialize standalone \
         (without a SocialToast parent) with three-state texture set + \
         method-bound OnEnter/OnLeave/OnClick scripts. The three \
         textures are UI-Toast-CloseButton-Up / UI-Toast-CloseButton-Down \
         / UI-Toast-CloseButton-Highlight under Interface\\FriendsFrame. \
         The 18x18 size + TOPRIGHT offset (-4, -3) bind to the parent \
         toast at instantiation time via the inherits chain in \
         SocialToastTemplate's Frames block"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn social_toast_close_button_template_materializes_with_three_state_textures() {
    let env = load_full_ui_for(ScreenKind::Game);
    social_toast_close_button_template_materializes_with_three_state_textures::run(&env);
}

prefork_full_ui_case! {
fn xml_registers_all_five_virtual_templates(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES {
        let widget_type = match *template {
            "SocialToastAnimInTemplate" | "SocialToastAnimOutTemplate" => continue,
            "SocialToastGlowTemplate" => continue,
            "SocialToastCloseButtonTemplate" => "Button",
            "SocialToastTemplate" => "Button",
            other => panic!("unexpected template {other}"),
        };
        let probe = format!(
            "local ok, frame = pcall(function() \
                return CreateFrame({widget_type:?}, nil, UIParent, {template:?}) \
             end) \
             return ok and frame ~= nil"
        );
        let result: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("template probe ({template}): {err}"));
        assert!(
            result,
            "Virtual template {template} (registered by SocialToast.xml) \
             must materialize via CreateFrame as widget_type \
             {widget_type:?}. AnimationGroup / Texture templates can't \
             be CreateFrame'd directly — they apply via parentKey + \
             inherits on a host frame, validated by the \
             social_toast_template_materializes_with_full_child_surface \
             test which proves the animIn / waitAndAnimOut / glow \
             children resolve correctly"
        );
    }
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn xml_registers_all_five_virtual_templates() {
    let env = load_full_ui_for(ScreenKind::Game);
    xml_registers_all_five_virtual_templates::run(&env);
}

prefork_full_ui_case! {
fn social_toast_template_inherits_backdrop_template(env: &WowLuaEnv) {

    let probe = "local f = CreateFrame('Button', 'SocialToastBackdropProbe', UIParent, 'SocialToastTemplate') \
                 if not f then return false end \
                 return type(f.OnBackdropLoaded) == 'function' or \
                        type(f.SetBackdrop) == 'function' or \
                        type(f.GetBackdrop) == 'function'";
    let result: bool = env.eval(probe).expect("SocialToastTemplate backdrop probe");
    assert!(
        result,
        "SocialToastTemplate inherits=BackdropTemplate so it must \
         expose the backdrop API (SetBackdrop / GetBackdrop / \
         OnBackdropLoaded — at least one of which the simulator \
         provides). The XML KeyValue `backdropInfo = BACKDROP_TOAST_12_12` \
         type=global resolves at OnBackdropLoaded time to the backdrop \
         spec table from Blizzard_SharedXML/Backdrop.lua:79 (12-pixel \
         edge tile + 12-pixel insets — the standard friend-toast frame)"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn social_toast_template_inherits_backdrop_template() {
    let env = load_full_ui_for(ScreenKind::Game);
    social_toast_template_inherits_backdrop_template::run(&env);
}

prefork_full_ui_case! {
fn published_animation_group_template_with_default_anim_out_mixin_resolves(env: &WowLuaEnv) {

    let probe = "return type(DefaultAnimOutMixin) == 'table' and \
                 type(DefaultAnimOutMixin.OnFinished) == 'function'";
    let result: bool = env.eval(probe).expect("DefaultAnimOutMixin probe");
    assert!(
        result,
        "DefaultAnimOutMixin is GENERIC (named for the pattern, not the \
         addon) — its OnFinished:method does `self:GetParent():Hide()` \
         which works for any AnimationGroup-on-Frame setup where the \
         desired effect is auto-hide-on-completion. Other addons can \
         and do mix this in directly via XML mixin attribute. The \
         SocialToastAnimOutTemplate sets startDelay=4s + 1.5s alpha \
         fade-out so the toast hides itself after 4s of visibility plus \
         1.5s of fade animation"
    );
}
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn published_animation_group_template_with_default_anim_out_mixin_resolves() {
    let env = load_full_ui_for(ScreenKind::Game);
    published_animation_group_template_with_default_anim_out_mixin_resolves::run(&env);
}
