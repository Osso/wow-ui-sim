#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be populated for ItemButton load tests")
}

fn item_button_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ItemButton")
}

fn item_button_toc() -> PathBuf {
    find_toc_file(&item_button_dir()).expect("Blizzard_ItemButton TOC should resolve")
}

const ITEM_BUTTON_FILES: &[&str] = &[
    "Shared/ItemButtonTemplate.lua",
    "Shared/ItemButtonTemplate.xml",
    "Mainline/ItemButtonTemplate.xml",
];

// Pinned PTR 12.1.7 ships one unified TOC that also lists the Mainline Lua
// sibling and gates the family XML on `mainline`.
const PTR_ITEM_BUTTON_FILES: &[&str] = &[
    "Shared/ItemButtonTemplate.lua",
    "Shared/ItemButtonTemplate.xml",
    "Mainline/ItemButtonTemplate.lua",
    "Mainline/ItemButtonTemplate.xml",
];

const ITEM_BUTTON_DEPENDENCIES: &[&str] = &[
    "Blizzard_Fonts_Shared",
    "Blizzard_SharedXML",
    "Blizzard_Colors",
];

// Blizzard_Fonts_Shared is not a dependency of the unified PTR TOC.
const PTR_ITEM_BUTTON_DEPENDENCIES: &[&str] = &["Blizzard_SharedXML", "Blizzard_Colors"];

fn item_button_files() -> &'static [&'static str] {
    if toc_layout::unified_toc_tree() {
        PTR_ITEM_BUTTON_FILES
    } else {
        ITEM_BUTTON_FILES
    }
}

fn item_button_dependencies() -> &'static [&'static str] {
    if toc_layout::unified_toc_tree() {
        PTR_ITEM_BUTTON_DEPENDENCIES
    } else {
        ITEM_BUTTON_DEPENDENCIES
    }
}

const ITEM_BUTTON_VIRTUAL_TEMPLATES: &[&str] = &[
    "CircularItemButtonTemplate",
    "CircularGiantItemButtonTemplate",
    "GiantItemButtonTemplate",
    "SimplePopupButtonTemplate",
    "LargeItemButtonTemplate",
    "SmallItemButtonTemplate",
    "EnchantingItemButtonAnimTemplate",
];

const ITEM_BUTTON_SHARED_METHODS: &[&str] = &[
    "OnItemContextChanged",
    "PostOnShow",
    "PostOnHide",
    "PostOnEvent",
    "SetMatchesSearch",
    "GetMatchesSearch",
    "UpdateItemContextMatching",
    "UpdateCraftedProfessionsQualityShown",
    "GetItemContextOverlayMode",
    "UpdateItemContextOverlay",
    "UpdateItemContextOverlayTextures",
    "Reset",
    "SetItemSource",
    "SetItemLocation",
    "SetItem",
    "SetItemInternal",
    "GetItemInfo",
    "GetItemID",
    "GetItem",
    "GetItemLink",
    "GetItemLocation",
    "SetItemButtonCount",
    "SetItemButtonAnchorPoint",
    "SetItemButtonScale",
    "GetItemButtonCount",
    "SetAlpha",
    "SetBagID",
    "GetBagID",
    "GetSlotAndBagID",
    "OnUpdateItemContextMatching",
    "RegisterBagButtonUpdateItemContextMatching",
    "GetItemButtonIconTexture",
];

const ITEM_BUTTON_MAINLINE_METHOD_OVERRIDES: &[&str] = &[
    "SetItemButtonTexture",
    "SetItemButtonTextureVertexColor",
    "SetItemButtonQuality",
    "SetItemButtonBorderVertexColor",
    "GetItemButtonBackgroundTexture",
];

const ITEM_BUTTON_MAINLINE_GLOBALS: &[&str] = &[
    "GetFormattedItemQuantity",
    "SetItemButtonCount",
    "GetItemButtonCount",
    "SetItemButtonStock",
    "GetItemButtonBackgroundTexture",
    "SetItemButtonTexture",
    "SetItemButtonTextureVertexColor",
    "SetItemButtonBorderVertexColor",
    "SetItemButtonDesaturated",
    "SetItemButtonNormalTextureVertexColor",
    "SetItemButtonNameFrameVertexColor",
    "SetItemButtonSlotVertexColor",
    "ClearItemButtonOverlay",
    "SetItemButtonBorder_Base",
    "SetItemButtonBorder",
    "SetItemButtonQuality",
    "SetItemButtonOverlay",
    "SetItemCraftingQualityOverlayOverride",
    "SetItemCraftingQualityOverlay",
    "ClearItemCraftingQualityOverlay",
    "SetItemButtonReagentCount",
    "HandleModifiedItemClick",
];

