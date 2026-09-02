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

fn report_shared_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ReportFrameShared")
}

fn report_shared_toc() -> PathBuf {
    report_shared_dir().join("Blizzard_ReportFrameShared.toc")
}

const SHARED_FILES: &[&str] = &["ReportFrameShared.lua", "ReportFrameShared.xml"];

const PUBLIC_MIXIN_TABLES: &[&str] = &[
    "SharedReportFrameMixin",
    "ScreenshotModeFrameMixin",
    "ReportingFrameMinorCategoryButtonMixin",
    "ReportButtonMixin",
    "ReportInfo",
    "ReportInfoMixin",
];

const SHARED_FRAME_LIFECYCLE_METHODS: &[&str] = &[
    "OnLoad",
    "OnHide",
    "OnEvent",
    "OnAttributeChanged",
    "Reset",
    "InitiateReport",
    "InitiateReportInternal",
    "ReportByType",
    "MajorTypeSelected",
    "SetMajorType",
    "AnchorMinorCategory",
    "SendReport",
    "SetMinorCategoryFlag",
    "SetupDropdownByReportType",
    "UpdateThankYouMessage",
    "UpdateHarmfulToMinorsMinorCategoryEnabled",
    "OnTakeScreenshotClicked",
    "CanDisplayMinorCategory",
    "ShouldDisplayTooltip",
    "ManageButton",
];

const REPORT_INFO_FACTORY_METHODS: &[&str] = &[
    "CreateReportInfoFromType",
    "CreateClubFinderReportInfo",
    "CreatePetReportInfo",
    "CreateGroupFinderPostingReportInfo",
    "CreateGroupFinderApplicantReportInfo",
    "CreateMailReportInfo",
    "CreateCraftingOrderReportInfo",
    "CreateNeighborhoodReportInfo",
    "CreateDecorReportInfo",
];

const REPORT_INFO_MIXIN_SETTERS: &[&str] = &[
    "Clear",
    "SetReportType",
    "SetReportTarget",
    "SetReportMajorCategory",
    "SetMinorCategoryFlags",
    "SetComment",
    "SetClubFinderGUID",
    "SetGroupFinderSearchResultID",
    "SetGroupFinderApplicantID",
    "SetMailIndex",
    "SetPetGUID",
    "SetCraftingOrderID",
    "SetNeighborhoodGUID",
    "SetPlotIndex",
    "SetReportedChatInline",
    "SetBasicReportInfo",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "ReportingFrameMinorCategoryButtonTemplate",
    "SharedReportFrameTemplate",
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
        find_toc_file(&report_shared_dir()).expect("Blizzard_ReportFrameShared TOC should resolve");
    assert_eq!(
        resolved,
        report_shared_toc(),
        "Blizzard_ReportFrameShared ships exactly one TOC at the bare \
         `Blizzard_ReportFrameShared.toc` path — no `_Mainline` flavor split. find_toc_file at \
         src/loader/mod.rs:65 tries `_Mainline.toc` first (miss), then bare (hit). The shared \
         addon is intentionally flavor-agnostic because both Game (Blizzard_ReportFrame) and \
         Glue (Blizzard_ReportFrameGlue) consumers depend on the same template/mixin surface"
    );
}

#[test]
fn toc_declares_eager_both_screens_with_no_hard_deps() {
    let toc = TocFile::from_file(&report_shared_toc())
        .expect("Blizzard_ReportFrameShared TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_ReportFrameShared must NOT declare `## LoadOnDemand: 1` — the shared template \
         and mixin tables have to be live before its dependents (Blizzard_ReportFrame on Game; \
         Blizzard_ReportFrameGlue on Login/CharacterSelect/CharacterCreate) can load. \
         Dependency-resolution in src/loader/mod.rs would refuse to load the dependents if the \
         shared addon were LOD"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_ReportFrameShared declares no `## AllowLoadGameType` directive — \
         is_game_type_restricted at src/toc.rs:294-302 returns FALSE when the metadata is \
         absent. The shared report dialog applies to every flavor without a server-side gate"
    );
    assert_eq!(
        toc.metadata.get("DefaultState").map(String::as_str),
        Some("enabled"),
        "DefaultState=enabled preserved verbatim — without it the eager-loader would skip the \
         shared addon and the Game/Glue dependents would refuse to load due to the unresolved \
         hard dep"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_ReportFrameShared declares no hard dependencies; its sole optional dependency is Blizzard_GlueParent"
    );
    assert!(
        toc.saved_variables().is_empty() && toc.saved_variables_per_character().is_empty(),
        "Blizzard_ReportFrameShared declares zero saved variables — submitted reports go \
         straight to C_ReportSystem.SendReport, never persisted client-side"
    );
}

