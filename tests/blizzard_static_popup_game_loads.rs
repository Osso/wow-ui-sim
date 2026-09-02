use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn static_popup_game_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_StaticPopup_Game")
}

fn static_popup_game_toc() -> PathBuf {
    static_popup_game_dir().join("Blizzard_StaticPopup_Game.toc")
}

const PUBLISHED_MIXINS: &[&str] = &[
    "GameDialogBaseMixin",
    "GameDialogMixin",
    "GameDialogCoverFrameMixin",
    "StaticPopupItemFrameMixin",
];

const BASE_MIXIN_METHODS: &[&str] = &["OnLoad", "SetCloseButtonToMinimize", "SetCloseButtonToHide"];

const GAME_DIALOG_SETUP_METHODS: &[&str] = &[
    "OnLoad",
    "Init",
    "SetupDecorationFrames",
    "SetupText",
    "SetupCloseButton",
    "SetupInsertedFrame",
    "SetupEditBox",
    "SetupDropdown",
    "SetupMoneyFrame",
    "SetupItemFrame",
    "SetupButtons",
    "SetupAlertIcon",
    "SetupStartDelay",
    "SetupExtraButton",
    "SetupProgressBar",
    "SetupAnchor",
    "SetupElementAnchoring",
];

const GAME_DIALOG_ACCESSOR_METHODS: &[&str] = &[
    "GetItemFrame",
    "GetEditBox",
    "GetButton",
    "GetButton1",
    "GetButton2",
    "GetButton3",
    "GetButton4",
    "GetButtons",
    "GetExtraFrame",
    "GetTextFontString",
    "GetButtonSizeInfo",
    "GetInitialWidth",
];

const GAME_DIALOG_LIFECYCLE_METHODS: &[&str] = &[
    "OnUpdate",
    "OnEvent",
    "OnShow",
    "OnHide",
    "OnHyperlinkClick",
    "OnHyperlinkEnter",
    "OnHyperlinkLeave",
    "OnCloseButtonClicked",
    "Resize",
    "SetText",
    "GetText",
    "SetFormattedText",
    "ClearTextScripts",
    "SetTextScripts",
];

const ITEM_FRAME_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "OnEnter",
    "OnLeave",
    "SetCustomOnEnter",
    "RetrieveInfo",
    "DisplayInfo",
    "DisplayInfoFromStandardCallback",
];

const COVER_FRAME_MIXIN_METHODS: &[&str] = &["Init", "OnKeyDown"];

const DEFS_UTIL_FUNCTIONS: &[&str] = &[
    "GetSelfResurrectDialogOptions",
    "OnResurrectButtonClick",
    "GetDefaultExpirationText",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "StaticPopupButtonTemplate",
    "StaticPopupBaseTemplate",
    "StaticPopupTemplate",
];

const NAMED_DIALOG_SLOTS: &[&str] = &[
    "StaticPopup1",
    "StaticPopup2",
    "StaticPopup3",
    "StaticPopup4",
];

const SHARED_DIALOG_KEYS: &[&str] = &[
    "XP_LOSS_NO_SICKNESS_NO_DURABILITY",
    "OKAY",
    "GENERIC_INPUT_BOX",
    "GENERIC_DROP_DOWN",
    "CONFIRM_OVERWRITE_EQUIPMENT_SET",
    "CONFIRM_SAVE_EQUIPMENT_SET",
    "CONFIRM_DELETE_EQUIPMENT_SET",
    "ERROR_CINEMATIC",
    "TOO_MANY_LUA_ERRORS",
    "USE_GUILDBANK_REPAIR",
];

const ATLAS_GLOBAL_NAMES: &[&str] = &[
    "GameDialogCloseButtonStateNormal",
    "GameDialogCloseButtonStatePressed",
    "GameDialogCloseButtonStateCondensedNormal",
    "GameDialogCloseButtonStateCondensedPressed",
    "GameDialogBackgroundTop",
    "GameDialogAlertTextureName",
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
    let resolved = find_toc_file(&static_popup_game_dir()).expect("StaticPopup_Game TOC resolves");
    assert_eq!(
        resolved,
        static_popup_game_toc(),
        "Bare TOC: per-flavor selection is inline via [Family]/[Game] \
         placeholders + [AllowLoadGameType] annotations (toc.rs:144-146)"
    );
}

