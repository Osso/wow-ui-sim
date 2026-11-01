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

fn cart_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HousingMarketCart")
}

fn cart_toc() -> PathBuf {
    cart_dir().join("Blizzard_HousingMarketCart.toc")
}

fn shopping_cart_toc() -> PathBuf {
    blizzard_ui_dir()
        .join("Blizzard_GenericShoppingCart")
        .join("Blizzard_GenericShoppingCart.toc")
}

fn load_housing_market_cart_with_dependency(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &shopping_cart_toc())
        .expect("Blizzard_GenericShoppingCart (the only declared dep) should load via LoD call");
    load_addon(&env.loader_env(), &cart_toc())
        .expect("Blizzard_HousingMarketCart should load via explicit Rust loader call");
}

fn assert_mixin_methods(env: &WowLuaEnv, mixin: &str, methods: &[&str], rationale: &str) {
    for method in methods {
        let exists: bool = env
            .eval(&format!("return type({mixin}['{method}']) == 'function'"))
            .unwrap_or_else(|err| panic!("{mixin}.{method} existence query failed: {err}"));
        assert!(exists, "{mixin} must expose `:{method}()` — {rationale}");
    }
}

#[test]
fn blizzard_housing_market_cart_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&cart_dir()).expect("Blizzard_HousingMarketCart TOC should resolve");
    assert_eq!(
        resolved,
        cart_toc(),
        "Blizzard_HousingMarketCart ships exactly one bare TOC — retail-only addon resolves via \
         `find_toc_file` fallthrough"
    );
}

#[test]
fn blizzard_housing_market_cart_toc_declares_lod_with_one_dependency() {
    let toc = TocFile::from_file(&cart_toc()).expect("HousingMarketCart TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_HousingMarketCart declares `## LoadOnDemand: 1` — pulled transitively as a \
         declared `## Dependencies` of Blizzard_HouseEditor (line 4 of HouseEditor.toc), so any \
         explicit LoD load of HouseEditor pulls this addon first"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_GenericShoppingCart".to_string()],
        "Single `## Dependencies:` entry: Blizzard_GenericShoppingCart (provides the base \
         ShoppingCartVisualsFrameMixin / ShoppingCartDataManagerMixin / \
         ShoppingCartServiceButtonTemplate / ShoppingCartVisualsFrameTemplate / \
         ShoppingCartPriceTemplate / ShoppingCartRemoveFromCartButtonTemplate widgets and the \
         ShoppingCartDataServices.PurchaseCart service that every cart variant builds on)"
    );
}

#[test]
fn blizzard_housing_market_cart_toc_is_retail_only_and_game_only() {
    let toc = TocFile::from_file(&cart_toc()).expect("HousingMarketCart TOC should parse");
    let toc_text = std::fs::read_to_string(cart_toc()).expect("TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: standard"),
        "Declares `## AllowLoadGameType: standard` — retail-only Midnight feature"
    );
    assert!(!toc.is_game_type_restricted());
    assert!(
        toc_text.contains("## AllowLoad: Game"),
        "Declares `## AllowLoad: Game` — addon is gated to the in-game phase only (NOT the Login \
         / CharacterSelect / CharacterCreate Glue screens), but the LoadOnDemand flag still keeps \
         it out of the Game-screen auto-discovery sweep until LoadAddOn is called"
    );
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Declares `## DefaultState: enabled` — addon is enabled by default in every character's \
         AddOns list (LoD addons can still be explicitly disabled, the default just controls the \
         initial enabled flag)"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "No `## SavedVariables*` — cart contents are server-driven via the bulk-purchase + \
         storage-entry events plus the C_CatalogShop / C_HousingDecor APIs"
    );
}

#[test]
fn blizzard_housing_market_cart_toc_lists_five_files_in_order() {
    let toc = TocFile::from_file(&cart_toc()).expect("HousingMarketCart TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_HousingMarketCartServices.lua".to_string(),
            "Blizzard_HousingMarketCartTemplates.lua".to_string(),
            "Blizzard_HousingMarketCartTemplates.xml".to_string(),
            "Blizzard_HousingMarketCartItemTemplates.lua".to_string(),
            "Blizzard_HousingMarketCartItemTemplates.xml".to_string(),
        ],
        "TOC body lists exactly 5 source files in this exact order — Services.lua FIRST publishes \
         the 3 small button-mixin tables (HousingMarketViewCartButtonMixin / \
         HousingMarketShowCartServiceMixin / HousingMarketHideCartServiceMixin) used as service \
         button mixins; Templates.lua THEN defines HousingMarketCartFrameMixin (CreateFromMixins \
         of ShoppingCartVisualsFrameMixin) and HousingMarketCartDataManagerMixin (CreateFromMixins \
         of ShoppingCartDataManagerMixin) plus the 2 StaticPopupDialogs entries; Templates.xml \
         registers HousingMarketShoppingCartServiceButtonTemplate / HousingMarketCartCheckoutButtonTemplate \
         / HousingMarketCartFrameTemplate (the umbrella cart frame template, virtual); \
         ItemTemplates.lua / ItemTemplates.xml LAST register the row mixins (Brace / Price / \
         PlaceInWorldButton / Item / RemoveInlineItemFromCartService / BundleRegistrant / \
         BundleHeader / BundleFooter / Bundle / BundleItem) and their virtual templates"
    );
}

