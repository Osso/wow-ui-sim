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

fn runeforge_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_RuneforgeUI")
}

fn runeforge_toc() -> PathBuf {
    runeforge_dir().join("Blizzard_RuneforgeUI.toc")
}

const RUNEFORGE_TOC_FILES: &[&str] = &[
    "Blizzard_RuneforgeUI_Bootstrap.lua",
    "Blizzard_RuneforgePowerList.xml",
    "Blizzard_RuneforgeModifierSlot.xml",
    "Blizzard_RuneforgeItemSlot.xml",
    "Blizzard_RuneforgeCreateFrame.xml",
    "Blizzard_RuneforgeCraftingFrame.xml",
    "Blizzard_RuneforgeCraftingTooltip.xml",
    "Blizzard_RuneforgeFrame.xml",
];

const RUNEFORGE_LUA_FILES: &[&str] = &[
    "Blizzard_RuneforgePowerList.lua",
    "Blizzard_RuneforgeModifierSlot.lua",
    "Blizzard_RuneforgeItemSlot.lua",
    "Blizzard_RuneforgeCreateFrame.lua",
    "Blizzard_RuneforgeCraftingFrame.lua",
    "Blizzard_RuneforgeCraftingTooltip.lua",
    "Blizzard_RuneforgeFrame.lua",
];

const PUBLIC_MIXINS: &[&str] = &[
    "RuneforgeFrameMixin",
    "RuneforgeCraftingFrameMixin",
    "RunforgeFrameTooltipMixin",
    "RuneforgeCreateFrameMixin",
    "RuneforgeCraftItemButtonMixin",
    "RuneforgeItemSlotMixin",
    "RuneforgeUpgradeItemSlotMixin",
    "RuneforgeModifierSlotMixin",
    "RuneforgeModifierSelectionMixin",
    "RuneforgeModifierSelectorFrameMixin",
    "RuneforgeModifierFrameMixin",
    "RuneforgePowerButtonMixin",
    "RuneforgePowerSlotMixin",
    "RuneforgePowerMixin",
    "RuneforgePowerListMixin",
    "RuneforgePowerFrameMixin",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "RuneforgeCraftingFrameTemplate",
    "RunforgeFrameTooltipTemplate",
    "RuneforgeCreateFrameTemplate",
    "RuneforgeItemSlotTemplate",
    "RuneforgeUpgradeItemSlotTemplate",
    "RuneforgeModifierSlotTemplate",
    "RuneforgeModifierSelectionTemplate",
    "RuneforgeModifierSelectorFrameTemplate",
    "RuneforgeModifierFrameTemplate",
    "RuneforgePowerButtonTemplate",
    "RuneforgePowerSlotTemplate",
    "RuneforgePowerTemplate",
    "RuneforgePowerListTemplate",
    "RuneforgePowerFrameTemplate",
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
    let resolved = find_toc_file(&runeforge_dir()).expect("Blizzard_RuneforgeUI TOC resolves");
    assert_eq!(
        resolved,
        runeforge_toc(),
        "Blizzard_RuneforgeUI ships exactly one bare TOC — no `_Mainline.toc` variant. \
         The runeforge legendary-crafting UI was a Shadowlands feature; the addon is \
         legacy retail-only and a sibling Classic flavor would never load it"
    );
}

#[test]
fn toc_declares_load_on_demand_with_blizzard_colors_dependency() {
    let toc = TocFile::from_file(&runeforge_toc()).expect("Blizzard_RuneforgeUI TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` so `is_load_on_demand()` returns true. \
         The runeforge UI must NOT load eagerly — it is opened on-demand when the \
         player interacts with a Runecarver NPC, gated by C_LegendaryCrafting events. \
         Eager-loading it would cost ~107 KB of Lua + XML at every game-screen entry \
         even for characters who never touch legendary crafting"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec!["Blizzard_Colors".to_string()],
        "TOC must declare exactly one hard `## Dependencies: Blizzard_Colors`. \
         Blizzard_Colors publishes the `LEGENDARY_ORANGE_COLOR` / quality-color tables \
         used by RuneforgePowerButtonTemplate.HoverFrame to tint the legendary-orange \
         power-name FontString. No other foundational addon is required because every \
         other reference (CallbackRegistryMixin, CurrencyDisplayGroupTemplate, \
         ScriptAnimatedModelSceneTemplate, PagedListTemplate, TemplatedListElementTemplate) \
         is part of SharedXML which loads before any addon"
    );
    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — the runeforge UI never observes sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — runeforge state is server-authoritative; the client \
         re-fetches available powers/modifiers/currencies on every UI open"
    );
}

