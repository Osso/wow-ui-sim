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

fn photo_sharing_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PhotoSharing")
}

fn photo_sharing_toc() -> PathBuf {
    photo_sharing_dir().join("Blizzard_PhotoSharing.toc")
}

const PHOTO_SHARING_TOC_FILES: &[&str] = &[
    "Blizzard_PhotoSharing.lua",
    "Blizzard_PhotoSharing.xml",
    "Blizzard_PhotoSharingBrowser.lua",
    "Blizzard_PhotoSharingBrowser.xml",
];

const PUBLIC_MIXINS: &[&str] = &[
    "PhotoSharingMixin",
    "PhotoSharingSubmitButtonMixin",
    "PhotoSharingCancelButtonMixin",
    "PhotoSharingBrowserMixin",
    "PhotoSharingBrowserPopupMixin",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &[
    "PhotoSharingFrame",
    "PhotoSharingBrowserFrame",
    "PhotoSharingBrowserPopup",
    "PhotoSharingBrowser",
];

const PUBLIC_VIRTUAL_TEMPLATES: &[&str] = &[
    "PhotoSharingBrowserTemplate",
    "PhotoSharingBrowserTemplatePopup",
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
fn blizzard_photo_sharing_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&photo_sharing_dir()).expect("Blizzard_PhotoSharing TOC resolves");
    assert_eq!(
        resolved,
        photo_sharing_toc(),
        "Blizzard_PhotoSharing ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The Mainline-only restriction is enforced via `## AllowLoadGameType: standard` \
         instead of a flavor-split TOC; the bare TOC is the canonical entry point and \
         carries the gametype gate inline rather than via the filename suffix"
    );

    let mainline = photo_sharing_dir().join("Blizzard_PhotoSharing_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_photo_sharing_toc_declares_eager_mainline_only_with_game_menu_dependency() {
    let toc = TocFile::from_file(&photo_sharing_toc()).expect("Blizzard_PhotoSharing TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC OMITS `## LoadOnDemand:` so `is_load_on_demand()` returns false — the \
         photo-sharing UI is eager-loaded so the screenshot-preview event handlers \
         (PHOTO_SHARING_SCREENSHOT_READY etc.) are wired up before the player presses \
         the in-game screenshot key; deferring would cause the very first screenshot \
         to silently miss the preview frame"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: standard` so `is_game_type_restricted()` \
         returns false (NOT restricted from Mainline). The `standard` token at \
         src/toc.rs:294-302 is the canonical Mainline alias — same as `mainline` — and \
         passes the allow-list, which means the addon is loadable on retail. Classic / \
         Hardcore / SoD installs would see `is_game_type_restricted()` return true and \
         would skip the addon, because the social photo-sharing service is \
         retail-only"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Default-Game-only `allows_screen` at src/toc.rs:311 returns true for \
         ScreenKind::Game when `## AllowLoad:` is omitted"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Omitted `## AllowLoad:` must NOT enable {screen:?} — the photo-sharing \
             preview is bound to in-world screenshot capture; glue screens have no \
             screenshot-ready event source"
        );
    }

    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_GameMenuEsc".to_string()],
        "Retail 12.1.0.69497 declares Blizzard_GameMenuEsc as PhotoSharing's hard dependency"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the photo-sharing UI is a stateless mirror of \
         server-authoritative authorization / upload-status state fetched via \
         C_PhotoSharing each open. The published-photo history lives on the social \
         service's server, not on the client"
    );
}

#[test]
fn blizzard_photo_sharing_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(photo_sharing_toc())
        .expect("Blizzard_PhotoSharing TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Photo Sharing"),
        "TOC must declare `## Title: Blizzard Photo Sharing` exactly. UNUSUAL: the \
         title uses the space-and-prose form (rather than the underscore-namespace \
         spelling like `Blizzard_PhotoSharing`); the majority pattern, suggesting the \
         addon was hand-named for in-AddOnList readability rather than scaffolded"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment` exactly"
    );
    assert!(
        raw.contains("## AllowLoadGameType: standard"),
        "TOC must declare `## AllowLoadGameType: standard` exactly — case-sensitive \
         single-token Mainline-only gametype lock. UNUSUAL compared to the \
         `## AllowLoadGameType: standard, mists` dual-flavor pattern: photo-sharing is \
         a single-flavor feature, NOT available in Mists Classic"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand` — the absence is what flags eager loading"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless mirror"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_GameMenuEsc"),
        "Retail 12.1.0.69497 declares Blizzard_GameMenuEsc on its Dependencies line"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` — Game-only is the default behavior when \
         the screen-restriction key is omitted; the addon relies on the default"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — absent version is the minority pattern \
         and matches a few small Mainline-only social/utility addons"
    );
}

#[test]
fn blizzard_photo_sharing_toc_lists_four_files_in_canonical_order() {
    let toc = TocFile::from_file(&photo_sharing_toc()).expect("Blizzard_PhotoSharing TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PHOTO_SHARING_TOC_FILES,
        "TOC body must list exactly 4 files in canonical Lua-then-XML pair order: the \
         preview frame's logic+layout (Blizzard_PhotoSharing.lua then \
         Blizzard_PhotoSharing.xml) is loaded before the auth-flow browser's logic+ \
         layout (Blizzard_PhotoSharingBrowser.lua then Blizzard_PhotoSharingBrowser.xml). \
         The Lua-before-XML ordering matters per pair: each Lua file declares its \
         mixins (PhotoSharingMixin / PhotoSharingBrowserMixin / etc.) so the XML's \
         `mixin=\"...\"` attribute can resolve at parse time"
    );
}

