#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn item_interaction_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ItemInteractionUI")
}

fn item_interaction_toc() -> PathBuf {
    item_interaction_dir().join("Blizzard_ItemInteractionUI.toc")
}

const ITEM_INTERACTION_FILES: &[&str] = &[
    "Blizzard_ItemInteractionUI_Bootstrap.lua",
    "Blizzard_ItemInteractionUI.xml",
];

const ITEM_INTERACTION_DEPENDENCIES: &[&str] = &["Blizzard_Colors"];

const ITEM_INTERACTION_MAIN_METHODS: &[&str] = &[
    "GetItemLocation",
    "GetInteractionType",
    "GetCost",
    "HasExtendedCurrencyCost",
    "HasCost",
    "CostsGold",
    "CostsCurrency",
    "UsesCharges",
    "OnEvent",
    "OnShow",
    "OnHide",
    "LoadInteractionFrameData",
    "SetupFrameSpecificData",
    "SetupChargeCurrency",
    "UpdateDescription",
    "UpdateDescriptionColor",
    "GetDescriptionColor",
    "UpdateCostFrame",
    "UpdateMoney",
    "UpdateCurrency",
    "UpdateCharges",
    "GetRechargeMessage",
    "GetButtonTooltip",
    "GetConfirmationDescription",
    "GetConfirmationInfo",
    "GetChargeConfirmationText",
    "InteractWithItem",
    "CompleteItemInteraction",
    "UpdateActionButtonState",
    "SetItemConversionExtendedCurrencyCost",
    "SetInteractionItem",
    "SetupEquipmentFlyout",
    "SetDynamicFlyoutSettings",
    "GetValidItemInteractionItemsCallback",
    "ShowFlyout",
    "SetInputItemSlotTooltip",
];

const ITEM_SLOT_METHODS: &[&str] = &[
    "OnLoad",
    "RefreshIcon",
    "RefreshTooltip",
    "OnClick",
    "OnDragStart",
    "OnReceiveDrag",
    "OnEnter",
    "OnLeave",
];

const ACTION_BUTTON_METHODS: &[&str] = &["OnEnter", "OnLeave", "OnClick"];

const CONVERSION_FRAME_METHODS: &[&str] = &[
    "OnLoad",
    "OnHide",
    "SetupConversionCelebration",
    "PlayConversionCelebration",
    "StopConversionCelebration",
    "UpdateArrow",
];

const CONVERSION_INPUT_METHODS: &[&str] = &[
    "OnLoad",
    "RefreshIcon",
    "RefreshTooltip",
    "OnClick",
    "OnDragStart",
    "OnReceiveDrag",
    "OnEnter",
    "OnLeave",
];

const CONVERSION_OUTPUT_METHODS: &[&str] = &["RefreshIcon", "OnEvent", "OnEnter", "OnLeave"];

const SECONDARY_MIXIN_NAMES: &[&str] = &[
    "ItemInteractionItemSlotMixin",
    "ItemInteractionActionButtonMixin",
    "ItemInteractionItemConversionFrameMixin",
    "ItemInteractionItemConversionInputSlotMixin",
    "ItemInteractionItemConversionOutputSlotMixin",
];

const STATIC_POPUP_KEYS: &[&str] = &[
    "ITEM_INTERACTION_CONFIRMATION",
    "ITEM_INTERACTION_CONFIRMATION_DELAYED",
    "ITEM_INTERACTION_CONFIRMATION_DELAYED_WITH_CHARGE_INFO",
];

fn load_item_interaction_ui(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &item_interaction_toc())
        .expect("Blizzard_ItemInteractionUI should load via explicit Rust loader call");
}

#[test]
fn blizzard_item_interaction_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&item_interaction_dir())
        .expect("Blizzard_ItemInteractionUI TOC should resolve");
    assert_eq!(
        resolved,
        item_interaction_toc(),
        "Blizzard_ItemInteractionUI ships exactly one bare TOC — the LoadOnDemand item-interaction \
         module resolves via `find_toc_file` fallthrough after the `_Mainline.toc` lookup misses"
    );
}