#[test]
fn toc_declares_metadata_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(runeforge_toc()).expect("Blizzard_RuneforgeUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Runeforge UI"),
        "TOC must declare `## Title: Blizzard Runeforge UI` exactly (space-and-prose form)"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly — the explicit one form gates the \
         addon out of the eager Game-screen sweep"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_Colors"),
        "TOC must declare `## Dependencies: Blizzard_Colors` exactly"
    );
    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare `## AllowLoad` — falls through to the None arm at \
         src/toc.rs:311 which returns Game-only. Glue screens have no NPC interaction \
         surface and no legendary crafting flow"
    );
    assert!(
        !raw.contains("## AllowLoadGameType"),
        "TOC must NOT declare `## AllowLoadGameType` — the addon is mainline-only by \
         default; runeforge legendary crafting was a Shadowlands feature with no \
         classic-flavor counterpart"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — server-authoritative state"
    );
    assert!(
        !raw.contains("## OptionalDep"),
        "TOC must NOT declare any `## OptionalDep` keys"
    );
}

#[test]
fn toc_lists_bootstrap_and_seven_xml_files() {
    let toc = TocFile::from_file(&runeforge_toc()).expect("Blizzard_RuneforgeUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, RUNEFORGE_TOC_FILES,
        "Retail 12.1.0.69497 lists the Runeforge bootstrap before its seven XML files"
    );
}

#[test]
fn allows_screen_returns_true_only_for_game_when_allowload_absent() {
    let toc = TocFile::from_file(&runeforge_toc()).expect("Blizzard_RuneforgeUI TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Absent `## AllowLoad` falls through to the None arm at src/toc.rs:311 which \
         returns `screen == ScreenKind::Game`. Game screen is the only valid context \
         for legendary crafting"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Glue screen {screen:?} must reject the runeforge UI — there is no NPC \
             interaction or C_LegendaryCrafting backend on glue screens"
        );
    }
}

#[test]
fn excluded_from_eager_discovery_due_to_load_on_demand() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_RuneforgeUI");
    assert!(
        !found,
        "Blizzard_RuneforgeUI must NOT appear in `discover_blizzard_addons_for_screen` \
         on the Game screen — `## LoadOnDemand: 1` puts it in the LoD pool, and the \
         pool only joins the eager set when another loaded addon hard-depends on it. \
         No other Blizzard addon hard-depends on Blizzard_RuneforgeUI (the runeforge \
         flow is opened by C_LegendaryCrafting events from server, not by template \
         inheritance), so it stays in the LoD pool until C_AddOns.LoadAddOn is called"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&ui, screen);
        let glue_found = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_RuneforgeUI");
        assert!(
            !glue_found,
            "Blizzard_RuneforgeUI must NOT appear on glue screen {screen:?} — both \
             LoadOnDemand AND Game-only `allows_screen` exclude it"
        );
    }
}

#[test]
fn root_directory_holds_seven_lua_xml_pairs_next_to_toc() {
    let dir = runeforge_dir();
    for lua in RUNEFORGE_LUA_FILES {
        let path = dir.join(lua);
        assert!(
            path.is_file(),
            "{} missing — every XML in the TOC needs its sibling Lua next to it because the XML loads its Lua chunk via `<Script file=\"...\"/>`",
            path.display()
        );
    }
    assert!(
        dir.join(RUNEFORGE_TOC_FILES[0]).is_file(),
        "Runeforge bootstrap missing"
    );
    for xml in &RUNEFORGE_TOC_FILES[1..] {
        let path = dir.join(xml);
        assert!(path.is_file(), "{} missing", path.display());
    }
}

