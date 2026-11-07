#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
#[cfg(feature = "client-ptr")]
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn item_upgrade_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ItemUpgradeUI")
}

fn item_upgrade_mainline_toc() -> PathBuf {
    find_toc_file(&item_upgrade_dir()).expect("Blizzard_ItemUpgradeUI TOC should resolve")
}

const ITEM_UPGRADE_MAINLINE_FILES: &[&str] = &[
    "Blizzard_ItemUpgradeUI_Bootstrap.lua",
    "Mainline/Blizzard_ItemUpgradeUI.lua",
    "Mainline/Blizzard_ItemUpgradeUI.xml",
];

const ITEM_UPGRADE_DEPENDENCIES: &[&str] = &["Blizzard_Colors"];

const ITEM_UPGRADE_MAIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "HasReachedTargetUpgradeLevel",
    "UpdateIfTargetReached",
    "OnEvent",
    "OnConfirm",
    "UpdateUpgradeItemInfo",
    "ApplyTargetUpgradeLevel",
    "InitDropdown",
    "UpdateButtonAndArrowStates",
    "PopulatePreviewFrames",
    "GetSeasonSourceStringForCostItem",
    "CanAnyCostsBeDowngradedTo",
    "GetTrinketUpgradeText",
    "GetTotalCostEntry",
    "CalculateTotalCostTable",
    "GetUpgradeCostTables",
    "CheckUpgradeLevel",
    "GetUpgradeCostString",
    "CanUpgradeToLevel",
    "GetUpgradeInfo",
    "GetInsufficientCostInfo",
    "PlayUpgradedCelebration",
    "OnTooltipReappearTimerComplete",
    "OnTooltipReappearComplete",
];

const ITEM_UPGRADE_BUTTON_METHODS: &[&str] = &["OnClick", "GetDisabledTooltip", "GetUpgradeFrame"];

const ITEM_UPGRADE_PREVIEW_METHODS: &[&str] = &[
    "OnShow",
    "OnEnter",
    "OnLeave",
    "GeneratePreviewTooltip",
    "ApplyColorToGlowNiceSlice",
];

const ITEM_UPGRADE_SLOT_METHODS: &[&str] = &[
    "OnLoad",
    "GetItemUpgradeItemsCallBack",
    "OnEnter",
    "OnLeave",
    "OnClick",
    "OnDrag",
];

const ITEM_UPGRADE_ITEM_INFO_METHODS: &[&str] = &["Setup"];

const ITEM_UPGRADE_COST_QUANTITY_METHODS: &[&str] = &["OnEnter", "OnLeave"];

const ITEM_UPGRADE_COST_ICON_METHODS: &[&str] = &["OnEnter"];

const SECONDARY_MIXIN_NAMES: &[&str] = &[
    "ItemUpgradeButtonMixin",
    "ItemUpgradePreviewMixin",
    "ItemUpgradeSlotMixin",
    "ItemUpgradeItemInfoMixin",
    "ItemUpgradeCostQuantityMixin",
    "ItemUpgradeCostIconMixin",
];

const FREE_HELPER_FUNCTIONS: &[&str] = &["ItemUpgradeFrame_Show", "ItemUpgradeFrame_Hide"];

const VIRTUAL_TEMPLATE_NAMES: &[&str] = &[
    "ItemUpgradePreviewBigTextTemplate",
    "ItemUpgradeCostQuantityTemplate",
    "ItemUpgradeCostIconTemplate",
    "ItemUpgradeTooltipTemplate",
    "ItemUpgradePreviewTemplate",
];

fn load_item_upgrade_ui(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &item_upgrade_mainline_toc())
        .expect("Blizzard_ItemUpgradeUI_Mainline should load via explicit Rust loader call");
}

#[cfg(feature = "client-ptr")]
fn load_full_game_ui_with_item_upgrade_lod() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &item_upgrade_mainline_toc())
        .expect("Blizzard_ItemUpgradeUI_Mainline should load via explicit Rust loader call");

    env
}

