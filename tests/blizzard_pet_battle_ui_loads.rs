#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be synced")

}

fn pet_battle_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PetBattleUI")
}

fn pet_battle_toc() -> PathBuf {
    pet_battle_dir().join("Blizzard_PetBattleUI.toc")
}

// `[Family]` is resolved to `Mainline` by `src/toc.rs:145` before path resolution,
// so the canonical 5-file body on a Mainline run substitutes `Mainline` for the
// `[Family]` placeholder on the third entry.
const PET_BATTLE_TOC_FILES: &[&str] = &[
    "Shared/Blizzard_PetBattleUIPatchwerks.xml",
    "Shared/Blizzard_PetBattleUI.lua",
    "Mainline/Blizzard_PetBattleUI.lua",
    "Shared/Blizzard_PetBattleUI.xml",
    "Shared/Localization.lua",
];

const REQUIRED_DEPS: &[&str] = &[
    "Blizzard_Colors",
    "Blizzard_MicroMenu",
    "Blizzard_RaidWarning",
    "Blizzard_UIModes",
];

const PUBLIC_MIXINS: &[&str] = &["MicroButtonFrameMixin"];

const PUBLIC_NAMED_FRAMES: &[&str] = &[
    "PetBattleFrame",
    "PetBattlePrimaryUnitTooltip",
    "PetBattlePrimaryAbilityTooltip",
    "StartSplash",
];

const PUBLIC_GLOBAL_FUNCTIONS: &[&str] = &[
    "PetBattleFrame_OnLoad",
    "PetBattleFrame_OnEvent",
    "PetBattleFrame_OnShow",
    "PetBattleFrame_OnHide",
    "PetBattleFrame_Display",
    "PetBattleFrame_Remove",
    "PetBattleFrame_UpdateAllActionButtons",
    "PetBattleAbilityButton_OnClick",
    "PetBattleForfeitButton_OnClick",
    "PetBattleCatchButton_OnClick",
];

const PUBLIC_VIRTUAL_TEMPLATES: &[&str] = &[
    "PetBattleUnitFrame",
    "PetBattleUnitFrameUnclickable",
    "PetBattleAuraTemplate",
    "PetBattleAuraHolderTemplate",
    "PetBattleUnitTooltipAuraTemplate",
    "PetBattlePetSelectionButtonTemplate",
    "PetBattleMiniUnitFrameAlly",
    "PetBattleMiniUnitFrameEnemy",
    "PetBattleUnitTooltipPetTypeStrengthTemplate",
    "PetBattleUnitTooltipTemplate",
    "PetBattleActionButtonTemplate",
    "PetBattleAbilityButtonTemplate",
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
fn blizzard_pet_battle_ui_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&pet_battle_dir()).expect("Blizzard_PetBattleUI TOC resolves");
    assert_eq!(
        resolved,
        pet_battle_toc(),
        "Blizzard_PetBattleUI ships exactly one bare TOC — no `_Mainline.toc` \
         variant. The pet battle UI is the WoW Pets & Battles minigame frontend \
         (the in-world battle camera, action bar, ability/aura/weather displays \
         that appear when the player engages a pet battle). The flavor split is \
         handled by the per-TOC `## AllowLoadGameType: standard, mists` flag plus \
         a `[Family]` placeholder in the file list (resolved to Mainline or \
         Classic at parse time) instead of a separate `_Mainline.toc` file"
    );

    let mainline = pet_battle_dir().join("Blizzard_PetBattleUI_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the bare TOC with \
         `## AllowLoadGameType: standard, mists` and the `[Family]` placeholder \
         is the canonical entry point for both Mainline and Classic Mists",
        mainline.display()
    );
}