#[test]
fn xml_files_embed_lua_via_script_directive_at_top_of_ui_root() {
    let dir = runeforge_dir();
    for (xml, expected_lua) in RUNEFORGE_TOC_FILES[1..]
        .iter()
        .zip(RUNEFORGE_LUA_FILES.iter())
    {
        let body = std::fs::read_to_string(dir.join(xml))
            .unwrap_or_else(|err| panic!("read {xml}: {err}"));
        let directive = format!(r#"<Script file="{expected_lua}"/>"#);
        assert!(
            body.contains(&directive),
            "{xml} must embed `{directive}` as a child of `<Ui>` so the Lua chunk \
             loads at XML parse time. The TOC lists only XML files; Lua reaches the \
             addon table exclusively through this embedded-script mechanism"
        );
    }
}

prefork_full_ui_case! {
fn loads_without_lua_errors_when_explicitly_loaded(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &runeforge_toc())
        .expect("Blizzard_RuneforgeUI should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_RuneforgeUI emitted Lua errors during explicit load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn is_addon_loaded_after_explicit_load(env: &WowLuaEnv) {

    let loaded_before: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_RuneforgeUI')")
        .expect("IsAddOnLoaded probe (pre-load) succeeds");
    assert!(
        !loaded_before,
        "C_AddOns.IsAddOnLoaded('Blizzard_RuneforgeUI') must return false BEFORE \
         explicit LoadAddOn — the eager Game-screen sweep skips LoadOnDemand addons \
         that no other loaded addon depends on"
    );

    load_addon(&env.loader_env(), &runeforge_toc())
        .expect("Blizzard_RuneforgeUI should load via Rust loader");

    let loaded_after: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_RuneforgeUI')")
        .expect("IsAddOnLoaded probe (post-load) succeeds");
    assert!(
        loaded_after,
        "C_AddOns.IsAddOnLoaded('Blizzard_RuneforgeUI') must return true after \
         explicit `load_addon` — the loader marks the addon as loaded in \
         `sim.addons[].loaded` (src/c_api/c_addons.rs:336)"
    );
}
}

prefork_full_ui_case! {
fn publishes_full_mixin_surface(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &runeforge_toc()).expect("RuneforgeUI loads");

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — the runeforge UI declares 16 mixins \
             across 7 Lua files. Master frame: RuneforgeFrameMixin (CallbackRegistryMixin \
             extension, owns the 8-callback registry for BaseItemChanged/PowerSelected/\
             ModifiersChanged/ItemSlotOnEnter|Leave/UpgradeItemChanged/\
             UpgradeItemSlotOnEnter|Leave). Child frames: RuneforgeCraftingFrameMixin / \
             RuneforgeCreateFrameMixin / RunforgeFrameTooltipMixin (note the typo \
             `Runforge` → `Runforge` carried verbatim from Blizzard's source) — all \
             three extend RuneforgeSystemMixin defined in Blizzard_FrameXML's \
             RuneforgeUtil. Item slots: RuneforgeItemSlotMixin → \
             RuneforgeUpgradeItemSlotMixin (extends via CreateFromMixins). Modifier \
             slots: RuneforgeModifierSlotMixin (extends RuneforgeEffectOwnerMixin) + \
             3 standalone helpers (RuneforgeModifierSelectionMixin / \
             RuneforgeModifierSelectorFrameMixin / RuneforgeModifierFrameMixin which \
             also extends RuneforgeSystemMixin). Power list: RuneforgePowerButtonMixin \
             (extends RuneforgePowerBaseMixin) → RuneforgePowerSlotMixin extends it; \
             plus RuneforgePowerMixin (data-row), RuneforgePowerListMixin (paged-list \
             container), RuneforgePowerFrameMixin (extends RuneforgeSystemMixin). Plus \
             RuneforgeCraftItemButtonMixin standalone. Each mixin is referenced by \
             an XML `mixin=\"...\"` attribute on its template"
        );
    }
}
}

prefork_full_ui_case! {
fn does_not_leak_virtual_templates_to_globals(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &runeforge_toc()).expect("RuneforgeUI loads");

    for template in VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — all 14 `virtual=\"true\"` templates live in \
             the template registry, NOT `_G`. Leaking any of them would let consumer \
             addons mutate the template definition and break every existing instance"
        );
    }
}
}