#[test]
fn toc_declares_optional_dep_singular_advisory_only() {
    let toc = TocFile::from_file(&report_shared_toc())
        .expect("Blizzard_ReportFrameShared TOC should parse");
    let optional_dep_singular = toc
        .metadata
        .get("OptionalDep")
        .map(String::as_str)
        .unwrap_or("");
    assert_eq!(
        optional_dep_singular, "Blizzard_GlueParent",
        "Retail 12.1.0.69497 declares Blizzard_GlueParent through singular OptionalDep metadata"
    );
    assert_eq!(
        toc.optional_deps(),
        vec!["Blizzard_GlueParent".to_string()],
        "the singular `## OptionalDep:` directive exposes the current GlueParent advisory dependency"
    );
}

#[test]
fn toc_lists_two_files_in_root_directory() {
    let toc = TocFile::from_file(&report_shared_toc())
        .expect("Blizzard_ReportFrameShared TOC should parse");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        listed, SHARED_FILES,
        "TOC body must list exactly these 2 files in this order: ReportFrameShared.lua loads \
         FIRST so the 6 mixin tables (SharedReportFrameMixin, ScreenshotModeFrameMixin, \
         ReportingFrameMinorCategoryButtonMixin, ReportButtonMixin, ReportInfo factory, \
         ReportInfoMixin) plus the file-local CopyTableWithCleanedPairs / \
         EnumerateTaintedKeysTable taint-sanitization helpers and ScreenshotReportErrorStrings \
         publish before ReportFrameShared.xml's `mixin=\"...Mixin\"` attributes resolve them"
    );
}

#[test]
fn allows_screen_returns_true_for_every_screen() {
    let toc = TocFile::from_file(&report_shared_toc())
        .expect("Blizzard_ReportFrameShared TOC should parse");
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "Blizzard_ReportFrameShared declares `## AllowLoad: Both` — must allow EVERY \
             screen because both the Game variant (Blizzard_ReportFrame) and the Glue variant \
             (Blizzard_ReportFrameGlue) hard-depend on it. src/toc.rs:307 maps `both` → \
             always true. (Screen tested: {screen:?})"
        );
    }
}

#[test]
fn included_in_eager_discovery_for_every_screen() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_ReportFrameShared");
        assert!(
            found,
            "Blizzard_ReportFrameShared must appear in eager auto-discovery on EVERY screen — \
             AllowLoad=Both passes the truthy branch at src/toc.rs:307. Required because both \
             Game and Glue dependents declare it as a hard `## Dependencies:` entry, and the \
             eager loader pulls deps in topological order so the shared addon must be \
             discoverable on whichever screen its dependents are loading on. (Screen tested: \
             {screen:?})"
        );
    }
}

#[test]
fn root_directory_holds_two_files_next_to_toc() {
    let mut entries: Vec<String> = std::fs::read_dir(report_shared_dir())
        .expect("Blizzard_ReportFrameShared directory should read")
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name != "Blizzard_ReportFrameShared.toc")
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            "ReportFrameShared.lua".to_string(),
            "ReportFrameShared.xml".to_string(),
        ],
        "Blizzard_ReportFrameShared/ root must hold exactly the lua/xml pair next to the TOC — \
         no per-flavor subdirectory and no localization stub"
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
            message.contains("ReportFrameShared")
                || message.contains("SharedReportFrameMixin")
                || message.contains("ReportInfoMixin")
                || message.contains("ScreenshotModeFrameMixin")
                || message.contains("ReportButtonMixin")
                || message.contains("ReportingFrameMinorCategoryButtonMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ReportFrameShared emitted Lua errors during eager Game-screen load. The \
         shared addon is the foundation: 6 mixin tables, 9 ReportInfo factory methods, 16 \
         ReportInfoMixin setters, 2 virtual templates and the named ReportScreenshotModeFrame \
         all publish here. Any error is a real load failure:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ReportFrameShared')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ReportFrameShared') must return true after the eager \
         Game sweep — confirms the loader registers the non-LOD addon with the loaded-set \
         without an explicit LoadAddOn call. Required for the dependent Blizzard_ReportFrame's \
         dependency-resolution to mark the dep as already-loaded"
    );
}
}

