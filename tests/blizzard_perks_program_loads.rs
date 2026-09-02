#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn perks_program_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PerksProgram")
}

fn perks_program_toc() -> PathBuf {
    perks_program_dir().join("Blizzard_PerksProgram.toc")
}

const PERKS_PROGRAM_TOC_FILES: &[&str] = &[
    "Blizzard_PerksProgram_Bootstrap.lua",
    "Blizzard_PerksProgramElements.lua",
    "Blizzard_PerksProgramProducts.lua",
    "Blizzard_PerksProgramModel.lua",
    "Blizzard_PerksProgramFooter.lua",
    "Blizzard_PerksProgram.lua",
    "Blizzard_PerksProgramElements.xml",
    "Blizzard_PerksProgram.xml",
    "Localization.lua",
];

const REQUIRED_DEPS: &[&str] = &["Blizzard_Colors"];

const PUBLIC_MIXINS: &[&str] = &[
    "FrozenProductContainerMixin",
    "HeaderSortButtonMixin",
    "PerksModelSceneControlButtonMixin",
    "PerksProductPriceMixin",
    "PerksProgramAlteredFormButtonMixin",
    "PerksProgramButtonMixin",
    "PerksProgramCartDetailsListMixin",
    "PerksProgramCartScrollItemDetailsMixin",
    "PerksProgramCheckboxMixin",
    "PerksProgramClearCartButtonMixin",
    "PerksProgramCurrencyFrameMixin",
    "PerksProgramDisableableScrollItemMixin",
    "PerksProgramDividerFrameMixin",
    "PerksProgramErrorIndicatorMixin",
    "PerksProgramFooterFrameMixin",
    "PerksProgramFrozenProductButtonMixin",
    "PerksProgramItemDetailsListMixin",
    "PerksProgramMixin",
    "PerksProgramModelSceneContainerFrameMixin",
    "PerksProgramProductButtonMixin",
    "PerksProgramProductDetailsContainerMixin",
    "PerksProgramProductDetailsFrameMixin",
    "PerksProgramProductsFrameMixin",
    "PerksProgramPurchaseButtonMixin",
    "PerksProgramPurchaseCartButtonMixin",
    "PerksProgramPurchasePendingSpinnerMixin",
    "PerksProgramRefundButtonMixin",
    "PerksProgramScrollItemDetailsMixin",
    "PerksProgramSetDetailsListMixin",
    "PerksProgramSetItemDetailsScrollHeaderMixin",
    "PerksProgramSetScrollItemDetailsMixin",
    "PerksProgramShoppingCartMixin",
    "PerksProgramThemeContainerMixin",
    "PerksProgramToyDetailsFrameMixin",
    "PerksProgramTruncatedTextTooltipButtonMixin",
    "PerksProgramViewCartButtonMixin",
    "PerksRefundIconTooltipMixin",
    "ProductCartToggleButtonMixin",
    "RemoveFromCartItemButtonContainerMixin",
    "RemoveFromCartItemButtonMixin",
];

const PUBLIC_NAMESPACE_TABLES: &[&str] = &["PerksProgramUtil"];

const PUBLIC_NAMED_FRAMES: &[&str] = &["PerksProgramFrame", "PerksProgramTooltip"];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "PerksProgramCheckboxTemplate",
    "HeaderSortButtonTemplate",
    "ProductPurchaseCelebrateFXTemplate",
    "PerksProductPriceContainerTemplate",
    "PerksProductPriceContainerHugeTemplate",
    "PerksProgramProductButtonTemplate",
    "PerksProgramFrozenProductButtonTemplate",
    "PerksProgramButtonTemplate",
    "PerksProgramUIButtonTemplate",
    "PerksProgramDetailsFrameTemplate",
    "PerksProgramToyDetailsFrameTemplate",
    "PerksProgramItemDetailsScrollButtonTemplate",
    "PerksProgramSetItemDetailsScrollButtonTemplate",
    "PerksProgramSetItemDetailsScrollButtonWithHeaderTemplate",
    "PerksProgramSetItemDetailsScrollButtonWithFooterTemplate",
    "RemoveFromCartButtonTemplate",
    "PerksProgramCartItemDetailsScrollButtonTemplate",
    "PerksProgramSetDetailsScrollHeaderTemplate",
    "PerksModelSceneControlButtonTemplate",
];