const ENCHANTING_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "SetItemLocationCallback",
    "GetItemLocation",
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
fn blizzard_item_button_find_toc_resolves_mainline_variant() {
    let resolved =
        find_toc_file(&item_button_dir()).expect("Blizzard_ItemButton TOC should resolve");
    assert_eq!(
        resolved.file_name().and_then(|name| name.to_str()),
        Some(toc_layout::expected_toc_name("Blizzard_ItemButton").as_str()),
        "find_toc_file must resolve the active profile's ItemButton TOC: the Retail cache \
         ships only `_Mainline.toc`, while the pinned PTR cache folds the Mainline flavor \
         into the bare unified TOC"
    );
}

#[test]
fn blizzard_item_button_toc_declares_three_dependencies_and_default_enabled() {
    let toc = TocFile::from_file(&item_button_toc()).expect("Blizzard_ItemButton TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_ItemButton omits `## LoadOnDemand:` — it is a foundational widget addon that \
         every item-bearing UI (bags, vendor, mail, auction, trade, void storage, professions) \
         depends on, so it auto-loads on the Game-screen sweep with `## DefaultState: enabled`"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        item_button_dependencies()
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>(),
        "Blizzard_ItemButton declares its `## Dep` entries in published order — one per \
         collaborator (ItemButtonUtil / FrameUtil / EventRegistry, NumberFontNormal and \
         ColorManager). The Retail TOC also lists Blizzard_Fonts_Shared; the unified PTR \
         TOC does not"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Blizzard_ItemButton declares zero `## OptionalDeps` — every helper / namespace it \
         touches lives in the three hard deps; nothing is conditional"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_ItemButton declares zero saved variables — it is a pure template / mixin / \
         helper module with no per-character state to persist"
    );
}

#[test]
fn blizzard_item_button_toc_is_mainline_only_and_loads_on_both_screens() {
    let toc = TocFile::from_file(&item_button_toc()).expect("Blizzard_ItemButton TOC should parse");
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_ItemButton declares `## AllowLoadGameType: mainline` — `mainline` matches the \
         retail game-type filter (src/toc.rs:294-302), so is_game_type_restricted returns false \
         and the auto-discovery sweep at src/loader/mod.rs:527 keeps it"
    );

    // The unified PTR TOC carries no `## AllowLoad` directive, so the addon is
    // in-game only there; the Retail TOC loads it on both screen families.
    let glue_allowed = !toc_layout::unified_toc_tree();
    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Blizzard_ItemButton must load on the Game screen — every item-bearing HUD surface \
         depends on the `ItemButton` intrinsic + ItemButtonMixin"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert_eq!(
            toc.allows_screen(screen),
            glue_allowed,
            "Blizzard_ItemButton glue-screen loading follows the active TOC's AllowLoad \
             directive. (Screen tested: {screen:?})"
        );
    }

    let raw =
        std::fs::read_to_string(item_button_toc()).expect("Blizzard_ItemButton TOC should read");
    if toc_layout::unified_toc_tree() {
        assert!(
            !raw.contains("## AllowLoadGameType"),
            "The unified PTR TOC scopes flavors per entry, so it must not declare a \
             top-level `## AllowLoadGameType`"
        );
    } else {
        assert!(
            raw.contains("## AllowLoadGameType: mainline"),
            "TOC must declare `## AllowLoadGameType: mainline` exactly — the Shared/Mainline file \
             layout exists so non-mainline flavors can ship a different lua/xml split, but the \
             current TOC scopes the addon to retail only"
        );
        assert!(
            raw.contains("## AllowLoad: Both"),
            "TOC must declare `## AllowLoad: Both` — item button surfaces appear on both the \
             in-game HUD (bags, vendor, etc.) and on glue screens (character select previews)"
        );
        assert!(
            raw.contains("## DefaultState: enabled"),
            "TOC must declare `## DefaultState: enabled` — auto-enabled because every \
             item-bearing UI depends on `ItemButton` intrinsic + ItemButtonMixin"
        );
    }
}

