use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn shared_xml_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_SharedXML")
}

fn shared_xml_toc() -> PathBuf {
    shared_xml_dir().join("Blizzard_SharedXML.toc")
}

const HARD_DEPS: &[&str] = &[
    "Blizzard_ProjectConstants",
    "Blizzard_Fonts_Shared",
    "Blizzard_SharedXMLBase",
    "Blizzard_PrintHandler",
    "Blizzard_Menu",
    "Blizzard_Colors",
    "Blizzard_HelpPlate",
    "Blizzard_Narration",
];

const CORNERSTONE_MIXINS: &[&str] = &[
    "BackdropTemplateMixin",
    "BaseLayoutMixin",
    "ResizeLayoutMixin",
    "VerticalLayoutMixin",
    "FontableFrameMixin",
    "DataProviderMixin",
    "AnimateWhileShownMixin",
    "ColumnDisplayMixin",
    "ContentFrameMixin",
    "BulletPointMixin",
    "ButtonGroupMixin",
    "AlphaHighlightButtonMixin",
    "ClickToDragMixin",
    "DirtiableMixin",
    "DefaultPanelMixin",
    "BarDividerMixin",
    "CharacterModelSceneMixin",
    "DeselectableRadioButtonGroupMixin",
];

const CORNERSTONE_UTIL_TABLES: &[&str] = &[
    "EventUtil",
    "FormattingUtil",
    "InterpolatorUtil",
    "GridLayoutUtil",
    "InputUtil",
    "LinkUtil",
    "NineSliceUtil",
    "PixelUtil",
    "RegionUtil",
    "ScriptAnimationUtil",
    "SecondsFormatter",
    "SortUtil",
    "StringUtil",
    "BindingUtil",
    "EasingUtil",
    "BenchmarkUtil",
    "PingUtil",
    "PlayerUtil",
    "HelpTip",
];

const POPULATED_CLASS_KEYS: &[&str] = &[
    "WARRIOR",
    "PALADIN",
    "HUNTER",
    "ROGUE",
    "PRIEST",
    "DEATHKNIGHT",
    "SHAMAN",
    "MAGE",
    "WARLOCK",
    "MONK",
    "DRUID",
    "DEMONHUNTER",
    "EVOKER",
];

fn fresh_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = fresh_env();

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    env
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&shared_xml_dir()).expect("SharedXML TOC resolves");
    assert_eq!(
        resolved,
        shared_xml_toc(),
        "Retail 12.1.0.69497 ships a single bare Blizzard_SharedXML.toc"
    );
    assert!(
        !shared_xml_dir()
            .join("Blizzard_SharedXML_Mainline.toc")
            .exists()
            && !shared_xml_dir()
                .join("Blizzard_SharedXML_Mists.toc")
                .exists(),
        "Retail 12.1.0.69497 does not ship profile-specific SharedXML TOCs"
    );
}

#[test]
fn toc_declares_eight_hard_deps_in_published_order() {
    let toc = TocFile::from_file(&shared_xml_toc()).expect("SharedXML TOC parses");

    let deps = toc.dependencies();
    assert_eq!(
        deps, HARD_DEPS,
        "Retail 12.1.0.69497 declares eight SharedXML dependencies in published \
         order, including ProjectConstants first and Narration last. Got: {deps:?}"
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
}

#[test]
fn mainline_toc_eager_load_with_allow_load_both_screens() {
    let toc = TocFile::from_file(&shared_xml_toc()).expect("Mainline TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT be LoadOnDemand — SharedXML is the foundational \
         primitives layer. 54+ downstream addons hard-dep on it; lazy \
         loading would force every dependent into LoD too"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_glue_only());

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "`## AllowLoad: Both` must allow {screen:?} — the Both keyword \
             enables eager load on game AND glue screens. Glue-screen UI \
             (login, charselect, charcreate) reuses the same SharedXML \
             primitive layer for layout / dropdowns / dialogs / nine-slice \
             panels"
        );
    }
}