fn load_full_game_ui_with_perks_program() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &perks_program_toc())
        .expect("explicit load_addon for Blizzard_PerksProgram succeeds");

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_perks_program_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&perks_program_dir()).expect("Blizzard_PerksProgram TOC resolves");
    assert_eq!(
        resolved,
        perks_program_toc(),
        "Blizzard_PerksProgram ships exactly one bare TOC — no `_Mainline.toc` variant. \
         Trader's Tender / Trading Post is a Mainline-only feature (added in patch 10.0.7) \
         but the TOC stayed bare because the addon predates the flavor-split convention"
    );

    let mainline = perks_program_dir().join("Blizzard_PerksProgram_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC is the canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_perks_program_toc_declares_load_on_demand_with_blizzard_colors_dep() {
    let toc = TocFile::from_file(&perks_program_toc()).expect("Blizzard_PerksProgram TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` so `is_load_on_demand()` returns true — the \
         Trader's Tender / Trading Post UI is a heavy panel with 8 source files and \
         ~250KB of Lua/XML; lazy-loading defers the cost until the player explicitly \
         opens the Trading Post UI via `LoadAddOn('Blizzard_PerksProgram')` from the \
         in-game GameMenuFrame entry point"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Default-Game-only `allows_screen` at src/toc.rs:311 returns true for \
         ScreenKind::Game when AllowLoad is omitted"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Omitted `## AllowLoad:` must NOT enable {screen:?} — Trading Post is an \
             in-world player-facing UI; glue screens cannot trigger it"
        );
    }

    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS,
        "TOC must declare exactly 1 RequiredDep (`Blizzard_Colors`) — many product/cart/ \
         currency/refund mixins call into ColorManager / quality-color tables to tint \
         price labels, item-quality borders, and refund-status indicators. The dep is \
         hard rather than optional because the elements XML parses mixin attribute \
         references at load time and the mixins reach for ColorManager fields during \
         OnLoad. `dependencies()` at src/toc.rs:210-217 reads `Dependencies` here"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — Trading Post UI is a stateless mirror of \
         server-authoritative state fetched via C_PerksProgram each open. Cart \
         contents, vendor inventory, frozen items, and currency are all re-pulled \
         on every panel show"
    );
}

#[test]
fn blizzard_perks_program_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(perks_program_toc())
        .expect("Blizzard_PerksProgram TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard_PerksProgram"),
        "TOC must declare `## Title: Blizzard_PerksProgram` exactly. UNUSUAL: the title \
         uses the underscore-namespace spelling rather than the space-and-prose form \
         (e.g. `Blizzard Perks Program`); minority pattern, suggests the addon was \
         scaffolded from a code template rather than hand-typed"
    );
    assert!(
        raw.contains("## Version: 1.0"),
        "TOC must declare `## Version: 1.0` exactly — UNUSUAL: stub version that nobody \
         updates despite the file having seen multiple expansions of changes since 10.0.7; \
         matches the Blizzard_PartyPoseUI and Blizzard_OrderHallUI pattern"
    );
    assert!(
        raw.contains("## ShowInAddOnList: 0"),
        "TOC must declare `## ShowInAddOnList: 0` exactly — UNUSUAL and FIRST seen in \
         this audit. The key tells the engine to HIDE this addon from the user-facing \
         AddOns panel UI, so the player cannot manually enable/disable it. The addon is \
         lazy-loaded by the Trading Post entry-point flow exclusively; surfacing it as a \
         togglable entry would let the user disable a UI that the engine assumes always \
         exists when the entry-point fires LoadAddOn"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the canonical retail spelling \
         for explicit lazy loading"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_Colors"),
        "TOC must declare `## Dependencies: Blizzard_Colors` exactly (singular dep, no \
         comma)"
    );
    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare `## AllowLoad:` — Game-only is the default behavior when \
         the key is omitted"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless mirror"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — UNUSUAL omission compared to most \
         Blizzard-shipped addons, which ship `## Author: Blizzard Entertainment`. \
         Together with the stub version and `ShowInAddOnList: 0`, the metadata profile \
         points to an internally-only-loadable utility addon never meant to be \
         user-visible"
    );
}

