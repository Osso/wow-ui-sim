#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addons_for_screen, find_toc_file, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn obliterum_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ObliterumUI")
}

fn obliterum_toc() -> PathBuf {
    obliterum_dir().join("Blizzard_ObliterumUI.toc")
}

const OBLITERUM_TOC_FILES: &[&str] = &[
    "Blizzard_ObliterumUI_Bootstrap.lua",
    "Blizzard_ObliterumUI.xml",
];

const PUBLIC_MIXINS: &[&str] = &["ObliterumForgeMixin", "ObliterumForgeItemSlotMixin"];

const NAMED_FRAMES: &[&str] = &["ObliterumForgeFrame"];

fn load_obliterum_ui_after_fork(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &obliterum_toc())
        .expect("Blizzard_ObliterumUI load_addon succeeds after eager Game-screen sweep");
}

#[test]
fn blizzard_obliterum_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&obliterum_dir()).expect("Blizzard_ObliterumUI TOC resolves");
    assert_eq!(
        resolved,
        obliterum_toc(),
        "Blizzard_ObliterumUI ships exactly one bare TOC — no `_Mainline.toc` variant. The \
         Obliterum Forge is Legion-era content (patch 7.0 cosmetic-token converter) that no \
         longer functions on retail (the forge NPCs were removed in BfA), but the addon \
         remains in the codebase for archive completeness; the bare-TOC slot resolves at \
         src/loader/mod.rs:67-92 after the `_Mainline.toc` lookup misses"
    );

    let mainline = obliterum_dir().join("Blizzard_ObliterumUI_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — flavor split is unnecessary because the \
         addon is essentially dead code preserved for nostalgia / data-driven NPC dialog \
         compatibility. The bare TOC is the single canonical entry point",
        mainline.display()
    );
}

#[test]
fn blizzard_obliterum_toc_declares_load_on_demand_with_no_dependencies() {
    let toc = TocFile::from_file(&obliterum_toc()).expect("Blizzard_ObliterumUI TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` — the forge UI is summoned by an NPC gossip \
         interaction (talking to the Obliterum Forge anvil) via UIPanelWindows + \
         OBLITERUM_FORGE_PENDING_ITEM_CHANGED event, so eager-loading would waste resources \
         on every login. The `## LoadOnDemand: 1` route at src/loader/mod.rs:530-534 keeps \
         the addon out of the eager Game-screen discovery sweep until something explicitly \
         calls load_addon"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.dependencies().is_empty(),
        "Zero `## RequiredDep:` / `## Dependencies:` — the forge UI has NO hard dependencies. \
         It inherits ButtonFrameTemplate / MagicButtonTemplate from foundational SharedXML / \
         FrameXML (always loaded) and calls C_TradeSkillUI / C_Item from built-in C_* \
         namespaces"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons. Every surface the forge UI touches \
         is either foundational FrameXML (UIPanelWindows, FrameUtil.RegisterFrameForUnitEvents, \
         GameTooltip) or a built-in C_* namespace"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — the forge UI is purely transactional (slot in an item, click \
         Obliterate, cast the spell). No persistent state to mirror across sessions; the \
         pending-item state is server-side and reissued via OBLITERUM_FORGE_PENDING_ITEM_CHANGED"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC OMITS `## AllowLoad:` so `allows_screen` at src/toc.rs:311 returns true for the \
         Game screen by default — the omitted-key default is Game-only (NOT all screens), \
         which matches the addon's purpose: the forge can only be opened by interacting with \
         an in-world NPC anvil"
    );
}

#[test]
fn blizzard_obliterum_toc_declares_load_on_demand_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(obliterum_toc()).expect("Blizzard_ObliterumUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Obliterum Forge UI"),
        "TOC must declare `## Title: Blizzard Obliterum Forge UI` exactly. The space-and-prose \
         title spelling (rather than the underscore-separated `Blizzard_ObliterumUI`) is the \
         human-readable label shown in the addon-manager UI"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the explicit `1` form (rather than \
         omitting the key) is the canonical retail spelling for LoD addons. The numeric value \
         is parsed via `is_load_on_demand` at src/toc.rs returning true for any non-zero value"
    );
    assert!(
        !raw.contains("## RequiredDep") && !raw.contains("## Dependencies"),
        "TOC must NOT declare any dependency keys — zero RequiredDep / Dependencies / \
         RequiredDeps. The forge UI is self-contained on foundational FrameXML"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure transactional UI, no \
         persistence"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` — the absence is the canonical retail spelling \
         for Game-only addons (default), distinct from explicit `## AllowLoad: Both` /  \
         `## AllowLoad: Game` forms"
    );
}

#[test]
fn blizzard_obliterum_toc_lists_bootstrap_then_xml() {
    let toc = TocFile::from_file(&obliterum_toc()).expect("Blizzard_ObliterumUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, OBLITERUM_TOC_FILES,
        "TOC body must list exactly 1 file: Blizzard_ObliterumUI.xml. The Lua file \
         (Blizzard_ObliterumUI.lua) is NOT listed in the TOC body — it is loaded indirectly \
         via the `<Script file=\"Blizzard_ObliterumUI.lua\"/>` element at the top of the XML \
         (line 3). This is the inverse-load pattern: TOC → XML → Lua, rather than the more \
         common TOC → Lua + XML side-by-side. The simulator's XML parser handles `<Script \
         file>` by enqueueing the Lua file for execution within the addon's load context"
    );
}