prefork_full_ui_case! {
fn six_public_mixin_tables_publish(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXIN_TABLES {
        let kind: String = env
            .eval(&format!("return type(_G['{mixin}'])"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at `_G` as a table — the shared addon's Lua chunk declares 6 \
             public mixin/factory tables: SharedReportFrameMixin (the base for the report \
             dialog frame, 18 methods), ScreenshotModeFrameMixin (drives ReportScreenshotModeFrame's \
             SetAlternateTopLevelParent / UIParent:Hide camera-mode lifecycle), \
             ReportingFrameMinorCategoryButtonMixin (CheckButton template's OnClick / OnEnter / \
             OnLeave / SetupButton / SetMinorCategoryEnabled), ReportButtonMixin (ReportButton \
             child's OnClick / UpdateButtonState / OnEnter / OnLeave), ReportInfo (factory with \
             9 Create* methods returning ReportInfoMixin instances), and ReportInfoMixin (16 \
             setters describing one specific report's payload before it ships to \
             C_ReportSystem.SendReport)"
        );
    }
}
}

prefork_full_ui_case! {
fn shared_report_frame_mixin_exposes_full_lifecycle_surface(env: &WowLuaEnv) {

    for method in SHARED_FRAME_LIFECYCLE_METHODS {
        let kind: String = env
            .eval(&format!("return type(SharedReportFrameMixin.{method})"))
            .unwrap_or_else(|err| {
                panic!("type(SharedReportFrameMixin.{method}) probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "SharedReportFrameMixin.{method} must be a function — the shared mixin owns the \
             entire report-dialog lifecycle. OnLoad applies the NineSliceUtil Dialog border, \
             builds a FlagsMixin minorCategoryFlags state, primes the \
             ReportingFrameMinorCategoryButtonTemplate frame pool, registers \
             REPORT_PLAYER_RESULT and REPORT_SCREENSHOT_READY, and configures the \
             major-category WowStyle1Dropdown. OnAttributeChanged dispatches the \
             `initiate_report` attribute through CopyTableWithCleanedPairs (taint-sanitized \
             deep-copy via securecallfunction) before invoking InitiateReportInternal. The \
             three virtual hooks CanDisplayMinorCategory / ShouldDisplayTooltip / ManageButton \
             are stubbed bodies (`function ... () end`) here; the Game and Glue addons override \
             each with their own semantics via CreateFromMixins"
        );
    }
}
}

prefork_full_ui_case! {
fn report_info_factory_publishes_nine_create_methods(env: &WowLuaEnv) {

    for method in REPORT_INFO_FACTORY_METHODS {
        let kind: String = env
            .eval(&format!("return type(ReportInfo.{method})"))
            .unwrap_or_else(|err| panic!("type(ReportInfo.{method}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ReportInfo.{method} must be a function — the ReportInfo factory exposes 9 \
             specialized constructors that each call CreateReportInfoFromType (which itself \
             calls CreateFromMixins(ReportInfoMixin) then SetReportType) before tagging the \
             returned object with the appropriate target metadata. Used by 24+ downstream \
             consumers in the Blizzard codebase to wire context-specific report payloads \
             before passing them to ReportFrame:InitiateReport — e.g. \
             CreateClubFinderReportInfo for Communities, CreatePetReportInfo for pet \
             reporting, CreateGroupFinderPostingReportInfo / CreateGroupFinderApplicantReportInfo \
             for LFGList, CreateMailReportInfo for MailFrame, CreateCraftingOrderReportInfo \
             for Professions, CreateNeighborhoodReportInfo / CreateDecorReportInfo for \
             Housing, CreateReportInfoFromType for the catch-all path"
        );
    }
}
}

prefork_full_ui_case! {
fn report_info_mixin_publishes_setters_and_clear(env: &WowLuaEnv) {

    for method in REPORT_INFO_MIXIN_SETTERS {
        let kind: String = env
            .eval(&format!("return type(ReportInfoMixin.{method})"))
            .unwrap_or_else(|err| panic!("type(ReportInfoMixin.{method}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ReportInfoMixin.{method} must be a function — the report-info instance mixin \
             exposes 15 Set*-style setters plus Clear. SendReport at line 218 reads three of \
             these (reportType / majorCategory / minorCategoryFlags) directly off self.reportInfo \
             before calling C_ReportSystem.SendReport. SetMailIndex(mailIndex) decrements by 1 \
             at line 457 because the C API uses 0-indexed mail entries while Lua uses \
             1-indexed pool indices"
        );
    }
}
}

prefork_full_ui_case! {
fn screenshot_mode_frame_mixin_drives_alt_top_level_parent_camera_mode(env: &WowLuaEnv) {

    let probe: bool = env
        .eval(
            "return type(ScreenshotModeFrameMixin.OnShow) == 'function' \
                and type(ScreenshotModeFrameMixin.OnHide) == 'function' \
                and type(ScreenshotModeFrameMixin.OnMouseUp) == 'function'",
        )
        .expect("ScreenshotModeFrameMixin probe should succeed");
    assert!(
        probe,
        "ScreenshotModeFrameMixin must expose OnShow / OnHide / OnMouseUp — these are wired \
         via XML <Scripts method=...> on ReportScreenshotModeFrame at lines 222-225. OnShow \
         calls SetAlternateTopLevelParent(self) and UIParent:Hide() so the screenshot frame \
         takes over as the visible top-level parent (gameplay UI is hidden during screenshot \
         capture); OnHide reverses with ClearAlternateTopLevelParent + UIParent:Show. \
         OnMouseUp on LeftButton plays SOUNDKIT.REPORT_SCREENSHOT_CAMERA, runs \
         C_Housing.CanTakeReportScreenshot validation, and either shows ScreenshotError or \
         calls C_ReportSystem.TakeReportScreenshot"
    );
}
}

prefork_full_ui_case! {
fn report_screenshot_mode_frame_publishes_under_implicit_root(env: &WowLuaEnv) {

    let frame_kind: String = env
        .eval("return type(ReportScreenshotModeFrame)")
        .expect("ReportScreenshotModeFrame probe should succeed");
    assert_eq!(
        frame_kind, "table",
        "ReportScreenshotModeFrame must publish at `_G` as a table — it is the ONLY named \
         non-virtual frame in Blizzard_ReportFrameShared, declared at \
         ReportFrameShared.xml:207 with `mixin=\"ScreenshotModeFrameMixin\"` \
         `inherits=\"TopLevelParentScaleFrameTemplate\"` `toplevel=\"true\"` \
         `setAllPoints=\"true\"` `ignoreParentScale=\"true\"` `enableKeyboard=\"true\"` \
         `hidden=\"true\"`. No `parent=` attribute means the loader uses the implicit \
         screen-context root (UIParent on Game, GlueParent on Glue). The frame becomes the \
         active top-level parent during screenshot-capture mode via SetAlternateTopLevelParent"
    );

    let toplevel: bool = env
        .eval("return ReportScreenshotModeFrame:IsToplevel()")
        .expect("ReportScreenshotModeFrame toplevel probe should succeed");
    assert!(
        toplevel,
        "ReportScreenshotModeFrame must report toplevel=true — the XML attribute marks it as a \
         top-level parent so Z-order lifts to the front when shown. Required for the \
         camera-mode workflow where this frame must visually obscure all other UI during \
         screenshot capture"
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
            "{template} must NOT publish at `_G` — declared as `virtual=\"true\"` so the \
             loader keeps it in the template registry only, never as a global. \
             ReportingFrameMinorCategoryButtonTemplate is the CheckButton template (350×41, \
             ClickCastList-ButtonBackground / ButtonHighlight / ReportList-ButtonSelect \
             atlases) consumed by SharedReportFrameMixin's CreateFramePool; \
             SharedReportFrameTemplate is the Frame template (400×190 ResizeLayoutFrame with \
             NineSlice Border, BACKGROUND/BORDER/ARTWORK/OVERLAY layers, ReportingMajorCategoryDropdown, \
             ScreenshotReportingFrame nested frame, Comment InputScrollFrame, ReportButton, \
             CloseButton, ThankYouText / TitleText / ReportString / Watermark FontStrings) \
             inherited by both ReportFrame (Game) and ReportFrame (Glue)"
        );
    }
}
}

prefork_full_ui_case! {
fn module_locals_stay_file_scoped(env: &WowLuaEnv) {

    let no_module_local_leak: bool = env
        .eval(
            "return _G['ScreenshotReportErrorStrings'] == nil \
                and _G['CopyTableWithCleanedPairs'] == nil \
                and _G['EnumerateTaintedKeysTable'] == nil \
                and _G['SanitizableTypeSet'] == nil",
        )
        .expect("Module-local probes should succeed");
    assert!(
        no_module_local_leak,
        "Blizzard_ReportFrameShared's file-local helpers must NOT leak into `_G`: \
         ScreenshotReportErrorStrings (the InvalidPlotScreenshotReason → error-string lookup \
         for housing-decor screenshot validation), CopyTableWithCleanedPairs (deep-copy \
         taint-sanitizer used by OnAttributeChanged to filter only string/number/boolean keys \
         out of the initiate_report attribute payload), EnumerateTaintedKeysTable (wraps pairs \
         in securecallfunction to walk the tainted attribute table without acquiring the \
         caller's taint), SanitizableTypeSet (the {{string=true,number=true,boolean=true}} \
         allow-set that gates the deep-copy). All scoped via the `do ... end` block at \
         lines 517-573 plus inner `local` declarations"
    );
}
}

#[test]
fn xml_declares_two_virtual_templates_and_one_named_frame() {
    let xml_text = std::fs::read_to_string(report_shared_dir().join("ReportFrameShared.xml"))
        .expect("ReportFrameShared.xml should read");
    assert!(
        xml_text.contains("name=\"ReportingFrameMinorCategoryButtonTemplate\"")
            && xml_text.contains("virtual=\"true\""),
        "ReportFrameShared.xml must declare ReportingFrameMinorCategoryButtonTemplate as \
         virtual=\"true\" — the CheckButton template at lines 3-20 is consumed by \
         SharedReportFrameMixin's CreateFramePool(\"CHECKBUTTON\", self, \
         \"ReportingFrameMinorCategoryButtonTemplate\") at OnLoad time"
    );
    assert!(
        xml_text.contains("name=\"SharedReportFrameTemplate\""),
        "ReportFrameShared.xml must declare SharedReportFrameTemplate — the virtual Frame at \
         lines 21-205 inherited by both Game and Glue's ReportFrame instances"
    );
    assert!(
        xml_text.contains("name=\"ReportScreenshotModeFrame\""),
        "ReportFrameShared.xml must declare ReportScreenshotModeFrame as the only named \
         non-virtual frame (lines 207-227) — the camera-mode frame opened via \
         ReportScreenshotModeFrame:Show() from SharedReportFrameMixin:OnTakeScreenshotClicked"
    );
    assert!(
        xml_text.contains("<OnLoad method=\"OnLoad\"/>")
            && xml_text.contains("<OnAttributeChanged method=\"OnAttributeChanged\"/>"),
        "SharedReportFrameTemplate must wire OnLoad / OnAttributeChanged via mixin-method \
         dispatch — the OnAttributeChanged handler is the modern protected-frame entry point: \
         InitiateReport sets `initiate_report` attribute via SetAttribute(...) which fires \
         OnAttributeChanged with the cleaned data payload, decoupling the protected/insecure \
         caller from the frame's secure environment"
    );
}