#[test]
fn blizzard_photo_sharing_appears_in_game_screen_eager_discovery_only() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_PhotoSharing");
    assert!(
        in_game,
        "Blizzard_PhotoSharing must auto-discover on Game screen — eager (no \
         LoadOnDemand) AND `## AllowLoadGameType: standard` allows Mainline AND \
         omitted `## AllowLoad:` defaults to Game-only, which together makes \
         `discover_blizzard_addons_for_screen(Game)` include the addon"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PhotoSharing");
        assert!(
            !found,
            "Blizzard_PhotoSharing must NOT auto-discover on screen {screen:?} — \
             default Game-only (`## AllowLoad:` omitted) restricts discovery to \
             ScreenKind::Game"
        );
    }
}

#[test]
fn blizzard_photo_sharing_appears_in_full_addon_inventory() {
    let ui = blizzard_ui_dir();
    let all = discover_all_blizzard_addons(&ui);
    let found = all.iter().any(|(name, _)| name == "Blizzard_PhotoSharing");
    assert!(
        found,
        "Blizzard_PhotoSharing must appear in `discover_all_blizzard_addons` — the \
         flavor-restriction filter applies to per-screen discovery, but the \
         flavor-agnostic full-inventory walk still reports the addon as on-disk"
    );
}

prefork_full_ui_case! {
fn blizzard_photo_sharing_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PhotoSharing")
                || message.contains("PhotoSharing")
                || message.contains("PhotoSharingMixin")
                || message.contains("PhotoSharingBrowser")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PhotoSharing emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_photo_sharing_is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PhotoSharing')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PhotoSharing') must return true after the \
         eager Game-screen sweep — no LoadOnDemand puts the addon in the eager set"
    );
}
}

prefork_full_ui_case! {
fn blizzard_photo_sharing_publishes_five_mixins(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table. Blizzard_PhotoSharing declares 5 \
             top-level mixin tables across two Lua files: PhotoSharingMixin / \
             PhotoSharingSubmitButtonMixin / PhotoSharingCancelButtonMixin (preview \
             frame's drive/submit/cancel handlers in Blizzard_PhotoSharing.lua) and \
             PhotoSharingBrowserMixin / PhotoSharingBrowserPopupMixin (auth-flow \
             OAuth-callback browser handlers in Blizzard_PhotoSharingBrowser.lua). \
             The XML's `mixin=\"PhotoSharing*Mixin\"` attribute on the named frames \
             resolves to these tables at template-parse time, so they must exist as \
             plain Lua tables (NOT wrapped or proxied) for the inheritance-merge step \
             in Mixin() to copy method keys onto each instance frame"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_photo_sharing_publishes_tab_list_global(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(_G.PHOTO_SHARING_TAB_LIST)")
        .expect("type probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.PHOTO_SHARING_TAB_LIST must publish as a table — it's the 2-entry \
         tab-order list (`PhotoSharingTitleEditBox` then \
         `PhotoSharingDescriptionEditBox`) consumed by `EditBox_HandleTabbing` \
         inside the inline OnTabPressed scripts in Blizzard_PhotoSharing.xml. \
         Declared at the top of Blizzard_PhotoSharing.lua with two integer-keyed \
         entries; this is the addon's only top-level non-mixin global"
    );

    let count: i64 = env
        .eval("return #_G.PHOTO_SHARING_TAB_LIST")
        .expect("length probe succeeds");
    assert_eq!(
        count, 2,
        "PHOTO_SHARING_TAB_LIST must have exactly 2 entries — Title editbox at index \
         1 and Description editbox at index 2, matching the visible top-to-bottom \
         layout of the post-creation form"
    );

    let first: String = env
        .eval("return _G.PHOTO_SHARING_TAB_LIST[1]")
        .expect("[1] probe succeeds");
    assert_eq!(first, "PhotoSharingTitleEditBox");

    let second: String = env
        .eval("return _G.PHOTO_SHARING_TAB_LIST[2]")
        .expect("[2] probe succeeds");
    assert_eq!(second, "PhotoSharingDescriptionEditBox");
}
}

