#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn garrison_base_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GarrisonBase")
}

fn garrison_base_toc() -> PathBuf {
    garrison_base_dir().join("Blizzard_GarrisonBase.toc")
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

#[test]
fn blizzard_garrison_base_resolves_single_unsuffixed_toc() {
    let resolved = find_toc_file(&garrison_base_dir())
        .expect("Blizzard_GarrisonBase directory must contain a discoverable TOC");
    let resolved_name = resolved
        .file_name()
        .expect("resolved TOC must have a filename")
        .to_str()
        .expect("resolved TOC filename must be utf-8");

    assert_eq!(
        resolved_name, "Blizzard_GarrisonBase.toc",
        "Blizzard_GarrisonBase ships a SINGLE unsuffixed TOC (no `_Mainline`/`_Vanilla` \
         variants); src/loader/mod.rs:65's `find_toc_file` falls through to the bare \
         `<addon>.toc` when no `_Mainline.toc` is present"
    );
}

#[test]
fn blizzard_garrison_base_toc_declares_colors_dependency_and_no_load_flags() {
    let toc = TocFile::from_file(&garrison_base_toc()).expect("Blizzard_GarrisonBase TOC parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_GarrisonBase has no `## LoadOnDemand` — it must load eagerly because \
         Blizzard_GameTooltip's xml line 101 nests a parentKey'd \
         GarrisonFollowerTooltipContentsTemplate inside the EmbeddedItemTooltip's \
         InternalEmbeddedItemTooltipTemplate, so the template registry has to know \
         about the GarrisonBase templates before GameTooltip resolves its inherits=..."
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GarrisonBase does not declare `## UseSecureEnvironment` — follower \
         tooltips read public follower data via C_Garrison.GetFollowerInfo, no \
         protected-action surface"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_GarrisonBase declares `## AllowLoadGameType: mainline` which \
         src/toc.rs:299 treats as the unrestricted retail game type — \
         `is_game_type_restricted()` returns false"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_GarrisonBase declares no `## SavedVariables` — the floating tooltips \
         do not persist position across sessions"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec!["Blizzard_Colors".to_string()],
        "Retail 12.1.0.69497 declares only `## Dependencies: Blizzard_Colors`. Got: {:?}",
        deps
    );

    let toc_text = std::fs::read_to_string(garrison_base_toc())
        .expect("Blizzard_GarrisonBase TOC should read");
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_GarrisonBase.toc declares `## DefaultState: enabled` — the follower \
         tooltip surface is core UI and on by default"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_GarrisonBase declares NO `## AllowLoad:` line — Game-only default per \
         src/toc.rs:311 (the login/character-select glue screens do not display garrison \
         follower tooltips)"
    );
}

#[test]
fn blizzard_garrison_base_toc_lists_only_xml_files() {
    let toc_text = std::fs::read_to_string(garrison_base_toc())
        .expect("Blizzard_GarrisonBase TOC should read");
    let xml_lines = [
        "GarrisonBaseUtils.xml",
        "AdventuresFollowerTooltip.xml",
        "FloatingGarrisonFollowerTooltip.xml",
        "GarrisonFollowerTooltip.xml",
    ];
    for xml in xml_lines.iter() {
        assert!(
            toc_text.contains(xml),
            "Blizzard_GarrisonBase TOC must list {xml} — the TOC enumerates ONLY the 4 \
             XML files; each .xml `<Script file=\"X.lua\"/>` directive includes its \
             matching .lua sibling, so the TOC delegates Lua loading to the XML loader"
        );
    }
    assert!(
        !toc_text.contains("GarrisonBaseUtils.lua\n"),
        "TOC must NOT list GarrisonBaseUtils.lua directly — the .lua siblings are \
         loaded via the .xml file's `<Script file=...>` include directive (see \
         GarrisonBaseUtils.xml line 4)"
    );
}

#[test]
fn blizzard_garrison_base_defaults_to_game_screen_only() {
    let toc = TocFile::from_file(&garrison_base_toc()).expect("Blizzard_GarrisonBase TOC parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Missing `## AllowLoad:` defaults to Game-only (src/toc.rs:311) — Game must be \
         allowed because Blizzard_GameTooltip depends on GarrisonBase's templates"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "Missing `## AllowLoad:` excludes Login — garrison follower tooltips are not \
         meaningful on the realm picker"
    );
    assert!(
        !toc.allows_screen(ScreenKind::CharacterSelect),
        "Missing `## AllowLoad:` excludes CharacterSelect for the same reason"
    );
}