#[test]
fn blizzard_item_button_toc_lists_three_files_lua_first_then_shared_xml_then_mainline_xml() {
    let toc = TocFile::from_file(&item_button_toc()).expect("Blizzard_ItemButton TOC should parse");
    let actual_files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        actual_files,
        item_button_files(),
        "TOC body must list exactly these files in this order: Shared\\ItemButtonTemplate.lua \
         loads FIRST (publishes ItemButtonMixin + ItemButtonConstants + GetItemButtonIconTexture \
         at file scope) so Shared\\ItemButtonTemplate.xml's `mixin=\"ItemButtonMixin\"` resolves \
         the table; Mainline\\ItemButtonTemplate.xml loads LAST and uses `<Script file=...>` to \
         pull in Mainline\\ItemButtonTemplate.lua before its 7 virtual templates instantiate \
         (each references CircularGiantItemButtonMixin / EnchantingItemButtonAnimMixin / \
         SetItemButtonTexture-style overrides). Backslashes in the TOC body are normalized to \
         forward slashes by push_file_entry (src/toc.rs:147)"
    );
}

#[test]
fn blizzard_item_button_directory_holds_three_entries() {
    let entries = std::fs::read_dir(item_button_dir())
        .expect("Blizzard_ItemButton directory should read")
        .count();
    assert_eq!(
        entries, 3,
        "Directory must hold exactly 3 entries (1 `_Mainline.toc` + 1 `Shared/` subdir + 1 \
         `Mainline/` subdir). The Shared/Mainline split exists so non-mainline flavors could \
         ship divergent xml/lua against the same intrinsic, but only Mainline ships today"
    );
}

#[test]
fn blizzard_item_button_auto_discovered_on_game_screen() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_ItemButton");
    assert!(
        found,
        "Blizzard_ItemButton must appear in the Game-screen auto-discovery set — the TOC sets \
         `## AllowLoad: Both` + `## AllowLoadGameType: mainline` and omits `LoadOnDemand`, so \
         the sweep at src/loader/mod.rs:527 keeps it for the standard retail Game screen"
    );
}

