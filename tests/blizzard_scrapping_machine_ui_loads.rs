use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn scrapping_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ScrappingMachineUI")
}

fn scrapping_toc() -> PathBuf {
    scrapping_dir().join("Blizzard_ScrappingMachineUI.toc")
}

const TOC_FILES: &[&str] = &[
    "Blizzard_ScrappingMachineUI_Bootstrap.lua",
    "Blizzard_ScrappingMachineUI.xml",
];

const SCRAPPING_MACHINE_MIXIN_METHODS: &[&str] = &[
    "SetupScrapButtonPool",
    "ClearAllScrapButtons",
    "ScrapItems",
    "UpdateScrapButtonState",
    "OnLoad",
    "OnShow",
    "OnEvent",
    "PlayItemChangeSounds",
    "CloseScrappingMachine",
    "OnHide",
];

const ITEM_SLOT_MIXIN_METHODS: &[&str] = &[
    "RefreshIcon",
    "ClearSlot",
    "Clear",
    "OnLoad",
    "OnEvent",
    "OnClick",
    "OnDragStart",
    "OnReceiveDrag",
    "OnMouseEnter",
    "OnMouseLeave",
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
        find_toc_file(&scrapping_dir()).expect("Blizzard_ScrappingMachineUI TOC resolves");
    assert_eq!(
        resolved,
        scrapping_toc(),
        "Blizzard_ScrappingMachineUI ships exactly one bare TOC — no flavor split. \
         The scrapping machine was a BfA-era system tied to the C_ScrappingMachineUI \
         server API, mainline-only by virtue of that backend's absence on classic"
    );
}

#[test]
fn toc_declares_minimal_load_on_demand_with_zero_dependencies() {
    let toc = TocFile::from_file(&scrapping_toc()).expect("Blizzard_ScrappingMachineUI TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` so `is_load_on_demand()` returns true. \
         The scrapping machine UI is opened on-demand when the player interacts with \
         a scrapping NPC; no other addon ever needs the frame template at parse time"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.dependencies().is_empty(),
        "Zero `## Dependencies:` — the TOC is famously minimal. The Lua chunk reaches \
         only well-bootstrapped FrameXML primitives (UIPanelWindows, CreateFramePool, \
         UnitFactionGroup, GameTooltip, OpenAllBags, ItemButtonUtil, Item:CreateFromItemLocation, \
         SetItemButtonQuality, ButtonFrameTemplate, MagicButtonTemplate, PlaySound, \
         SOUNDKIT, PlayerGetTimerunningSeasonID), all of which load before any addon \
         via Blizzard_FrameXML / Blizzard_SharedXML core bootstrap. Declaring no \
         hard dep means the addon trusts the implicit FrameXML-loads-first ordering"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — pure leaf"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — scrapping state is server-authoritative; the pending \
         scrap list re-fetches from C_ScrappingMachineUI on every UI open"
    );
}

#[test]
fn toc_declares_metadata_in_raw_bytes_with_carried_over_blizzard_typo() {
    let raw = std::fs::read_to_string(scrapping_toc())
        .expect("Blizzard_ScrappingMachineUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Scraping Machine UI"),
        "TOC must declare `## Title: Blizzard Scraping Machine UI` exactly — note the \
         typo: `Scraping` (one `p`) in the title vs `Scrapping` (two `p`s) in the \
         folder name and `ScrappingMachine*` in every Lua identifier. The typo is \
         carried verbatim from Blizzard's source and predates the systems addon \
         being moved out of Blizzard_FrameXML; pinning it guards against a \
         well-intentioned 'fix' that would change the displayed AddOns-list label"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly"
    );
    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare `## AllowLoad` — falls through to None arm at \
         src/toc.rs:311 returning Game-only. Glue screens have no scrapping NPC"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare any `## Dependencies` keys — the addon is a pure leaf"
    );
    assert!(
        !raw.contains("## OptionalDep"),
        "TOC must NOT declare any `## OptionalDep` keys"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables` keys"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — relies on implicit Blizzard ownership"
    );
}

#[test]
fn toc_lists_bootstrap_then_xml_with_lua_loaded_via_script_directive() {
    let toc = TocFile::from_file(&scrapping_toc()).expect("Blizzard_ScrappingMachineUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "Retail 12.1.0.69497 lists the ScrappingMachine bootstrap before the XML file"
    );
}

#[test]
fn allows_screen_returns_true_only_for_game_when_allowload_absent() {
    let toc = TocFile::from_file(&scrapping_toc()).expect("Blizzard_ScrappingMachineUI TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Absent `## AllowLoad` falls through to None arm at src/toc.rs:311 returning \
         Game-only"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must reject the scrapping UI — no NPC \
             interaction or C_ScrappingMachineUI backend on glue screens"
        );
    }
}

