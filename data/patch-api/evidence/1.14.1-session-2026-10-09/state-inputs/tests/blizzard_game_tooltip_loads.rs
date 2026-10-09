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

fn game_tooltip_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_GameTooltip")
}

fn game_tooltip_mainline_toc() -> PathBuf {
    game_tooltip_dir().join("Blizzard_GameTooltip_Mainline.toc")
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
fn blizzard_game_tooltip_picks_mainline_toc_variant() {
    let resolved = find_toc_file(&game_tooltip_dir())
        .expect("Blizzard_GameTooltip directory must contain a discoverable TOC");
    let resolved_name = resolved
        .file_name()
        .expect("resolved TOC must have a filename")
        .to_str()
        .expect("resolved TOC filename must be utf-8");

    assert_eq!(
        resolved_name, "Blizzard_GameTooltip_Mainline.toc",
        "Blizzard_GameTooltip ships a single `_Mainline.toc` and `find_toc_file` \
         (src/loader/mod.rs:65) prefers the Mainline-suffixed variant when present"
    );
}

#[test]
fn blizzard_game_tooltip_mainline_toc_declares_three_deps_and_no_load_flags() {
    let toc = TocFile::from_file(&game_tooltip_mainline_toc())
        .expect("Blizzard_GameTooltip Mainline TOC parse");

    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_GameTooltip has no `## LoadOnDemand` — the tooltip frames must be \
         eagerly created at load time so any addon (or the world cursor) can request a \
         tooltip immediately without waiting for an LOD request"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_GameTooltip does not declare `## UseSecureEnvironment` — tooltip \
         construction runs in the standard taint environment (it only reads cursor / \
         unit data, never issues protected actions)"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_GameTooltip declares `## AllowLoadGameType: mainline` which \
         src/toc.rs:299 treats as the unrestricted retail game type — \
         `is_game_type_restricted()` returns false on standard mainline retail"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_GameTooltip declares no `## SavedVariables` — there is no per-account \
         tooltip preference (anchor preferences are stored under the `Settings` panel addon)"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "Blizzard_EditMode".to_string(),
            "Blizzard_GarrisonBase".to_string(),
            "Blizzard_Colors".to_string(),
        ],
        "`## Dependencies: Blizzard_EditMode, Blizzard_GarrisonBase, Blizzard_Colors` — \
         EditMode provides the EditModeHudTooltipSystemTemplate that wraps \
         GameTooltipDefaultContainer (line 242), GarrisonBase provides the \
         GarrisonFollowerTooltipContentsTemplate consumed at line 101, and Blizzard_Colors \
         provides NORMAL_FONT_COLOR / GREEN_FONT_COLOR used throughout the \
         TOOLTIP_QUEST_REWARDS_STYLE_* dictionaries. Got: {:?}",
        deps
    );

    let toc_text = std::fs::read_to_string(game_tooltip_mainline_toc())
        .expect("Blizzard_GameTooltip Mainline TOC should read");
    assert!(
        toc_text.contains("## DefaultState: enabled"),
        "Blizzard_GameTooltip_Mainline.toc declares `## DefaultState: enabled` — the \
         tooltip subsystem is core UI and on by default"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline"),
        "Blizzard_GameTooltip_Mainline.toc declares `## AllowLoadGameType: mainline` so \
         it loads on retail; classic flavors ship a separate tooltip addon variant"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_GameTooltip_Mainline.toc declares NO `## AllowLoad:` line — this \
         means `allows_screen` falls through to the Game-only default (src/toc.rs:311). \
         The login glue screen uses GlueParent's own GlueTooltip, not GameTooltip"
    );
}

#[test]
fn blizzard_game_tooltip_defaults_to_game_screen_only() {
    let toc = TocFile::from_file(&game_tooltip_mainline_toc())
        .expect("Blizzard_GameTooltip Mainline TOC parse");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Missing `## AllowLoad:` defaults to Game-only (src/toc.rs:311) — Game must be \
         allowed because the Esc-key game menu, action bars, and unit frames all rely \
         on GameTooltip"
    );
    assert!(
        !toc.allows_screen(ScreenKind::Login),
        "Missing `## AllowLoad:` excludes Login — the login glue screen uses GlueTooltip \
         (a separate addon), not the in-world GameTooltip"
    );
    assert!(
        !toc.allows_screen(ScreenKind::CharacterSelect),
        "Missing `## AllowLoad:` excludes CharacterSelect for the same reason — the \
         character-select glue screen also uses GlueTooltip"
    );
}