prefork_full_ui_case! {
fn blizzard_item_button_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_ItemButton")
                || message.contains("ItemButtonMixin")
                || message.contains("ItemButtonTemplate")
                || message.contains("CircularGiantItemButtonMixin")
                || message.contains("EnchantingItemButtonAnimMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ItemButton emitted addon-specific Lua errors during the Game-screen sweep:\n  \
         {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_button_is_addon_loaded_after_game_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ItemButton')")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ItemButton') must return true after the Game-screen \
         sweep — confirms the loader registers the addon with the loaded-set without an \
         explicit load_addon call"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_button_publishes_constants_table_with_two_context_match_values(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(ItemButtonConstants)")
        .expect("ItemButtonConstants probe should succeed");
    assert_eq!(
        kind, "table",
        "ItemButtonConstants must publish at `_G` as a table — Shared/ItemButtonTemplate.lua:1-8 \
         declares it at file scope so GetItemContextOverlayMode / UpdateItemContextOverlayTextures \
         can compare against ContextMatch.Standard / RuneForging without a global lookup"
    );

    let standard: f64 = env
        .eval("return ItemButtonConstants.ContextMatch.Standard")
        .expect("Standard probe should succeed");
    assert_eq!(
        standard, 1.0,
        "ItemButtonConstants.ContextMatch.Standard must equal 1 — keyed in the constants table \
         literal so UpdateItemContextOverlayTextures(Standard) draws the dim 0/0/0/0.8 overlay \
         covering the whole button"
    );

    let runeforging: f64 = env
        .eval("return ItemButtonConstants.ContextMatch.RuneForging")
        .expect("RuneForging probe should succeed");
    assert_eq!(
        runeforging, 2.0,
        "ItemButtonConstants.ContextMatch.RuneForging must equal 2 — \
         UpdateItemContextOverlayTextures(RuneForging) swaps the overlay to the \
         `runecarving-icon-bag-item-glow` atlas centered on the icon (bagged-item runeforging cue)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_button_mixin_publishes_with_shared_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(ItemButtonMixin)")
        .expect("ItemButtonMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "ItemButtonMixin must publish at `_G` as a table — Shared/ItemButtonTemplate.lua:10 \
         creates the empty table at file scope before binding 32 methods to it"
    );

    for method in ITEM_BUTTON_SHARED_METHODS {
        let kind: String = env
            .eval(&format!("return type(ItemButtonMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("ItemButtonMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemButtonMixin.{method} must publish as a function — Shared/ItemButtonTemplate.lua \
             binds 32 methods total (item-context tracking, item-source / location / link \
             accessors, count / scale / alpha forwarders, bag-id helpers). Missing this method \
             implies the Shared lua never executed or the mixin table was overwritten"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_button_mixin_carries_mainline_method_overrides(env: &WowLuaEnv) {

    for method in ITEM_BUTTON_MAINLINE_METHOD_OVERRIDES {
        let kind: String = env
            .eval(&format!("return type(ItemButtonMixin['{method}'])"))
            .unwrap_or_else(|err| panic!("ItemButtonMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "ItemButtonMixin.{method} must publish as a function — \
             Mainline/ItemButtonTemplate.lua:404-422 defines five method overrides on \
             ItemButtonMixin (SetItemButtonTexture, SetItemButtonTextureVertexColor, \
             SetItemButtonQuality, SetItemButtonBorderVertexColor, GetItemButtonBackgroundTexture) \
             that delegate to the file-local *_Base helpers. Loaded by \
             Mainline/ItemButtonTemplate.xml:3 via `<Script file=\"Mainline\\ItemButtonTemplate.lua\"/>` \
             — missing this method proves the Mainline xml's Script directive never fired"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_button_publishes_circular_giant_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(CircularGiantItemButtonMixin)")
        .expect("CircularGiantItemButtonMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "CircularGiantItemButtonMixin must publish at `_G` as a table — \
         Mainline/ItemButtonTemplate.lua:424 creates the table at file scope, then binds \
         SetItemButtonQuality which paints an auction-house atlas border instead of the \
         standard WhiteIconFrame. Consumed by CircularGiantItemButtonTemplate (54x54 button \
         that auction listings use for item previews)"
    );

    let set_quality: String = env
        .eval("return type(CircularGiantItemButtonMixin.SetItemButtonQuality)")
        .expect("CircularGiantItemButtonMixin.SetItemButtonQuality probe should succeed");
    assert_eq!(
        set_quality, "function",
        "CircularGiantItemButtonMixin.SetItemButtonQuality must publish as a function — single \
         override that calls ColorManager.GetAtlasDataForAuctionHouseItemQuality(quality) and \
         applies the atlas to IconBorder, falling back to the bare border when quality is nil"
    );
}
}

prefork_full_ui_case! {
fn blizzard_item_button_publishes_enchanting_anim_mixin_with_lifecycle_methods(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(EnchantingItemButtonAnimMixin)")
        .expect("EnchantingItemButtonAnimMixin probe should succeed");
    assert_eq!(
        kind, "table",
        "EnchantingItemButtonAnimMixin must publish at `_G` as a table — \
         Mainline/ItemButtonTemplate.lua:448 creates the table, then binds 6 methods. Consumed \
         by EnchantingItemButtonAnimTemplate via `mixin=\"EnchantingItemButtonAnimMixin\"` and \
         the four `<OnLoad/OnShow/OnHide/OnEvent method=\"...\">` script bindings"
    );

    for method in ENCHANTING_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(EnchantingItemButtonAnimMixin['{method}'])"
            ))
            .unwrap_or_else(|err| {
                panic!("EnchantingItemButtonAnimMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "EnchantingItemButtonAnimMixin.{method} must publish as a function — the mixin owns \
             6 methods: OnLoad wires the AugmentBorderAnim OnFinished handler to hide the \
             texture, OnShow/OnHide register/unregister ENCHANT_SPELL_COMPLETED via FrameUtil, \
             OnEvent plays the burst effect (175) when the enchanted item matches \
             GetItemLocation(), and SetItemLocationCallback / GetItemLocation are the \
             location-source plumbing"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_button_publishes_mainline_global_helpers(env: &WowLuaEnv) {

    for global in ITEM_BUTTON_MAINLINE_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G['{global}'])"))
            .unwrap_or_else(|err| panic!("global {global} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{global} must publish at `_G` as a function — \
             Mainline/ItemButtonTemplate.lua exposes 22 free helpers (count / texture / quality / \
             border / overlay / crafting-quality plumbing plus HandleModifiedItemClick) at file \
             scope. These are the API surface that bag / mail / vendor / auction frames use \
             instead of calling ItemButtonMixin methods directly. Missing this global proves \
             Mainline\\ItemButtonTemplate.lua never executed"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_button_virtual_templates_stay_nil_at_global_scope(env: &WowLuaEnv) {

    for template in ITEM_BUTTON_VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G['{template}'])"))
            .unwrap_or_else(|err| panic!("template {template} probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "{template} must NOT publish at `_G` — Mainline/ItemButtonTemplate.xml declares 7 \
             `virtual=\"true\"` frames (CircularItemButton, CircularGiantItemButton, \
             GiantItemButton, SimplePopupButton, LargeItemButton, SmallItemButton, \
             EnchantingItemButtonAnim). The loader keeps virtual frames in the template registry \
             only — they are consumed via `inherits=\"...\"` by downstream addons (auction, \
             mail, professions, encounter journal) and never resolved through global scope"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_item_button_intrinsic_creates_via_create_frame(env: &WowLuaEnv) {

    let intrinsic_table_global: String = env
        .eval("return type(_G['ItemButton'])")
        .expect("ItemButton global probe should succeed");
    assert_eq!(
        intrinsic_table_global, "nil",
        "_G['ItemButton'] must stay nil — Shared/ItemButtonTemplate.xml:4 declares the frame as \
         `intrinsic=\"true\"`, which `register_virtual_or_intrinsic` (preparation.rs:75) routes \
         to the template registry with `parent_override=None` so no top-level instance is \
         spawned. Intrinsics surface through the `CreateFrame(\"ItemButton\", ...)` widget-type \
         arg, not through global scope"
    );

    let create_kind: String = env
        .eval(
            "local btn = CreateFrame('ItemButton', 'TestItemButton_Intrinsic', UIParent) \
             return type(btn)",
        )
        .expect("CreateFrame('ItemButton') probe should succeed");
    assert_eq!(
        create_kind, "table",
        "CreateFrame('ItemButton', name, parent) must succeed and return a frame — proves the \
         loader registered `ItemButton` as a widget-type intrinsic. The created button inherits \
         the Shared XML's NormalTexture / PushedTexture / HighlightTexture, the 11 parentKey \
         children (icon, Count, Stock, searchOverlay, ItemContextOverlay, IconBorder, \
         IconOverlay, IconOverlay2, NormalTexture, PushedTexture, HighlightTexture), the \
         showMatchHighlight=true KeyValue, and the ItemButtonMixin method set"
    );

    let mixin_count: f64 = env
        .eval("return TestItemButton_Intrinsic.showMatchHighlight and 1 or 0")
        .expect("KeyValue probe should succeed");
    assert_eq!(
        mixin_count, 1.0,
        "TestItemButton_Intrinsic.showMatchHighlight must be truthy — Shared/ItemButtonTemplate.xml \
         declares `<KeyValue key=\"showMatchHighlight\" value=\"true\" type=\"boolean\"/>` so \
         every ItemButton-intrinsic instance starts with the match-highlight overlay opted in"
    );
}
}

#[test]
fn modified_click_xml_default_and_set_round_trip() {
    let env = load_full_game_ui();
    let default: String = env.eval("return GetModifiedClick('CHATLINK')").unwrap();
    assert_eq!(default, "SHIFT-BUTTON1");
    env.exec("SetModifiedClick('CHATLINK', 'CTRL')").unwrap();
    let updated: String = env.eval("return GetModifiedClick('CHATLINK')").unwrap();
    assert_eq!(updated, "CTRL");
}

#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
#[test]
fn retail_original_modified_click_setting_returns_nil_for_unknown_action() {
    let env = load_full_game_ui();
    let is_nil: bool = env
        .eval(
            r#"
            local setting = CreateFromMixins(ModifiedClickSettingMixin)
            setting:Init("stub-n-unknown-action", "STUB_N_UNKNOWN_ACTION", "NONE")
            return setting:GetValueDerived() == nil
            "#,
        )
        .expect("selected Blizzard ModifiedClickSetting consumer should read the action");

    assert!(
        is_nil,
        "the original settings consumer must preserve nil for an unknown modified-click action"
    );
}

#[test]
fn modified_item_click_routes_by_action_and_modifier() {
    let env = load_full_game_ui();
    env.state()
        .borrow_mut()
        .set_mouse_button_down("LeftButton", true);
    let result: String = env
        .eval(
            r#"
        local routes = {}
        ChatFrameUtil.InsertLink = function() table.insert(routes, "chat"); return true end
        DressUpItemLocation = function() table.insert(routes, "dress"); return true end
        DressUpLink = function() table.insert(routes, "dress-link"); return true end
        A_Admin.SetShiftKeyDown(true)
        HandleModifiedItemClick("|Hitem:123|h[item]|h")
        A_Admin.SetShiftKeyDown(false)
        A_Admin.SetControlKeyDown(true)
        HandleModifiedItemClick("|Hitem:123|h[item]|h")
        return table.concat(routes, ",")
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "chat,dress",
        "each modifier must select its bound action"
    );
}

#[test]
#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
fn retail_original_merchant_consumes_item_button_display_helpers() {
    use wow_ui_sim::loader::{
        discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen,
        register_load_with_addons,
    };
    use wow_ui_sim::lua_api::globals::real::merchant_buyback::BuybackItem;
    let ui = blizzard_ui_dir();
    let env = WowLuaEnv::new().expect("normal Retail Game environment");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    assert!(
        env.eval::<bool>(
            r#"
        return SetItemButtonCount == nil and SetItemButtonTexture == nil
            and SetItemButtonTextureVertexColor == nil
            and SetItemButtonNormalTextureVertexColor == nil
    "#
        )
        .unwrap()
    );
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));

    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    let owner = startup
        .iter()
        .position(|a| a.name == "Blizzard_ItemButton")
        .unwrap();
    let merchant = startup
        .iter()
        .position(|a| a.name == "Blizzard_UIPanels_Game")
        .unwrap();
    assert!(
        owner < merchant,
        "FrameXML.toc:7 is the transitive owner dependency"
    );
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env.state().borrow_mut().merchant_buyback_items = vec![
        BuybackItem {
            item_id: 2840,
            quantity: 7,
            price: 20,
            num_available: 1,
            is_usable: false,
            is_bound: false,
        },
        BuybackItem {
            item_id: 6948,
            quantity: 3,
            price: 30,
            num_available: 1,
            is_usable: true,
            is_bound: false,
        },
    ];
    env.exec(r#"
        local function original(func, path, line)
            local info = debug.getinfo(func, 'S')
            local source = info.source:gsub('\\', '/')
            assert(source:find(path, 1, true), source)
            assert(info.linedefined == line, path .. ': ' .. info.linedefined)
        end
        local owner = 'Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua'
        original(SetItemButtonCount, owner, 9)
        original(SetItemButtonTexture, owner, 94)
        original(SetItemButtonTextureVertexColor, owner, 111)
        original(SetItemButtonNormalTextureVertexColor, owner, 146)
        original(MerchantFrame_UpdateBuybackInfo, 'Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua', 506)
        -- Calls all four globals (MerchantFrame.lua:532,534,544-550).
        MerchantFrame_UpdateBuybackInfo()
        for index, quantity in ipairs({7, 3}) do
            local button = _G['MerchantItem' .. index .. 'ItemButton']
            local _, icon = GetBuybackItemInfo(index)
            assert(button.Count:IsShown() and tonumber(button.Count:GetText()) == quantity)
            assert(GetItemButtonIconTexture(button):GetTexture() == icon)
            local expectedR, expectedGB = index == 1 and 0.9 or 1, index == 1 and 0 or 1
            for _, texture in ipairs({GetItemButtonIconTexture(button), button:GetNormalTexture()}) do
                local r, g, b = texture:GetVertexColor()
                assert(math.abs(r - expectedR) < 0.001 and g == expectedGB and b == expectedGB)
            end
        end
    "#).expect("original MerchantFrame buyback updates count, icon, and both vertex colors");
    assert!(
        !env.state()
            .borrow()
            .lua_errors
            .iter()
            .any(|error| error.contains("SetItemButtonCount")
                || error.contains("SetItemButtonTexture")
                || error.contains("SetItemButtonNormalTextureVertexColor")),
        "unexpected item-button startup error: {:?}",
        env.state().borrow().lua_errors
    );
}