#[test]
fn excluded_from_eager_discovery_due_to_load_on_demand() {
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
            .any(|(name, _)| name == "Blizzard_ScrappingMachineUI");
        assert!(
            !found,
            "Blizzard_ScrappingMachineUI must NOT appear in eager discovery on \
             screen {screen:?} — `## LoadOnDemand: 1` puts it in the LoD pool, and \
             no other Blizzard addon hard-depends on it (ScrappingMachineFrame is \
             opened by C_ScrappingMachineUI events from the server, not by template \
             inheritance), so the LoD pool never promotes it to the eager set"
        );
    }
}

#[test]
fn root_directory_holds_bootstrap_lua_and_xml_next_to_toc() {
    let dir = scrapping_dir();
    assert!(dir.join("Blizzard_ScrappingMachineUI.lua").is_file());
    assert!(dir.join("Blizzard_ScrappingMachineUI.xml").is_file());
    assert!(dir.join("Blizzard_ScrappingMachineUI.toc").is_file());
    assert!(dir
        .join("Blizzard_ScrappingMachineUI_Bootstrap.lua")
        .is_file());

    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("read addon dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        4,
        "Retail 12.1.0.69497 adds a bootstrap Lua beside the TOC, XML, and embedded-script Lua: {entries:?}"
    );
}

#[test]
fn xml_embeds_lua_via_script_directive_at_top_of_ui_root() {
    let body = std::fs::read_to_string(scrapping_dir().join("Blizzard_ScrappingMachineUI.xml"))
        .expect("read xml");
    assert!(
        body.contains(r#"<Script file="Blizzard_ScrappingMachineUI.lua"/>"#),
        "XML must embed `<Script file=\"Blizzard_ScrappingMachineUI.lua\"/>` as a \
         child of `<Ui>` so the Lua chunk loads at XML parse time. The TOC lists \
         only the XML file; Lua reaches the addon table exclusively through this \
         embedded-script mechanism"
    );
}

prefork_full_ui_case! {
fn loads_without_lua_errors_when_explicitly_loaded(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &scrapping_toc())
        .expect("Blizzard_ScrappingMachineUI should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ScrappingMachineUI emitted Lua errors during explicit load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_transitions_false_to_true_across_explicit_load(env: &WowLuaEnv) {

    let loaded_before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ScrappingMachineUI')")
        .expect("IsAddOnLoaded probe (pre-load) succeeds");
    assert!(
        !loaded_before,
        "C_AddOns.IsAddOnLoaded('Blizzard_ScrappingMachineUI') must return false \
         before explicit LoadAddOn — the eager sweep skips LoadOnDemand addons"
    );

    load_addon(&env.loader_env(), &scrapping_toc())
        .expect("Blizzard_ScrappingMachineUI should load");

    let loaded_after: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_ScrappingMachineUI')")
        .expect("IsAddOnLoaded probe (post-load) succeeds");
    assert!(loaded_after);
}
}

prefork_full_ui_case! {
fn publishes_scrapping_machine_mixin_with_ten_methods(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &scrapping_toc()).expect("ScrappingMachineUI loads");

    let kind: String = env
        .eval("return type(_G.ScrappingMachineMixin)")
        .expect("ScrappingMachineMixin probe");
    assert_eq!(
        kind, "table",
        "_G.ScrappingMachineMixin must publish as a table — the master frame mixin \
         declared at line 2 of the Lua chunk as `ScrappingMachineMixin = {{}}` then \
         populated method-by-method. NOT created via CreateFromMixins; the addon \
         predates mixin inheritance and inherits XML-side via \
         `inherits=\"ButtonFrameTemplate\"`"
    );

    for method in SCRAPPING_MACHINE_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(ScrappingMachineMixin.{method})"))
            .unwrap_or_else(|err| panic!("type(ScrappingMachineMixin.{method}) failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "ScrappingMachineMixin.{method} must be a function — the mixin publishes \
             10 methods: pool setup (SetupScrapButtonPool builds a 3×3 grid via \
             CreateFramePool), pool teardown (ClearAllScrapButtons enumerates active \
             buttons via :EnumerateActive() and calls :ClearSlot()), action methods \
             (ScrapItems wraps C_ScrappingMachineUI.ScrapItems, \
             UpdateScrapButtonState gates the SCRAP button on \
             C_ScrappingMachineUI.HasScrappableItems), 4 lifecycle handlers (OnLoad \
             builds the pool + sets faction-keyed background atlas + portrait + \
             title; OnShow registers 8 events incl. BAG_UPDATE / \
             SCRAPPING_MACHINE_PENDING_ITEM_CHANGED / SCRAPPING_MACHINE_ITEM_ADDED / \
             SCRAPPING_MACHINE_ITEM_REMOVED / UPDATE_TRADESKILL_CAST_STOPPED + 3 \
             unit-events for player spell-cast tracking, opens all bags; OnEvent \
             dispatches the 8 events; OnHide unregisters and closes), \
             PlayItemChangeSounds (timerunning-season-keyed sound dispatch), \
             CloseScrappingMachine (clears slots and calls \
             C_ScrappingMachineUI.CloseScrappingMachine)"
        );
    }
}
}