#[test]
fn blizzard_obliterum_does_not_appear_in_eager_discovery_for_any_screen() {
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
            .any(|(name, _)| name == "Blizzard_ObliterumUI");
        assert!(
            !found,
            "Blizzard_ObliterumUI must NOT auto-discover on screen {screen:?} — `## \
             LoadOnDemand: 1` excludes the addon from every eager-discovery sweep. The LoD \
             addon must be summoned via an explicit `load_addon` call (e.g. \
             `C_AddOns.LoadAddOn('Blizzard_ObliterumUI')` triggered by gossip interaction)"
        );
    }
}

#[test]
fn blizzard_obliterum_appears_in_full_addon_inventory() {
    let ui = blizzard_ui_dir();
    let all_addons = discover_all_blizzard_addons(&ui);
    let found = all_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_ObliterumUI");
    assert!(
        found,
        "Blizzard_ObliterumUI must appear in `discover_all_blizzard_addons` — the function \
         walks the full Interface/BlizzardUI directory and returns every addon directory \
         that contains a parseable TOC, regardless of LoadOnDemand or AllowLoad. This is the \
         inventory probe used by the addon-manager UI; LoD addons must be visible here so \
         users can manually enable/disable them"
    );
}

prefork_full_ui_case! {
    fn blizzard_obliterum_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_obliterum_ui_after_fork(env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_ObliterumUI")
                || message.contains("ObliterumForgeFrame")
                || message.contains("ObliterumForgeMixin")
                || message.contains("ObliterumForgeItemSlotMixin")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ObliterumUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
    fn blizzard_obliterum_is_addon_loaded_after_explicit_load(env: &WowLuaEnv) {
    load_obliterum_ui_after_fork(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ObliterumUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_ObliterumUI') must return true after the explicit \
         load_addon call — the LoD-routed addon is bookkept by the simulator's addon \
         registry once load_addon completes"
    );
}
}

prefork_full_ui_case! {
    fn blizzard_obliterum_publishes_two_mixin_tables(env: &WowLuaEnv) {
    load_obliterum_ui_after_fork(env);

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_ObliterumUI.lua declares 2 mixins \
             at module top: ObliterumForgeMixin (line 9, owns OnLoad / OnEvent / OnShow / \
             OnHide / ObliterateItem / UpdateObliterateButtonState — the parent frame's \
             event-driven lifecycle and Obliterate-button enable/disable logic), \
             ObliterumForgeItemSlotMixin (line 59, owns OnLoad / OnEvent / RefreshIcon / \
             ClearSlot / OnClick / OnDragStart / OnReceiveDrag / OnMouseEnter / OnMouseLeave \
             — the item-slot button's drag-drop + tooltip + click-to-clear behavior). Each \
             mixin is referenced by the XML's `mixin=\"...\"` attribute so the framework \
             copies its keys onto the corresponding frame at instantiation"
        );
    }
}
}

prefork_full_ui_case! {
    fn blizzard_obliterum_creates_named_frame_after_load(env: &WowLuaEnv) {
    load_obliterum_ui_after_fork(env);

    for frame_name in NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame_name})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame_name}) probe failed: {err}"));
        assert!(
            kind == "table" || kind == "userdata",
            "_G.{frame_name} must be a frame (table or userdata) — the named non-virtual \
             `<Frame name=\"ObliterumForgeFrame\">` at Blizzard_ObliterumUI.xml line 5 \
             instantiates immediately at XML parse time and publishes to _G via the \
             standard frame-name binding. It inherits `ButtonFrameTemplate`, sets \
             `parent=\"UIParent\"`, `toplevel=\"true\"`, `enableMouse=\"true\"`, mixin = \
             ObliterumForgeMixin, hidden=\"true\". Got type {kind} for {frame_name}"
        );
    }
}
}

prefork_full_ui_case! {
    fn blizzard_obliterum_registers_ui_panel_window_entry(env: &WowLuaEnv) {
    load_obliterum_ui_after_fork(env);

    let entry_kind: String = env
        .eval("return type(_G.UIPanelWindows.ObliterumForgeFrame)")
        .expect("UIPanelWindows.ObliterumForgeFrame probe succeeds");
    assert_eq!(
        entry_kind, "table",
        "UIPanelWindows.ObliterumForgeFrame must publish as a table — \
         Blizzard_ObliterumUI.lua line 1 registers the entry: `UIPanelWindows[\
         \"ObliterumForgeFrame\"] = {{ area = \"left\", pushable = 3, showFailedFunc = \
         C_TradeSkillUI.CloseObliterumForge }}`. The UIPanelWindows table is consumed by \
         FrameXML's UIParent panel-management system to position / stack / dismiss the frame \
         when other panels open. The forge gets `area = \"left\"` (the standard left-side \
         panel slot, shared with the spellbook / talent UI), `pushable = 3` (priority for \
         the multi-panel push-stack), and a `showFailedFunc` callback that calls \
         C_TradeSkillUI.CloseObliterumForge to inform the server when the panel cannot be \
         shown"
    );

    let area: String = env
        .eval("return UIPanelWindows.ObliterumForgeFrame.area")
        .expect("UIPanelWindows.ObliterumForgeFrame.area probe succeeds");
    assert_eq!(
        area, "left",
        "UIPanelWindows.ObliterumForgeFrame.area must equal `left` — the forge competes for \
         the left-side panel slot with other crafting/trade-skill UIs"
    );

    let pushable: i64 = env
        .eval("return UIPanelWindows.ObliterumForgeFrame.pushable")
        .expect("UIPanelWindows.ObliterumForgeFrame.pushable probe succeeds");
    assert_eq!(
        pushable, 3,
        "UIPanelWindows.ObliterumForgeFrame.pushable must equal 3 — the integer encodes the \
         panel's stacking priority within the left-area push-stack"
    );
}
}