#[test]
fn dependencies_chain_pulls_six_blizzard_addons_via_plural_dependencies_key() {
    let toc = TocFile::from_file(&static_popup_game_toc()).expect("TOC parses");

    let expected_deps = vec![
        "Blizzard_StaticPopup".to_string(),
        "Blizzard_ItemButton".to_string(),
        "Blizzard_AutoComplete".to_string(),
        "Blizzard_MoneyFrame".to_string(),
        "Blizzard_AccessibilityTemplates".to_string(),
        "Blizzard_GameMenuEsc".to_string(),
    ];

    assert_eq!(
        toc.dependencies(),
        expected_deps,
        "Plural `## Dependencies:` parsed at toc.rs:210-217. Order: \
         StaticPopup (dispatcher), ItemButton (SetItemButtonTexture/Quality), \
         AutoComplete (EditBox template), MoneyFrame (Money templates), \
         AccessibilityTemplates (UserScaledFrameTemplate), GameMenuEsc \
         (escape-menu integration). Got: {:?}",
        toc.dependencies()
    );
}

#[test]
fn allow_load_game_resolves_to_game_screen_only() {
    let toc = TocFile::from_file(&static_popup_game_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` matches Game (toc.rs:308, case-insensitive)"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: game` excludes glue screens — glue dialogs live in \
         Blizzard_StaticPopup_Glue"
    );
    assert!(!toc.allows_screen(ScreenKind::CharacterSelect));
    assert!(!toc.allows_screen(ScreenKind::CharacterCreate));
}

#[test]
fn no_game_type_restriction_at_addon_level() {
    let toc = TocFile::from_file(&static_popup_game_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "No addon-level `## AllowLoadGameType` — filtering is per-file via \
         inline annotations. None-branch returns false (toc.rs:294-302)"
    );
}

#[test]
fn toc_is_eager_with_no_secure_env_or_saved_vars() {
    let toc = TocFile::from_file(&static_popup_game_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "Eager: dialog surface (StaticPopup1..4, GameDialogMixin) must be \
         ready before consumer addons run"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.default_enabled());
}

#[test]
fn toc_raw_bytes_pin_three_metadata_directives() {
    let raw = std::fs::read_to_string(static_popup_game_toc()).expect("TOC reads utf-8");

    let expected_directives = [
        "## Title: Blizzard_StaticPopup_Game",
        "## Dependencies: Blizzard_StaticPopup, Blizzard_ItemButton, Blizzard_AutoComplete, Blizzard_MoneyFrame, Blizzard_AccessibilityTemplates, Blizzard_GameMenuEsc",
        "## AllowLoad: game",
    ];

    for directive in expected_directives {
        assert!(
            raw.contains(directive),
            "Raw TOC must pin `{directive}` — 3 metadata lines + 11 body \
             entries; each directive is load-bearing"
        );
    }

    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDep"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## UseSecureEnvironment"));
}

#[test]
fn body_substitutes_family_and_filters_classic_only_entries() {
    let toc = TocFile::from_file(&static_popup_game_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let expected = [
        "GameDialogDefsUtil.lua",
        "GameDialogDefs.lua",
        "Mainline/GameDialogDefs.lua",
        "GameDialog.lua",
        "Mainline/GameDialog.lua",
        "GameDialog.xml",
        "Mainline/StaticPopupSpecial.xml",
    ];

    assert_eq!(
        body.len(),
        expected.len(),
        "7 retained body entries: 11 raw lines minus 4 filtered by inline \
         AllowLoadGameType (1 wrath/cata/mists + 3 classic-only). \
         [Family] → Mainline/ per toc.rs:145. Got: {body:?}"
    );

    for (i, want) in expected.iter().enumerate() {
        assert_eq!(
            &body[i], want,
            "Body entry {i}: expected {want}, got {}",
            body[i]
        );
    }
}

#[test]
fn body_orders_definitions_before_logic_before_layout() {
    let toc = TocFile::from_file(&static_popup_game_toc()).expect("TOC parses");

    let body: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert_eq!(
        body[0], "GameDialogDefsUtil.lua",
        "DefsUtil first: dialog defs reference its helpers at file scope \
         (eg GetDefaultExpirationText bound to dialogInfo.GetExpirationText)"
    );

    assert!(
        body.iter().position(|f| f == "GameDialog.lua").unwrap()
            > body.iter().position(|f| f == "GameDialogDefs.lua").unwrap(),
        "Defs (data) before GameDialog.lua (logic)"
    );

    assert_eq!(
        body[5], "GameDialog.xml",
        "XML last: mixin=\"…\" attrs are resolved at template-registration \
         time, so mixin tables must already exist in _G"
    );
}

#[test]
fn appears_in_eager_discovery_on_game_screen_only() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let game_found = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_StaticPopup_Game");
    assert!(
        game_found,
        "Game-screen eager sweep must include StaticPopup_Game"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_StaticPopup_Game");
        assert!(!found, "{screen:?} sweep must exclude StaticPopup_Game");
    }
}