prefork_full_ui_case! {
fn publishes_item_slot_mixin_with_ten_methods(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &scrapping_toc()).expect("ScrappingMachineUI loads");

    let kind: String = env
        .eval("return type(_G.ScrappingMachineItemSlotMixin)")
        .expect("ScrappingMachineItemSlotMixin probe");
    assert_eq!(
        kind, "table",
        "_G.ScrappingMachineItemSlotMixin must publish as a table — the per-slot \
         button mixin attached to the ScrappingMachineItemSlot virtual template"
    );

    for method in ITEM_SLOT_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!(
                "return type(ScrappingMachineItemSlotMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("type(ScrappingMachineItemSlotMixin.{method}) failed: {err}")
            });
        assert_eq!(
            method_kind, "function",
            "ScrappingMachineItemSlotMixin.{method} must be a function — 10 methods: \
             RefreshIcon (resolves item via \
             C_ScrappingMachineUI.GetCurrentPendingScrapItemLocationByIndex, builds \
             Item:CreateFromItemLocation, defers icon/quality binding via \
             :ContinueWithCancelOnItemLoad), ClearSlot (hides Icon/IconBorder/\
             IconOverlay textures and clears itemLink), Clear (full reset incl. \
             cancelling itemDataLoadedCancelFunc deferred-load handle), 7 script \
             handlers (OnLoad registers Left+Right click + Left drag + \
             SCRAPPING_MACHINE_PENDING_ITEM_CHANGED event; OnEvent calls RefreshIcon \
             then re-anchors GameTooltip if owner; OnClick clears local state + \
             calls C_ScrappingMachineUI.RemoveItemToScrap + DropPendingScrapItemFromCursor; \
             OnDragStart and OnReceiveDrag both delegate to OnClick; OnMouseEnter \
             shows GameTooltip with itemLink hyperlink; OnMouseLeave hides tooltip)"
        );
    }
}
}

prefork_full_ui_case! {
fn registers_scrapping_machine_frame_in_uipanelwindows_at_module_load(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &scrapping_toc()).expect("ScrappingMachineUI loads");

    let area: String = env
        .eval(
            "return UIPanelWindows.ScrappingMachineFrame and \
             UIPanelWindows.ScrappingMachineFrame.area or 'missing'",
        )
        .expect("UIPanelWindows.ScrappingMachineFrame.area probe succeeds");
    assert_eq!(
        area, "left",
        "UIPanelWindows[\"ScrappingMachineFrame\"].area must equal `left` AFTER \
         OnLoad runs. The Lua chunk sets the entry TWICE: line 1 at module top with \
         `area = \"center\"`, then OnLoad (line 42) overwrites it via \
         `UIPanelWindows[self:GetName()] = {{ area = \"left\", ... }}`. After the \
         eager startup OnLoad fires (the named ScrappingMachineFrame is created on \
         addon load, OnLoad runs synchronously), so the module-load-time `center` is \
         shadowed by the OnLoad-time `left`"
    );

    let pushable: f64 = env
        .eval("return UIPanelWindows.ScrappingMachineFrame.pushable")
        .expect("pushable probe");
    assert_eq!(pushable, 3.0, "pushable=3 lets bag/character frames stack");

    let has_failed_func: bool = env
        .eval("return type(UIPanelWindows.ScrappingMachineFrame.showFailedFunc) == 'function'")
        .expect("showFailedFunc probe");
    assert!(
        has_failed_func,
        "showFailedFunc must reference C_ScrappingMachineUI.CloseScrappingMachine so \
         a failed open never leaves dangling NPC state on the server"
    );
}
}