#[test]
fn blizzard_game_tooltip_auto_loads_on_game_and_skips_login() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GameTooltip");
    assert!(
        in_game,
        "Blizzard_GameTooltip has no `## LoadOnDemand` and defaults to Game-only — it \
         MUST appear in Game-screen auto-discovery (it is itself a dependency of \
         Blizzard_GameMenu among many others)"
    );

    let login_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Login);
    let in_login = login_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_GameTooltip");
    assert!(
        !in_login,
        "Game-only default excludes Blizzard_GameTooltip from Login auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_loads_via_full_game_ui_without_errors(env: &WowLuaEnv) {

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("GameTooltip")
                || message.contains("ShoppingTooltip")
                || message.contains("EmbeddedItemTooltip")
                || message.contains("HealthBar_OnValueChanged")
                || message.contains("TooltipConstants")
                || message.contains("TOOLTIP_QUEST_REWARDS_STYLE")
        })
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Blizzard_GameTooltip emitted Lua errors during the full Game-screen load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_is_addon_loaded_returns_true_after_full_game_ui_load(env: &WowLuaEnv) {

    let post_load: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_GameTooltip') and true or false")
        .expect("IsAddOnLoaded probe should succeed");

    assert!(
        post_load,
        "C_AddOns.IsAddOnLoaded('Blizzard_GameTooltip') must return truthy after a \
         successful full-game-UI load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_publishes_top_level_uiparent_frames(env: &WowLuaEnv) {

    let frames: (bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GameTooltip) == 'table', \
                    type(EmbeddedItemTooltip) == 'table', \
                    type(GameNoHeaderTooltip) == 'table', \
                    type(GameSmallHeaderTooltip) == 'table', \
                    type(ShoppingTooltip1) == 'table', \
                    type(ShoppingTooltip2) == 'table', \
                    type(GameTooltipDefaultContainer) == 'table'",
        )
        .expect("Top-level frame probes should succeed");
    assert_eq!(
        frames,
        (true, true, true, true, true, true, true),
        "GameTooltip.xml publishes 7 top-level UIParent-parented frames as named \
         globals: GameTooltip (line 249 — the canonical right-click/cursor tooltip with \
         supportsItemComparison KeyValue and an ItemTooltip parentKey'd EmbeddedItemTooltip \
         child), EmbeddedItemTooltip (line 276 — the inline item display used in quest \
         reward popups; carries a BottomFontString and its own ItemTooltip child), \
         GameNoHeaderTooltip (line 313 — variant that uses GameTooltipText for both lines \
         instead of a larger font for line 1), GameSmallHeaderTooltip (line 322 — variant \
         with SystemFont_Med2 line 1 instead of GameTooltipHeaderText), ShoppingTooltip1 \
         (line 239) and ShoppingTooltip2 (line 240) — TOOLTIP-strata, hidden, \
         clampedToScreen=true, used by GameTooltip.shoppingTooltips for item comparison \
         when hovering equippable items, and GameTooltipDefaultContainer (line 242) — \
         LOW-strata EditModeHudTooltipSystemTemplate hidden host that the EditMode anchor \
         system uses to position GameTooltip in the HUD layout"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_virtual_templates_are_applied_via_inheritance(env: &WowLuaEnv) {

    let templates_dont_leak: (bool, bool, bool, bool, bool) = env
        .eval(
            "return _G.GameTooltipTemplate == nil, \
                    _G.InternalEmbeddedItemTooltipTemplate == nil, \
                    _G.ShoppingTooltipTemplate == nil, \
                    _G.TooltipStatusBarTemplate == nil, \
                    _G.TooltipProgressBarTemplate == nil",
        )
        .expect("template-leak probe should succeed");
    assert_eq!(
        templates_dont_leak,
        (true, true, true, true, true),
        "GameTooltip.xml publishes 5 virtual templates: GameTooltipTemplate (line 4), \
         InternalEmbeddedItemTooltipTemplate (line 39), ShoppingTooltipTemplate (line \
         109), TooltipStatusBarTemplate (line 145), TooltipProgressBarTemplate (line \
         172). All declare `virtual=\"true\"` so they live only in the XML template \
         registry, not as `_G.*` Lua globals — the canonical contract for templates"
    );

    let template_application: (bool, bool, bool) = env
        .eval(
            "return type(GameTooltip.StatusBar) == 'table', \
                    type(GameTooltip.ItemTooltip) == 'table', \
                    type(EmbeddedItemTooltip.ItemTooltip) == 'table'",
        )
        .expect("template-application probe should succeed");
    assert_eq!(
        template_application,
        (true, true, true),
        "Templates are applied via inheritance: GameTooltip inherits GameTooltipTemplate \
         and gets the parentKey'd StatusBar child (GameTooltipTemplate xml line 9 — proves \
         GameTooltipTemplate registered and applied); GameTooltip's own Frames block \
         (line 254) instantiates an InternalEmbeddedItemTooltipTemplate as ItemTooltip \
         parentKey (proves InternalEmbeddedItemTooltipTemplate registered); \
         EmbeddedItemTooltip likewise nests an ItemTooltip child (xml line 287) — both \
         only exist if the template registry resolved the inherits=... attribute"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_publishes_three_mixins(env: &WowLuaEnv) {

    let mixins: (bool, bool, bool) = env
        .eval(
            "return type(GameTooltipDataMixin) == 'table', \
                    type(GameTooltipUnitHealthBarMixin) == 'table', \
                    type(GameTooltipUnitHealthBarSecureMixin) == 'table'",
        )
        .expect("Mixin probes should succeed");
    assert_eq!(
        mixins,
        (true, true, true),
        "GameTooltip.lua publishes 3 mixin globals: GameTooltipDataMixin (line 937 — \
         `CreateFromMixins(TooltipDataHandlerMixin)`, owns OnLoad/RefreshData/\
         RefreshDataNextUpdate/OnEvent/SetWorldCursor and the GetItem/GetSpell/GetUnit \
         shims that delegate to TooltipUtil.GetDisplayedItem/Spell/Unit), \
         GameTooltipUnitHealthBarMixin (line 1025 — owns OnLoad/SetWatch/StopUpdates/\
         ClearWatch/ResetUnitHealth/UpdateUnitHealth/OnUpdate; the unit-health StatusBar \
         that polls UnitHealth/UnitHealthMax in OnUpdate), and \
         GameTooltipUnitHealthBarSecureMixin (line 1079 — secure-environment override \
         attached via secureMixin=... on the StatusBar; provides ResetUnitHealth/\
         UpdateUnitHealth that read from the secure unit cache instead of UnitHealth())"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_data_mixin_extends_tooltip_data_handler(env: &WowLuaEnv) {

    let methods: (bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GameTooltipDataMixin.OnLoad) == 'function', \
                    type(GameTooltipDataMixin.RefreshData) == 'function', \
                    type(GameTooltipDataMixin.RefreshDataNextUpdate) == 'function', \
                    type(GameTooltipDataMixin.OnEvent) == 'function', \
                    type(GameTooltipDataMixin.SetWorldCursor) == 'function'",
        )
        .expect("GameTooltipDataMixin method probes should succeed");
    assert_eq!(
        methods,
        (true, true, true, true, true),
        "GameTooltipDataMixin (line 937) defines 5 own methods on top of the \
         TooltipDataHandlerMixin base copied in via CreateFromMixins: OnLoad (line 939 — \
         chains GameTooltip_OnLoad, seeds shoppingTooltips={{ShoppingTooltip1,2}}, hides \
         the BattlePet tooltip), RefreshData (line 945 — clears shouldRefreshData and \
         calls RebuildFromTooltipInfo), RefreshDataNextUpdate (line 950 — sets \
         updateTooltipTimer=0 and shouldRefreshData=true so the next OnUpdate tick \
         rebuilds), OnEvent (line 955 — handles TOOLTIP_DATA_UPDATE by calling \
         RefreshDataNextUpdate when dataInstanceID matches), SetWorldCursor (line 964 — \
         the world-mouseover entry point: dispatches by Enum.WorldCursorAnchorType — \
         Default → GameTooltip_SetDefaultAnchor, Cursor → SetOwner with \
         ANCHOR_CURSOR/ANCHOR_CURSOR_RIGHT, Nameplate → SetOwner+SetObjectTooltipPosition; \
         uses securecallfunction on GetPrimaryTooltipInfo to prevent prior-tooltip taint \
         from leaking into ProcessInfo)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_health_bar_global_publishes_value_changed_handler(env: &WowLuaEnv) {

    let handler: bool = env
        .eval("return type(HealthBar_OnValueChanged) == 'function'")
        .expect("HealthBar_OnValueChanged probe should succeed");
    assert!(
        handler,
        "HealthBar_OnValueChanged (HealthBar.lua:2 — top-level file in the Blizzard_\
         GameTooltip dir, NOT in Mainline\\) is the canonical OnValueChanged handler for \
         the unit-health StatusBar inside tooltips. It maps health fraction to RGB via a \
         smooth/non-smooth gradient: smooth-mode interpolates green→yellow→red across \
         (max-min) range; non-smooth uses fixed green (1,0,0). It respects \
         self.lockColor — when set, the bar's color is owned by external code and the \
         handler is a no-op. The TOC enumerates HealthBar.lua at the top level (no \
         per-line `[Family]` marker), proving it loads on every game type"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_publishes_quest_reward_styles(env: &WowLuaEnv) {

    let style_count: f64 = env
        .eval(
            "local n = 0 \
             for _, name in ipairs({ \
                'TOOLTIP_QUEST_REWARDS_STYLE_DEFAULT', \
                'TOOLTIP_QUEST_REWARDS_STYLE_WORLD_QUEST', \
                'TOOLTIP_QUEST_REWARDS_STYLE_NO_HEADER', \
                'TOOLTIP_QUEST_REWARDS_STYLE_CONTRIBUTION', \
                'TOOLTIP_QUEST_REWARDS_STYLE_PVP_BOUNTY', \
                'TOOLTIP_QUEST_REWARDS_STYLE_ISLANDS_QUEUE', \
                'TOOLTIP_QUEST_REWARDS_STYLE_EMISSARY_REWARD', \
                'TOOLTIP_QUEST_REWARDS_STYLE_CALLING_REWARD', \
                'TOOLTIP_QUEST_REWARDS_PRIORITIZE_CURRENCY_OVER_ITEM', \
                'TOOLTIP_QUEST_REWARDS_STYLE_QUEST_CHOICE', \
                'TOOLTIP_QUEST_REWARDS_STYLE_NONE', \
                'TOOLTIP_QUEST_REWARDS_STYLE_INITIATIVE_TASK' \
             }) do \
                if type(_G[name]) == 'table' then n = n + 1 end \
             end \
             return n",
        )
        .expect("Quest-reward style table count probe should succeed");
    assert_eq!(
        style_count, 12.0,
        "GameTooltip.lua publishes 12 TOOLTIP_QUEST_REWARDS_STYLE_* dictionaries \
         (lines 17-118 + 87-96 for the PRIORITIZE_CURRENCY_OVER_ITEM variant): each is a \
         {{headerText, headerColor, prefixBlankLineCount, postHeaderBlankLineCount, \
         wrapHeaderText, fullItemDescription, atLeastShowAzerite?, \
         prioritizeCurrencyOverItem?, showCollectionText?, clampFavorToCycleCap?}} \
         dictionary consumed by GameTooltip_AddQuestRewardsToTooltip (line 193) to choose \
         header text, blank-line padding, and reward-formatting policy. \
         Found {style_count} of the 12 expected styles"
    );

    let default_style: (bool, f64, f64, bool, bool) = env
        .eval(
            "return TOOLTIP_QUEST_REWARDS_STYLE_DEFAULT.headerText == QUEST_REWARDS, \
                    TOOLTIP_QUEST_REWARDS_STYLE_DEFAULT.prefixBlankLineCount, \
                    TOOLTIP_QUEST_REWARDS_STYLE_DEFAULT.postHeaderBlankLineCount, \
                    TOOLTIP_QUEST_REWARDS_STYLE_DEFAULT.wrapHeaderText, \
                    TOOLTIP_QUEST_REWARDS_STYLE_DEFAULT.fullItemDescription",
        )
        .expect("DEFAULT style probe should succeed");
    assert_eq!(
        default_style,
        (true, 1.0, 0.0, true, true),
        "TOOLTIP_QUEST_REWARDS_STYLE_DEFAULT (lines 17-24) is the canonical default \
         applied when no style is passed: headerText is the QUEST_REWARDS localization \
         constant (whose en_US value is `Rewards` in current builds — the addon stores \
         the constant reference, not the literal string), prefixBlankLineCount=1, \
         postHeaderBlankLineCount=0, wrapHeaderText=true, fullItemDescription=true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_publishes_backdrop_styles(env: &WowLuaEnv) {

    let backdrop_styles: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GAME_TOOLTIP_BACKDROP_STYLE_DEFAULT_DARK) == 'table', \
                    type(GAME_TOOLTIP_BACKDROP_STYLE_AZERITE_ITEM) == 'table', \
                    type(GAME_TOOLTIP_BACKDROP_STYLE_CORRUPTED_ITEM) == 'table', \
                    type(GAME_TOOLTIP_BACKDROP_STYLE_RUNEFORGE_LEGENDARY) == 'table', \
                    type(GAME_TOOLTIP_BACKDROP_STYLE_CLASS_TALENT) == 'table', \
                    type(GAME_TOOLTIP_TEXTUREKIT_BACKDROP_STYLES) == 'table'",
        )
        .expect("Backdrop style probes should succeed");
    assert_eq!(
        backdrop_styles,
        (true, true, true, true, true, true),
        "GameTooltip.lua publishes 5 GAME_TOOLTIP_BACKDROP_STYLE_* dictionaries (lines \
         326-364 — DEFAULT_DARK / AZERITE_ITEM / CORRUPTED_ITEM / RUNEFORGE_LEGENDARY / \
         CLASS_TALENT — each carries a layoutType plus optional overlayAtlasTop/Bottom + \
         padding) and the GAME_TOOLTIP_TEXTUREKIT_BACKDROP_STYLES lookup (line 366 — maps \
         a textureKit string e.g. `jailerstower` to one of the backdrop style tables, \
         used by SharedTooltip_SetBackdropStyle to switch the tooltip's overlay textures \
         based on the source NPC/item's texture kit"
    );

    let azerite_layout: (String, String, f64) = env
        .eval(
            "return GAME_TOOLTIP_BACKDROP_STYLE_AZERITE_ITEM.layoutType, \
                    GAME_TOOLTIP_BACKDROP_STYLE_AZERITE_ITEM.overlayAtlasTop, \
                    GAME_TOOLTIP_BACKDROP_STYLE_AZERITE_ITEM.overlayAtlasTopScale",
        )
        .expect("AZERITE backdrop probe should succeed");
    assert_eq!(
        azerite_layout,
        (
            "TooltipAzeriteLayout".to_string(),
            "AzeriteTooltip-Topper".to_string(),
            0.75,
        ),
        "GAME_TOOLTIP_BACKDROP_STYLE_AZERITE_ITEM (lines 330-340) drives the \
         azerite-item overlay: layoutType=TooltipAzeriteLayout chooses the \
         AzeriteTooltip nine-slice, overlayAtlasTop=AzeriteTooltip-Topper places the \
         decorative gold lattice atop the tooltip at 0.75 scale, padding {{6,6,6,6}}"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_publishes_helper_functions(env: &WowLuaEnv) {

    let helpers: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GameTooltip_OnLoad) == 'function', \
                    type(GameTooltip_OnShow) == 'function', \
                    type(GameTooltip_OnHide) == 'function', \
                    type(GameTooltip_OnUpdate) == 'function', \
                    type(GameTooltip_Hide) == 'function', \
                    type(GameTooltip_HideTooltip) == 'function'",
        )
        .expect("Lifecycle helper probes should succeed");
    assert_eq!(
        helpers,
        (true, true, true, true, true, true),
        "GameTooltip.lua publishes the canonical 6 lifecycle helpers as bare globals \
         (consumed via XML `<OnX function=\"GameTooltip_OnX\"/>` script binding): \
         GameTooltip_OnLoad (line 306), GameTooltip_OnShow (line 370), GameTooltip_OnHide \
         (line 383), GameTooltip_OnUpdate (line 437), GameTooltip_Hide (line 604 — \
         convenience wrapper that calls GameTooltip:Hide() — referenced as a one-arg \
         helper in legacy XML), GameTooltip_HideTooltip (line 610 — null-safe variant: \
         `if tooltip then tooltip:Hide() end`)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_publishes_status_and_progress_bar_helpers(env: &WowLuaEnv) {

    let bar_helpers: (bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(GameTooltip_ClearStatusBars) == 'function', \
                    type(GameTooltip_ShowStatusBar) == 'function', \
                    type(GameTooltip_AddStatusBar) == 'function', \
                    type(GameTooltip_ClearProgressBars) == 'function', \
                    type(GameTooltip_ShowProgressBar) == 'function', \
                    type(GameTooltip_AddProgressBar) == 'function', \
                    type(GameTooltip_ClearAllStatusBars) == 'function'",
        )
        .expect("Status/progress bar helper probes should succeed");
    assert_eq!(
        bar_helpers,
        (true, true, true, true, true, true, true),
        "GameTooltip.lua publishes the StatusBar trio (lines 490-531 — \
         GameTooltip_ClearStatusBars / ShowStatusBar / AddStatusBar — driving the \
         TooltipStatusBarTemplate Pool used for honor/reputation/XP gains) and the \
         ProgressBar trio (lines 533-555 — GameTooltip_ClearProgressBars / \
         ShowProgressBar / AddProgressBar — driving the TooltipProgressBarTemplate Pool \
         used for quest/objective progress overlays); plus \
         GameTooltip_ClearAllStatusBars (line 502 — combined clear)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_publishes_embedded_item_tooltip_api(env: &WowLuaEnv) {

    let api: (bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            "return type(EmbeddedItemTooltip_UpdateSize) == 'function', \
                    type(EmbeddedItemTooltip_Hide) == 'function', \
                    type(EmbeddedItemTooltip_Clear) == 'function', \
                    type(EmbeddedItemTooltip_PrepareForItem) == 'function', \
                    type(EmbeddedItemTooltip_PrepareForSpell) == 'function', \
                    type(EmbeddedItemTooltip_SetItemByID) == 'function', \
                    type(EmbeddedItemTooltip_SetItemByQuestReward) == 'function', \
                    type(EmbeddedItemTooltip_SetCurrencyByID) == 'function'",
        )
        .expect("EmbeddedItemTooltip API probes should succeed");
    assert_eq!(
        api,
        (true, true, true, true, true, true, true, true),
        "GameTooltip.lua publishes 8 of the EmbeddedItemTooltip_* helper API (lines \
         751-934): UpdateSize / Hide / Clear / PrepareForItem / PrepareForSpell / \
         SetItemByID / SetItemByQuestReward / SetCurrencyByID — these drive the \
         InternalEmbeddedItemTooltipTemplate's Icon + Count + Text + child Tooltip + \
         FollowerTooltip layout; PrepareForItem hides spell/follower children and sets \
         the Icon container, SetItemByQuestReward dispatches to spell/currency/item by \
         the rewardType enum"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_constants_table_publishes(env: &WowLuaEnv) {

    let wrap: (String, bool) = env
        .eval(
            "return type(TooltipConstants), \
                    TooltipConstants and TooltipConstants.WrapText or false",
        )
        .expect("TooltipConstants probe should succeed");
    assert_eq!(
        wrap,
        ("table".to_string(), true),
        "TooltipConstants (line 2) is the canonical tooltip-feature switch table; \
         WrapText=true means GameTooltip_AddQuestRewardsToTooltip and similar helpers \
         enable line wrapping by default — addons override this single boolean to disable \
         wrapping in narrow tooltip surfaces"
    );
}
}

prefork_full_ui_case! {
fn blizzard_game_tooltip_main_tooltip_uses_uiparent_with_tooltip_strata(env: &WowLuaEnv) {

    let strata: (String, String) = env
        .eval(
            "return GameTooltip:GetFrameStrata(), \
                    GameTooltip:GetParent():GetName()",
        )
        .expect("GameTooltip strata/parent probes should succeed");
    assert_eq!(
        strata,
        ("TOOLTIP".to_string(), "UIParent".to_string()),
        "GameTooltip is parented to UIParent (xml line 249, `parent=\"UIParent\"`) and \
         inherits TOOLTIP frame strata from SharedTooltipTemplate — TOOLTIP is the \
         topmost strata in the canonical strata stack, so tooltips render above DIALOG / \
         FULLSCREEN_DIALOG modal panels"
    );

    let shopping_strata: (String, bool) = env
        .eval(
            "return ShoppingTooltip1:GetFrameStrata(), \
                    ShoppingTooltip1:IsClampedToScreen()",
        )
        .expect("ShoppingTooltip1 strata/clamp probe should succeed");
    assert_eq!(
        shopping_strata,
        ("TOOLTIP".to_string(), true),
        "ShoppingTooltip1 (xml line 239) is `frameStrata=\"TOOLTIP\"` and \
         `clampedToScreen=\"true\"` — the comparison popups must ride at TOOLTIP strata \
         alongside GameTooltip and clamp into the visible viewport when the source \
         tooltip is near a screen edge"
    );
}
}