prefork_full_ui_case! {
fn blizzard_photo_sharing_creates_named_frames(env: &WowLuaEnv) {

    for frame in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a frame after XML load. The 4 named non-virtual \
             frames are: PhotoSharingFrame (the preview/post-creation panel from \
             Blizzard_PhotoSharing.xml, parented to UIParent, inherits \
             SettingsFrameTemplate), PhotoSharingBrowserFrame (the OAuth WebKit shell \
             from Blizzard_PhotoSharingBrowser.xml, inherits DefaultPanelTemplate), \
             PhotoSharingBrowser (the Browser child INSIDE PhotoSharingBrowserFrame, \
             registered globally via `name=\"PhotoSharingBrowser\"`), and \
             PhotoSharingBrowserPopup (resolves to the inner Browser child due to a \
             name COLLISION with the outer top-level Frame; the late-registered \
             child Browser overwrites the earlier top-level Frame in `_G` — verified \
             separately by its parent identity)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_photo_sharing_browser_popup_global_pins_inner_browser_collision_winner(env: &WowLuaEnv) {

    // The XML declares both a top-level Frame and its nested Browser child with
    // name="PhotoSharingBrowserPopup". The child is registered last and owns the
    // shared global, which is required by Blizzard_PhotoSharingBrowser.lua:73-74.
    let parent_name: String = env
        .eval("return PhotoSharingBrowserPopup:GetParent():GetName()")
        .expect("popup Browser ownership probe succeeds");
    assert_eq!(
        parent_name, "PhotoSharingBrowserPopup",
        "The nested Browser must own `_G.PhotoSharingBrowserPopup`; its parent is the \
         same-named outer Frame rather than UIParent"
    );

    let navigation_contract: (String, String, bool) = env
        .eval(
            r#"
            local navigateToResult = PhotoSharingBrowserPopup:NavigateTo("https://example.invalid")
            local navigateHomeResult = PhotoSharingBrowser:NavigateHome("PhotoSharing")
            return type(PhotoSharingBrowserPopup.NavigateTo),
                type(PhotoSharingBrowser.NavigateHome),
                navigateToResult == nil and navigateHomeResult == nil
            "#,
        )
        .expect("Browser navigation methods should be callable");
    assert_eq!(
        navigation_contract,
        ("function".to_string(), "function".to_string(), true),
        "Browser:NavigateTo and Browser:NavigateHome must be callable no-result methods. \
         Blizzard_PhotoSharingBrowser.lua calls them for popup URLs and the PhotoSharing \
         home page; the simulator performs no external browser navigation"
    );
}
}

prefork_full_ui_case! {
fn blizzard_photo_sharing_does_not_leak_virtual_templates_to_globals(env: &WowLuaEnv) {

    for tmpl in PUBLIC_VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G.{tmpl})"))
            .unwrap_or_else(|err| panic!("type(_G.{tmpl}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{tmpl} must NOT exist as a global. The 2 Browser virtual templates \
             (PhotoSharingBrowserTemplate and PhotoSharingBrowserTemplatePopup, both \
             with `virtual=\"true\"` in Blizzard_PhotoSharingBrowser.xml) live in the \
             template registry only — they are pure layout/script blueprints \
             instantiated via `inherits=\"PhotoSharingBrowser*Template\"` on the two \
             concrete Browser children. Leaking them into `_G` would be a regression \
             in the XML loader's virtual-flag respect"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_photo_sharing_c_photo_sharing_authorization_probe_works(env: &WowLuaEnv) {

    // PhotoSharingMixin:OnLoad calls self:UpdatePublishButton() which calls
    // C_PhotoSharing.IsAuthorized() — verified callable here so the mixin's
    // OnLoad path doesn't silently misbehave.
    let kind: String = env
        .eval("return type(C_PhotoSharing.IsAuthorized)")
        .expect("IsAuthorized type probe succeeds");
    assert_eq!(
        kind, "function",
        "C_PhotoSharing.IsAuthorized must be a callable function — registered by \
         `register_all` in src/lua_api/globals/photo_sharing.rs and consumed by \
         PhotoSharingMixin:UpdatePublishButton at Blizzard_PhotoSharing.lua:19. The \
         OnLoad path calls UpdatePublishButton at line 41, so a missing or \
         non-callable IsAuthorized would surface as a load-time Lua error"
    );

    let default: bool = env
        .eval("return C_PhotoSharing.IsAuthorized()")
        .expect("IsAuthorized() call succeeds");
    assert!(
        !default,
        "C_PhotoSharing.IsAuthorized() must return false by default — the sim has no \
         real photo-sharing service, so the SimState-backed flag stays false until \
         flipped by `A_Admin.SetPhotoSharingAuthorized(true)`. PhotoSharingMixin:OnLoad \
         exercises this default path and must take the `else` branch (showing the \
         `Sign in` button) without erroring"
    );
}
}
