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

fn report_frame_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ReportFrame")
}

fn report_frame_toc() -> PathBuf {
    report_frame_dir().join("Blizzard_ReportFrame.toc")
}

const REPORT_FILES: &[&str] = &["ReportFrame.lua", "ReportFrame.xml"];

const HARD_DEPENDENCIES: &[&str] = &["Blizzard_ReportFrameShared", "Blizzard_GameMenuEsc"];

const OVERRIDE_METHODS: &[&str] = &[
    "CanDisplayMinorCategory",
    "ShouldDisplayTooltip",
    "ManageButton",
];

const INHERITED_BASE_METHODS: &[&str] = &[
    "OnLoad",
    "OnHide",
    "OnEvent",
    "Reset",
    "InitiateReport",
    "InitiateReportInternal",
    "ReportByType",
    "MajorTypeSelected",
    "SetMajorType",
    "SendReport",
    "SetupDropdownByReportType",
    "UpdateThankYouMessage",
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
    let resolved =
        find_toc_file(&report_frame_dir()).expect("Blizzard_ReportFrame TOC should resolve");
    assert_eq!(
        resolved,
        report_frame_toc(),
        "Blizzard_ReportFrame ships exactly one TOC at the bare \
         `Blizzard_ReportFrame.toc` path — no `_Mainline` flavor split (the retail/classic \
         variants live in sibling addons Blizzard_ReportFrameGlue and Blizzard_ReportFrameShared \
         instead, sharing the same SharedReportFrameMixin base). find_toc_file at \
         src/loader/mod.rs:65 tries `_Mainline.toc` first (miss), then bare (hit) and returns it"
    );
}

#[test]
fn toc_declares_eager_game_only_with_two_hard_deps() {
    let toc =
        TocFile::from_file(&report_frame_toc()).expect("Blizzard_ReportFrame TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_ReportFrame must NOT declare `## LoadOnDemand: 1` — the in-game player-report \
         dialog (right-click → Report Player) is the entry point for /report, MailFrame, \
         CommunitiesFrame, ClubFinder, LFGList, Calendar, ItemRefHandlers and UnitPopupShared, \
         so it has to be live the moment any of those panels can spawn the dialog"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_ReportFrame declares no `## AllowLoadGameType` directive — \
         is_game_type_restricted at src/toc.rs:294-302 returns FALSE when the metadata is \
         absent. The report dialog applies to every retail flavor without a server-side gate"
    );
    assert_eq!(
        toc.metadata.get("DefaultState").map(String::as_str),
        Some("enabled"),
        "DefaultState=enabled preserved verbatim — without it the eager-loader would skip the \
         addon entirely on the Game screen"
    );
    let deps = toc.dependencies();
    assert_eq!(
        deps.iter().map(String::as_str).collect::<Vec<_>>(),
        HARD_DEPENDENCIES,
        "Retail 12.1.0.69497 declares ReportFrameShared and GameMenuEsc as ReportFrame hard dependencies"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_ReportFrame declares zero `## OptionalDeps:` — every collaborator (the shared \
         template, the report-system C API) is either a hard dep or part of the always-loaded \
         core surface"
    );
    assert!(
        toc.saved_variables().is_empty() && toc.saved_variables_per_character().is_empty(),
        "Blizzard_ReportFrame declares zero saved variables — submitted reports go straight to \
         C_ReportSystem.SendReport, never persisted client-side"
    );
}

#[test]
fn toc_lists_two_files_in_root_directory() {
    let toc =
        TocFile::from_file(&report_frame_toc()).expect("Blizzard_ReportFrame TOC should parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        listed, REPORT_FILES,
        "TOC body must list exactly these 2 files in this order: ReportFrame.lua loads FIRST so \
         `ReportFrameMixin = CreateFromMixins(SharedReportFrameMixin)` plus the three override \
         method definitions publish before ReportFrame.xml's `mixin=\"ReportFrameMixin\"` \
         attribute resolves them at frame instantiation"
    );
}

#[test]
fn allows_screen_returns_true_only_for_game() {
    let toc =
        TocFile::from_file(&report_frame_toc()).expect("Blizzard_ReportFrame TOC should parse");
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_ReportFrame declares `## AllowLoad: Game` — must allow the in-game screen. \
         The report dialog is meaningless from glue screens; that responsibility belongs to \
         Blizzard_ReportFrameGlue (with `## AllowLoad: Glue`)"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_ReportFrame must NOT allow non-Game screens — `## AllowLoad: Game` matches \
             the Game branch only at src/toc.rs:308. (Screen tested: {screen:?})"
        );
    }
}

#[test]
fn included_in_eager_discovery_for_game_screen() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_ReportFrame");
    assert!(
        found,
        "Blizzard_ReportFrame must appear in eager auto-discovery on the Game screen — no \
         LoadOnDemand gate, no AllowLoadGameType restriction, AllowLoad=Game passes the \
         ScreenKind::Game branch at src/toc.rs:308. The downstream LFGList / MailFrame / \
         CommunitiesFrame addons reference `ReportFrame:InitiateReport(...)` directly with no \
         C_AddOns.LoadAddOn guard, so the addon must already be live"
    );
}