#[test]
fn blizzard_pet_battle_ui_toc_declares_eager_dual_flavor_with_four_deps() {
    let toc = TocFile::from_file(&pet_battle_toc()).expect("Blizzard_PetBattleUI TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 0` explicitly — pet battles are entered \
         via in-world combat triggers (not a panel button) so the OnLoad / OnEvent \
         chain must wire up before the engine fires PET_BATTLE_OPENING_START. \
         Lazy-loading would mean the first pet battle opens with no UI"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Default-Game-only `allows_screen` at src/toc.rs:311 returns true for \
         ScreenKind::Game when AllowLoad is omitted — pet battles are an \
         in-world feature"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Omitted `## AllowLoad:` must NOT enable {screen:?} — pet battles \
             only happen in 3D world space, glue screens have no battle camera"
        );
    }

    assert_eq!(
        toc.dependencies(),
        REQUIRED_DEPS,
        "TOC must declare exactly 2 RequiredDeps in order: `Blizzard_Colors` \
         (the panel uses ColorManager / quality-color tables for ability tooltip \
         tinting and pet-quality borders) then `Blizzard_MicroMenu` (the per-family \
         MicroButtonFrameMixin:OnShow calls `MicroMenu:OverrideMicroMenuPosition` \
         to reposition the micro menu while the pet-battle HUD is visible). Both \
         deps are hard rather than optional because the inherits / mixin lookups \
         resolve at parse time. `dependencies()` at src/toc.rs:210-217 reads \
         `Dependencies` here as the canonical retail spelling"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — no soft sibling addons"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — pet battle UI is a pure live-state mirror of the \
         active pet battle fetched from `C_PetBattles.*` on every event tick; \
         the persistent pet collection lives in Blizzard_Collections (PetJournal), \
         not here"
    );
}

#[test]
fn blizzard_pet_battle_ui_toc_declares_metadata_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(pet_battle_toc()).expect("Blizzard_PetBattleUI TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Pet Battle UI"),
        "TOC must declare `## Title: Blizzard Pet Battle UI` exactly — the \
         space-and-prose form rather than the underscore-namespace form like \
         `Blizzard_PetBattleUI`; the human-readable label is the majority \
         pattern for older Blizzard-shipped addons"
    );
    assert!(
        raw.contains("## LoadOnDemand: 0"),
        "TOC must declare `## LoadOnDemand: 0` exactly — UNUSUAL: the explicit-zero \
         spelling is the MINORITY form for eager addons; most eager addons OMIT \
         the key entirely. The explicit `0` here is a defensive declaration so \
         that flipping the value to `1` is a one-byte hot-fix without altering \
         the TOC structure"
    );
    assert!(
        raw.contains("## AllowLoadGameType: standard, mists"),
        "TOC must declare `## AllowLoadGameType: standard, mists` exactly — \
         UNUSUAL: a comma-separated dual-flavor accept list. `standard` covers \
         modern Mainline retail; `mists` covers the Mists of Pandaria classic \
         flavor where pet battles were originally introduced (5.0). \
         `is_game_type_restricted` at src/toc.rs:294-302 splits on `,` and returns \
         false (i.e. allow-Mainline) as long as ANY token matches `mainline` or \
         `standard`, so this TOC loads on Mainline. Other Classic flavors (Vanilla \
         pre-Mists, BC, Wrath, Cata) cannot run pet battles and so are filtered out"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_Colors, Blizzard_MicroMenu"),
        "TOC must declare `## Dependencies: Blizzard_Colors, Blizzard_MicroMenu` \
         exactly — comma-separated 2-dep list, ordered Colors-first because the \
         color-quality tables are referenced by both shared and family files \
         while the micro-menu override only fires from the family-specific mixin"
    );
    assert!(
        raw.contains("[Family]\\Blizzard_PetBattleUI.lua"),
        "TOC must declare a `[Family]\\Blizzard_PetBattleUI.lua` line — the \
         `[Family]` placeholder is resolved to `Mainline` by src/toc.rs:145 \
         before path resolution, allowing one TOC to drive both Mainline (which \
         calls EditModeManagerFrame:BlockEnteringEditMode in OnShow) and Classic \
         (which is a no-op stub) without a flavor-split TOC"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:` — Game-only is the default; pet \
         battles use `## AllowLoadGameType:` for the flavor filter, NOT \
         `## AllowLoad:` for the screen filter"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any `## SavedVariables*` keys — pure stateless \
         mirror of live pet-battle state"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare any `## OptionalDeps:` — zero soft siblings"
    );
    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — the addon omits the author key, \
         like Blizzard_PerksProgram and Blizzard_PersonalResourceDisplay; the \
         minimal-metadata profile is consistent across the older eager-loading \
         feature addons"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT declare `## Version:` — UNUSUAL omission compared to most \
         Blizzard-shipped addons; together with the missing `## Author:` the \
         metadata is minimal"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:` — the engine treats the absence \
         as enabled-by-default, no need to spell it out"
    );
}

#[test]
fn blizzard_pet_battle_ui_toc_lists_five_files_patchwerks_first() {
    let toc = TocFile::from_file(&pet_battle_toc()).expect("Blizzard_PetBattleUI TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PET_BATTLE_TOC_FILES,
        "TOC body must list exactly 5 files: \
         (1) Shared/Blizzard_PetBattleUIPatchwerks.xml — 36 virtual `<Texture>` \
         templates carving the PetBattleHud atlas into named texture references \
         (BattleHUD-Top, BattleHUD-Versus, MainPet-HealthBarFrame, Timer-Frame, \
         Buff-Divider, etc.); MUST load FIRST so the main XML's `inherits=\"...\"` \
         attributes resolve at parse time. \
         (2) Shared/Blizzard_PetBattleUI.lua — the 65KB main script that declares \
         the 109 free global functions (PetBattleFrame_OnLoad / PetBattleFrame_\
         OnEvent / PetBattleAbilityButton_OnClick / PetBattleFrameTurnTimer_OnUpdate \
         / etc.) and the PET_BATTLE_WEATHER_TEXTURES table; loads BEFORE the \
         Family-specific Lua so the Family overrides can reference shared \
         functions. \
         (3) Mainline/Blizzard_PetBattleUI.lua — resolved from the `[Family]` \
         placeholder; declares MicroButtonFrameMixin and the per-family \
         PetBattleFrame_OnShow / OnHide overrides that call \
         EditModeManagerFrame:BlockEnteringEditMode (Mainline-only — Classic's \
         version of the same file is a no-op stub since Classic predates EditMode). \
         (4) Shared/Blizzard_PetBattleUI.xml — the 62KB main layout that \
         materializes the 4 named non-virtual frames and 14 virtual templates; \
         loads AFTER all Lua so the script handlers bind by name. \
         (5) Shared/Localization.lua — a 50-byte stub with the comment \
         `-- This file is executed at the end of addon load`; runs LAST per \
         convention so locale-specific font / size overrides apply after the XML \
         materializes the named frames"
    );
}