#[test]
fn blizzard_item_interaction_toc_declares_load_on_demand_with_single_dependency() {
    let toc = TocFile::from_file(&item_interaction_toc())
        .expect("Blizzard_ItemInteractionUI TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_ItemInteractionUI declares `## LoadOnDemand: 1` — the addon stays unloaded \
         until the PlayerInteractionFrameManager fires `ItemInteraction_LoadUI` in response to a \
         PLAYER_INTERACTION_MANAGER_FRAME_SHOW event with the ItemInteraction interaction type \
         (Blizzard_UIPanels_Game/Shared/PlayerInteractionFrameManager.lua wires the trigger)"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        ITEM_INTERACTION_DEPENDENCIES
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>(),
        "Blizzard_ItemInteractionUI declares exactly one `## Dependencies:` entry — \
         Blizzard_Colors. ColorManager is consumed by GetDescriptionColor / UpdateDescriptionColor \
         to paint the description text (gold for affordable interactions, red when the player \
         lacks the cost). All other templates the XML inherits — PortraitFrameTemplate, \
         InsetFrameTemplate, CurrencyDisplayGroupTemplate, BackpackTokenTemplate, MagicButtonTemplate, \
         SmallMoneyFrameTemplate, ThinGoldEdgeTemplate, _UI-Frame-InnerBotTile, _UI-Frame-BtnBotTile, \
         and the ItemButton intrinsic — come from auto-loaded foundational addons \
         (Blizzard_SharedXML / Blizzard_TokenUI / Blizzard_ItemButton) without explicit deps"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_ItemInteractionUI declares zero `## OptionalDeps` — every helper / template / \
         API namespace it touches is provided unconditionally by the foundational addon set"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_ItemInteractionUI declares zero saved variables — interaction state is server- \
         driven (C_ItemInteraction.GetItemInteractionInfo / GetChargeInfo) and resets every time \
         the panel opens; no client-side persistence"
    );
}

#[test]
fn blizzard_item_interaction_toc_omits_allow_load_metadata() {
    let toc = TocFile::from_file(&item_interaction_toc())
        .expect("Blizzard_ItemInteractionUI TOC should parse");
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_ItemInteractionUI omits `## AllowLoadGameType:` — `is_game_type_restricted` \
         (src/toc.rs:294-302) returns false when the metadata key is missing, so retail clients \
         keep the addon eligible. Item interaction surfaces (CleanseCorruption, RunecarverScrapping, \
         ItemConversion) ship as part of the mainline retail experience"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_ItemInteractionUI omits `## AllowLoad:` — `allows_screen` (src/toc.rs:311) \
         defaults missing AllowLoad to Game-only. The interaction panel is an in-game NPC \
         interface (rune carver, corruption cleanser, item converter) so glue-screen access is \
         meaningless"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Blizzard_ItemInteractionUI must NOT load on glue screens — missing AllowLoad \
             defaults to Game-only at src/toc.rs:311. (Screen tested: {screen:?})"
        );
    }

    let raw = std::fs::read_to_string(item_interaction_toc())
        .expect("Blizzard_ItemInteractionUI TOC should read");
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the on-demand panel stays out of the \
         Game-screen auto-discovery sweep until the trigger fires"
    );
    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare any `## AllowLoad*` metadata — relying on the Game-only default \
         keeps the on-demand panel out of glue-screen sweeps without an explicit allow"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:` — the addon is LoadOnDemand and its state is \
         fully server-driven; there is no on/off toggle to seed"
    );
}

#[test]
fn blizzard_item_interaction_toc_lists_bootstrap_then_xml() {
    let toc = TocFile::from_file(&item_interaction_toc())
        .expect("Blizzard_ItemInteractionUI TOC should parse");
    assert_eq!(
        toc.files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        ITEM_INTERACTION_FILES,
        "TOC body must list the bootstrap before Blizzard_ItemInteractionUI.xml. The lua sibling \
         is loaded by the XML's `<Script file=\"Blizzard_ItemInteractionUI.lua\"/>` directive \
         before frame elements resolve its mixins"
    );
}

#[test]
fn blizzard_item_interaction_directory_holds_four_entries() {
    let entries = std::fs::read_dir(item_interaction_dir())
        .expect("Blizzard_ItemInteractionUI directory should read")
        .count();
    assert_eq!(
        entries, 4,
        "Directory must hold exactly 4 entries: the TOC, bootstrap, lua, and XML files"
    );
}

