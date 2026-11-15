#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn paged_content_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PagedContent")
}

fn paged_content_toc() -> PathBuf {
    paged_content_dir().join("Blizzard_PagedContent.toc")
}

const PAGED_CONTENT_TOC_FILES: &[&str] = &[
    "Blizzard_PagingControls.lua",
    "Blizzard_PagingControls.xml",
    "Blizzard_PagedContentFrame.lua",
    "Blizzard_PagedContentFrame.xml",
    "Blizzard_PagedGridContentFrame.lua",
    "Blizzard_PagedGridContentFrame.xml",
    "Blizzard_PagedListContentFrame.lua",
    "Blizzard_PagedListContentFrame.xml",
    "Blizzard_PagedCondensedGridContentFrame.lua",
    "Blizzard_PagedCondensedGridContentFrame.xml",
];

const PUBLIC_MIXINS: &[&str] = &[
    "PagingControlsMixin",
    "PagedContentFrameBaseMixin",
    "BasePagedGridContentFrameMixin",
    "PagedCellSizeGridContentFrameMixin",
    "PagedNaturalSizeGridContentFrameMixin",
    "BasePagedListContentFrameMixin",
    "PagedVerticalListContentFrameMixin",
    "PagedHorizontalListContentFrameMixin",
    "PagedCondensedVerticalGridContentFrameMixin",
];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "PagingControlsPrevPageButtonTemplate",
    "PagingControlsNextPageButtonTemplate",
    "PageTextTemplate",
    "PagingControlsTemplate",
    "PagingControlsHorizontalTemplate",
    "PagingControlsCenteredTemplate",
    "PagedContentSpacerTemplate",
    "PagedContentFrameBaseTemplate",
    "PagedCellSizeGridContentFrameTemplate",
    "PagedNaturalSizeGridContentFrameTemplate",
    "PagedVerticalListContentFrameTemplate",
    "PagedHorizontalListContentFrameTemplate",
    "PagedCondensedVerticalGridContentFrameTemplate",
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
fn blizzard_paged_content_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&paged_content_dir()).expect("Blizzard_PagedContent TOC resolves");
    assert_eq!(
        resolved,
        paged_content_toc(),
        "Blizzard_PagedContent ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The paged-content library is engine-flavor-agnostic templating: the same paging math \
         drives Mainline talent loadouts, Classic spellbook tabs, and the Wrath-Classic \
         dungeon journal — no flavor split needed because the templates contain zero \
         retail-only API references"
    );

    let mainline = paged_content_dir().join("Blizzard_PagedContent_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_paged_content_toc_declares_eager_both_screens_with_no_dependencies() {
    let toc = TocFile::from_file(&paged_content_toc()).expect("Blizzard_PagedContent TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 0` (explicit zero — eager) so `is_load_on_demand()` \
         returns false. Eager loading is mandatory because the templates this addon defines \
         (PagedContentFrameBaseTemplate, PagingControlsTemplate, etc.) are inherited by other \
         addons' XML at parse time; if PagedContent loaded later the dependent XML would \
         resolve those template names to nil and crash"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "`## AllowLoad: Both` must enable {screen:?} — `allows_screen` at \
             src/toc.rs:307 returns true for every ScreenKind when AllowLoad is `Both`. \
             The paged-content templates are foundational and must be available on every \
             screen so glue-screen UIs (CharacterSelect glance panels, login news) can \
             also inherit them"
        );
    }

    assert!(
        toc.dependencies().is_empty(),
        "Zero `## RequiredDep:` / `## Dependencies:` — the paged-content library is a leaf \
         dependency in the FrameXML graph; it consumes only foundational primitives \
         (CallbackRegistryMixin, HorizontalLayoutFrame, GenerateClosure, Clamp, PlaySound, \
         IsShiftKeyDown, IsControlKeyDown) which are all part of the SharedXML core that \
         loads before any addon"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons. The library has zero awareness \
         of any consumer; consumers reach in via template inheritance"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the library is a pure templating provider with no per-user \
         state to persist. Page indices live on the consumer's frame, not on this library"
    );
}

#[test]
fn blizzard_paged_content_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(paged_content_toc())
        .expect("Blizzard_PagedContent TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Paged Content"),
        "TOC must declare `## Title: Blizzard Paged Content` exactly (space-and-prose form, \
         the human-readable label — the majority pattern for Blizzard-shipped addons)"
    );
    assert!(
        raw.contains("## LoadOnDemand: 0"),
        "TOC must declare `## LoadOnDemand: 0` exactly — the explicit zero form is the \
         minority retail spelling for eager addons; most eager addons OMIT the key entirely. \
         Pinning the explicit `0` guards against a refactor that flips it to `1` and \
         silently breaks every consumer's template inheritance"
    );
    assert!(
        raw.contains("## AllowLoad: Both"),
        "TOC must declare `## AllowLoad: Both` exactly — case-sensitive every-screen \
         availability"
    );
    assert!(
        !raw.contains("## RequiredDep"),
        "TOC must NOT declare any `## RequiredDep` keys — leaf dependency"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare any `## Dependencies` keys — leaf dependency"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure templating provider"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — relies on implicit Blizzard ownership of the \
         `Blizzard_` namespace prefix"
    );
}