prefork_full_ui_case! {
fn registers_runeforge_frame_in_uipanelwindows_table(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &runeforge_toc()).expect("RuneforgeUI loads");

    let area: String = env
        .eval("return UIPanelWindows.RuneforgeFrame and UIPanelWindows.RuneforgeFrame.area or 'missing'")
        .expect("UIPanelWindows.RuneforgeFrame.area probe succeeds");
    assert_eq!(
        area, "left",
        "UIPanelWindows[\"RuneforgeFrame\"].area must equal `left` — \
         Blizzard_RuneforgeFrame.lua:2 sets `UIPanelWindows[\"RuneforgeFrame\"] = \
         {{ area = \"left\", pushable = 3, showFailedFunc = \
         C_LegendaryCrafting.CloseRuneforgeInteraction }}` so the UIParent panel \
         manager docks the frame on the left side, allows up to 3 simultaneous \
         left-side panels, and tears down the C_LegendaryCrafting NPC interaction \
         when the panel manager forces a close"
    );

    let pushable: f64 = env
        .eval("return UIPanelWindows.RuneforgeFrame.pushable")
        .expect("UIPanelWindows.RuneforgeFrame.pushable probe succeeds");
    assert_eq!(
        pushable, 3.0,
        "pushable=3 lets bag/character/spellbook frames stack alongside the runeforge \
         panel without auto-closing it"
    );

    let has_failed_func: bool = env
        .eval("return type(UIPanelWindows.RuneforgeFrame.showFailedFunc) == 'function'")
        .expect("showFailedFunc probe succeeds");
    assert!(
        has_failed_func,
        "showFailedFunc must be the C_LegendaryCrafting.CloseRuneforgeInteraction \
         function reference (server-side teardown) so a failed open never leaves \
         dangling NPC state"
    );
}
}

prefork_full_ui_case! {
fn publishes_named_non_virtual_frames_at_global_scope(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &runeforge_toc()).expect("RuneforgeUI loads");

    let frame_kind: String = env
        .eval("return type(_G.RuneforgeFrame)")
        .expect("RuneforgeFrame probe succeeds");
    assert_eq!(
        frame_kind, "table",
        "_G.RuneforgeFrame must publish as a table — the only top-level non-virtual \
         frame, declared at Blizzard_RuneforgeFrame.xml:5 with toplevel=\"true\" \
         parent=\"UIParent\" mixin=\"RuneforgeFrameMixin\" hidden=\"true\" \
         dimensions 588×676. Carries the OnLoad/OnShow/OnHide/OnEvent script bindings"
    );

    let tooltip_kind: String = env
        .eval("return type(_G.RuneforgeFrameResultTooltip)")
        .expect("RuneforgeFrameResultTooltip probe succeeds");
    assert_eq!(
        tooltip_kind, "table",
        "_G.RuneforgeFrameResultTooltip must publish — the result-preview GameTooltip \
         declared at Blizzard_RuneforgeFrame.xml:53 as `<GameTooltip \
         name=\"RuneforgeFrameResultTooltip\" parentKey=\"ResultTooltip\" \
         inherits=\"RunforgeFrameTooltipTemplate\"/>`. The XML comment on line 52 \
         (`<!-- Tooltips must have a name. -->`) is the engine constraint: anonymous \
         GameTooltip frames cannot resolve their texture pool. parentKey=ResultTooltip \
         also exposes it as `RuneforgeFrame.ResultTooltip` for the mixin code"
    );

    let hidden: bool = env
        .eval("return RuneforgeFrame:IsShown() == false")
        .expect("IsShown probe succeeds");
    assert!(
        hidden,
        "RuneforgeFrame must start hidden — `hidden=\"true\"` in XML and \
         UIPanelWindows-managed visibility (only ShowUIPanel(RuneforgeFrame) opens it)"
    );
}
}

prefork_full_ui_case! {
fn callback_registry_mixin_publishes_eight_runeforge_events(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &runeforge_toc()).expect("RuneforgeUI loads");

    let event_count: f64 = env
        .eval(
            "local count = 0; \
             for _ in pairs(RuneforgeFrameMixin.Event or {}) do count = count + 1 end; \
             return count",
        )
        .expect("RuneforgeFrameMixin.Event count probe succeeds");
    assert_eq!(
        event_count, 8.0,
        "RuneforgeFrameMixin.Event must hold 8 callback keys — published via \
         `RuneforgeFrameMixin:GenerateCallbackEvents({{ \"BaseItemChanged\", \
         \"PowerSelected\", \"ModifiersChanged\", \"ItemSlotOnEnter\", \
         \"ItemSlotOnLeave\", \"UpgradeItemChanged\", \"UpgradeItemSlotOnEnter\", \
         \"UpgradeItemSlotOnLeave\" }})` at Blizzard_RuneforgeFrame.lua:34. \
         CallbackRegistryMixin.GenerateCallbackEvents stamps Event[name]=name on the \
         mixin so child mixins can register without typo-ing the keys at the call site"
    );
}
}