#[test]
fn blizzard_item_interaction_excluded_from_every_screen_auto_discovery() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_ItemInteractionUI");
        assert!(
            !found,
            "Blizzard_ItemInteractionUI must be filtered out of auto-discovery on every \
             ScreenKind. The TOC declares `## LoadOnDemand: 1`, and \
             discover_blizzard_addons_for_screen routes LoD addons into the lod_pool (src/loader/mod.rs:530-535) \
             rather than the eager `addons` set. (Screen tested: {screen:?})"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_item_interaction_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_ItemInteractionUI")
                || message.contains("ItemInteractionMixin")
                || message.contains("ItemInteractionItemSlotMixin")
                || message.contains("ItemInteractionActionButtonMixin")
                || message.contains("ItemInteractionItemConversionFrameMixin")
                || message.contains("ItemInteractionItemConversionInputSlotMixin")
                || message.contains("ItemInteractionItemConversionOutputSlotMixin")
                || message.contains("ItemInteractionFrame")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ItemInteractionUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_is_addon_loaded_via_explicit_load(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ItemInteractionUI')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ItemInteractionUI') must return true after the \
         explicit load_addon call — confirms the loader registers the addon with the loaded-set \
         even though the auto-discovery sweep skipped it (LoadOnDemand)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_main_mixin_publishes_with_thirty_six_methods(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    let kind: String = env
        .eval("return type(ItemInteractionMixin)")
        .expect("ItemInteractionMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "ItemInteractionMixin must publish at `_G` as a table — \
         Blizzard_ItemInteractionUI.lua:127 creates the empty table at file scope before binding \
         36 methods to it"
    );

    for method in ITEM_INTERACTION_MAIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(ItemInteractionMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("ItemInteractionMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemInteractionMixin.{method} must publish as a function — the master mixin owns 36 \
             methods (interaction-type / cost / charge accessors, OnEvent / OnShow / OnHide \
             lifecycle, LoadInteractionFrameData / SetupFrameSpecificData / SetupChargeCurrency \
             setup pipeline, UpdateDescription / UpdateMoney / UpdateCurrency / UpdateCharges / \
             UpdateActionButtonState refresh helpers, InteractWithItem / CompleteItemInteraction \
             confirmation flow, SetInteractionItem item-slot wiring, SetupEquipmentFlyout / \
             SetDynamicFlyoutSettings / ShowFlyout / SetInputItemSlotTooltip flyout plumbing). \
             Missing this method implies the .lua never executed via `<Script file=...>` or the \
             mixin table was overwritten"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_publishes_five_secondary_mixins(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    for mixin in SECONDARY_MIXIN_NAMES {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} type probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must publish at `_G` as a table after Blizzard_ItemInteractionUI loads — \
             the .lua creates 5 secondary mixin tables at file scope (lines 746, 810, 858, 910, \
             984) before binding methods to them. Each mixin is consumed by an XML element via \
             `mixin=\"...\"`: ItemSlot drives the main item drop target, ActionButton drives the \
             confirm button (MagicButtonTemplate), and the three ItemConversion* mixins drive the \
             input → arrow → output conversion sub-flow used by ItemConversion-type interactions"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_item_slot_mixin_carries_eight_lifecycle_methods(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    for method in ITEM_SLOT_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(ItemInteractionItemSlotMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("ItemInteractionItemSlotMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "ItemInteractionItemSlotMixin.{method} must publish as a function — the slot mixin \
             owns 8 methods: OnLoad initializes the registered slot, RefreshIcon paints the item \
             texture and IconBorder color, RefreshTooltip drives the description summary, OnClick \
             routes left/right-click to ClearItem / pickup, OnDragStart starts the cursor pickup, \
             OnReceiveDrag accepts the cursor item, OnEnter / OnLeave drive the GameTooltip"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_action_button_mixin_carries_three_methods(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    for method in ACTION_BUTTON_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(ItemInteractionActionButtonMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("ItemInteractionActionButtonMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "ItemInteractionActionButtonMixin.{method} must publish as a function — the confirm \
             button mixin owns 3 methods: OnEnter drives the disabled-reason tooltip, OnLeave \
             hides it, OnClick routes to ItemInteractionFrame:InteractWithItem() which kicks the \
             confirmation StaticPopup flow"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_conversion_frame_mixin_carries_six_methods(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    for method in CONVERSION_FRAME_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(ItemInteractionItemConversionFrameMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("ItemInteractionItemConversionFrameMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "ItemInteractionItemConversionFrameMixin.{method} must publish as a function — the \
             conversion-pane mixin owns 6 methods: OnLoad / OnHide lifecycle, \
             SetupConversionCelebration wires the AnimationHolder.ConversionFlash anim group, \
             PlayConversionCelebration / StopConversionCelebration drive the celebration burst, \
             UpdateArrow toggles the AnimatedArrow / DimArrow swap when an input item is valid"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_conversion_input_slot_mixin_carries_eight_methods(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    for method in CONVERSION_INPUT_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(ItemInteractionItemConversionInputSlotMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("ItemInteractionItemConversionInputSlotMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "ItemInteractionItemConversionInputSlotMixin.{method} must publish as a function — \
             the input-slot mixin mirrors ItemInteractionItemSlotMixin's 8-method shape (OnLoad / \
             RefreshIcon / RefreshTooltip / OnClick / OnDragStart / OnReceiveDrag / OnEnter / \
             OnLeave) but routes through the conversion-specific PulseEmptySlotGlow animation \
             and InputSlot_Flash overlays during the celebration"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_conversion_output_slot_mixin_carries_four_methods(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    for method in CONVERSION_OUTPUT_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(ItemInteractionItemConversionOutputSlotMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("ItemInteractionItemConversionOutputSlotMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "ItemInteractionItemConversionOutputSlotMixin.{method} must publish as a function — \
             the output-slot mixin owns 4 methods (RefreshIcon / OnEvent / OnEnter / OnLeave). \
             OnEvent listens for ITEM_INTERACTION_ITEM_CONVERSION_RESULT to paint the converted \
             item, RefreshIcon writes the icon, OnEnter / OnLeave drive the result tooltip"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_named_frame_publishes_with_portrait_template_chain(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    let kind: String = env
        .eval("return type(ItemInteractionFrame)")
        .expect("ItemInteractionFrame probe should succeed");
    assert_eq!(
        kind, "table",
        "ItemInteractionFrame must publish at `_G` as a table — declared at \
         Blizzard_ItemInteractionUI.xml:5 with `name=\"ItemInteractionFrame\"` \
         `inherits=\"PortraitFrameTemplate\"` `toplevel=\"true\"` `parent=\"UIParent\"` \
         `enableMouse=\"true\"` `mixin=\"ItemInteractionMixin\"` `hidden=\"true\"`. The \
         PortraitFrameTemplate chain provides the close button, title, and portrait container; \
         hidden=\"true\" keeps it invisible until ItemInteraction_LoadUI -> ShowUIPanel fires"
    );

    let name: String = env
        .eval("return ItemInteractionFrame:GetName()")
        .expect("ItemInteractionFrame:GetName() probe should succeed");
    assert_eq!(name, "ItemInteractionFrame");

    let visible: bool = env
        .eval("return ItemInteractionFrame:IsShown()")
        .expect("ItemInteractionFrame:IsShown() probe should succeed");
    assert!(
        !visible,
        "ItemInteractionFrame must report IsShown=false on load — XML declares `hidden=\"true\"` \
         and the frame stays invisible until C_PlayerInteractionManager fires \
         PLAYER_INTERACTION_MANAGER_FRAME_SHOW for an ItemInteraction-type interaction"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_registers_ui_panel_window_entry(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    let area: String = env
        .eval("return tostring(UIPanelWindows['ItemInteractionFrame'].area)")
        .expect("UIPanelWindows entry probe should succeed");
    assert_eq!(
        area, "left",
        "UIPanelWindows['ItemInteractionFrame'].area must equal \"left\" — \
         Blizzard_ItemInteractionUI.lua:1 registers the panel as a left-docked panel via \
         `UIPanelWindows[\"ItemInteractionFrame\"] = {{area = \"left\", pushable = 3, ...}}` so \
         ShowUIPanel docks it at the standard CharacterFrame slot"
    );

    let pushable: f64 = env
        .eval("return UIPanelWindows['ItemInteractionFrame'].pushable")
        .expect("pushable probe should succeed");
    assert_eq!(
        pushable, 3.0,
        "UIPanelWindows['ItemInteractionFrame'].pushable must equal 3 — interaction surfaces \
         coexist with the bag / character / quest panels at priority 3, so opening the frame \
         pushes lower-priority panels aside"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_interaction_registers_three_static_popup_dialogs(env: &WowLuaEnv) {
    load_item_interaction_ui(env);

    for key in STATIC_POPUP_KEYS {
        let kind: String = env
            .eval(&format!("return type(StaticPopupDialogs['{key}'])"))
            .unwrap_or_else(|err| panic!("StaticPopupDialogs.{key} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "StaticPopupDialogs['{key}'] must publish as a table — \
             Blizzard_ItemInteractionUI.lua:9-83 registers 3 confirmation dialogs (Confirmation, \
             ConfirmationDelayed, ConfirmationDelayedWithChargeInfo). The base Confirmation drives \
             instant interactions; the Delayed variant adds a 5-second acceptDelay so the player \
             must read before clicking; the WithChargeInfo variant additionally surfaces the \
             charge-count subtext"
        );
    }
}
}