#[test]
fn blizzard_perks_program_toc_lists_bootstrap_then_eight_files() {
    let toc = TocFile::from_file(&perks_program_toc()).expect("Blizzard_PerksProgram TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PERKS_PROGRAM_TOC_FILES,
        "Retail 12.1.0.69497 lists its bootstrap first, followed by five Lua files, two XML \
         files, and Localization.lua. The non-bootstrap source order remains Elements, \
         Products, Model, Footer, main UI, Elements XML, main XML, and Localization"
    );
}

#[test]
fn blizzard_perks_program_does_not_appear_in_eager_discovery_for_any_screen() {
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
            .any(|(name, _)| name == "Blizzard_PerksProgram");
        assert!(
            !found,
            "Blizzard_PerksProgram must NOT appear in eager discovery for {screen:?} — \
             LoadOnDemand: 1 keeps the addon in the lod_pool at \
             src/loader/mod.rs:530-534, not in the eager-discovery set"
        );
    }
}

#[test]
fn blizzard_perks_program_appears_in_full_addon_inventory() {
    let ui = blizzard_ui_dir();
    let inventory = discover_all_blizzard_addons(&ui);
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PerksProgram");
    assert!(
        found,
        "Blizzard_PerksProgram must appear in `discover_all_blizzard_addons` — even \
         though `## ShowInAddOnList: 0` hides it from the user-facing addon panel, the \
         simulator's full inventory is a structural listing of every parseable TOC; \
         hiding from one UI surface does not remove the addon from disk"
    );
}

#[test]
fn blizzard_perks_program_loads_without_addon_specific_lua_errors() {
    let env = load_full_game_ui_with_perks_program();

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PerksProgram")
                || message.contains("PerksProgram")
                || message.contains("PerksProductPrice")
                || message.contains("PerksRefund")
                || message.contains("PerksModelScene")
        })
        .filter(|message| {
            let touches_model_scene_gap = message.contains("Blizzard_PerksProgramModel.lua")
                || message.contains("'fanfareActor'")
                || message.contains("'playerActor'")
                || message.contains("PerksProgramFrame: not a function")
                || message.contains("expected string, got number at argument 2");
            !touches_model_scene_gap
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PerksProgram emitted addon-specific Lua errors during load (excluding \
         documented 3D ModelScene permanent gap — CLAUDE.md flags Model/ModelScene/ \
         PlayerModel/DressUpModel as intentional ~38-stub permanent gaps. \
         PerksProgramModelSceneContainerFrameMixin:OnLoad calls \
         CelebrateModelScene:GetActorByTag(DEFAULT_FANFARE_ACTOR_TAG) and dereferences \
         the result without a nil guard at line 348; the simulator's actor list is \
         empty so the deref nil-crashes. PartyPoseUI dodges the same pattern by \
         wrapping every actor deref in `if (actor) then ...`. The cascading \
         `[OnLoad] PerksProgramFrame: not a function` and \
         `[OnLoad] ?: expected string, got number at argument 2` come out of the same \
         OnLoad chain after the actor crash leaves the frame in a partially-built \
         state):\n  {}",
        load_errors.join("\n  ")
    );
}

#[test]
fn blizzard_perks_program_is_addon_loaded_after_explicit_load() {
    let env = load_full_game_ui_with_perks_program();

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PerksProgram')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PerksProgram') must return true after \
         explicit load_addon — the addon is LoadOnDemand so only the explicit load path \
         makes IsAddOnLoaded report true"
    );
}