#[test]
fn blizzard_pet_battle_ui_appears_in_game_screen_eager_discovery_only() {
    let ui = blizzard_ui_dir();

    let game_addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_PetBattleUI");
    assert!(
        in_game,
        "Blizzard_PetBattleUI must appear in Game-screen eager discovery — \
         eager (LoadOnDemand: 0) and Game-only by default, with \
         AllowLoadGameType containing `standard` accepted by \
         `is_game_type_restricted` for Mainline runs"
    );

    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&ui, screen);
        let in_glue = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PetBattleUI");
        assert!(
            !in_glue,
            "Blizzard_PetBattleUI must NOT appear in {screen:?} eager discovery \
             — default Game-only `allows_screen` filters glue screens"
        );
    }
}

#[test]
fn blizzard_pet_battle_ui_appears_in_full_addon_inventory() {
    let ui = blizzard_ui_dir();
    let inventory = discover_all_blizzard_addons(&ui);
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PetBattleUI");
    assert!(
        found,
        "Blizzard_PetBattleUI must appear in `discover_all_blizzard_addons` — \
         the full inventory walks every parseable TOC under Interface/BlizzardUI \
         regardless of LoadOnDemand or AllowLoadGameType"
    );
}

prefork_full_ui_case! {
fn blizzard_pet_battle_ui_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PetBattleUI")
                || message.contains("PetBattleFrame")
                || message.contains("PetBattleUnit")
                || message.contains("PetBattleAbility")
                || message.contains("PetBattleAura")
                || message.contains("PetBattleAction")
                || message.contains("PetBattleMiniUnitFrame")
                || message.contains("PetBattlePetSelection")
                || message.contains("MicroButtonFrameMixin")
                || message.contains("StartSplash")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PetBattleUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_pet_battle_ui_is_addon_loaded_after_eager_sweep(env: &WowLuaEnv) {

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PetBattleUI')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_PetBattleUI') must return true after \
         the eager Game-screen sweep — the addon is eager so the Game-screen \
         sweep loads it directly"
    );
}
}

