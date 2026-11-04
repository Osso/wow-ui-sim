#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn frame_xml_util_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_FrameXMLUtil/Blizzard_FrameXMLUtil.toc")
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
fn blizzard_frame_xml_util_toc_uses_dep_alias_and_allow_load_game() {
    let toc = TocFile::from_file(&frame_xml_util_toc()).expect("Blizzard_FrameXMLUtil TOC parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_FrameXMLUtil has no `## LoadOnDemand` line — this is the high-level \
         utility library tier (ItemUtil / AuraUtil / CalendarUtil / CommunitiesUtil / \
         AchievementUtil / AzeriteUtil / MapUtil / DifficultyUtil / FadingFrame_* / \
         CooldownFrame_* / AnimationSystem helpers + ~25 other Util namespaces) that \
         every panel addon depends on, so it MUST be eagerly loaded at startup"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_FrameXMLUtil does not declare `## UseSecureEnvironment` — utility \
         libraries live in the standard taint environment"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_FrameXMLUtil declares no `## AllowLoadGameType:` line at the file \
         level (per-line bracketed AllowLoadGameType annotations gate individual files \
         instead — e.g. `[AllowLoadGameType mainline]` on AchievementUtil.lua), so \
         `is_game_type_restricted()` returns false at the addon level"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_FrameXMLUtil declares no `## SavedVariables` — utility code only, no \
         per-character persistence"
    );

    let toc_text = std::fs::read_to_string(frame_xml_util_toc())
        .expect("Blizzard_FrameXMLUtil TOC should read");
    assert!(
        toc_text.contains("## Dep: Blizzard_SharedXMLGame"),
        "Blizzard_FrameXMLUtil uses the `## Dep:` (singular, repeated-line) shorthand \
         instead of the canonical `## Dependencies:`. Three `## Dep:` lines declare \
         Blizzard_SharedXMLGame / Blizzard_Colors / Blizzard_StaticPopup — these supply \
         the SharedXMLGame helpers used across the Util files, the FontColor / \
         ColorMixin surface, and the StaticPopup_Show plumbing"
    );
    assert!(
        toc_text.contains("## Dep: Blizzard_Colors"),
        "Blizzard_FrameXMLUtil declares `## Dep: Blizzard_Colors` (line 3) — supplies \
         FontColor / ColorMixin / NORMAL_FONT_COLOR / GREEN_FONT_COLOR globals consumed \
         by ItemUtil / TitleUtil / AchievementUtil tooltip routines"
    );
    assert!(
        toc_text.contains("## Dep: Blizzard_StaticPopup"),
        "Blizzard_FrameXMLUtil declares `## Dep: Blizzard_StaticPopup` (line 4) — \
         supplies StaticPopup_Show / StaticPopup_Hide consumed by ItemUtil's \
         disenchant/refund confirmation paths"
    );
    assert!(
        toc_text.contains("## AllowLoad: Game"),
        "Blizzard_FrameXMLUtil declares `## AllowLoad: Game` — utility code is \
         in-world only, glue/login screens have their own utility addons"
    );

    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_SharedXMLGame".to_string(),
            "Blizzard_Colors".to_string(),
            "Blizzard_StaticPopup".to_string(),
        ],
        "repeated `## Dep:` directives must be exposed in source order"
    );
}

#[test]
fn blizzard_frame_xml_util_allows_only_game_screen() {
    let toc = TocFile::from_file(&frame_xml_util_toc()).expect("Blizzard_FrameXMLUtil TOC parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` must permit the Game screen (src/toc.rs:307)"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "`## AllowLoad: Game` must reject the Login screen — utility code is \
         in-world only"
    );
    assert!(
        !toc.allows_screen(ScreenKind::CharacterSelect),
        "`## AllowLoad: Game` must reject CharacterSelect — utility code is \
         in-world only"
    );
}