#[test]
fn root_directory_holds_two_files_next_to_toc() {
    let mut entries: Vec<String> = std::fs::read_dir(report_frame_dir())
        .expect("Blizzard_ReportFrame directory should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name != "Blizzard_ReportFrame.toc")
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec!["ReportFrame.lua".to_string(), "ReportFrame.xml".to_string()],
        "Blizzard_ReportFrame/ root must hold exactly the lua/xml pair next to the TOC — no \
         per-flavor subdirectory and no localization stub"
    );
}

prefork_full_ui_case! {
fn loads_without_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("ReportFrame") || message.contains("ReportFrameMixin"))
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ReportFrame emitted Lua errors during eager Game-screen load. The addon's load \
         path is the simplest possible: 1 Lua chunk publishing ReportFrameMixin via \
         CreateFromMixins, 1 XML chunk instantiating one named frame inheriting \
         SharedReportFrameTemplate. Any error is a real load failure:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ReportFrame')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ReportFrame') must return true after the eager Game \
         sweep — confirms the loader registers the non-LOD addon with the loaded-set without an \
         explicit LoadAddOn call"
    );

    let dep_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ReportFrameShared')")
        .expect("IsAddOnLoaded(Blizzard_ReportFrameShared) probe should succeed");
    assert!(
        dep_loaded,
        "Blizzard_ReportFrameShared must also be loaded — declared as the sole hard \
         `## Dependencies:` entry, eager-loaded itself via `## AllowLoad: Both` so it pre-loads \
         on every screen"
    );
}
}

prefork_full_ui_case! {
fn publishes_named_top_level_frame_under_uiparent(env: &WowLuaEnv) {

    let frame_kind: String = env
        .eval("return type(ReportFrame)")
        .expect("ReportFrame probe should succeed");
    assert_eq!(
        frame_kind, "table",
        "ReportFrame must publish at `_G` as a table — declared at ReportFrame.xml:3 with \
         `mixin=\"ReportFrameMixin\"` `inherits=\"SharedReportFrameTemplate\"` \
         `parent=\"UIParent\"` `toplevel=\"true\"` `enableMouse=\"true\"` `hidden=\"true\"` \
         `frameStrata=\"DIALOG\"`. This is the ONLY named frame in the addon; everything else is \
         inherited from SharedReportFrameTemplate"
    );

    let parented_to_uiparent: bool = env
        .eval("return ReportFrame:GetParent() == UIParent")
        .expect("ReportFrame parent probe should succeed");
    assert!(
        parented_to_uiparent,
        "ReportFrame must be parented to UIParent — `parent=\"UIParent\"` XML attribute. The \
         frame uses `frameStrata=\"DIALOG\"` so it stacks above gameplay UI when the player \
         right-clicks → Report"
    );

    let strata: String = env
        .eval("return ReportFrame:GetFrameStrata()")
        .expect("ReportFrame strata probe should succeed");
    assert_eq!(
        strata, "DIALOG",
        "ReportFrame's strata must be DIALOG so the report dialog stacks above gameplay UI \
         (default MEDIUM) but below TOOLTIP. Inherited from the XML attribute, propagated by \
         the loader to the Rust widget Frame strata field"
    );
}
}

prefork_full_ui_case! {
fn report_frame_mixin_extends_shared_via_create_from_mixins(env: &WowLuaEnv) {

    let mixin_kind: String = env
        .eval("return type(ReportFrameMixin)")
        .expect("ReportFrameMixin probe should succeed");
    assert_eq!(
        mixin_kind, "table",
        "ReportFrameMixin must publish at `_G` as a table — declared at ReportFrame.lua:1 via \
         `ReportFrameMixin = CreateFromMixins(SharedReportFrameMixin)`. The CreateFromMixins \
         helper from Blizzard_SharedXMLBase shallow-copies SharedReportFrameMixin's keys into a \
         fresh table so ReportFrameMixin inherits the base lifecycle/state methods at \
         table-build time (NOT call-chain time like RemixArtifactFrameMixin's \
         TalentFrameBaseMixin.OnLoad(self) pattern)"
    );

    let shared_kind: String = env
        .eval("return type(SharedReportFrameMixin)")
        .expect("SharedReportFrameMixin probe should succeed");
    assert_eq!(
        shared_kind, "table",
        "SharedReportFrameMixin must exist as a table provided by Blizzard_ReportFrameShared — \
         this is the parent mixin that CreateFromMixins copies from. Without it ReportFrame.lua \
         line 1 would silently produce an empty table and every method dispatch on the live \
         frame would nil-error"
    );
}
}