#[test]
fn blizzard_garrison_base_auto_loads_on_game_and_skips_login() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GarrisonBase");
    assert!(
        in_game,
        "Blizzard_GarrisonBase has no `## LoadOnDemand` and defaults to Game-only — it \
         MUST appear in Game-screen auto-discovery (Blizzard_GameTooltip depends on \
         it for the EmbeddedItemTooltip's GarrisonFollowerTooltip child)"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GarrisonBase");
    assert!(
        !in_login,
        "Game-only default excludes Blizzard_GarrisonBase from Login auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_garrison_base_loads_via_full_game_ui_without_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Garrison")
                || message.contains("Adventures")
                || message.contains("Floating")
                || message.contains("GarrAutoCombatUtil")
                || message.contains("FollowerOptions")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_GarrisonBase emitted Lua errors during the full Game-screen load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_GarrisonBase') and true or false")
        .expect("IsAddOnLoaded probe should succeed");

    assert!(
        post_load,
        "C_AddOns.IsAddOnLoaded('Blizzard_GarrisonBase') must return truthy after a \
         successful full-game-UI load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_publishes_top_level_named_tooltips(env: &WowLuaEnv) {

    let from_garrison_follower_tooltip_xml: (bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GarrisonFollowerTooltip) == 'table', \
                    type(GarrisonFollowerAbilityTooltip) == 'table', \
                    type(GarrisonFollowerAbilityWithoutCountersTooltip) == 'table', \
                    type(GarrisonFollowerMissionAbilityWithoutCountersTooltip) == 'table', \
                    type(GarrisonShipyardFollowerTooltip) == 'table'",
        )
        .expect("GarrisonFollowerTooltip.xml frames probe should succeed");
    assert_eq!(
        from_garrison_follower_tooltip_xml,
        (true, true, true, true, true),
        "GarrisonFollowerTooltip.xml publishes 5 named non-floating tooltips (lines \
         4-32, all `toplevel=\"true\" movable=\"false\"`): GarrisonFollowerTooltip (the \
         hover follower stat tooltip), GarrisonFollowerAbilityTooltip (the ability \
         hover with mechanic counters), GarrisonFollowerAbilityWithoutCountersTooltip \
         (variant without the counter row for follower lists), \
         GarrisonFollowerMissionAbilityWithoutCountersTooltip (the mission-page version), \
         GarrisonShipyardFollowerTooltip (the WoD shipyard boat-follower tooltip)"
    );

    let from_floating_xml: (bool, bool, bool, bool) = env
        .eval(
            "return type(FloatingGarrisonFollowerTooltip) == 'table', \
                    type(FloatingGarrisonShipyardFollowerTooltip) == 'table', \
                    type(FloatingGarrisonFollowerAbilityTooltip) == 'table', \
                    type(FloatingGarrisonMissionTooltip) == 'table'",
        )
        .expect("FloatingGarrisonFollowerTooltip.xml frames probe should succeed");
    assert_eq!(
        from_floating_xml,
        (true, true, true, true),
        "FloatingGarrisonFollowerTooltip.xml publishes 4 named MOVABLE floating \
         tooltips (lines 240/258/424/442, all `movable=\"true\" toplevel=\"true\" \
         parent=\"UIParent\" frameStrata=\"TOOLTIP\" clampedToScreen=\"true\" \
         hidden=\"true\"`): FloatingGarrisonFollowerTooltip (the hyperlink-anchored \
         floating follower tooltip), FloatingGarrisonShipyardFollowerTooltip (boat \
         variant), FloatingGarrisonFollowerAbilityTooltip (ability link), \
         FloatingGarrisonMissionTooltip (mission link)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_publishes_virtual_templates(env: &WowLuaEnv) {

    let templates_dont_leak: (bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return _G.AdventuresLevelPortraitTemplate == nil, \
                    _G.GarrisonFollowerAbilityTemplate == nil, \
                    _G.GarrisonFollowerTooltipContentsTemplate == nil, \
                    _G.GarrisonFollowerTooltipTemplate == nil, \
                    _G.GarrisonShipyardFollowerTooltipTemplate == nil, \
                    _G.GarrisonFollowerAbilityTooltipTemplate == nil, \
                    _G.GarrisonFollowerAbilityWithoutCountersTooltipTemplate == nil, \
                    _G.GarrisonFollowerMissionAbilityWithoutCountersTooltipTemplate == nil, \
                    _G.GarrisonFollowerPortraitTemplate == nil",
        )
        .expect("template-leak probe should succeed");
    assert_eq!(
        templates_dont_leak,
        (true, true, true, true, true, true, true, true, true),
        "Blizzard_GarrisonBase publishes 9 virtual templates across all 4 XML files: \
         AdventuresLevelPortraitTemplate (AdventuresFollowerTooltip.xml line 5 — \
         mixin=AdventuresLevelPortraitMixin), GarrisonFollowerAbilityTemplate \
         (FloatingGarrisonFollowerTooltip.xml line 5 — the ability-row layout), \
         GarrisonFollowerTooltipContentsTemplate (line 59 — \
         the canonical contents template referenced by Blizzard_GameTooltip's \
         EmbeddedItemTooltip xml line 101), GarrisonFollowerTooltipTemplate (line 165 — \
         hosts contents+TooltipBackdropTemplate), GarrisonShipyardFollowerTooltipTemplate \
         (line 168), GarrisonFollowerAbilityTooltipTemplate (line 278), \
         GarrisonFollowerAbilityWithoutCountersTooltipTemplate (line 336), \
         GarrisonFollowerMissionAbilityWithoutCountersTooltipTemplate (line 377), \
         GarrisonFollowerPortraitTemplate (GarrisonBaseUtils.xml line 6 — \
         mixin=GarrisonFollowerPortraitMixin). All declare `virtual=\"true\"` so they \
         do NOT leak as `_G.*` Lua globals — they live only in the XML template registry"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_publishes_three_mixins(env: &WowLuaEnv) {

    let mixins: (bool, bool, bool) = env
        .eval(
            "return type(GarrisonFollowerPortraitMixin) == 'table', \
                    type(AdventuresLevelPortraitMixin) == 'table', \
                    type(GarrAutoCombatUtil) == 'table'",
        )
        .expect("Mixin/namespace probes should succeed");
    assert_eq!(
        mixins,
        (true, true, true),
        "Blizzard_GarrisonBase publishes 3 namespace tables: \
         GarrisonFollowerPortraitMixin (GarrisonBaseUtils.lua:425 — drives the \
         GarrisonFollowerPortraitTemplate; owns SetPortraitIcon / SetQuality / \
         SetQualityColor / SetNoLevel / SetLevel / SetILevel / SetupPortrait), \
         AdventuresLevelPortraitMixin (AdventuresFollowerTooltip.lua:6 — drives the \
         AdventuresLevelPortraitTemplate; owns SetupPortrait that sets the \
         Adventurers-Followers-Frame atlas + portrait icon + level text), \
         GarrAutoCombatUtil (GarrisonBaseUtils.lua:544 — namespace table NOT a mixin; \
         owns GetFollowerAutoCombatSpells / CreateTextureMarkupForTooltipSpellIcon / \
         GetAuraTypeAtlasesFromPreviewMask / GetAtlasMarkupFromPreviewMask / \
         AddAuraToTooltip / IsAbilityEvent — the auto-combat (Shadowlands adventure) \
         spell tooltip helpers)"
    );

    let portrait_methods: (bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GarrisonFollowerPortraitMixin.SetPortraitIcon) == 'function', \
                    type(GarrisonFollowerPortraitMixin.SetQuality) == 'function', \
                    type(GarrisonFollowerPortraitMixin.SetQualityColor) == 'function', \
                    type(GarrisonFollowerPortraitMixin.SetNoLevel) == 'function', \
                    type(GarrisonFollowerPortraitMixin.SetLevel) == 'function', \
                    type(GarrisonFollowerPortraitMixin.SetILevel) == 'function', \
                    type(GarrisonFollowerPortraitMixin.SetupPortrait) == 'function'",
        )
        .expect("GarrisonFollowerPortraitMixin method probe should succeed");
    assert_eq!(
        portrait_methods,
        (true, true, true, true, true, true, true),
        "GarrisonFollowerPortraitMixin defines 7 methods (lines 427-509): \
         SetPortraitIcon (sets the texture from a fileID), SetQuality (maps \
         Enum.ItemQuality to ITEM_QUALITY_COLORS for the border tint), \
         SetQualityColor (raw RGB setter for non-standard tints), SetNoLevel \
         (hides the level overlay), SetLevel (sets level text), SetILevel (sets \
         item-level text + shows the secondary plate), SetupPortrait (the canonical \
         entry point — chains all of the above based on the followerInfo struct)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_publishes_garrison_follower_options_per_follower_type(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(GarrisonFollowerOptions)")
        .expect("GarrisonFollowerOptions type probe should succeed");
    assert_eq!(
        kind, "table",
        "GarrisonFollowerOptions (GarrisonBaseUtils.lua:4) is the canonical \
         per-follower-type config table keyed by Enum.GarrisonFollowerType — owns the \
         strings/sounds/layout-scalars consumed by every GarrisonMission UI variant"
    );

    let entries: (bool, bool, bool, bool, bool) = env
        .eval(
            "local F = Enum.GarrisonFollowerType \
             return type(GarrisonFollowerOptions[F.FollowerType_6_0_GarrisonFollower]) == 'table', \
                    type(GarrisonFollowerOptions[F.FollowerType_6_0_Boat]) == 'table', \
                    type(GarrisonFollowerOptions[F.FollowerType_7_0_GarrisonFollower]) == 'table', \
                    type(GarrisonFollowerOptions[F.FollowerType_8_0_GarrisonFollower]) == 'table', \
                    type(GarrisonFollowerOptions[F.FollowerType_9_0_GarrisonFollower]) == 'table'",
        )
        .expect("GarrisonFollowerOptions per-type probes should succeed");
    assert_eq!(
        entries,
        (true, true, true, true, true),
        "GarrisonFollowerOptions has entries for all 5 historical follower-type \
         expansions (GarrisonBaseUtils.lua:5/68/131/196/261): WoD garrison follower, \
         WoD shipyard boat, Legion class hall champion, BfA war campaign champion, \
         Shadowlands adventure follower. Each entry carries a ~30-field config block \
         (abilityTooltipFrame, displayCounterAbilityInPlaceOfMechanic, \
         followerListCounterNumPerRow, garrisonType, missionFrame, \
         missionPageAssignFollowerSound, partyNotFullText, showILevelInFollowerList, \
         strings sub-table, etc.)"
    );

    let wod_garrison: (String, String, bool) = env
        .eval(
            "local cfg = GarrisonFollowerOptions[Enum.GarrisonFollowerType.FollowerType_6_0_GarrisonFollower] \
             return cfg.missionFrame, cfg.abilityTooltipFrame, cfg.isPrimaryFollowerType",
        )
        .expect("WoD garrison config probe should succeed");
    assert_eq!(
        wod_garrison,
        (
            "GarrisonMissionFrame".to_string(),
            "GarrisonFollowerAbilityTooltip".to_string(),
            true,
        ),
        "GarrisonFollowerOptions[FollowerType_6_0_GarrisonFollower] (lines 5-66) \
         routes to GarrisonMissionFrame for the mission UI and to \
         GarrisonFollowerAbilityTooltip for ability tooltips, isPrimaryFollowerType=true \
         (only the first WoD garrison follower type is primary — other entries are \
         secondary aliases)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_publishes_helper_globals_from_garrison_base_utils(env: &WowLuaEnv) {

    let helpers: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GetPrimaryGarrisonFollowerType) == 'function', \
                    type(ShouldShowFollowerAbilityBorder) == 'function', \
                    type(ShouldShowILevelInFollowerList) == 'function', \
                    type(IsGarrisonLandingPageFeatured) == 'function', \
                    type(ShowGarrisonLandingPage) == 'function', \
                    type(DoesFollowerMatchCurrentGarrisonType) == 'function'",
        )
        .expect("GarrisonBaseUtils helper probe should succeed");
    assert_eq!(
        helpers,
        (true, true, true, true, true, true),
        "GarrisonBaseUtils.lua publishes 6 bare-global helpers (lines 326-422): \
         GetPrimaryGarrisonFollowerType (line 326 — maps a garrTypeID to the canonical \
         primary follower-type id, chosen by the isPrimaryFollowerType flag in the \
         options table), ShouldShowFollowerAbilityBorder (line 335 — guards the \
         ability border quad based on followerType + abilityInfo.isTrait flags), \
         ShouldShowILevelInFollowerList (line 340 — reads showILevelInFollowerList from \
         the per-type options), IsGarrisonLandingPageFeatured (line 348 — the in-world \
         minimap banner gating helper), ShowGarrisonLandingPage (line 353 — the canonical \
         entry point that opens the GarrisonLandingPage frame for a given garrTypeID), \
         DoesFollowerMatchCurrentGarrisonType (line 412 — used by the FollowerList \
         filter to exclude wrong-type followers)"
    );

    let talent_string_helper: bool = env
        .eval("return type(GetGarrisonTalentCostString) == 'function'")
        .expect("GetGarrisonTalentCostString probe should succeed");
    assert!(
        talent_string_helper,
        "GetGarrisonTalentCostString (line 511) — formats the gold + currency cost row \
         shown on garrison-talent hover with optional abbreviation and color codes; \
         consumed by every order-hall talent tree (Legion class hall, BfA war campaign, \
         Shadowlands covenant)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_publishes_garr_auto_combat_util_methods(env: &WowLuaEnv) {

    let methods: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GarrAutoCombatUtil.GetFollowerAutoCombatSpells) == 'function', \
                    type(GarrAutoCombatUtil.CreateTextureMarkupForTooltipSpellIcon) == 'function', \
                    type(GarrAutoCombatUtil.GetAuraTypeAtlasesFromPreviewMask) == 'function', \
                    type(GarrAutoCombatUtil.GetAtlasMarkupFromPreviewMask) == 'function', \
                    type(GarrAutoCombatUtil.AddAuraToTooltip) == 'function', \
                    type(GarrAutoCombatUtil.IsAbilityEvent) == 'function'",
        )
        .expect("GarrAutoCombatUtil method probe should succeed");
    assert_eq!(
        methods,
        (true, true, true, true, true, true),
        "GarrAutoCombatUtil (line 544) defines 6 methods consumed by the Shadowlands \
         9.0 adventure auto-combat tooltip: GetFollowerAutoCombatSpells (line 546 — \
         queries C_Garrison.GetFollowerAutoCombatSpells with optional auto-attack \
         filter), CreateTextureMarkupForTooltipSpellIcon (line 555 — formats the inline \
         icon TextureMarkup string), GetAuraTypeAtlasesFromPreviewMask (line 559 — \
         decodes the bitfield that says which aura icons (buff/debuff/healing/damage) \
         the spell shows in preview mode), GetAtlasMarkupFromPreviewMask (line 576 — \
         flattens the icon list into a markup blob for the tooltip), AddAuraToTooltip \
         (line 590 — appends an aura row to a GameTooltip), IsAbilityEvent (line 611 — \
         the boolean predicate for spell-event filtering during auto-combat replay)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_publishes_garrison_follower_tooltip_helpers(env: &WowLuaEnv) {

    let helpers: (bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GarrisonFollowerTooltip_Show) == 'function', \
                    type(GarrisonFollowerTooltip_ShowWithData) == 'function', \
                    type(GarrisonFollowerTooltipTemplate_BuildDefaultDataForID) == 'function', \
                    type(GarrisonFollowerAbilityTooltip_Show) == 'function', \
                    type(ShowGarrisonFollowerAbilityTooltip) == 'function', \
                    type(HideGarrisonFollowerAbilityTooltip) == 'function', \
                    type(ShowGarrisonFollowerMissionAbilityTooltip) == 'function'",
        )
        .expect("GarrisonFollowerTooltip helper probe should succeed");
    assert_eq!(
        helpers,
        (true, true, true, true, true, true, true),
        "GarrisonFollowerTooltip.lua publishes 7 of the canonical follower-tooltip \
         entry points: GarrisonFollowerTooltip_Show (line 2 — primary entry; takes 21 \
         positional args including 4 abilities + 4 traits + xp/level/itemLevel), \
         GarrisonFollowerTooltip_ShowWithData (line 32 — struct-based variant that \
         reads from a data table), GarrisonFollowerTooltipTemplate_BuildDefaultDataForID \
         (line 49 — fetches C_Garrison.GetFollowerInfo and constructs the default \
         tooltip payload), GarrisonFollowerAbilityTooltip_Show (line 85 — sets up the \
         ability tooltip from a garrFollowerAbilityID), \
         ShowGarrisonFollowerAbilityTooltip (line 90 — the canonical anchor-and-show \
         helper consumed by hover handlers), HideGarrisonFollowerAbilityTooltip (line \
         99), ShowGarrisonFollowerMissionAbilityTooltip (line 120 — the \
         mission-page-specific variant that anchors to the mission threat row)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_publishes_floating_tooltip_toggle_helpers(env: &WowLuaEnv) {

    let helpers: (bool, bool, bool, bool, bool) = env
        .eval(
            "return type(FloatingGarrisonFollower_Toggle) == 'function', \
                    type(FloatingGarrisonFollower_Show) == 'function', \
                    type(GarrisonFollowerTooltipTemplate_SetGarrisonFollower) == 'function', \
                    type(FloatingGarrisonFollowerAbility_Toggle) == 'function', \
                    type(FloatingGarrisonMission_Toggle) == 'function'",
        )
        .expect("FloatingGarrison helper probe should succeed");
    assert_eq!(
        helpers,
        (true, true, true, true, true),
        "FloatingGarrisonFollowerTooltip.lua publishes the floating-tooltip Toggle/Show \
         API: FloatingGarrisonFollower_Toggle (line 10 — toggles \
         FloatingGarrisonFollowerTooltip; rebuilds the template if the followerID \
         changed), FloatingGarrisonFollower_Show (line 24 — the explicit-show variant \
         that takes a tooltipFrame and follower data), \
         GarrisonFollowerTooltipTemplate_SetGarrisonFollower (line 59 — the layout \
         engine that fills in the contents-template fields from the data struct), \
         FloatingGarrisonFollowerAbility_Toggle (line 548 — toggles the ability-link \
         tooltip), FloatingGarrisonMission_Toggle (line 625 — toggles the mission-link \
         tooltip and anchors to the screen center)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_garrison_base_floating_tooltips_use_uiparent_with_tooltip_strata(env: &WowLuaEnv) {

    let strata: (String, String, bool) = env
        .eval(
            "return FloatingGarrisonFollowerTooltip:GetFrameStrata(), \
                    FloatingGarrisonFollowerTooltip:GetParent():GetName(), \
                    FloatingGarrisonFollowerTooltip:IsClampedToScreen()",
        )
        .expect("FloatingGarrisonFollowerTooltip strata probe should succeed");
    assert_eq!(
        strata,
        ("TOOLTIP".to_string(), "UIParent".to_string(), true),
        "FloatingGarrisonFollowerTooltip (xml line 240) inherits \
         GarrisonFollowerTooltipTemplate (line 165) which sets `parent=\"UIParent\" \
         frameStrata=\"TOOLTIP\" clampedToScreen=\"true\"` — the floating tooltip rides \
         at the topmost strata above modal panels and clamps into the viewport when \
         dragged near a screen edge"
    );

    let mission_strata: (String, String, bool) = env
        .eval(
            "return FloatingGarrisonMissionTooltip:GetFrameStrata(), \
                    FloatingGarrisonMissionTooltip:GetParent():GetName(), \
                    FloatingGarrisonMissionTooltip:IsClampedToScreen()",
        )
        .expect("FloatingGarrisonMissionTooltip strata probe should succeed");
    assert_eq!(
        mission_strata,
        ("TOOLTIP".to_string(), "UIParent".to_string(), true),
        "FloatingGarrisonMissionTooltip (xml line 442) declares \
         `parent=\"UIParent\" frameStrata=\"TOOLTIP\" clampedToScreen=\"true\"` — the \
         mission-link popup rides at the topmost strata above modal panels and clamps \
         into the viewport (unlike the in-world hover variants, this frame inherits \
         TooltipBackdropTemplate directly rather than GarrisonFollowerTooltipTemplate)"
    );
}
}