#[test]
fn blizzard_housing_market_cart_directory_holds_six_entries() {
    let dir = cart_dir();
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HousingMarketCart directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        6,
        "Directory ships exactly 6 entries (5 source + 1 TOC, no flavor subdirectory). Got: \
         {entries:?}"
    );
    assert!(entries.contains(&"Blizzard_HousingMarketCart.toc".to_string()));
    assert!(entries.contains(&"Blizzard_HousingMarketCartServices.lua".to_string()));
    assert!(entries.contains(&"Blizzard_HousingMarketCartTemplates.lua".to_string()));
    assert!(entries.contains(&"Blizzard_HousingMarketCartTemplates.xml".to_string()));
    assert!(entries.contains(&"Blizzard_HousingMarketCartItemTemplates.lua".to_string()));
    assert!(entries.contains(&"Blizzard_HousingMarketCartItemTemplates.xml".to_string()));
}

#[test]
fn blizzard_housing_market_cart_excluded_from_all_screen_auto_discovery() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
        ScreenKind::Game,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_HousingMarketCart");
        assert!(
            !discovered,
            "Must NOT appear in {screen:?} auto-discovery — `## LoadOnDemand: 1` excludes the \
             addon from every ScreenKind pass; pulled exclusively as a transitive Dependencies \
             entry of Blizzard_HouseEditor"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HousingMarketCart/")
                || e.contains("Blizzard_HousingMarketCart\\")
                || e.contains("HousingMarketCart")
                || e.contains("HousingMarketView")
                || e.contains("HousingMarketShow")
                || e.contains("HousingMarketHide")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Loading Blizzard_HousingMarketCart must not emit any addon-specific Lua errors. Got {} \
         errors: {:?}",
        related.len(),
        related
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_formats_numeric_prices(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    let result: bool = env.eval(r#"
        local price, sale = HousingMarketCartPriceMixin:GetPriceText(120, 75, 200)
        local regular, noSale = HousingMarketCartPriceMixin:GetPriceText(42, nil, 200)
        return price == '|c' .. GRAY_FONT_COLOR:GenerateHexColor() .. '120|r'
            and sale == '|c' .. GREEN_FONT_COLOR:GenerateHexColor() .. '75|r'
            and regular == '|c' .. WHITE_FONT_COLOR:GenerateHexColor() .. '42|r'
            and noSale == ''
    "#).expect("housing cart formats numeric prices");
    assert!(result);
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_is_addon_loaded_via_explicit_lod_call(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingMarketCart')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_HousingMarketCart') must return true after the explicit \
         LoD load — proves the loader registered the addon name"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_shopping_cart_dependency_loads_via_explicit_lod_call(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_GenericShoppingCart')")
        .expect("GenericShoppingCart IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "Blizzard_GenericShoppingCart (the only declared dep, also LoadOnDemand) must be loaded \
         before HousingMarketCart — both addons are LoD so neither is in the auto-discovery \
         sweep; the test driver explicitly LoDs GenericShoppingCart first to satisfy the dep"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_publishes_event_namespace_global(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    let namespace: String = env
        .eval("return HOUSING_MARKET_EVENT_NAMESPACE")
        .expect("HOUSING_MARKET_EVENT_NAMESPACE lookup should succeed");
    assert_eq!(
        namespace, "HousingMarketEvents",
        "HOUSING_MARKET_EVENT_NAMESPACE must publish the literal string 'HousingMarketEvents' as \
         a `_G` global — used by every Cart service button (KeyValue eventNamespace=global) and \
         by HousingMarketCartFrameMixin:GetEventNamespace as the per-namespace event-routing key \
         that lets the shared shopping-cart infrastructure dispatch events to the housing variant"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_frame_mixin_publishes_twenty_one_methods(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    assert_mixin_methods(
        &env,
        "HousingMarketCartFrameMixin",
        &[
            "OnLoad",
            "OnShow",
            "StartCurrencyRefreshTicker",
            "StopCurrencyRefreshTicker",
            "OnHide",
            "OnEvent",
            "OnSimpleCheckoutClosed",
            "PlayHearthsteelBalanceUpdateAnim",
            "GetEventNamespace",
            "SetCartShown",
            "GetElementInitInfoFunc",
            "SetupDataManager",
            "SetupDividerPredicates",
            "GetTotalPrice",
            "GetNumItemsInCart",
            "AddItemToList",
            "RemoveItemFromList",
            "UpdateNumItemsInCart",
            "IsBundleInCart",
            "GetNumDecorPlaced",
            "GetCartCurrencyInfo",
        ],
        "HousingMarketCartFrameMixin owns 21 methods on the umbrella cart frame template \
         (CreateFromMixins of ShoppingCartVisualsFrameMixin): lifecycle (OnLoad / OnShow / OnHide \
         / OnEvent), Hearthsteel virtual-currency refresh ticker (StartCurrencyRefreshTicker / \
         StopCurrencyRefreshTicker / PlayHearthsteelBalanceUpdateAnim), simple-checkout \
         integration (OnSimpleCheckoutClosed routed via HouseEditor.SimpleCheckoutClosed \
         EventRegistry callback), event-namespace plumbing (GetEventNamespace returns \
         HOUSING_MARKET_EVENT_NAMESPACE), show/hide gating (SetCartShown), data-manager / row \
         template setup (GetElementInitInfoFunc / SetupDataManager / SetupDividerPredicates / \
         GetTotalPrice), per-row queries (GetNumItemsInCart / AddItemToList / RemoveItemFromList \
         / UpdateNumItemsInCart / IsBundleInCart / GetNumDecorPlaced), and currency-info readout \
         (GetCartCurrencyInfo)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_data_manager_mixin_publishes_nineteen_methods(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    assert_mixin_methods(
        &env,
        "HousingMarketCartDataManagerMixin",
        &[
            "Init",
            "GetNumItemsInCart",
            "IsBundleInCart",
            "GetNumDecorPlaced",
            "AddToCart",
            "RemoveFromCartInternal",
            "ClearCartInternal",
            "ClearCart",
            "PromoteHelper",
            "Promote",
            "BULK_PURCHASE_RESULT_RECEIVED",
            "PromotePendingHelper",
            "HOUSING_STORAGE_ENTRY_UPDATED",
            "HOUSING_DECOR_PREVIEW_LIST_UPDATED",
            "HOUSING_DECOR_ADD_TO_PREVIEW_LIST",
            "HOUSING_DECOR_PREVIEW_LIST_REMOVE_FROM_WORLD",
            "OnUpdateItemInfo",
            "PlaceInWorld",
            "SetPlaceInWorldCallback",
        ],
        "HousingMarketCartDataManagerMixin owns 19 methods (CreateFromMixins of \
         ShoppingCartDataManagerMixin): Init, query helpers (GetNumItemsInCart / IsBundleInCart \
         / GetNumDecorPlaced), mutation (AddToCart / RemoveFromCartInternal / ClearCartInternal / \
         ClearCart), promote (PromoteHelper / Promote / PromotePendingHelper for the \
         decor-preview promotion flow), 5 frame-event handler methods named after the events \
         they consume (BULK_PURCHASE_RESULT_RECEIVED + the 4 HOUSING_DECOR_* + \
         HOUSING_STORAGE_ENTRY_UPDATED), OnUpdateItemInfo (item-info refresh callback), and \
         place-in-world plumbing (PlaceInWorld / SetPlaceInWorldCallback for the decor-place \
         button on each row)",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_item_mixins_publish_method_counts(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    assert_mixin_methods(
        &env,
        "HousingMarketCartBraceMixin",
        &["InitBraces"],
        "Brace mixin shows top/bottom brace decorations between bundle rows — single \
         InitBraces(hasTopBrace, hasBottomBrace) entry point",
    );
    assert_mixin_methods(
        &env,
        "HousingMarketCartPriceMixin",
        &["GetCurrencyInfo", "GetPriceText"],
        "Price mixin formats Hearthsteel price strings with optional sale-price strikethrough \
         and player-currency comparison — 2 methods",
    );
    assert_mixin_methods(
        &env,
        "PlaceInWorldButtonMixin",
        &["OnEnter", "OnLeave"],
        "Place-in-world button uses the house-chest-icon atlas and shows a tooltip on hover — 2 \
         methods",
    );
    assert_mixin_methods(
        &env,
        "HousingMarketCartItemMixin",
        &[
            "OnLoad",
            "GetPlaceInWorldData",
            "InitItem",
            "Refresh",
            "OnDragStart",
            "SetSelection",
            "OnEnter",
            "OnLeave",
            "UpdatePreviewStatusIcon",
        ],
        "Item row mixin owns 9 methods on the standalone (non-bundle) cart row",
    );
    assert_mixin_methods(
        &env,
        "HousingRemoveInlineItemFromCartServiceMixin",
        &["GetEventData"],
        "Remove-inline service button mixin routes its click event through the shopping-cart \
         service infrastructure — single GetEventData entry point",
    );
    assert_mixin_methods(
        &env,
        "HousingMarketCartBundleHeaderMixin",
        &["Init", "Refresh"],
        "Bundle header mixin (CreateFromMixins of HousingMarketCartBundleRegistrant) — 2 methods",
    );
    assert_mixin_methods(
        &env,
        "HousingMarketCartBundleFooterMixin",
        &["Init"],
        "Bundle footer mixin (CreateFromMixins of HousingMarketCartBundleRegistrant) — 1 method",
    );
    assert_mixin_methods(
        &env,
        "HousingMarketCartBundleMixin",
        &[
            "OnLoad",
            "OnBundleHovered",
            "OnBundleLeft",
            "InitItem",
            "Refresh",
        ],
        "Bundle umbrella mixin (CreateFromMixins of HousingMarketCartBundleRegistrant) — 5 \
         methods including the bundle-hover-broadcast pair OnBundleHovered/OnBundleLeft",
    );
    assert_mixin_methods(
        &env,
        "HousingMarketCartBundleItemMixin",
        &[
            "OnLoad",
            "GetPlaceInWorldData",
            "OnDragStart",
            "InitItem",
            "Refresh",
            "SetSelection",
            "OnEnter",
            "OnLeave",
            "UpdatePreviewStatusIcon",
        ],
        "Bundle-item row mixin (CreateFromMixins of HousingMarketCartBundleRegistrant) — 9 \
         methods, mirrors HousingMarketCartItemMixin's row surface for items contained in a \
         bundle",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_services_mixins_publish_method_counts(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    assert_mixin_methods(
        &env,
        "HousingMarketViewCartButtonMixin",
        &["UpdateNumItemsInCart"],
        "View-cart button mixin owns the ItemCountText badge — single \
         UpdateNumItemsInCart(numItemsInCart) entry point that drives the count text on the \
         hidden-cart shortcut button",
    );
    assert_mixin_methods(
        &env,
        "HousingMarketShowCartServiceMixin",
        &["GetEventData"],
        "Show-cart service button mixin returns the literal `shown=true` event payload — single \
         GetEventData entry point routed through the shopping-cart service infrastructure to \
         flip the cart's visibility on click",
    );
    assert_mixin_methods(
        &env,
        "HousingMarketHideCartServiceMixin",
        &["GetEventData"],
        "Hide-cart service button mixin mirrors the show variant with `shown=false` — single \
         GetEventData entry point",
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_static_popup_dialogs_register(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    for key in [
        "HOUSING_MARKET_CLEAR_CART_CONFIRMATION",
        "HOUSING_MARKET_PURCHASE_FAILURE",
    ] {
        let registered: bool = env
            .eval(&format!(
                "return type(StaticPopupDialogs['{key}']) == 'table'"
            ))
            .expect("StaticPopupDialogs lookup should succeed");
        assert!(
            registered,
            "StaticPopupDialogs['{key}'] must register as a table after addon load — \
             HousingMarketCartTemplates.lua wires both popups at file scope (Clear-Cart \
             confirmation OnAccept calls ClearCartInternal; Purchase-Failure shows the bulk \
             purchase error response surface routed through BULK_PURCHASE_RESULT_RECEIVED)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_market_cart_all_xml_templates_stay_virtual(env: &WowLuaEnv) {
    load_housing_market_cart_with_dependency(env);
    for template in [
        "HousingMarketShoppingCartServiceButtonTemplate",
        "HousingMarketCartCheckoutButtonTemplate",
        "HousingMarketCartFrameTemplate",
        "PlaceInWorldButtonTemplate",
        "HousingMarketCartPriceTemplate",
        "HousingMarketCartItemTemplate",
        "HousingMarketCartBundleHeaderTemplate",
        "HousingMarketCartBundleFooterTemplate",
        "HousingMarketCartBraceTemplate",
        "HousingMarketCartBundleTemplate",
        "HousingMarketCartBundleItemTemplate",
    ] {
        let value_type: String = env
            .eval(&format!("return type(_G['{template}'])"))
            .expect("template _G lookup should succeed");
        assert_eq!(
            value_type, "nil",
            "{template} must NOT publish to `_G` — every Frame/Button declared in the \
             HousingMarketCart XML files carries `virtual=\"true\"`, since this addon ships \
             ONLY templates (no top-level non-virtual frames). Consumers — Blizzard_HouseEditor \
             and the Blizzard catalog shop UI — instantiate their own named frames inheriting \
             these templates"
        );
    }
}
}