#[test]
fn blizzard_item_upgrade_find_toc_resolves_mainline_variant() {
    let resolved =
        find_toc_file(&item_upgrade_dir()).expect("Blizzard_ItemUpgradeUI TOC should resolve");
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some(toc_layout::expected_toc_name("Blizzard_ItemUpgradeUI").as_str()),
        "find_toc_file resolves the active profile's ItemUpgradeUI TOC: the Retail cache ships \
         `_Mainline.toc` and `_Mists.toc` variants, while the pinned PTR cache folds both \
         flavors into the bare unified TOC"
    );
}

#[test]
fn blizzard_item_upgrade_toc_declares_load_on_demand_with_single_dependency() {
    let toc = TocFile::from_file(&item_upgrade_mainline_toc())
        .expect("Blizzard_ItemUpgradeUI_Mainline TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_ItemUpgradeUI declares `## LoadOnDemand: 1` — the addon stays unloaded until \
         the player opens an item-upgrader NPC and PLAYER_INTERACTION_MANAGER_FRAME_SHOW with the \
         ItemUpgrade interaction type fires the `ItemUpgrade_LoadUI` trigger registered by \
         Blizzard_UIPanels_Game/Shared/PlayerInteractionFrameManager.lua"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        ITEM_UPGRADE_DEPENDENCIES
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>(),
        "Blizzard_ItemUpgradeUI declares exactly one `## Dependencies:` entry — Blizzard_Colors. \
         ColorManager paints the cost-quantity text (red when the player can't afford, white \
         when affordable) and the upgrade-arrow glow color via \
         ItemUpgradePreviewMixin:ApplyColorToGlowNiceSlice. All other templates the XML inherits \
         — PortraitFrameTemplate, ResizeLayoutFrame, UIPanelButtonTemplate, TruncatedButtonTemplate, \
         DisabledTooltipButtonTemplate, ThinGoldEdgeTemplate, SharedTooltipTemplate, \
         CurrencyLayoutFrameIconTemplate, GameFontHighlight — come from auto-loaded foundational \
         addons (Blizzard_SharedXML / Blizzard_TokenUI / Blizzard_Fonts_Shared) without explicit \
         deps; the ItemButton intrinsic comes from Blizzard_ItemButton"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_ItemUpgradeUI declares zero `## OptionalDeps` — every helper / template / API \
         namespace it touches (C_ItemUpgrade, C_CurrencyInfo, C_Item, GameTooltip*, \
         CreateBaseTooltipInfo, TooltipDataLineType / TooltipDataItemBindingType enums) is \
         provided unconditionally by the foundational addon set"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_ItemUpgradeUI declares zero saved variables — upgrade state is server-driven \
         (C_ItemUpgrade.GetItemUpgradeItemInfo / GetItemHyperlink / GetUpgradeCostInfo) and \
         resets every time the panel opens; no client-side persistence"
    );
}

#[test]
fn blizzard_item_upgrade_toc_declares_mainline_game_type_restriction() {
    let toc = TocFile::from_file(&item_upgrade_mainline_toc())
        .expect("Blizzard_ItemUpgradeUI_Mainline TOC should parse");
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_ItemUpgradeUI_Mainline declares `## AllowLoadGameType: mainline` — \
         `is_game_type_restricted` (src/toc.rs:294-302) treats `mainline` and `standard` as the \
         unrestricted retail flavor and returns false. The addon ships across retail Midnight \
         alongside the Mists variant (which restricts to `mists` instead)"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_ItemUpgradeUI_Mainline omits `## AllowLoad:` — `allows_screen` (src/toc.rs:311) \
         defaults missing AllowLoad to Game-only. The upgrade panel is an in-game NPC interface \
         (item-upgrader vendors at every major hub) so glue-screen access is meaningless"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_ItemUpgradeUI_Mainline must NOT load on glue screens — missing AllowLoad \
             defaults to Game-only at src/toc.rs:311. (Screen tested: {screen:?})"
        );
    }

    let raw = std::fs::read_to_string(item_upgrade_mainline_toc())
        .expect("Blizzard_ItemUpgradeUI_Mainline TOC should read");
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the on-demand panel stays out of the \
         Game-screen auto-discovery sweep until the trigger fires"
    );
    assert!(
        raw.contains("## AllowLoadGameType: mainline"),
        "TOC must admit the `mainline` game type — the Retail `_Mainline.toc` restricts to \
         `mainline` alone while the unified PTR TOC lists `mainline, mists`"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` (different from `## AllowLoadGameType:`) — relying \
         on the Game-only default keeps the on-demand panel out of glue-screen sweeps"
    );
}