#[test]
fn full_game_load_emits_no_addon_specific_lua_errors() {
    let env = load_full_ui_for(ScreenKind::Game);

    let errors = env.state().borrow().lua_errors.clone();
    let needles = [
        "GameDialog.lua",
        "GameDialogDefs.lua",
        "GameDialogDefsUtil.lua",
        "GameDialog.xml",
        "GameDialogBaseMixin",
        "GameDialogMixin",
        "GameDialogCoverFrameMixin",
        "StaticPopupItemFrameMixin",
        "StaticPopup_Game",
        "Blizzard_StaticPopup_Game",
    ];

    let matched: Vec<&String> = errors
        .iter()
        .filter(|e| needles.iter().any(|n| e.contains(n)))
        .collect();

    assert!(
        matched.is_empty(),
        "Full Game-screen load must emit zero StaticPopup_Game-specific \
         Lua errors. Found {} matching errors: {:#?}",
        matched.len(),
        matched
    );
}

#[test]
fn is_addon_loaded_reports_true_after_eager_sweep() {
    let env = load_full_ui_for(ScreenKind::Game);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_StaticPopup_Game')")
        .expect("IsAddOnLoaded query");
    assert!(
        loaded,
        "IsAddOnLoaded('Blizzard_StaticPopup_Game') = true after eager sweep"
    );
}

#[test]
fn equipment_set_confirmation_popup_resizes_to_compact_dialog() {
    let env = load_full_ui_for(ScreenKind::Game);

    let (width, height, text_width): (f64, f64, f64) = env
        .eval(
            r#"
            local dialog = StaticPopup_Show("CONFIRM_SAVE_EQUIPMENT_SET", "kk", nil, "kk")
            assert(dialog ~= nil, "CONFIRM_SAVE_EQUIPMENT_SET dialog should be shown")
            return dialog:GetWidth(), dialog:GetHeight(), dialog.Text:GetWidth()
            "#,
        )
        .expect("equipment set StaticPopup dimensions");

    assert!(
        (300.0..=380.0).contains(&width),
        "Equipment-set confirmation dialog should keep Blizzard's compact popup width \
         instead of stretching to a viewport-derived size. Got width={width}, \
         text_width={text_width}"
    );
    assert!(
        height <= 180.0,
        "Equipment-set confirmation dialog should be a compact confirmation box, not a \
         screen-tall overlay. Got height={height}, width={width}, text_width={text_width}"
    );
}

#[test]
fn publishes_four_mixin_tables_at_global_scope() {
    let env = load_full_ui_for(ScreenKind::Game);

    for mixin in PUBLISHED_MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} type probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at _G as a table — GameDialog.lua creates \
             3 ({{}}, CreateFromMixins, {{}}) + StaticPopupItemFrameMixin"
        );
    }
}

#[test]
fn base_mixin_carries_three_close_button_methods() {
    let env = load_full_ui_for(ScreenKind::Game);

    for method in BASE_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(GameDialogBaseMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("GameDialogBaseMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameDialogBaseMixin.{method} = function — base owns OnLoad \
             (BG atlas), SetCloseButtonTo{{Minimize,Hide}} (atlas pairs)"
        );
    }
}

#[test]
fn game_dialog_mixin_inherits_from_base_via_create_from_mixins() {
    let env = load_full_ui_for(ScreenKind::Game);

    for method in BASE_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(GameDialogMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("GameDialogMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameDialogMixin.{method} inherited via \
             CreateFromMixins(GameDialogBaseMixin) at GameDialog.lua:39"
        );
    }
}