prefork_full_ui_case! {
fn blizzard_pet_battle_ui_publishes_micro_button_frame_mixin(env: &WowLuaEnv) {

    for mixin in PUBLIC_MIXINS {
        let kind: String = env
            .eval(&format!("return type(_G.{mixin})"))
            .unwrap_or_else(|err| panic!("type(_G.{mixin}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — the Mainline family file \
             declares MicroButtonFrameMixin at module top with a single OnShow \
             method that calls `MicroMenu:OverrideMicroMenuPosition(self, \
             \"TOPLEFT\", self, \"TOPLEFT\", -3, 4, true)` to reposition the \
             micro menu while the pet-battle HUD is visible. The Classic family \
             file declares the same mixin name with different x/y offsets \
             (-10/27 vs the Mainline -3/4), the only behavior split between \
             flavors. Note: this is the addon's ONLY public mixin — the rest of \
             the panel is wired via free global functions like \
             PetBattleFrame_OnLoad bound by name to `<Scripts>` `<OnLoad \
             function=\"...\"/>` blocks in the main XML"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_pet_battle_ui_publishes_global_script_handlers(env: &WowLuaEnv) {

    for func in PUBLIC_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G.{func})"))
            .unwrap_or_else(|err| panic!("type(_G.{func}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{func} must publish as a function — Blizzard_PetBattleUI.lua \
             declares 109 free global functions across 65KB of script. The \
             critical ones are the OnLoad / OnEvent / OnShow / OnHide / Display / \
             Remove / UpdateAllActionButtons handlers bound by name from the main \
             XML's <Scripts> blocks. Free globals (rather than mixins) are the \
             older Blizzard-FrameXML pattern from before mixins were introduced; \
             pet battles shipped in 5.0 (Mists of Pandaria, 2012) when the \
             FrameXML codebase still leaned on global function pointers; the \
             pattern was preserved through subsequent expansions because the \
             script handlers in the XML use `<OnLoad function=\"...\"/>` syntax \
             which resolves the named global at parse time"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_pet_battle_ui_creates_named_frames(env: &WowLuaEnv) {

    for frame in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a frame userdata (FrameRef reports as \
             `'table'` via the custom __type metamethod). \
             Blizzard_PetBattleUI.xml ships exactly 4 named non-virtual frames: \
             PetBattleFrame (parent=UIParent, setAllPoints=true, hidden=true — \
             the panel root with the BottomFrame action bar / TurnTimer / \
             abilityButtons / PetSelectionFrame, the WeatherFrame status \
             indicator at the top, and the unit-frame pair — registers 12 \
             PET_BATTLE_* events in OnLoad), PetBattlePrimaryUnitTooltip \
             (inherits PetBattleUnitTooltipTemplate — the dedicated unit-info \
             tooltip for hovering a pet portrait), PetBattlePrimaryAbilityTooltip \
             (inherits SharedPetBattleAbilityTooltipTemplate — the ability \
             tooltip anchored BOTTOMRIGHT -5 +120, declared in shared XML so \
             both Mainline and Classic share the layout), and StartSplash \
             (parent=UIParent, hidden=true — the `VS!` splash overlay that \
             plays when PET_BATTLE_OPENING_DONE fires; uses scale + alpha \
             AnimationGroups to flash the SplashTexture child)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_pet_battle_ui_does_not_leak_virtual_templates_to_globals(env: &WowLuaEnv) {

    for template in PUBLIC_VIRTUAL_TEMPLATES {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual XML templates are pinned to \
             the template registry, NOT `_G`. Leaking any of these as a global \
             would let consumer addons mutate the template definition and break \
             every existing instance. Blizzard_PetBattleUI.xml declares 14 \
             virtual templates: 12 functional templates (PetBattleUnitFrame / \
             PetBattleUnitFrameUnclickable for ally vs enemy, PetBattleAuraTemplate \
             / PetBattleAuraHolderTemplate / PetBattleUnitTooltipAuraTemplate \
             for buff/debuff display, PetBattlePetSelectionButtonTemplate / \
             PetBattleMiniUnitFrameAlly / PetBattleMiniUnitFrameEnemy for the \
             pet-swap row, PetBattleUnitTooltipPetTypeStrengthTemplate / \
             PetBattleUnitTooltipTemplate for the tooltip layout, \
             PetBattleActionButtonTemplate / PetBattleAbilityButtonTemplate for \
             the action bar slots) plus 2 debug helpers (DebugTexture / \
             DebugTextureBlack — gated overlays for layout debugging that \
             remain in shipped XML for engineering convenience). Patchwerks \
             XML adds 36 more virtual `<Texture>` templates (BattleHUD-Top, \
             Timer-BG, MainPet-HealthBarFrame, etc.) but those are leaf texture \
             definitions consumed via `inherits=\"...\"` on `<Texture>` \
             elements; they are not Frame templates and so cannot leak via the \
             normal frame-template path"
        );
    }
}
}