#[test]
fn mainline_toc_raw_bytes_pin_metadata_lines_and_no_savedvars() {
    let raw = std::fs::read_to_string(shared_xml_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard_SharedXML"));
    assert!(raw.contains("## AllowLoad: Both"));
    assert!(raw.contains(
        "## Dependencies: Blizzard_ProjectConstants, Blizzard_Fonts_Shared, \
         Blizzard_SharedXMLBase, Blizzard_PrintHandler, Blizzard_Menu, \
         Blizzard_Colors, Blizzard_HelpPlate, Blizzard_Narration"
    ));
    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## AllowLoadGameType"));

    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare SavedVariables — SharedXML is a primitives \
         library, persistent state belongs in domain-specific addons"
    );
    assert!(!raw.contains("## SavedVariablesPerCharacter"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(
        raw.contains("## Deprecated. Retained only for addons"),
        "TOC must contain the deprecation comment block — flags \
         HybridScrollFrame.lua/.xml as legacy compatibility shims kept only \
         for third-party addons that haven't migrated to ScrollBox/View"
    );
}

#[test]
fn mainline_toc_body_starts_with_localization_machinery_then_constants() {
    let raw = std::fs::read_to_string(shared_xml_toc()).expect("TOC reads utf-8");

    let body_lines: Vec<&str> = raw
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    let first_three = &body_lines[..3];
    assert_eq!(
        first_three,
        [
            "Shared\\LocalizationMachinery.lua",
            "SharedConstants.lua",
            "ColorUtil.lua",
        ],
        "TOC body MUST start with these 3 entries in order. (1) \
         Shared\\LocalizationMachinery.lua bootstraps the Localization() \
         helper used by every later file's localized strings. (2) \
         SharedConstants.lua publishes shared UI constants. (3) ColorUtil.lua \
         publishes color helpers consumed by later widget files. Got: {first_three:?}"
    );
}

#[test]
fn mainline_toc_body_count_breakdown_matches_filesystem_layout() {
    let toc = TocFile::from_file(&shared_xml_toc()).expect("Mainline TOC parses");

    let lua_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "lua"))
        .count();
    let xml_count = toc
        .files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "xml"))
        .count();

    assert_eq!(
        lua_count + xml_count,
        218,
        "Retail 12.1.0.69497 TOC body has 218 active Mainline file entries. \
         Got lua={lua_count} xml={xml_count}"
    );
    assert_eq!(
        lua_count, 145,
        "Retail 12.1.0.69497 TOC body has 145 active Mainline Lua entries. \
         Got: {lua_count}"
    );
    assert_eq!(
        xml_count, 73,
        "Retail 12.1.0.69497 TOC body has 73 active Mainline XML entries. \
         Got: {xml_count}"
    );
}

#[test]
fn shared_subdir_holds_localization_and_widget_subdirs() {
    let shared_dir = shared_xml_dir().join("Shared");
    assert!(shared_dir.is_dir());

    for subdir in [
        "Button",
        "ButtonTray",
        "Dialog",
        "Frame",
        "FrameTemplate",
        "InputBox",
        "LoadSystem",
        "ModelSceneCameras",
        "Scroll",
        "Selector",
        "Slider",
        "Tabs",
        "TabSystem",
    ] {
        assert!(
            shared_dir.join(subdir).is_dir(),
            "Shared/{subdir} subdirectory must exist — holds the \
             cross-flavor widget primitives that both Mainline and Mists \
             flavors share"
        );
    }

    assert!(shared_dir.join("Localization.lua").is_file());
    assert!(shared_dir.join("LocalizationMachinery.lua").is_file());
    assert!(shared_dir.join("VASErrorLookup.lua").is_file());
}

#[test]
fn mainline_subdir_holds_retail_only_overrides() {
    let mainline_dir = shared_xml_dir().join("Mainline");
    assert!(mainline_dir.is_dir());

    assert!(
        shared_xml_dir().join("ClassColors.lua").is_file(),
        "ClassColors.lua lives at the Blizzard_SharedXML root and populates \
         RAID_CLASS_COLORS for the retail classes plus Plunderstorm entries"
    );
    assert!(
        mainline_dir
            .join("MajorFactionRenownSharedTemplates.lua")
            .is_file(),
        "Mainline-only — Major Faction renown UI is Dragonflight+ retail"
    );
    assert!(
        mainline_dir.join("ModelFrames.lua").is_file(),
        "Mainline-only — extended model frame primitives for the retail \
         dressing room"
    );
    assert!(mainline_dir.join("ModelFrameMixin.lua").is_file());
    assert!(mainline_dir.join("ModelControlButtonMixin.lua").is_file());
    assert!(mainline_dir.join("NineSliceLayouts.lua").is_file());
    assert!(mainline_dir.join("Sound.lua").is_file());
    assert!(mainline_dir.join("ScrollDefine.lua").is_file());
}