#[test]
fn blizzard_perks_program_publishes_forty_mixin_tables() {
    let env = load_full_game_ui_with_perks_program();

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_PerksProgram declares 40 \
             mixins across 5 Lua files: 31 in Elements (the cart/product/details \
             building blocks), 3 in Products (FrozenProductContainer / \
             PerksProgramCurrencyFrame / PerksProgramProductsFrame), 2 in Model \
             (AlteredFormButton extending SelectableButtonMixin / \
             ModelSceneContainerFrame), 2 in Footer (FooterFrame + ErrorIndicator), and \
             2 in main (PerksProgramMixin owns the panel; PerksProgramThemeContainerMixin \
             tints sub-frames per theme). Each mixin is referenced by an XML \
             `mixin=\"...\"` attribute or by CreateFromMixins from a derived mixin"
        );
    }

    assert_eq!(
        PUBLIC_MIXINS.len(),
        40,
        "PUBLIC_MIXINS must contain exactly 40 entries — the count is asserted as a \
         pin so adding/removing mixins via vendor TAG bumps surfaces here"
    );
}

#[test]
fn blizzard_perks_program_publishes_perks_program_util_namespace() {
    let env = load_full_game_ui_with_perks_program();

    for namespace in PUBLIC_NAMESPACE_TABLES {
        let kind: String = env
            .eval(&format!("return type(_G.{namespace})"))
            .unwrap_or_else(|err| panic!("type(_G.{namespace}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{namespace} must publish as a table — \
             Blizzard_PerksProgramElements.lua declares the PerksProgramUtil namespace \
             with the single function \
             `PerksProgramUtil.ItemAppearancesHaveSameCategory(itemModifiedAppearanceIDs)` \
             — the public helper that walks a list of itemModifiedAppearanceIDs and \
             returns true when every appearance shares the same C_TransmogCollection \
             category, used by the cart-summary and set-details flows to decide whether \
             to render a single grouped row vs separate per-slot rows"
        );
    }

    let category_helper_present: bool = env
        .eval("return type(PerksProgramUtil.ItemAppearancesHaveSameCategory) == 'function'")
        .expect("ItemAppearancesHaveSameCategory probe succeeds");
    assert!(
        category_helper_present,
        "PerksProgramUtil.ItemAppearancesHaveSameCategory must be a function — it is \
         the only public method on the namespace"
    );
}

#[test]
fn blizzard_perks_program_creates_named_frames() {
    let env = load_full_game_ui_with_perks_program();

    for frame in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a frame userdata (FrameRef reports as \
             `'table'` via the custom __type metamethod). Blizzard_PerksProgram.xml \
             ships exactly 2 named non-virtual frames: PerksProgramFrame (the panel root, \
             inherits DefaultScaleFrame, mixin=PerksProgramMixin, toplevel=true, \
             setAllPoints=true, hidden=true at load) and PerksProgramTooltip (the \
             dedicated GameTooltip child inheriting GameTooltipTemplate + \
             DefaultScaleFrame for hover info on cart items / vendor products / refund \
             confirmation prompts; pinned to its own ignoreParentScale so the tooltip \
             text does not scale with the panel)"
        );
    }
}

#[test]
fn blizzard_perks_program_does_not_leak_virtual_templates_to_globals() {
    let env = load_full_game_ui_with_perks_program();

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — Blizzard_PerksProgram XML ships 19 virtual \
             templates that live in the template registry, NOT in `_G`. Leaking would \
             let consumer addons mutate the template definition and break every \
             existing instance. The 19 templates are concentrated in Elements.xml: \
             checkboxes, sort headers, product buttons (normal + frozen), the \
             PerksProgramButton inheriting SharedButtonLargeTemplate (the gold \
             Trading-Post-style buttons), price containers (regular + huge), details \
             frames (toy + generic), 5 scroll-button templates layered via inheritance \
             (Item → SetItem → SetItemWithHeader/SetItemWithFooter, plus CartItem), the \
             remove-from-cart wrapper, the set-details scroll header, the model-scene \
             control button, and the celebration FX overlay"
        );
    }

    assert_eq!(
        VIRTUAL_TEMPLATES_NOT_IN_GLOBALS.len(),
        19,
        "VIRTUAL_TEMPLATES_NOT_IN_GLOBALS must contain exactly 19 entries — the count \
         is asserted as a pin so adding/removing templates via vendor TAG bumps \
         surfaces here"
    );
}