prefork_full_ui_case! {
fn override_methods_publish_on_report_frame_mixin(env: &WowLuaEnv) {

    for method in OVERRIDE_METHODS {
        let kind: String = env
            .eval(&format!("return type(ReportFrameMixin.{method})"))
            .unwrap_or_else(|err| panic!("type(ReportFrameMixin.{method}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ReportFrameMixin.{method} must be a function — declared with `--override` comment \
             marker in ReportFrame.lua. CanDisplayMinorCategory hides BTag/GuildName/Name minor \
             categories based on report context (clubFinderGUID, isBnetReport, \
             ClubFinderRequestType). ShouldDisplayTooltip returns true unconditionally to enable \
             tooltip text on minor-category buttons. ManageButton calls SetEnabled(isActive) so \
             the Report submit button greys out until a category is selected"
        );
    }
}
}

prefork_full_ui_case! {
fn inherited_base_methods_carry_through_via_create_from_mixins(env: &WowLuaEnv) {

    for method in INHERITED_BASE_METHODS {
        let kind: String = env
            .eval(&format!("return type(ReportFrameMixin.{method})"))
            .unwrap_or_else(|err| panic!("type(ReportFrameMixin.{method}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ReportFrameMixin.{method} must be a function inherited from SharedReportFrameMixin \
             via CreateFromMixins — the shallow-copy semantics mean every key on the parent \
             table at line 1's evaluation time is copied to ReportFrameMixin. OnLoad applies \
             NineSliceUtil Dialog border, builds the FlagsMixin minorCategoryFlags state, \
             primes the ReportingFrameMinorCategoryButtonTemplate frame pool, registers \
             REPORT_PLAYER_RESULT / REPORT_SCREENSHOT_READY, and configures the major-category \
             dropdown. The inherited InitiateReport is the public entry point called by \
             Blizzard_Calendar / FriendsFrame / Communities / LFGList / MailFrame / \
             UnitPopupShared / Professions / Housing / ItemRefHandlers when the player elects \
             to report another character"
        );
    }
}
}

prefork_full_ui_case! {
fn registers_for_report_lifecycle_events(env: &WowLuaEnv) {

    let event_count: f64 = env
        .eval(
            "local n = 0 \
             if ReportFrame:IsEventRegistered('REPORT_PLAYER_RESULT') then n = n + 1 end \
             if ReportFrame:IsEventRegistered('REPORT_SCREENSHOT_READY') then n = n + 1 end \
             return n",
        )
        .expect("Event-registration probe should succeed");
    assert_eq!(
        event_count, 2.0,
        "ReportFrame must register exactly two events at OnLoad time: REPORT_PLAYER_RESULT (the \
         async server ack that flips the panel into the thank-you state via UpdateThankYouMessage) \
         and REPORT_SCREENSHOT_READY (fires when the screenshot subsystem hands back a usable \
         preview, reveals the ScreenshotPreview texture and unblocks the submit button). \
         Registration happens inside SharedReportFrameMixin:OnLoad via self:RegisterEvent at \
         ReportFrameShared.lua:9-10 — confirmation that CreateFromMixins-inherited OnLoad ran \
         on the live derived frame"
    );
}
}

prefork_full_ui_case! {
fn xml_declares_no_virtual_templates_at_global_scope(env: &WowLuaEnv) {

    let no_template_leak: bool = env
        .eval(
            "return _G['SharedReportFrameTemplate'] == nil \
                and _G['ReportingFrameMinorCategoryButtonTemplate'] == nil",
        )
        .expect("Template global-leak probe should succeed");
    assert!(
        no_template_leak,
        "Blizzard_ReportFrame ships zero virtual templates of its own — its XML defines exactly \
         one named non-virtual frame (ReportFrame inheriting SharedReportFrameTemplate). The \
         inherited templates SharedReportFrameTemplate / ReportingFrameMinorCategoryButtonTemplate \
         live in Blizzard_ReportFrameShared with `virtual=\"true\"` so the loader keeps them in \
         the template registry only, NEVER as globals"
    );
}
}

#[test]
fn xml_uses_pure_inheritance_with_no_inline_children() {
    let xml_text = std::fs::read_to_string(report_frame_dir().join("ReportFrame.xml"))
        .expect("ReportFrame.xml should read");
    assert!(
        xml_text.contains("inherits=\"SharedReportFrameTemplate\""),
        "ReportFrame.xml must declare `inherits=\"SharedReportFrameTemplate\"` — every visible \
         child (Border NineSlice, ReportButton, ReportingMajorCategoryDropdown, \
         ScreenshotReportingFrame, Comment, MinorReportDescription, ReportString, ThankYouText, \
         Watermark) is supplied by the inherited template, not declared inline"
    );
    assert!(
        xml_text.contains("mixin=\"ReportFrameMixin\""),
        "ReportFrame.xml must wire the mixin via `mixin=\"ReportFrameMixin\"` — required so the \
         inherited <Scripts> handlers in SharedReportFrameTemplate (declared via method= \
         attributes) dispatch against the override-aware ReportFrameMixin table rather than \
         falling back to the parent SharedReportFrameMixin"
    );
    assert!(
        !xml_text.contains("<Layers>") && !xml_text.contains("<Frames>"),
        "ReportFrame.xml must NOT declare any <Layers>/<Frames> blocks — the entire visual \
         structure is inherited from SharedReportFrameTemplate. Adding inline children here \
         would diverge from the glue twin (Blizzard_ReportFrameGlue's GlueReportFrame) which \
         shares the same template; both rely on the shared XML being the single source of truth"
    );
}