#[test]
fn game_dialog_mixin_carries_seventeen_setup_methods() {
    let env = load_full_ui_for(ScreenKind::Game);

    for method in GAME_DIALOG_SETUP_METHODS {
        let kind: String = env
            .eval(&format!("return type(GameDialogMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("GameDialogMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameDialogMixin.{method} = function — setup surface \
             decomposes Init() into per-region steps for dialog reuse"
        );
    }
}

#[test]
fn game_dialog_mixin_carries_twelve_accessor_methods() {
    let env = load_full_ui_for(ScreenKind::Game);

    for method in GAME_DIALOG_ACCESSOR_METHODS {
        let kind: String = env
            .eval(&format!("return type(GameDialogMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("GameDialogMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameDialogMixin.{method} = function — accessor exposes child \
             frames through stable names for the dispatcher"
        );
    }
}

#[test]
fn game_dialog_mixin_carries_fourteen_lifecycle_and_text_methods() {
    let env = load_full_ui_for(ScreenKind::Game);

    for method in GAME_DIALOG_LIFECYCLE_METHODS {
        let kind: String = env
            .eval(&format!("return type(GameDialogMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("GameDialogMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameDialogMixin.{method} = function — lifecycle wires script \
             handlers + hyperlinks + Resize + text accessors"
        );
    }
}

#[test]
fn item_frame_mixin_carries_eight_methods() {
    let env = load_full_ui_for(ScreenKind::Game);

    for method in ITEM_FRAME_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(StaticPopupItemFrameMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("StaticPopupItemFrameMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "StaticPopupItemFrameMixin.{method} = function — handles \
             GET_ITEM_INFO_RECEIVED, tooltip, DisplayInfo variants"
        );
    }
}

#[test]
fn cover_frame_mixin_carries_init_and_keydown() {
    let env = load_full_ui_for(ScreenKind::Game);

    for method in COVER_FRAME_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(GameDialogCoverFrameMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("GameDialogCoverFrameMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameDialogCoverFrameMixin.{method} = function — fullScreenCover \
             Frame; Init stores hideOnEscape, OnKeyDown handles ESCAPE"
        );
    }
}

#[test]
fn defs_util_table_carries_three_helper_functions() {
    let env = load_full_ui_for(ScreenKind::Game);

    let kind: String = env
        .eval("return type(GameDialogDefsUtil)")
        .expect("GameDialogDefsUtil probe");
    assert_eq!(
        kind, "table",
        "GameDialogDefsUtil = global table — created at file scope with 3 \
         helpers consumed by GameDialogDefs.lua"
    );

    for helper in DEFS_UTIL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(GameDialogDefsUtil['{helper}'])"))
            .unwrap_or_else(|err| panic!("GameDialogDefsUtil.{helper} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "GameDialogDefsUtil.{helper} = function — resurrect-dialog \
             helpers + GetDefaultExpirationText time-left formatter"
        );
    }
}

#[test]
fn close_button_atlas_globals_publish_as_strings() {
    let env = load_full_ui_for(ScreenKind::Game);

    for name in ATLAS_GLOBAL_NAMES {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            kind, "string",
            "{name} = string global — atlas/texture path consumed by \
             OnLoad / SetCloseButtonTo* / SetupAlertIcon at runtime"
        );
    }
}

#[test]
fn money_frame_on_load_global_publishes_as_function() {
    let env = load_full_ui_for(ScreenKind::Game);

    let kind: String = env
        .eval("return type(GameDialog_MoneyFrameOnLoad)")
        .expect("GameDialog_MoneyFrameOnLoad probe");
    assert_eq!(
        kind, "function",
        "GameDialog_MoneyFrameOnLoad = global function — OnLoad shim \
         (GameDialog.lua:844) wired by XML <OnLoad>"
    );
}

#[test]
fn xml_registers_three_virtual_templates() {
    let env = load_full_ui_for(ScreenKind::Game);

    for template in VIRTUAL_TEMPLATES {
        let probe = format!(
            "local ok, frame = pcall(function() \
                return CreateFrame('Frame', nil, UIParent, {template:?}) \
             end) \
             return ok and frame ~= nil"
        );
        let result: bool = env
            .eval(&probe)
            .unwrap_or_else(|err| panic!("template probe ({template}): {err}"));
        assert!(
            result,
            "Virtual template {template} must materialize via CreateFrame \
             — virtual=\"true\", consumed by inheritance"
        );
    }
}

#[test]
fn four_named_dialog_slots_materialize_as_hidden_dialogs() {
    let env = load_full_ui_for(ScreenKind::Game);

    for name in NAMED_DIALOG_SLOTS {
        let kind: String = env
            .eval(&format!("return type({name})"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{name} = table — XML declares 4 named StaticPopup1..4 frames \
             inheriting StaticPopupTemplate, hidden=true (4-slot pool)"
        );
    }
}

#[test]
fn shared_dialog_definitions_seed_into_dispatcher() {
    let env = load_full_ui_for(ScreenKind::Game);

    for key in SHARED_DIALOG_KEYS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupDialogs['{key}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupDialogs[{key}] probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "StaticPopupDialogs['{key}'] seeded — GameDialogDefs.lua \
             injects 186 dialog defs into the dispatcher"
        );
    }
}

#[test]
fn pet_battle_queue_ready_frame_materializes_from_mainline_special_xml() {
    let env = load_full_ui_for(ScreenKind::Game);

    let kind: String = env
        .eval("return type(PetBattleQueueReadyFrame)")
        .expect("PetBattleQueueReadyFrame probe");
    assert_eq!(
        kind, "table",
        "PetBattleQueueReadyFrame = frame — Mainline/StaticPopupSpecial.xml \
         is mainline-gated and retained by toc.rs:141-143 body filter"
    );
}
