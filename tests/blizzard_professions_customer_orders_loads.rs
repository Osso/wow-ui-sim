#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn customer_orders_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ProfessionsCustomerOrders")
}

fn customer_orders_toc() -> PathBuf {
    customer_orders_dir().join("Blizzard_ProfessionsCustomerOrders.toc")
}

const CUSTOMER_ORDERS_TOC_FILES: &[&str] = &[
    "Blizzard_ProfessionsCustomerOrders_Bootstrap.lua",
    "Blizzard_ProfessionsCustomerOrdersForm.xml",
    "Blizzard_ProfessionsCustomerOrdersRecipeCategoryList.xml",
    "Blizzard_ProfessionsCustomerOrdersRecipeList.xml",
    "Blizzard_ProfessionsCustomerOrdersBrowseOrders.xml",
    "Blizzard_ProfessionsCustomerOrdersMyOrders.xml",
    "Blizzard_ProfessionsCustomerOrders.xml",
    "Registration.lua",
];

const REQUIRED_DEPS: &[&str] = &["Blizzard_ProfessionsTemplates", "Blizzard_AuctionHouseUI"];

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "ProfessionsCustomerOrdersMixin",
    "ProfessionsCustomerOrdersFrameTabMixin",
    "ProfessionsCustomerOrdersBrowsePageMixin",
    "ProfessionsCustomerOrdersMyOrdersMixin",
    "ProfessionsCustomerOrderListElementMixin",
    "ProfessionsCustomerOrdersRecipeListMixin",
    "ProfessionsCustomerOrdersRecipeListElementMixin",
    "ProfessionsCustomerOrdersRecipeCategoryListMixin",
    "ProfessionsCustomerOrdersCategoryButtonMixin",
    "ProfessionsCustomerOrderFormMixin",
    "ProfessionsCustomerListingsElementMixin",
];

const NAMED_NON_VIRTUAL_TOP_LEVEL_FRAMES: &[&str] = &["ProfessionsCustomerOrdersFrame"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "ProfessionsCustomerOrdersFrameTabTemplate",
    "ProfessionsCustomerOrdersRecipeListElementTemplate",
    "ProfessionsCustomerOrdersRecipeListTemplate",
    "ProfessionsCustomerOrdersCategoryButtonTemplate",
    "ProfessionsCustomerOrdersRecipeCategoryListTemplate",
    "ProfessionsCustomerOrdersBrowseOrdersTemplate",
    "ProfessionsCustomerOrderListElementTemplate",
    "ProfessionsCustomerOrdersMyOrdersTemplate",
    "ProfessionsCustomerListingsElementTemplate",
    "ProfessionsCustomerOrderFormTemplate",
];

fn auction_house_ui_toc() -> PathBuf {
    blizzard_ui_dir()
        .join("Blizzard_AuctionHouseUI")
        .join("Blizzard_AuctionHouseUI_Mainline.toc")
}

fn professions_templates_toc() -> PathBuf {
    blizzard_ui_dir()
        .join("Blizzard_ProfessionsTemplates")
        .join("Blizzard_ProfessionsTemplates.toc")
}