#[test]
fn blizzard_frame_xml_util_auto_loads_on_game_and_skips_login() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FrameXMLUtil");
    assert!(
        in_game,
        "Blizzard_FrameXMLUtil has no `## LoadOnDemand` line and `## AllowLoad: Game`, \
         so it MUST appear in Game-screen auto-discovery"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_FrameXMLUtil");
    assert!(
        !in_login,
        "`## AllowLoad: Game` excludes Blizzard_FrameXMLUtil from Login auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_loads_via_full_game_ui_without_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("FrameXMLUtil")
                || message.contains("ItemUtil")
                || message.contains("AuraUtil")
                || message.contains("CalendarUtil")
                || message.contains("CommunitiesUtil")
                || message.contains("CooldownFrame_")
                || message.contains("FadingFrame_")
                || message.contains("AnimationSystem")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_FrameXMLUtil emitted Lua errors during the full Game-screen load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_FrameXMLUtil') and true or false")
        .expect("IsAddOnLoaded probe should succeed");
    assert!(
        post_load,
        "After full Game-screen load, IsAddOnLoaded('Blizzard_FrameXMLUtil') must \
         return true — auto-discovery picks up the addon (no LoadOnDemand) and \
         `mark_addon_loaded` registers it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_core_item_and_aura_namespaces(env: &WowLuaEnv) {

    let core_namespaces: (bool, bool, bool, bool) = env
        .eval(
            "return type(ItemUtil) == 'table', \
                    type(ItemButtonUtil) == 'table', \
                    type(AuraUtil) == 'table', \
                    type(CalendarUtil) == 'table'",
        )
        .expect("Core util namespace probe should succeed");
    assert_eq!(
        core_namespaces,
        (true, true, true, true),
        "Blizzard_FrameXMLUtil publishes the core utility namespaces consumed by every \
         panel addon: ItemUtil (ItemUtil.lua — GetItemDetails / GetItemHyperlink / \
         PickupBagItem / IteratePlayerInventory / FilterOwnedItems), ItemButtonUtil \
         (ItemUtil.lua — Event registry, ItemContextEnum, GetItemContext / \
         OpenAndFilterBags for the unified bag/character-frame filter), AuraUtil \
         (AuraUtil.lua — aura iteration helpers and FindAuraByName), CalendarUtil \
         (CalendarUtil.lua — date formatting and event-status mapping)"
    );
}
}

#[cfg(feature = "retail-12-1-0")]
prefork_full_ui_case! {
fn blizzard_frame_xml_util_restores_patch_12_1_difficulty_color_delegates(env: &WowLuaEnv) {
    let result: String = env
        .eval(
            r#"
            local function matchesVendor(functionName, ...)
                local color, highlight = DifficultyUtil[functionName](...)
                local vendorColor, vendorHighlight = _G[functionName](...)
                return color == vendorColor and highlight == vendorHighlight
            end

            if not matchesVendor("GetDifficultyColor", Enum.RelativeContentDifficulty.Impossible) then return "difficulty" end
            if not matchesVendor("GetQuestDifficultyColor", UnitEffectiveLevel("player"), false) then return "quest" end
            if not matchesVendor("GetCreatureDifficultyColor", UnitEffectiveLevel("player") + 5) then return "creature" end
            if not matchesVendor("GetRelativeDifficultyColor", 10, 15) then return "relative-delegate" end
            if not matchesVendor("GetScalingQuestDifficultyColor", UnitEffectiveLevel("player") + 5) then return "scaling-delegate" end

            local color = DifficultyUtil.GetRelativeDifficultyColor(10, 15)
            if color ~= QuestDifficultyColors.impossible then return "relative-plus-five" end
            color = DifficultyUtil.GetRelativeDifficultyColor(10, 13)
            if color ~= QuestDifficultyColors.verydifficult then return "relative-plus-three" end
            color = DifficultyUtil.GetRelativeDifficultyColor(10, 6)
            if color ~= QuestDifficultyColors.difficult then return "relative-minus-four" end

            local playerLevel = UnitEffectiveLevel("player")
            color = DifficultyUtil.GetScalingQuestDifficultyColor(playerLevel + 5)
            if color ~= QuestDifficultyColors.impossible then return "scaling-plus-five" end
            color = DifficultyUtil.GetScalingQuestDifficultyColor(playerLevel + 3)
            if color ~= QuestDifficultyColors.verydifficult then return "scaling-plus-three" end
            color = DifficultyUtil.GetScalingQuestDifficultyColor(playerLevel)
            if color ~= QuestDifficultyColors.difficult then return "scaling-zero" end
            return "ok"
            "#,
        )
        .expect("12.1 DifficultyUtil delegates should survive full Game UI startup");

    assert_eq!(result, "ok");
}
}

#[cfg(not(feature = "retail-12-1-0"))]
prefork_full_ui_case! {
fn blizzard_frame_xml_util_does_not_publish_patch_12_1_difficulty_color_delegates(env: &WowLuaEnv) {

    let result: bool = env
        .eval(
            r#"
            return DifficultyUtil.GetCreatureDifficultyColor == nil
                and DifficultyUtil.GetDifficultyColor == nil
                and DifficultyUtil.GetQuestDifficultyColor == nil
                and DifficultyUtil.GetRelativeDifficultyColor == nil
                and DifficultyUtil.GetScalingQuestDifficultyColor == nil
            "#,
        )
        .expect("pre-12.1 DifficultyUtil surface should be queryable");

    assert!(result, "12.1 DifficultyUtil delegates leaked into an older epoch");
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_communities_and_pvp_namespaces(env: &WowLuaEnv) {

    let group_namespaces: (bool, bool, bool, bool, bool) = env
        .eval(
            "return type(CommunitiesUtil) == 'table', \
                    type(PartyUtil) == 'table', \
                    type(ArenaUtil) == 'table', \
                    type(PVPUtil) == 'table', \
                    type(DifficultyUtil) == 'table'",
        )
        .expect("Group util namespace probe should succeed");
    assert_eq!(
        group_namespaces,
        (true, true, true, true, true),
        "Blizzard_FrameXMLUtil publishes group/community-related namespaces: \
         CommunitiesUtil (~26 helpers — GetMemberRGB / SortClubs / GetMemberInfo / \
         DoesAnyCommunityHaveUnreadMessages / OpenInviteDialog / FindGuildStreamByType / \
         AddLookingForLines), PartyUtil (party-roster helpers), ArenaUtil (arena \
         enemy-team helpers), PVPUtil (PVPUtil.lua — battleground/pvp helpers, \
         mainline-only), DifficultyUtil (DifficultyUtil.lua — instance difficulty \
         enum-to-name mapping)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_achievement_and_collection_namespaces(env: &WowLuaEnv) {

    let achieve_namespaces: (bool, bool, bool, bool) = env
        .eval(
            "return type(AchievementUtil) == 'table', \
                    type(AzeriteUtil) == 'table', \
                    type(AzeriteEssenceUtil) == 'table', \
                    type(CollectionWardrobeUtil) == 'table'",
        )
        .expect("Achievement/collection util probe should succeed");
    assert_eq!(
        achieve_namespaces,
        (true, true, true, true),
        "Blizzard_FrameXMLUtil publishes mainline-only achievement and collection \
         namespaces (each gated by `[AllowLoadGameType mainline]` on the TOC line, so \
         the file is loaded on standard retail): AchievementUtil (AchievementUtil.lua \
         — achievement criteria and shield-icon helpers), AzeriteUtil (AzeriteUtil.lua \
         — has-selected-power / preview helpers), AzeriteEssenceUtil \
         (AzeriteEssenceUtil.lua — essence display formatting), \
         CollectionWardrobeUtil (CollectionsUtil.lua — wardrobe set/source helpers)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_mainline_subdir_namespaces(env: &WowLuaEnv) {

    let mainline_namespaces: (bool, bool, bool, bool) = env
        .eval(
            "return type(MapUtil) == 'table', \
                    type(MinimapUtil) == 'table', \
                    type(QuestUtil) == 'table', \
                    type(ReputationUtil) == 'table'",
        )
        .expect("Mainline subdir util probe should succeed");
    assert_eq!(
        mainline_namespaces,
        (true, true, true, true),
        "Blizzard_FrameXMLUtil's TOC `[Family]\\<file>.lua` entries (mapped to \
         `Mainline\\<file>.lua` by src/toc.rs:145) load the Mainline subdir variants: \
         MapUtil (Mainline/MapUtil.lua — GetDisplayableMapForPlayer, map-id resolution), \
         MinimapUtil (Mainline/MinimapUtil.lua — minimap-icon coordinate helpers), \
         QuestUtil (Mainline/QuestUtils.lua — quest tag/threshold helpers), \
         ReputationUtil (Mainline/ReputationUtil.lua — paragon/major-faction helpers)"
    );

    let other_mainline: (bool, bool, bool, bool) = env
        .eval(
            "return type(TraitUtil) == 'table', \
                    type(PlayerSpellsUtil) == 'table', \
                    type(RAFUtil) == 'table', \
                    type(GetExpansionName) == 'function'",
        )
        .expect("Additional Mainline subdir util probe should succeed");
    assert_eq!(
        other_mainline,
        (true, true, true, true),
        "Mainline subdir also publishes TraitUtil (Mainline/TraitUtil.lua — talent-tree \
         delves-companion frame callbacks), PlayerSpellsUtil (FrameTabs and \
         SpellBookCategories enums for the spellbook tab UI), RAFUtil (Mainline/\
         RAFUtil.lua — recruit-a-friend helpers), and the bare GetExpansionName(expansion) \
         global from Mainline/ExpansionUtil.lua (the file does not declare an \
         `ExpansionUtil` table — only this one global function that resolves \
         `EXPANSION_NAME<id>` localization strings via _G[tag])"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_runeforge_and_renown_namespaces(env: &WowLuaEnv) {

    let mixins: (bool, bool, bool, bool) = env
        .eval(
            "return type(RuneforgeUtil) == 'table', \
                    type(RuneforgeEffectOwnerMixin) == 'table', \
                    type(RuneforgeSystemMixin) == 'table', \
                    type(RenownRewardUtil) == 'table'",
        )
        .expect("Runeforge/renown probe should succeed");
    assert_eq!(
        mixins,
        (true, true, true, true),
        "RuneforgeUtil.lua publishes RuneforgeUtil + four mixins for the Shadowlands \
         legendary crafter: RuneforgeCovenantSigilMixin, RuneforgePowerBaseMixin, \
         RuneforgeEffectOwnerMixin, RuneforgeSystemMixin (= CreateFromMixins(\
         RuneforgeEffectOwnerMixin)). RenownRewardUtil.lua publishes the namespace \
         used by major-faction renown reward icons. All four are mainline-gated"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_currency_and_profession_namespaces(env: &WowLuaEnv) {

    let currency_namespaces: (bool, bool, bool, bool) = env
        .eval(
            "return type(CurrencyContainerUtil) == 'table', \
                    type(ProfessionsUtil) == 'table', \
                    type(TitleUtil) == 'table', \
                    type(AdventureGuideUtil) == 'table'",
        )
        .expect("Currency/profession util probe should succeed");
    assert_eq!(
        currency_namespaces,
        (true, true, true, true),
        "Blizzard_FrameXMLUtil publishes CurrencyContainerUtil (CurrencyContainer.lua \
         — currency-amount formatting), ProfessionsUtil (ProfessionsUtil.lua — \
         crafting reagent-availability helpers), TitleUtil (TitleUtil.lua — player \
         title display formatting), AdventureGuideUtil (AdventureGuideUtil.lua — \
         encounter-journal navigation helpers used by IsInInstance / dungeon-map \
         lookups)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_cooldown_and_fading_helpers(env: &WowLuaEnv) {

    let cooldown_helpers: (bool, bool, bool) = env
        .eval(
            "return type(CooldownFrame_Set) == 'function', \
                    type(CooldownFrame_Clear) == 'function', \
                    type(CooldownFrame_SetDisplayAsPercentage) == 'function'",
        )
        .expect("CooldownFrame_ probe should succeed");
    assert_eq!(
        cooldown_helpers,
        (true, true, true),
        "Cooldown.lua publishes the CooldownFrame_* helper family used by every \
         action-bar / aura-icon cooldown swipe: CooldownFrame_Set(self, start, \
         duration, enable, forceShowDrawEdge, modRate) (the canonical entry point), \
         CooldownFrame_Clear (cancel a swipe), CooldownFrame_SetDisplayAsPercentage \
         (numeric remaining display)"
    );

    let fading_helpers: (bool, bool, bool, bool, bool) = env
        .eval(
            "return type(FadingFrame_OnLoad) == 'function', \
                    type(FadingFrame_Show) == 'function', \
                    type(FadingFrame_OnUpdate) == 'function', \
                    type(FadingFrame_SetFadeInTime) == 'function', \
                    type(FadingFrame_SetHoldTime) == 'function'",
        )
        .expect("FadingFrame_ probe should succeed");
    assert_eq!(
        fading_helpers,
        (true, true, true, true, true),
        "FadingFrame.lua publishes FadingFrame_OnLoad / _Show / _OnUpdate / \
         _SetFadeInTime / _SetHoldTime / _SetFadeOutTime / _GetRemainingTime / \
         _CopyTimes — the auto-fade timer driver used by the chat alert text, raid \
         warning text, and similar HUD overlays"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_animation_system_helpers(env: &WowLuaEnv) {

    let anim_helpers: (bool, bool) = env
        .eval(
            "return type(SetUpAnimation) == 'function', \
                    type(CancelAnimations) == 'function'",
        )
        .expect("AnimationSystem probe should succeed");
    assert_eq!(
        anim_helpers,
        (true, true),
        "AnimationSystem.lua publishes SetUpAnimation(frame, animTable, postFunc, \
         reverse) — the table-driven animation builder consumed by the alert toast / \
         talking-head / glow-emitter modules — and CancelAnimations(frame) to flush \
         queued animations on a frame"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_item_button_util_event_registry(env: &WowLuaEnv) {

    let registry: (bool, bool, bool, bool) = env
        .eval(
            "return type(ItemButtonUtil.RegisterCallback) == 'function', \
                    type(ItemButtonUtil.UnregisterCallback) == 'function', \
                    type(ItemButtonUtil.TriggerEvent) == 'function', \
                    type(ItemButtonUtil.Event) == 'table'",
        )
        .expect("ItemButtonUtil registry probe should succeed");
    assert_eq!(
        registry,
        (true, true, true, true),
        "ItemUtil.lua wires ItemButtonUtil to a CallbackRegistry instance: \
         RegisterCallback / UnregisterCallback / TriggerEvent forward to the inner \
         registry, and the .Event table mirrors the registry's event enum. Backs the \
         filter-bags / open-and-filter-character-frame interactions across the \
         crafting and item-context UIs"
    );
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_item_context_enums(env: &WowLuaEnv) {

    let enums: (bool, bool) = env
        .eval(
            "return type(ItemButtonUtil.ItemContextEnum) == 'table', \
                    type(ItemButtonUtil.ItemContextMatchResult) == 'table'",
        )
        .expect("ItemButtonUtil enum probe should succeed");
    assert_eq!(
        enums,
        (true, true),
        "ItemUtil.lua declares ItemButtonUtil.ItemContextEnum (item-filter context \
         labels — Scrapping, PickRuneforgeBaseItem, ReplaceRuneforgeItem, ...) and \
         ItemButtonUtil.ItemContextMatchResult (Match / Mismatch / DoesNotApply) used \
         by the bag / character-frame filter-result coloring pipeline"
    );
}
}

prefork_full_ui_case! {
fn area_poi_timed_consumer_and_arguments(env: &WowLuaEnv) {
    env.eval::<()>(r#"
        local f = C_AreaPoiInfo.IsAreaPOITimed
        assert(type(f) == 'function', 'IsAreaPOITimed must be published')
        local function results(...) return select('#', ...), ... end
        local n, timed, hidden = results(f(-2147483648))
        assert(n == 2 and timed == false and hidden == nil, 'unknown POI return shape')
        n, timed, hidden = results(f('-2147483648'))
        assert(n == 2 and timed == false and hidden == nil, 'numeric string input')
        n, timed, hidden = results(f(1099.9))
        assert(n == 2 and timed == true and hidden == false, 'timed POI with visible timer')
        n, timed, hidden = results(f('7459'))
        assert(n == 2 and timed == true and hidden == true, 'timed POI with hidden timer')
        for _, value in ipairs({false, {}, 'not a number', 2147483648}) do
            local ok = pcall(f, value)
            assert(not ok, 'invalid argument must raise')
        end
        assert(not pcall(f), 'missing argument must raise')
        assert(not pcall(f, nil), 'nil argument must raise')

        local shown = AreaPoiUtil.TryShowTooltip(nil, nil, {areaPoiID = -2147483648})
        assert(shown == false, 'unknown POI without tooltip content must not show')
    "#).expect("selected AreaPoiUtil consumer and API contract");
}
}

prefork_full_ui_case! {
fn blizzard_frame_xml_util_publishes_dragonriding_and_arena_namespaces(env: &WowLuaEnv) {

    let extras: (bool, bool, bool, bool) = env
        .eval(
            "return type(DragonridingUtil) == 'table', \
                    type(AreaPoiUtil) == 'table', \
                    type(CampaignUtil) == 'table', \
                    type(ItemTransmogInfoMixin) == 'table'",
        )
        .expect("Dragonriding/area POI probe should succeed");
    assert_eq!(
        extras,
        (true, true, true, true),
        "DragonridingUtil.lua publishes the Dragonflight skyriding namespace; \
         AreaPoiUtil.lua publishes the area-POI tooltip helpers (mainline-only); \
         CampaignUtil.lua publishes the Shadowlands/War Within campaign-progress \
         namespace; ItemTransmogInfoMixin (ItemUtil.lua) is the shared transmog-info \
         table mixed into every transmog-aware item display"
    );
}
}