#[test]
fn blizzard_item_upgrade_toc_lists_lua_then_xml_under_mainline_subdir() {
    let toc = TocFile::from_file(&item_upgrade_mainline_toc())
        .expect("Blizzard_ItemUpgradeUI_Mainline TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        ITEM_UPGRADE_MAINLINE_FILES,
        "TOC body must list the bootstrap before the Mainline lua and XML files. The Mainline \
         source subdirectory remains isolated from the Mists variant"
    );
}

#[test]
fn blizzard_item_upgrade_directory_matches_pinned_profile_layout() {
    let mut entries: Vec<String> = std::fs::read_dir(item_upgrade_dir())
        .expect("Blizzard_ItemUpgradeUI directory should read")
        .map(|entry| {
            entry
                .expect("Blizzard_ItemUpgradeUI directory entry should read")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    entries.sort();
    let expected: Vec<&str> = if toc_layout::unified_toc_tree() {
        vec![
            "Blizzard_ItemUpgradeUI.toc",
            "Blizzard_ItemUpgradeUI_Bootstrap.lua",
            "Mainline",
        ]
    } else {
        vec![
            "Blizzard_ItemUpgradeUI_Bootstrap.lua",
            "Blizzard_ItemUpgradeUI_Mainline.toc",
            "Blizzard_ItemUpgradeUI_Mists.toc",
            "Mainline",
            "Mists",
        ]
    };
    assert_eq!(
        entries, expected,
        "Directory entries must match the active profile's pinned manifest. The Retail manifest \
         lists 7 files under Blizzard_ItemUpgradeUI (data/blizzard-ui-files/retail.txt:2296-2302): \
         the shared bootstrap, the `_Mainline` and `_Mists` flavor TOCs, and the Mainline and \
         Mists source directories — 5 top-level entries. The pinned PTR manifest lists 4 files \
         (data/blizzard-ui-files/ptr.txt:2536-2539) because PTR ships one unified \
         Blizzard_ItemUpgradeUI.toc whose `[Family]` placeholders select the Mainline files, and \
         no Mists source directory — 3 top-level entries"
    );
}

#[test]
fn blizzard_item_upgrade_excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_ItemUpgradeUI");
        assert!(
            !found,
            "Blizzard_ItemUpgradeUI must be filtered out of auto-discovery on every ScreenKind. \
             The TOC declares `## LoadOnDemand: 1`, and discover_blizzard_addons_for_screen \
             routes LoD addons into the lod_pool (src/loader/mod.rs:530-535) rather than the \
             eager `addons` set. (Screen tested: {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_ItemUpgradeUI")
                || message.contains("ItemUpgradeMixin")
                || message.contains("ItemUpgradeButtonMixin")
                || message.contains("ItemUpgradePreviewMixin")
                || message.contains("ItemUpgradeSlotMixin")
                || message.contains("ItemUpgradeItemInfoMixin")
                || message.contains("ItemUpgradeCostQuantityMixin")
                || message.contains("ItemUpgradeCostIconMixin")
                || message.contains("ItemUpgradeFrame")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ItemUpgradeUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_is_addon_loaded_via_explicit_load(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ItemUpgradeUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ItemUpgradeUI') must return true after the explicit \
         load_addon call — the loader strips the flavor suffix (`_Mainline`) when registering \
         the addon with the loaded-set so addons can probe each other by the canonical bare \
         name regardless of which flavor TOC actually loaded"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_main_mixin_publishes_with_twenty_six_methods(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    let kind: String = env
        .eval("return type(ItemUpgradeMixin)")
        .expect("ItemUpgradeMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "ItemUpgradeMixin must publish at `_G` as a table — \
         Mainline/Blizzard_ItemUpgradeUI.lua:17 creates the empty table at file scope before \
         binding 26 methods to it. The master mixin owns the entire upgrade-frame lifecycle \
         (OnLoad / OnShow / OnHide / OnEvent / OnConfirm), the upgrade-level state machine \
         (HasReachedTargetUpgradeLevel / UpdateIfTargetReached / ApplyTargetUpgradeLevel / \
         CanUpgradeToLevel / CheckUpgradeLevel / GetUpgradeInfo), the dropdown / button / arrow \
         UI plumbing (InitDropdown / UpdateButtonAndArrowStates / UpdateUpgradeItemInfo), the \
         cost calculation pipeline (GetTotalCostEntry / CalculateTotalCostTable / \
         GetUpgradeCostTables / GetUpgradeCostString / GetSeasonSourceStringForCostItem / \
         CanAnyCostsBeDowngradedTo / GetInsufficientCostInfo), the trinket-special-case helper \
         (GetTrinketUpgradeText), the preview-pane wiring (PopulatePreviewFrames), and the \
         upgrade-celebration animation (PlayUpgradedCelebration / OnTooltipReappearTimerComplete \
         / OnTooltipReappearComplete)"
    );

    for method in ITEM_UPGRADE_MAIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(ItemUpgradeMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("ItemUpgradeMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemUpgradeMixin.{method} must publish as a function. Missing this method implies \
             the .lua never executed via the TOC body order or the mixin table was overwritten \
             before the method bound"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_publishes_six_secondary_mixins(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for mixin in SECONDARY_MIXIN_NAMES {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} type probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at `_G` as a table after Blizzard_ItemUpgradeUI loads — the \
             .lua creates 6 secondary mixin tables at file scope (lines 680, 739, 903, 980, \
             1027, 1087) before binding methods to them. ItemUpgradeButtonMixin drives the \
             UPGRADE button (UIPanelButtonTemplate + TruncatedButtonTemplate + \
             DisabledTooltipButtonTemplate); ItemUpgradePreviewMixin drives the embedded \
             tooltip-style left/right preview frames inheriting SharedTooltipTemplate; \
             ItemUpgradeSlotMixin drives the ItemButton drop target; \
             ItemUpgradeItemInfoMixin drives the ResizeLayoutFrame item-info pane; \
             ItemUpgradeCostQuantityMixin drives each cost-row FontString hover surface; \
             ItemUpgradeCostIconMixin drives each cost-row icon hover surface inheriting \
             CurrencyLayoutFrameIconTemplate"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_button_mixin_carries_three_methods(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for method in ITEM_UPGRADE_BUTTON_METHODS {
        let kind: String = env
            .eval(&format!("return type(ItemUpgradeButtonMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("ItemUpgradeButtonMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemUpgradeButtonMixin.{method} must publish as a function — the upgrade button \
             mixin owns 3 methods: OnClick fires C_ItemUpgrade.UpgradeItem after the static-popup \
             confirmation flow, GetDisabledTooltip surfaces the insufficient-cost / \
             max-level-reached / not-tradeable error text via the DisabledTooltipButtonTemplate \
             tooltip, GetUpgradeFrame walks back up to the parent ItemUpgradeFrame for state \
             access"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_preview_mixin_carries_five_methods(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for method in ITEM_UPGRADE_PREVIEW_METHODS {
        let kind: String = env
            .eval(&format!("return type(ItemUpgradePreviewMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("ItemUpgradePreviewMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemUpgradePreviewMixin.{method} must publish as a function — the preview-tooltip \
             mixin owns 5 methods: OnShow / OnEnter / OnLeave drive the hover behavior on the \
             embedded LeftItemPreviewFrame / RightItemPreviewFrame / ItemHoverPreviewFrame \
             tooltips, GeneratePreviewTooltip builds the synthetic upgraded / current item \
             tooltip via ProcessInfo / GetItemByID with synthetic stat bonuses applied, \
             ApplyColorToGlowNiceSlice repaints the surrounding glow nine-slice color via \
             ColorManager based on quality"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_slot_mixin_carries_six_methods(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for method in ITEM_UPGRADE_SLOT_METHODS {
        let kind: String = env
            .eval(&format!("return type(ItemUpgradeSlotMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("ItemUpgradeSlotMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemUpgradeSlotMixin.{method} must publish as a function — the upgrade-slot mixin \
             owns 6 methods: OnLoad seeds the EquipmentFlyout-driven valid-item callback, \
             GetItemUpgradeItemsCallBack walks the player's bag/equipped to populate the flyout \
             list, OnEnter / OnLeave drive the GameTooltip with the slotted item, OnClick routes \
             left-click to flyout open and right-click to clear, OnDrag handles the cursor item \
             pickup/drop"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_item_info_mixin_carries_one_method(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for method in ITEM_UPGRADE_ITEM_INFO_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(ItemUpgradeItemInfoMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("ItemUpgradeItemInfoMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemUpgradeItemInfoMixin.{method} must publish as a function — the item-info-pane \
             mixin owns exactly 1 method: Setup populates the ResizeLayoutFrame with the item \
             name (quality-colored), level, and bonus / corruption / catalyst metadata. The \
             frame layouts itself via ResizeLayoutFrame's automatic measurement"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_cost_quantity_mixin_carries_two_methods(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for method in ITEM_UPGRADE_COST_QUANTITY_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(ItemUpgradeCostQuantityMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("ItemUpgradeCostQuantityMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "ItemUpgradeCostQuantityMixin.{method} must publish as a function — the \
             cost-quantity FontString mixin owns 2 methods: OnEnter shows a GameTooltip with the \
             player-current / required count plus the season-source string \
             (\"Earned in The War Within Season 3\"), OnLeave hides it"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_cost_icon_mixin_carries_one_method(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for method in ITEM_UPGRADE_COST_ICON_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(ItemUpgradeCostIconMixin['{method}'])"
            ))
            .unwrap_or_else(|err| panic!("ItemUpgradeCostIconMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemUpgradeCostIconMixin.{method} must publish as a function — the cost-icon mixin \
             owns exactly 1 method: OnEnter dispatches to SetCurrencyByID for currency costs or \
             ProcessInfo with CreateBaseTooltipInfo('GetItemByID', itemID) for item costs, \
             excluding the SellPrice and ItemBinding tooltip lines and appending the \
             costSourceString as a normal-color line"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_publishes_two_free_helper_functions(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for helper in FREE_HELPER_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({helper})"))
            .unwrap_or_else(|err| panic!("{helper} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{helper} must publish at `_G` as a function — Mainline/Blizzard_ItemUpgradeUI.lua:3 \
             and :10 define the 2 free-helper globals: ItemUpgradeFrame_Show calls \
             ShowUIPanel(ItemUpgradeFrame) and falls back to C_ItemUpgrade.CloseItemUpgrade if \
             the ShowUIPanel handshake fails (the ItemUpgrade UI cannot open if a non-pushable \
             panel already occupies the left slot — pushable=0); ItemUpgradeFrame_Hide simply \
             calls HideUIPanel(ItemUpgradeFrame). These are the entry points the \
             PlayerInteractionFrameManager registers via `loadFunc` for the ItemUpgrade \
             interaction type"
        );
    }
}
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_item_upgrade_publishes_hide_wrapper_from_pinned_bootstrap() {
    let env = load_full_game_ui_with_item_upgrade_lod();

    let (kind, source): (String, String) = env
        .eval(
            r#"
            assert(type(HideItemUpgradeFrame) == 'function',
                'pinned PTR bootstrap defines HideItemUpgradeFrame')
            local info = debug.getinfo(HideItemUpgradeFrame, 'S')
            return type(HideItemUpgradeFrame), (info and info.source or ''):gsub('\\', '/')
            "#,
        )
        .expect("item upgrade hide wrapper provenance should be queryable");
    assert_eq!(kind, "function");
    assert!(
        source.ends_with("Blizzard_ItemUpgradeUI/Blizzard_ItemUpgradeUI_Bootstrap.lua"),
        "HideItemUpgradeFrame must come from the pinned PTR bootstrap, not from a simulator stub: \
         data/blizzard-ui-files/ptr.txt:2537 pins \
         Blizzard_ItemUpgradeUI/Blizzard_ItemUpgradeUI_Bootstrap.lua, whose line 13 defines \
         `function HideItemUpgradeFrame()` guarding on C_AddOns.IsAddOnLoaded and delegating to \
         ItemUpgradeFrame_Hide. Source reported: {source}"
    );

    env.exec("HideItemUpgradeFrame()")
        .expect("pinned wrapper must run against the loaded addon without a Lua error");
    let shown: bool = env
        .eval("return ItemUpgradeFrame:IsShown()")
        .expect("ItemUpgradeFrame:IsShown() probe should succeed");
    assert!(
        !shown,
        "HideItemUpgradeFrame must delegate to ItemUpgradeFrame_Hide -> HideUIPanel, leaving the \
         never-shown ItemUpgradeFrame hidden"
    );
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_named_frame_publishes_with_portrait_template_chain(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    let kind: String = env
        .eval("return type(ItemUpgradeFrame)")
        .expect("ItemUpgradeFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "ItemUpgradeFrame must publish at `_G` as a table — declared at \
         Mainline/Blizzard_ItemUpgradeUI.xml:144 with `name=\"ItemUpgradeFrame\"` \
         `inherits=\"PortraitFrameTemplate\"` `mixin=\"ItemUpgradeMixin\"` `toplevel=\"true\"` \
         `parent=\"UIParent\"` `enableMouse=\"true\"` `hidden=\"true\"`. The PortraitFrameTemplate \
         chain provides the close button, title, and portrait container (which the OnLoad sets \
         to Interface\\Icons\\UI_ItemUpgrade); hidden=\"true\" keeps it invisible until \
         ItemUpgradeFrame_Show -> ShowUIPanel fires"
    );

    let name: String = env
        .eval("return ItemUpgradeFrame:GetName()")
        .expect("ItemUpgradeFrame:GetName() probe should succeed");
    assert_eq!(name, "ItemUpgradeFrame");

    let visible: bool = env
        .eval("return ItemUpgradeFrame:IsShown()")
        .expect("ItemUpgradeFrame:IsShown() probe should succeed");
    assert!(
        !visible,
        "ItemUpgradeFrame must report IsShown=false on load — XML declares `hidden=\"true\"` and \
         the frame stays invisible until ItemUpgradeFrame_Show fires"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_registers_ui_panel_window_entry(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    let area: String = env
        .eval("return tostring(UIPanelWindows['ItemUpgradeFrame'].area)")
        .expect("UIPanelWindows entry probe should succeed");
    assert_eq!(
        area, "left",
        "UIPanelWindows['ItemUpgradeFrame'].area must equal \"left\" — \
         Mainline/Blizzard_ItemUpgradeUI.lua:1 registers the panel as a left-docked panel via \
         `UIPanelWindows[\"ItemUpgradeFrame\"] = {{ area = \"left\", pushable = 0 }}` so \
         ShowUIPanel docks it at the standard CharacterFrame slot"
    );

    let pushable: f64 = env
        .eval("return UIPanelWindows['ItemUpgradeFrame'].pushable")
        .expect("pushable probe should succeed");
    assert_eq!(
        pushable, 0.0,
        "UIPanelWindows['ItemUpgradeFrame'].pushable must equal 0 — pushable=0 means the upgrade \
         frame cannot be pushed aside by any other panel; opening any left-docked panel during \
         an upgrade session closes the upgrade frame outright (the upgrade session is cancelled \
         via ItemUpgradeFrame_Show's IsShown() guard that calls C_ItemUpgrade.CloseItemUpgrade)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_upgrade_virtual_templates_stay_nil_at_global_scope(env: &WowLuaEnv) {
    load_item_upgrade_ui(env);

    for template in VIRTUAL_TEMPLATE_NAMES {
        let kind: String = env
            .eval(&format!("return type(_G['{template}'])"))
            .unwrap_or_else(|err| panic!("{template} probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G['{template}'] must be nil — virtual templates only register in the template \
             registry, NOT at `_G`. Blizzard_ItemUpgradeUI ships 5 virtual templates: \
             ItemUpgradePreviewBigTextTemplate (FontString, the headline text in preview \
             tooltips), ItemUpgradeCostQuantityTemplate (FontString, hoverable cost-amount \
             label wired to ItemUpgradeCostQuantityMixin), ItemUpgradeCostIconTemplate (Frame \
             inheriting CurrencyLayoutFrameIconTemplate wired to ItemUpgradeCostIconMixin), \
             ItemUpgradeTooltipTemplate (GameTooltip inheriting SharedTooltipTemplate wired to \
             ItemUpgradePreviewMixin — base class for the embedded preview tooltips), \
             ItemUpgradePreviewTemplate (GameTooltip inheriting ItemUpgradeTooltipTemplate at \
             frameStrata=MEDIUM — used by LeftItemPreviewFrame / RightItemPreviewFrame). Each \
             is consumed only via XML `inherits=\"...\"` resolution"
        );
    }
}
}