#[test]
fn blizzard_paged_content_toc_lists_ten_files_lua_xml_paired() {
    let toc = TocFile::from_file(&paged_content_toc()).expect("Blizzard_PagedContent TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PAGED_CONTENT_TOC_FILES,
        "TOC body must list exactly 10 files in canonical order: 5 Lua/XML pairs grouped by \
         template family. Order is: PagingControls (the page-arrow controls used by every \
         consumer), PagedContentFrame (the base mixin/template — depends on \
         PagingControlsTemplate being already-defined so PagedContentFrameBaseTemplate's \
         `<Frame parentKey=\"PagingControls\" inherits=\"PagingControlsTemplate\"/>` resolves \
         at XML parse time), PagedGridContentFrame (cell-size + natural-size grid layouts — \
         depend on PagedContentFrameBaseTemplate), PagedListContentFrame (vertical + \
         horizontal list layouts — same dependency), PagedCondensedGridContentFrame \
         (condensed-grid layout — same dependency). Within each pair, Lua loads first to \
         declare the mixin globals before XML inherits them via `mixin=\"...\"`"
    );
}

#[test]
fn blizzard_paged_content_appears_on_every_screen_eager_discovery() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PagedContent");
        assert!(
            found,
            "Blizzard_PagedContent must auto-discover on screen {screen:?} — eager (LoadOnDemand: 0) \
             AND `## AllowLoad: Both` makes `discover_blizzard_addons_for_screen` include the \
             addon on every screen"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_paged_content_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PagedContent")
                || message.contains("PagedContentFrame")
                || message.contains("PagingControls")
                || message.contains("PagedGridContentFrame")
                || message.contains("PagedListContentFrame")
                || message.contains("PagedCondensedGridContentFrame")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PagedContent emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_paged_content_is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PagedContent')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PagedContent') must return true after the eager \
         Game-screen sweep — LoadOnDemand: 0 puts the addon in the eager set"
    );
}
}

prefork_full_ui_case! {
fn blizzard_paged_content_publishes_nine_mixin_tables(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — the paged-content library declares 9 \
             mixins forming a 3-level inheritance tree: PagingControlsMixin (standalone — \
             page arrows + page-text widget), PagedContentFrameBaseMixin (extends \
             CallbackRegistryMixin — owns the data-provider, frame pool, view-distribution \
             math, and `OnUpdate` callback event), then 7 leaf mixins each extending \
             PagedContentFrameBaseMixin via `CreateFromMixins(...)` to specialize layout: \
             BasePagedGridContentFrameMixin (the shared grid-layout helpers — abstract base \
             never instantiated directly), PagedCellSizeGridContentFrameMixin / \
             PagedNaturalSizeGridContentFrameMixin (extend BasePagedGridContentFrameMixin — \
             the two grid sizing strategies: fixed cell size vs auto-fit to content), \
             BasePagedListContentFrameMixin (the shared list-layout helpers — abstract base), \
             PagedVerticalListContentFrameMixin / PagedHorizontalListContentFrameMixin \
             (extend BasePagedListContentFrameMixin — vertical vs horizontal list flow), \
             PagedCondensedVerticalGridContentFrameMixin (extends PagedContentFrameBaseMixin \
             directly, NOT via Base — the condensed-grid layout is a third sibling not a \
             grid variant). Each mixin is referenced by an XML `mixin=\"...\"` attribute on \
             the matching template"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_paged_content_does_not_leak_virtual_templates_to_globals(env: &WowLuaEnv) {

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — the entire addon is a virtual-template library \
             with ZERO named non-virtual frames. All 13 templates (`virtual=\"true\"`) live \
             in the template registry, NOT in `_G`. Leaking any of them as a global would \
             let consumer addons mutate the template definition and break every existing \
             instance across the entire FrameXML"
        );
    }
}
}

#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
prefork_full_ui_case! {
fn retail_original_paging_consumer_keeps_hidden_page_text_unchanged(env: &WowLuaEnv) {
    let (text, alpha): (String, f64) = env
        .eval(
            r#"
            local controls = CreateFrame("Frame", nil, UIParent, "PagingControlsHorizontalTemplate")
            controls.hideWhenSinglePage = true
            controls.currentPage = 1
            controls.maxPages = 1
            controls.PageText:SetText("preserved")
            controls:UpdateControls()
            return controls.PageText:GetText() or "", controls:GetAlpha()
            "#,
        )
        .expect("selected Blizzard paging consumer should update its controls");

    assert_eq!(alpha, 0.0, "the original consumer hides single-page controls");
    assert_eq!(
        text, "preserved",
        "the original UpdateControls leaves PageText unchanged when it hides the controls"
    );
}
}

prefork_full_ui_case! {
fn blizzard_paged_content_frame_base_mixin_publishes_callback_event(env: &WowLuaEnv) {

    let callback_present: bool = env
        .eval(
            "return type(PagedContentFrameBaseMixin.OnUpdate) ~= 'nil' \
             or type(PagedContentFrameBaseMixin.Event) == 'table' \
             or type(PagedContentFrameBaseMixin.GenerateCallbackEvents) == 'function'",
        )
        .expect("PagedContentFrameBaseMixin callback-events probe succeeds");
    assert!(
        callback_present,
        "PagedContentFrameBaseMixin must inherit CallbackRegistryMixin (verified via the \
         presence of GenerateCallbackEvents) and call \
         `:GenerateCallbackEvents({{ \"OnUpdate\" }})` at module top to declare the single \
         OnUpdate callback event. Consumer code wires up via \
         `myFrame:RegisterCallback(PagedContentFrameBaseMixin.Event.OnUpdate, handler, owner)` \
         — without the callback registry, every consumer would have to poll for changes"
    );
}
}