fn assert_directory_omits_symbol(directory: &std::path::Path, symbol: &str) {
    for entry in std::fs::read_dir(directory).expect("Blizzard source directory should read") {
        let path = entry.expect("Blizzard source entry should read").path();
        if path.is_dir() {
            assert_directory_omits_symbol(&path, symbol);
            continue;
        }

        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if !matches!(extension, "lua" | "xml" | "toc") {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("Blizzard source file should read");
        assert!(
            !source.contains(symbol),
            "{symbol} unexpectedly exists in {}",
            path.display()
        );
    }
}

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

fn load_customer_orders_with_deps(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &professions_templates_toc())
        .expect("Blizzard_ProfessionsTemplates loads cleanly");
    load_addon(&env.loader_env(), &auction_house_ui_toc())
        .expect("Blizzard_AuctionHouseUI (Mainline) loads cleanly");
    load_addon(&env.loader_env(), &customer_orders_toc())
        .expect("Blizzard_ProfessionsCustomerOrders loads cleanly");
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_customer_orders_does_not_publish_snapshot_only_hide_wrapper() {
    assert_directory_omits_symbol(&blizzard_ui_dir(), "HideProfessionsCustomerOrdersFrame");

    let env = load_full_game_ui();
    load_customer_orders_with_deps(&env);
    let result: (String, bool) = env
        .eval(
            "return type(ProfessionsCustomerOrdersFrame), HideProfessionsCustomerOrdersFrame == nil",
        )
        .expect("customer orders wrapper visibility should be queryable");
    assert_eq!(result, ("table".to_string(), true));
}

#[test]
fn blizzard_professions_customer_orders_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&customer_orders_dir())
        .expect("Blizzard_ProfessionsCustomerOrders TOC resolves");
    assert_eq!(
        resolved,
        customer_orders_toc(),
        "Blizzard_ProfessionsCustomerOrders ships exactly one bare TOC — no \
         `_Mainline.toc` variant. Cross-flavor crafting-orders panel"
    );

    let mainline = customer_orders_dir().join("Blizzard_ProfessionsCustomerOrders_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {}",
        mainline.display()
    );
}

#[test]
fn blizzard_professions_customer_orders_toc_declares_lod_game_only_addon() {
    let toc = TocFile::from_file(&customer_orders_toc())
        .expect("Blizzard_ProfessionsCustomerOrders TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the customer-orders panel is loaded \
         explicitly via the player-interaction path that fires \
         PlayerInteractionType.ProfessionsCustomerOrders, then \
         UIParentLoadAddOn('Blizzard_ProfessionsCustomerOrders') opens the panel"
    );
    assert!(!toc.is_load_first());
    assert!(
        !toc.is_secure_env(),
        "TOC must NOT declare `## UseSecureEnvironment:` — non-protected display panel"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC has no `## AllowLoadGameType:` directive — `is_game_type_restricted()` \
         returns false when the metadata key is absent (cross-flavor)"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC has no `## AllowLoad:` directive — `allows_screen` defaults to Game-only \
         when the key is absent. Crafting orders only matter in-world where the player \
         has access to a profession crafter NPC / mailbox"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Default Game-only screen gate must EXCLUDE {screen:?}"
        );
    }
}

#[test]
fn blizzard_professions_customer_orders_toc_declares_two_dependencies() {
    let toc = TocFile::from_file(&customer_orders_toc())
        .expect("Blizzard_ProfessionsCustomerOrders TOC parses");

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "TOC must declare exactly 2 deps in this order: Blizzard_ProfessionsTemplates \
         (publishes the shared crafting-table primitives + Professions.CreateNewOrderInfo \
         used by OnRecipeSelected to materialize a new order from itemID/spellID/skillLineAbilityID) \
         and Blizzard_AuctionHouseUI (publishes MoneyDisplayFrameTemplate / \
         ThinGoldEdgeTemplate + the auction-house frame primitives the customer-orders \
         panel re-uses for the money frame at the bottom — XML inline comment at \
         Blizzard_ProfessionsCustomerOrders.xml line 17 reads `<!-- Copied from \
         AuctionHouseFrame -->`)"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` declared"
    );
}

#[test]
fn blizzard_professions_customer_orders_toc_declares_no_saved_variables() {
    let toc = TocFile::from_file(&customer_orders_toc())
        .expect("Blizzard_ProfessionsCustomerOrders TOC parses");

    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare zero account-wide `## SavedVariables:` — pure stateless panel; \
         all order state pulls from the live C_CraftingOrders queries"
    );
    assert!(
        toc.saved_variables_per_character().is_empty(),
        "TOC must declare zero `## SavedVariablesPerCharacter:` — UI is fully derived \
         from server-authoritative crafting-orders state"
    );
}