#[test]
fn eager_discovery_includes_addon_on_all_four_screens() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_SharedXML");
        assert!(
            found,
            "Blizzard_SharedXML MUST appear in eager discovery on \
             {screen:?} — `## AllowLoad: Both` makes it eligible on every \
             screen, and 54+ downstream addons hard-dep on it"
        );
    }
}

prefork_full_ui_case! {
    fn full_game_load_emits_no_addon_specific_lua_errors(env: &WowLuaEnv) {

        let errors: Vec<String> = env.state().borrow().lua_errors.clone();
        let relevant: Vec<&String> = errors
            .iter()
            .filter(|e| {
                e.contains("Blizzard_SharedXML")
                    || e.contains("ProjectConstants")
                    || e.contains("LayoutFrame.lua")
                    || e.contains("NineSlice.lua")
                    || e.contains("Backdrop.lua")
                    || e.contains("HelpTip.lua")
                    || e.contains("ColorUtil.lua")
            })
            .collect();
        assert!(
            relevant.is_empty(),
            "Eager load via full Game UI discovery must emit zero \
             SharedXML-specific Lua errors. Got:\n  {}",
            relevant
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }
}

prefork_full_ui_case! {
    fn is_addon_loaded_returns_true_after_full_game_discovery(env: &WowLuaEnv) {

        let loaded: bool = env
            .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SharedXML')")
            .expect("IsAddOnLoaded probe");
        assert!(
            loaded,
            "C_AddOns.IsAddOnLoaded('Blizzard_SharedXML') must be true after \
             eager discovery — DefaultState=enabled + non-LoD eager addon"
        );

        for dep in HARD_DEPS {
            let dep_loaded: bool = env
                .eval(&format!("return C_AddOns.IsAddOnLoaded('{dep}')"))
                .unwrap_or_else(|_| panic!("{dep} IsAddOnLoaded probe failed"));
            assert!(
                dep_loaded,
                "Hard dep {dep} must be loaded — declared in `## Dependencies` \
                 so the loader pulls it in before SharedXML"
            );
        }
    }
}

prefork_full_ui_case! {
    fn project_constants_published_with_mainline_id(env: &WowLuaEnv) {

        let mainline_id_kind: String = env
            .eval("return type(WOW_PROJECT_MAINLINE)")
            .expect("WOW_PROJECT_MAINLINE probe");
        assert_eq!(
            mainline_id_kind, "number",
            "_G.WOW_PROJECT_MAINLINE must be a number — published by \
             ProjectConstants.lua, the canonical numeric id used in flavor \
             checks: `if WOW_PROJECT_ID == WOW_PROJECT_MAINLINE then`"
        );

        let project_id_kind: String = env
            .eval("return type(WOW_PROJECT_ID)")
            .expect("WOW_PROJECT_ID probe");
        assert_eq!(
            project_id_kind, "number",
            "_G.WOW_PROJECT_ID must be a number — set by the C client to \
             identify the running flavor; ProjectConstants.lua declares the \
             comparison constants"
        );
    }
}

prefork_full_ui_case! {
    fn raid_class_colors_populated_for_all_thirteen_retail_classes(env: &WowLuaEnv) {

        let kind: String = env
            .eval("return type(RAID_CLASS_COLORS)")
            .expect("RAID_CLASS_COLORS probe");
        assert_eq!(
            kind, "table",
            "_G.RAID_CLASS_COLORS must be a table — Mainline/ClassColors.lua \
             line 1 declares `RAID_CLASS_COLORS = {{}}` then populates it via \
             `C_ClassColor.GetClassColor(className)` for each class. The table \
             is the canonical class color lookup used by 30+ downstream UI \
             components"
        );

        for class_key in POPULATED_CLASS_KEYS {
            let entry_kind: String = env
                .eval(&format!("return type(RAID_CLASS_COLORS.{class_key})"))
                .unwrap_or_else(|_| panic!("RAID_CLASS_COLORS.{class_key} probe failed"));
            assert_eq!(
                entry_kind, "table",
                "RAID_CLASS_COLORS.{class_key} must be a table (ColorMixin \
                 instance with r/g/b/a fields and GenerateHexColor method). \
                 Missing any one means the corresponding class portrait / \
                 nameplate / chat-color rendering would fall back to a default \
                 gray"
            );

            let has_color_str: String = env
                .eval(&format!(
                    "return type(RAID_CLASS_COLORS.{class_key}.colorStr)"
                ))
                .unwrap_or_else(|_| panic!("colorStr probe for {class_key} failed"));
            assert_eq!(
                has_color_str, "string",
                "RAID_CLASS_COLORS.{class_key}.colorStr must be a string — \
                 ClassColors.lua line 27-29 iterates the populated table and \
                 calls v:GenerateHexColor() to set the colorStr cache. Used by \
                 chat hyperlink coloring without recomputing the hex per call"
            );
        }
    }
}

prefork_full_ui_case! {
    fn publishes_eighteen_cornerstone_mixin_tables(env: &WowLuaEnv) {

        for mixin in CORNERSTONE_MIXINS {
            let kind: String = env
                .eval(&format!("return type({mixin})"))
                .unwrap_or_else(|_| panic!("{mixin} probe failed"));
            assert_eq!(
                kind, "table",
                "_G.{mixin} must be a table — every downstream addon that \
                 composes via CreateFromMixins({mixin}) relies on this being \
                 present. Missing any one means the corresponding widget family \
                 fails to instantiate"
            );
        }
    }
}

prefork_full_ui_case! {
    fn publishes_nineteen_cornerstone_util_namespace_tables(env: &WowLuaEnv) {

        for util in CORNERSTONE_UTIL_TABLES {
            let kind: String = env
                .eval(&format!("return type({util})"))
                .unwrap_or_else(|_| panic!("{util} probe failed"));
            assert_eq!(
                kind, "table",
                "_G.{util} must be a table — used by 50+ downstream call sites \
                 across Blizzard_FrameXML and addons (e.g., EventUtil.\
                 ContinueOnAddOnLoaded for deferred init, FormattingUtil for \
                 number-formatting, GridLayoutUtil for grid layouts)"
            );
        }
    }
}

prefork_full_ui_case! {
    fn data_provider_mixin_inherits_callback_registry_capability(env: &WowLuaEnv) {

        let inherits: bool = env
            .eval(
                "return type(DataProviderMixin) == 'table' and \
                 type(DataProviderMixin.RegisterCallback) == 'function'",
            )
            .expect("DataProviderMixin probe");
        assert!(
            inherits,
            "DataProviderMixin must inherit from CallbackRegistryMixin — \
             declared as `DataProviderMixin = CreateFromMixins(\
             CallbackRegistryMixin)`. Without RegisterCallback, ScrollBox / \
             ScrollView consumers couldn't subscribe to OnInsert / OnRemove / \
             OnSort events that drive UI re-rendering"
        );
    }
}

prefork_full_ui_case! {
    fn resize_layout_mixin_inherits_base_layout_capability(env: &WowLuaEnv) {

        let inherits: bool = env
            .eval(
                "return type(ResizeLayoutMixin) == 'table' and \
                 type(ResizeLayoutMixin.Layout) == 'function'",
            )
            .expect("ResizeLayoutMixin probe");
        assert!(
            inherits,
            "ResizeLayoutMixin must inherit from BaseLayoutMixin — declared as \
             `ResizeLayoutMixin = CreateFromMixins(BaseLayoutMixin)`. Layout() \
             is the abstract method ResizeLayoutFrame instances call to \
             compute their final size based on layoutIndex children"
        );
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_smooth_progress_helper_is_global_not_interpolator_method() {
    let env = load_full_game_ui();

    let (global_type, namespaced_type, smoothed): (String, String, f64) = env
        .eval(
            r#"
            return type(GetSmoothProgressChange),
                type(InterpolatorUtil.GetSmoothProgressChange),
                GetSmoothProgressChange(100, 0, 100, 1)
            "#,
        )
        .expect("smooth progress snapshot mismatch probe should succeed");

    assert_eq!(global_type, "function");
    assert_eq!(namespaced_type, "nil");
    assert!((smoothed - 70.0).abs() < f64::EPSILON);
}

prefork_full_ui_case! {
    fn pixel_util_provides_screen_resolution_helpers(env: &WowLuaEnv) {

        for fn_name in [
            "GetNearestPixelSize",
            "SetWidth",
            "SetHeight",
            "SetSize",
            "SetPoint",
        ] {
            let kind: String = env
                .eval(&format!("return type(PixelUtil.{fn_name})"))
                .unwrap_or_else(|_| panic!("PixelUtil.{fn_name} probe failed"));
            assert_eq!(
                kind, "function",
                "PixelUtil.{fn_name} must be a function — pixel-snapping API \
                 that 100+ widget templates use to align edges to physical \
                 pixels. Without it, widget borders drift sub-pixel and \
                 produce blurry edges at non-integer UI scales"
            );
        }
    }
}

prefork_full_ui_case! {
    fn nine_slice_util_helpers_are_callable(env: &WowLuaEnv) {

        let kind: String = env
            .eval("return type(NineSliceUtil.ApplyLayout)")
            .expect("NineSliceUtil.ApplyLayout probe");
        assert_eq!(
            kind, "function",
            "NineSliceUtil.ApplyLayout must be a function — applies a \
             NineSliceLayout table (defined in Mainline/NineSliceLayouts.lua) \
             to a NineSlicePanelTemplate frame, configuring the 9 corner/edge/\
             center textures with atlas + offset values per slice"
        );
    }
}

prefork_full_ui_case! {
    fn help_tip_global_published_with_show_method(env: &WowLuaEnv) {

        let kind: String = env.eval("return type(HelpTip)").expect("HelpTip probe");
        assert_eq!(
            kind, "table",
            "_G.HelpTip must be a table — the tutorial-tooltip surface used \
             across glue + game UI. HelpTip.lua declares `HelpTip = {{ }}` and \
             attaches Show / Hide / HideAllSystem methods. Consumed by the \
             HelpPlate dep (which is itself a hard Dependency)"
        );

        let show_kind: String = env
            .eval("return type(HelpTip.Show)")
            .expect("HelpTip.Show probe");
        assert_eq!(
            show_kind, "function",
            "HelpTip.Show must be a function — entry point for displaying a \
             help-tip popup attached to a frame. Used by tutorial systems \
             across the entire UI"
        );
    }
}

prefork_full_ui_case! {
    fn deprecated_hybrid_scroll_frame_still_loads_for_addon_compat(env: &WowLuaEnv) {

        let kind: String = env
            .eval("return type(HybridScrollFrame_CreateButtons)")
            .expect("HybridScrollFrame_CreateButtons probe");
        assert_eq!(
            kind, "function",
            "HybridScrollFrame_CreateButtons must be a function — the \
             deprecated HybridScrollFrame.lua/.xml are explicitly retained \
             under the `## Deprecated. Retained only for addons` comment block \
             in the TOC body. Removing them would break third-party addons \
             that haven't migrated to ScrollBox/View. The function is the \
             primary entry point HybridScrollFrame consumers invoke at OnLoad"
        );
    }
}

prefork_full_ui_case! {
    fn shared_xml_publishes_192_total_top_level_globals_observable_via_publication_count(env: &WowLuaEnv) {

        let mixin_count: i64 = env
            .eval(
                "local n = 0; \
                 for k, v in pairs(_G) do \
                   if type(k) == 'string' and type(v) == 'table' and \
                      k:match('Mixin$') then \
                     n = n + 1 \
                   end \
                 end \
                 return n",
            )
            .expect("mixin global count probe");
        assert!(
            mixin_count >= 100,
            "_G must hold at least 100 *Mixin tables after full Game UI load — \
             SharedXML alone publishes 171 mixins, plus the SharedXMLBase + \
             downstream addons add more. A drop below 100 means a major \
             publication failure (e.g., a foundational file failed to execute \
             and downstream files swallowed the error). Got: {mixin_count}"
        );
    }
}