prefork_full_ui_case! {
fn publishes_named_non_virtual_frame_at_global_scope(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &scrapping_toc()).expect("ScrappingMachineUI loads");

    let frame_kind: String = env
        .eval("return type(_G.ScrappingMachineFrame)")
        .expect("ScrappingMachineFrame probe");
    assert_eq!(
        frame_kind, "table",
        "_G.ScrappingMachineFrame must publish as a table — the only top-level \
         non-virtual frame, declared at line 45 of the XML with toplevel=\"true\" \
         parent=\"UIParent\" inherits=\"ButtonFrameTemplate\" \
         mixin=\"ScrappingMachineMixin\" hidden=\"true\" dimensions 333×278. \
         Carries OnLoad/OnEvent/OnShow/OnHide script bindings"
    );

    let hidden: bool = env
        .eval("return ScrappingMachineFrame:IsShown() == false")
        .expect("IsShown probe");
    assert!(
        hidden,
        "ScrappingMachineFrame must start hidden — `hidden=\"true\"` in XML, opened \
         only via ShowUIPanel(ScrappingMachineFrame) when the player interacts with \
         a scrapping NPC"
    );
}
}

prefork_full_ui_case! {
fn does_not_leak_virtual_template_or_module_locals_to_globals(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &scrapping_toc()).expect("ScrappingMachineUI loads");

    let template_kind: String = env
        .eval("return type(_G.ScrappingMachineItemSlot)")
        .expect("ScrappingMachineItemSlot probe");
    assert_eq!(
        template_kind, "nil",
        "_G.ScrappingMachineItemSlot must be nil — the `<Button \
         name=\"ScrappingMachineItemSlot\" virtual=\"true\">` template lives in the \
         template registry, NOT in `_G`. Pool-instantiated buttons created via \
         CreateFramePool(\"BUTTON\", parent, \"ScrappingMachineItemSlot\") are \
         anonymous; they reach the user only via parentKey on their parent"
    );

    let module_local_kind: String = env
        .eval("return type(_G.itemChangeSoundsByTimerunningID)")
        .expect("itemChangeSoundsByTimerunningID probe");
    assert_eq!(
        module_local_kind, "nil",
        "_G.itemChangeSoundsByTimerunningID must be nil — the timerunning-season → \
         soundkit lookup table at line 107 of the Lua chunk is declared with \
         `local` scope. Leaking it would let consumers spuriously add sound mappings \
         for arbitrary season IDs"
    );
}
}

prefork_full_ui_case! {
fn frame_pool_seeds_nine_scrap_button_slots_after_onload_grid(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &scrapping_toc()).expect("ScrappingMachineUI loads");

    let active_count: f64 = env
        .eval(
            "local count = 0; \
             if ScrappingMachineFrame and ScrappingMachineFrame.ItemSlots and \
                ScrappingMachineFrame.ItemSlots.scrapButtons then \
                 for _ in ScrappingMachineFrame.ItemSlots.scrapButtons:EnumerateActive() do \
                     count = count + 1 \
                 end \
             end; \
             return count",
        )
        .expect("scrapButtons EnumerateActive count probe");
    assert_eq!(
        active_count, 9.0,
        "ScrappingMachineFrame.ItemSlots.scrapButtons must hold 9 active slots after \
         OnLoad runs SetupScrapButtonPool — the inner loop iterates `columnNum=3` × \
         `rowNum=3` and calls `:Acquire()` then `:Show()` on each button. The 9-slot \
         grid is the BfA-era hardcoded scrapping capacity, the same number of pending \
         items C_ScrappingMachineUI.ValidateScrappingList expects to walk"
    );
}
}