#[test]
fn blizzard_professions_customer_orders_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(customer_orders_toc())
        .expect("Blizzard_ProfessionsCustomerOrders TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Professions Customer Orders"),
        "TOC must declare `## Title: Blizzard Professions Customer Orders` — \
         space-and-prose form"
    );
    assert!(
        raw.contains("## Author: Blizzard Entertainment"),
        "TOC must declare `## Author: Blizzard Entertainment`"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_ProfessionsTemplates, Blizzard_AuctionHouseUI"),
        "TOC must declare the 2-dep `## Dependencies:` line as a single comma-separated \
         entry"
    );

    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare `## AllowLoad:` or `## AllowLoadGameType:`"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare `## SavedVariables:` or `## SavedVariablesPerCharacter:`"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare `## OptionalDeps:`"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:`"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:`"
    );
}

#[test]
fn blizzard_professions_customer_orders_toc_lists_bootstrap_then_seven_files_in_canonical_order() {
    let toc = TocFile::from_file(&customer_orders_toc())
        .expect("Blizzard_ProfessionsCustomerOrders TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, CUSTOMER_ORDERS_TOC_FILES,
        "Retail 12.1.0.69497 lists its bootstrap first, then the six XML entries and \
         Registration.lua. Each XML continues to load its paired Lua through Script directives"
    );
}

#[test]
fn blizzard_professions_customer_orders_does_not_appear_in_eager_discovery() {
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
            .any(|(name, _)| name == "Blizzard_ProfessionsCustomerOrders");
        assert!(
            !found,
            "Blizzard_ProfessionsCustomerOrders must NOT appear in eager discovery for \
             {screen:?} — `## LoadOnDemand: 1` excludes it. Loaded explicitly when the \
             player interacts with a crafting orders NPC / mailbox"
        );
    }
}

#[test]
fn blizzard_professions_customer_orders_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_ProfessionsCustomerOrders");
    assert!(
        found,
        "Blizzard_ProfessionsCustomerOrders must appear in `discover_all_blizzard_addons`"
    );
}

prefork_full_ui_case! {
fn blizzard_professions_customer_orders_loads_explicitly_after_dependencies(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_customer_orders_with_deps(&env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_ProfessionsCustomerOrders")
                || message.contains("ProfessionsCustomerOrdersFrame")
                || message.contains("ProfessionsCustomerOrder")
                || message.contains("ProfessionsCustomerListings")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ProfessionsCustomerOrders emitted addon-specific Lua errors during \
         load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_professions_customer_orders_publishes_eleven_mixin_globals(env: &WowLuaEnv) {
    load_customer_orders_with_deps(&env);

    for name in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{name})"))
            .unwrap_or_else(|err| panic!("type(_G.{name}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{name} must publish as a table — Blizzard_ProfessionsCustomerOrders \
             exposes 11 mixins to global scope. ProfessionsCustomerOrdersMixin owns \
             the root frame; ProfessionsCustomerOrdersFrameTabMixin owns the Browse / \
             Orders tab buttons; ProfessionsCustomerOrdersBrowsePageMixin / \
             ProfessionsCustomerOrdersMyOrdersMixin own the 2 page modes; \
             ProfessionsCustomerOrderListElementMixin / \
             ProfessionsCustomerListingsElementMixin / \
             ProfessionsCustomerOrdersRecipeListElementMixin extend \
             TableBuilderRowMixin via CreateFromMixins for the scrollable list rows; \
             ProfessionsCustomerOrdersRecipeListMixin / \
             ProfessionsCustomerOrdersRecipeCategoryListMixin own the category / \
             recipe browse panes; ProfessionsCustomerOrdersCategoryButtonMixin owns \
             the per-category button; ProfessionsCustomerOrderFormMixin owns the \
             heaviest piece — the 64KB order-detail edit form"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_customer_orders_publishes_mode_enum_global(env: &WowLuaEnv) {
    load_customer_orders_with_deps(&env);

    let mode_kind: String = env
        .eval("return type(_G.ProfessionsCustomerOrdersMode)")
        .expect("type(ProfessionsCustomerOrdersMode) probe succeeds");
    assert_eq!(
        mode_kind, "table",
        "_G.ProfessionsCustomerOrdersMode must publish as a table — created via \
         EnumUtil.MakeEnum('Browse', 'Orders') at line 1 of \
         Blizzard_ProfessionsCustomerOrders.lua. Used by SelectMode / the \
         modeToTabIdx lookup / the BrowseTab + OrdersTab KeyValue mode bindings"
    );

    let browse_value: i64 = env
        .eval("return ProfessionsCustomerOrdersMode.Browse")
        .expect("ProfessionsCustomerOrdersMode.Browse probe succeeds");
    let orders_value: i64 = env
        .eval("return ProfessionsCustomerOrdersMode.Orders")
        .expect("ProfessionsCustomerOrdersMode.Orders probe succeeds");
    assert_ne!(
        browse_value, orders_value,
        "EnumUtil.MakeEnum must assign distinct integer values to Browse and Orders"
    );
}
}

prefork_full_ui_case! {
fn blizzard_professions_customer_orders_named_top_level_frame_is_in_global_env(env: &WowLuaEnv) {
    load_customer_orders_with_deps(&env);

    for frame in NAMED_NON_VIRTUAL_TOP_LEVEL_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must exist as a frame table — \
             Blizzard_ProfessionsCustomerOrders defines exactly 1 named non-virtual \
             top-level frame: ProfessionsCustomerOrdersFrame (the root Frame inheriting \
             PortraitFrameTemplate, parent=UIParent, toplevel=true, hidden=true \
             initially, size 825x568, anchored CENTER, hosting Form / BrowseOrders / \
             MyOrdersPage / BrowseTab / OrdersTab / MoneyFrameInset / MoneyFrameBorder \
             children). The 2 sub-tabs ProfessionsCustomerOrdersFrameBrowseTab and \
             ProfessionsCustomerOrdersFrameOrdersTab are also published via the \
             $parent name expansion + parentKey, but are nested-non-top-level"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_customer_orders_virtual_templates_not_in_global_env(env: &WowLuaEnv) {
    load_customer_orders_with_deps(&env);

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the template \
             registry, NOT in the global environment. \
             Blizzard_ProfessionsCustomerOrders ships 10 virtual templates: \
             ProfessionsCustomerOrdersFrameTabTemplate (the Browse/Orders tab Button \
             inheriting PanelTabButtonTemplate); 4 list-element Button templates that \
             extend TableBuilderRowMixin via CreateFromMixins (RecipeListElement, \
             CategoryButton, OrderListElement, ListingsElement); 5 page-template \
             Frames (RecipeListTemplate, RecipeCategoryListTemplate, \
             BrowseOrdersTemplate, MyOrdersTemplate, OrderFormTemplate)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_customer_orders_registers_one_ui_panel_with_uiparent(env: &WowLuaEnv) {
    load_customer_orders_with_deps(&env);

    let registration_present: bool = env
        .eval(
            "return UIPanelWindows ~= nil \
                and UIPanelWindows.ProfessionsCustomerOrdersFrame ~= nil",
        )
        .expect("UIPanelWindows query succeeds");
    assert!(
        registration_present,
        "UIPanelWindows.ProfessionsCustomerOrdersFrame must be populated after load — \
         Registration.lua (the LAST file in the TOC body) calls \
         RegisterUIPanel(ProfessionsCustomerOrdersFrame, {{area='doublewide', \
         xoffset=20, pushable=0, allowOtherPanels=1, checkFit=1}}) at the end of the \
         addon load sequence. The `area='doublewide'` slot is the wide central panel \
         area used for AH-style panels (note the AuctionHouseFrame dependency); \
         `pushable=0` prevents the frame from being pushed aside by other UI panels"
    );
}
}
