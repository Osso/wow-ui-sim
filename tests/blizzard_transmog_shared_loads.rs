use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn transmog_shared_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_TransmogShared")
}

fn transmog_shared_toc() -> PathBuf {
    transmog_shared_dir().join("Blizzard_TransmogShared.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &["Blizzard_StaticPopup_Game", "Blizzard_GameTooltip"];

const TRANSMOG_UTIL_FUNCTIONS: &[&str] = &[
    "GetInfoForEquippedSlot",
    "CanEnchantSource",
    "GetWeaponInfoForEnchant",
    "GetBestWeaponInfoForIllusionDressup",
    "GetSlotID",
    "GetSlotName",
    "CreateTransmogLocation",
    "GetTransmogLocation",
    "GetCorrespondingHandTransmogLocation",
    "GetTransmogLocationLookupKey",
    "GetSetIcon",
    "IsSecondaryTransmoggedForItemLocation",
    "GetItemLocationFromTransmogLocation",
    "IsCategoryLegionArtifact",
    "IsCategoryRangedWeapon",
    "IsValidTransmogSlotID",
    "OpenCollectionToItem",
    "OpenCollectionToSet",
    "OpenCollectionUI",
    "GetEmptyItemTransmogInfoList",
    "CreateCustomSetSlashCommand",
    "ParseCustomSetSlashCommand",
    "GetWardrobeModelSetupData",
    "GetWardrobeModelSetupGearData",
    "GetUseTransmogSkin",
    "GetCameraVariation",
    "ToggleFavorite",
    "IsValidItemTransmogInfoList",
    "IsCustomSetCollected",
];

const TRANSMOG_LOCATION_METHODS: &[&str] = &[
    "Set",
    "IsAppearance",
    "IsIllusion",
    "IsEitherHand",
    "IsMainHand",
    "IsOffHand",
    "IsRangedSlot",
    "IsSecondary",
    "IsEqual",
    "GetSlot",
    "GetSlotID",
    "GetType",
    "GetSlotName",
    "GetArmorCategoryID",
    "GetLookupKey",
    "GetData",
];

const ITEM_MODEL_BASE_METHODS: &[&str] = &[
    "OnLoad",
    "OnModelLoaded",
    "OnMouseUp",
    "OnMouseDown",
    "OnEnter",
    "OnLeave",
    "OnUpdate",
    "OnShow",
    "Reload",
    "UpdateCamera",
    "SetDesaturated",
    "ToggleFavorite",
    "GetAppearanceInfo",
    "GetCollectionFrame",
    "GetIllusionLink",
    "GetAppearanceLink",
    "CanCheckDressUpClick",
];

const WARDROBE_SETS_DATA_PROVIDER_METHODS: &[&str] = &[
    "SortSets",
    "GetBaseSets",
    "GetBaseSetByID",
    "GetUsableSets",
    "GetAvailableSets",
    "GetVariantSets",
    "GetSetSourceData",
    "GetSetSourceCounts",
    "GetBaseSetData",
    "GetSetSourceTopCounts",
    "IsBaseSetNew",
    "ResetBaseSetNewStatus",
    "GetSortedSetSources",
    "ClearSets",
    "ClearBaseSets",
    "ClearVariantSets",
    "ClearUsableSets",
    "ClearAvailableSets",
    "GetIconForSet",
    "DetermineFavorites",
    "RefreshFavorites",
];

fn fresh_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&transmog_shared_dir()).expect("TransmogShared TOC resolves");
    assert_eq!(
        resolved,
        transmog_shared_toc(),
        "Bare TOC — no flavor suffix. Resolved via the bare-TOC path \
         in find_toc_file at src/loader/mod.rs:65-95"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_is_load_on_demand_with_two_dependencies() {
    let toc = TocFile::from_file(&transmog_shared_toc()).expect("TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "`## LoadOnDemand: 1` — but multiple non-LoD addons \
         (Blizzard_FrameXML, Blizzard_UIPanels_Game, \
         Blizzard_ObjectiveTracker, Blizzard_RecruitAFriend) declare \
         it as a hard dep, so pull_required_lod_addons promotes it to \
         the eager Game pool at module-load time"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps.len(),
        TOC_DEPENDENCIES.len(),
        "Must declare exactly {} hard deps. Got {}: {:?}",
        TOC_DEPENDENCIES.len(),
        deps.len(),
        deps
    );
    for expected in TOC_DEPENDENCIES {
        assert!(
            deps.iter().any(|d| d == expected),
            "TOC must declare `{expected}` — Blizzard_StaticPopup_Game \
             provides StaticPopupDialogs registry (used for the \
             TRANSMOG_FAVORITE_WARNING dialog at lua:1-12), \
             Blizzard_GameTooltip provides the tooltip surface ItemModel \
             OnEnter touches at lua:781-786. Got: {deps:?}"
        );
    }

    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "AllowLoadGameType absent → not restricted"
    );
    assert!(toc.default_enabled());
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_game_restricts_to_in_world_only() {
    let toc = TocFile::from_file(&transmog_shared_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` → toc.rs:308 returns true for Game"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — `AllowLoad: \
             Game` explicitly disallows glue. Even though glue-loaded \
             Blizzard_ChatFrameBase lists TransmogShared in OptionalDeps, \
             the allows_screen filter at loader/mod.rs:527 keeps it out \
             of the glue pool entirely"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn toc_raw_bytes_pin_four_directives_and_single_body_file() {
    let raw = std::fs::read_to_string(transmog_shared_toc()).expect("TOC reads utf-8");

    let expected_lines = [
        "## Title: Blizzard_TransmogShared",
        "## LoadOnDemand: 1",
        "## AllowLoad: Game",
        "## Dependencies: Blizzard_StaticPopup_Game, Blizzard_GameTooltip",
        "Blizzard_TransmogShared.lua",
    ];

    for line in expected_lines {
        assert!(
            raw.contains(line),
            "Raw TOC must pin `{line}` — TransmogShared has the \
             minimum-viable LoD shape: 4 metadata directives + 1 \
             pure-lua body file (no XML, no UI templates). Despite \
             LoadOnDemand=1, the dep-promotion path keeps it loaded \
             eagerly on Game"
        );
    }

    let body_files: Vec<&str> = raw
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("##") && !l.starts_with('#'))
        .collect();
    assert_eq!(
        body_files,
        vec!["Blizzard_TransmogShared.lua"],
        "TOC body must be exactly 1 entry — no XML, no Localization \
         trailer, no flavor-overrides"
    );

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("[Family]"));
    assert!(!raw.contains(".xml"));
}
}

post_wrath_ui_fixture! {
#[test]
fn promoted_into_game_eager_discovery_via_dependents() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_TransmogShared");
    assert!(
        found,
        "Blizzard_TransmogShared must appear in Game eager discovery — \
         despite LoadOnDemand=1, multiple non-LoD addons \
         (Blizzard_FrameXML, Blizzard_UIPanels_Game, \
         Blizzard_ObjectiveTracker, Blizzard_RecruitAFriend) declare \
         it as a hard dep, so pull_required_lod_addons \
         (loader/mod.rs:553-) promotes it from the lod_pool into the \
         eager addons map"
    );
}
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TransmogShared");
        assert!(
            !found,
            "Blizzard_TransmogShared must NOT appear on {screen:?} — \
             `AllowLoad: Game` is checked at loader/mod.rs:527 BEFORE \
             pool partitioning, so the addon never enters either pool \
             on glue screens. Even if Blizzard_ChatFrameBase \
             (AllowLoad: both, OptionalDeps: TransmogShared) loads on \
             glue, its optional dep is unresolvable there"
        );
    }
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in TOC_DEPENDENCIES {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Hard-dep directory `{dep}` must exist on disk — without \
             it the dependency-resolution path can't find a TOC and \
             load_addon would fail at TransmogShared load time"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "{dep} must have a discoverable TOC"
        );
    }
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn runtime_load_keeps_removed_inventory_slot_global_private() {
    let env = fresh_game_env();
    env.apply_post_event_workarounds();

    let result: String = env
        .eval(
            r#"
            if GetInventorySlotInfo ~= nil then
                return "public_before_load"
            end

            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_TransmogShared")
            if not loaded then
                return "load_failed:" .. tostring(reason)
            end
            if GetInventorySlotInfo ~= nil then
                return "public_after_load"
            end

            local location = TransmogUtil.GetTransmogLocation("HEADSLOT", Enum.TransmogType.Appearance, false)
            if not location then
                return "missing_head_location"
            end
            return "ok"
            "#,
        )
        .expect("runtime TransmogShared compatibility probe should run");

    assert_eq!(result, "ok");
}

post_wrath_ui_fixture! {
#[test]
fn direct_load_keeps_removed_inventory_slot_global_private() {
    let env = fresh_game_env();
    env.apply_post_event_workarounds();
    // Retail 12.1.0 removed the public global (`src/lua_api/globals/register.rs`
    // registers `inventory_slot` under `not(retail-12-1-0)`). WowForever is a
    // pre-12.1 client and keeps it public, so assert the load does not change the
    // profile's surface instead of hard-coding the Retail removal.
    let before: bool = env
        .eval("return GetInventorySlotInfo ~= nil")
        .expect("GetInventorySlotInfo surface probe");
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let result: String = env
        .eval(
            r#"
            if GetInventorySlotInfo ~= nil then
                return "public_after_load"
            end

            local location = TransmogUtil.GetTransmogLocation("HEADSLOT", Enum.TransmogType.Appearance, false)
            if not location then
                return "missing_head_location"
            end
            return "ok"
            "#,
        )
        .expect("direct TransmogShared compatibility probe should run");

    let after: bool = env
        .eval("return GetInventorySlotInfo ~= nil")
        .expect("GetInventorySlotInfo surface probe");
    assert_eq!(
        after, before,
        "loading Blizzard_TransmogShared must not change the public \
         GetInventorySlotInfo surface"
    );
    if before {
        assert_eq!(result, "public_after_load");
    } else {
        assert_eq!(result, "ok");
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_publishes_transmog_util_table() {
    let env = fresh_game_env();
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let kind: String = env
        .eval("return type(TransmogUtil)")
        .expect("TransmogUtil probe");
    assert_eq!(
        kind, "table",
        "TransmogUtil global must exist as a table — declared at \
         lua:57-59 with a single initial field `HiddenModelFrame = nil` \
         used as a lazily-created `DressUpModel` for illusion-attachment \
         checks (CanEnchantSource at lua:96-114)"
    );

    for fn_name in TRANSMOG_UTIL_FUNCTIONS {
        let fn_kind: String = env
            .eval(&format!("return type(TransmogUtil.{fn_name})"))
            .unwrap_or_else(|err| panic!("TransmogUtil.{fn_name} probe failed: {err}"));
        assert_eq!(
            fn_kind, "function",
            "TransmogUtil.{fn_name} must be a function — declared in \
             Blizzard_TransmogShared.lua, exposed cross-addon to \
             Collections, FrameXML, ObjectiveTracker, RecruitAFriend, \
             UIPanels_Game, and the LoD Transmog panel itself"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_publishes_transmog_location_mixin() {
    let env = fresh_game_env();
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let kind: String = env
        .eval("return type(TransmogLocationMixin)")
        .expect("TransmogLocationMixin probe");
    assert_eq!(
        kind, "table",
        "TransmogLocationMixin must be a table — declared at lua:530, \
         used by TransmogUtil.CreateTransmogLocation (lua:167-190) \
         which `CreateFromMixins(TransmogLocationMixin)` and calls \
         :Set(locationData) to populate slot/slotID/type/modification \
         fields"
    );

    for method in TRANSMOG_LOCATION_METHODS {
        let m_kind: String = env
            .eval(&format!("return type(TransmogLocationMixin.{method})"))
            .unwrap_or_else(|err| panic!("TransmogLocationMixin.{method} probe failed: {err}"));
        assert_eq!(
            m_kind, "function",
            "TransmogLocationMixin.{method} must be a function"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_publishes_item_model_base_mixin() {
    let env = fresh_game_env();
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let kind: String = env
        .eval("return type(ItemModelBaseMixin)")
        .expect("ItemModelBaseMixin probe");
    assert_eq!(
        kind, "table",
        "ItemModelBaseMixin must be a table — declared at lua:652. \
         Comment at lua:650-651 notes: assumes association with a \
         DressUpModel; intent is that consumers inherit via \
         CreateFromMixins. Provides default OnLoad/OnModelLoaded/\
         OnMouseUp/OnMouseDown/OnEnter/OnLeave/OnUpdate/OnShow handlers \
         + Reload/UpdateCamera/SetDesaturated/ToggleFavorite + 4 \
         override-stubs (GetAppearanceInfo, GetCollectionFrame, \
         GetIllusionLink, GetAppearanceLink) intended for inheritor \
         override"
    );

    for method in ITEM_MODEL_BASE_METHODS {
        let m_kind: String = env
            .eval(&format!("return type(ItemModelBaseMixin.{method})"))
            .unwrap_or_else(|err| panic!("ItemModelBaseMixin.{method} probe failed: {err}"));
        assert_eq!(
            m_kind, "function",
            "ItemModelBaseMixin.{method} must be a function"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_publishes_wardrobe_sets_data_provider_mixin() {
    let env = fresh_game_env();
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let kind: String = env
        .eval("return type(WardrobeSetsDataProviderMixin)")
        .expect("WardrobeSetsDataProviderMixin probe");
    assert_eq!(
        kind, "table",
        "WardrobeSetsDataProviderMixin must be a table — declared at \
         lua:920. The 21-method mixin caches base/usable/available/\
         variant set queries (the C_TransmogSets.* calls are expensive \
         to repeat) and exposes Clear* invalidators for refresh-on-\
         event. Used by Wardrobe sets tab and any addon that needs the \
         shared sort+favorite logic"
    );

    for method in WARDROBE_SETS_DATA_PROVIDER_METHODS {
        let m_kind: String = env
            .eval(&format!(
                "return type(WardrobeSetsDataProviderMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("WardrobeSetsDataProviderMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            m_kind, "function",
            "WardrobeSetsDataProviderMixin.{method} must be a function"
        );
    }
}
}

post_wrath_ui_fixture! {
/// Read the `TransmogSlotOrder` declaration from the pinned profile source so the
/// expectation follows the shipped body: PTR 12.1.7 and WowForever list 14
/// entries including `INVSLOT_RANGED` at `Blizzard_TransmogShared.lua:28`
/// (`:14-29`); Retail-era sources list 13 (`:14-28`).
fn pinned_transmog_slot_order() -> Vec<String> {
    let path = transmog_shared_dir().join("Blizzard_TransmogShared.lua");
    let source =
        std::fs::read_to_string(&path).expect("pinned Blizzard_TransmogShared.lua reads utf-8");
    let mut lines = source
        .lines()
        .skip_while(|line| !line.starts_with("TransmogSlotOrder ="));
    lines.next().expect("pinned TransmogSlotOrder declaration");
    lines
        .take_while(|line| line.trim() != "};")
        .map(|line| line.trim().trim_end_matches(',').to_string())
        .filter(|entry| !entry.is_empty())
        .collect()
}

#[test]
fn explicit_load_publishes_transmog_slot_order() {
    let env = fresh_game_env();
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let kind: String = env
        .eval("return type(TransmogSlotOrder)")
        .expect("TransmogSlotOrder probe");
    assert_eq!(
        kind, "table",
        "TransmogSlotOrder global must be a table — declared at \
         Blizzard_TransmogShared.lua:14 in display order. Drives both the \
         CreateCustomSetSlashCommand serializer and the \
         ParseCustomSetSlashCommand parser at lua:328-404"
    );

    let expected = pinned_transmog_slot_order();
    let count: i64 = env
        .eval("return #TransmogSlotOrder")
        .expect("TransmogSlotOrder length probe");
    assert_eq!(
        count,
        expected.len() as i64,
        "TransmogSlotOrder must publish exactly the entries the pinned source \
         declares at Blizzard_TransmogShared.lua:14 ({expected:?})"
    );
    for (index, slot) in expected.iter().enumerate() {
        let matches: bool = env
            .eval(&format!(
                "return TransmogSlotOrder[{}] == {slot}",
                index + 1
            ))
            .unwrap_or_else(|err| panic!("TransmogSlotOrder[{index}] probe failed: {err}"));
        assert!(
            matches,
            "TransmogSlotOrder[{index}] must be `{slot}` in the pinned source order"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_initializes_empty_transmog_slots_table() {
    let env = fresh_game_env();
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let kind: String = env
        .eval("return type(TRANSMOG_SLOTS)")
        .expect("TRANSMOG_SLOTS probe");
    assert_eq!(
        kind, "table",
        "TRANSMOG_SLOTS global must be a table — declared empty at \
         lua:52, populated by InitializeSlotLocationInfo (called \
         inline at lua:646 inside the do-block at lua:613-647). The \
         module-load path INVOKES InitializeSlotLocationInfo \
         immediately, but the C_TransmogOutfitInfo.GetAllSlotLocationInfo \
         stub returns nothing in the simulator, so the table stays \
         empty after load — that's expected"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_publishes_initialize_slot_location_info_global() {
    let env = fresh_game_env();
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let kind: String = env
        .eval("return type(InitializeSlotLocationInfo)")
        .expect("InitializeSlotLocationInfo probe");
    assert_eq!(
        kind, "function",
        "InitializeSlotLocationInfo must be a global function — \
         declared at lua:614 inside the trailing `do ... end` block. \
         Walks C_TransmogOutfitInfo.GetAllSlotLocationInfo's appearance \
         + illusion lists, calling TransmogUtil.CreateTransmogLocation \
         per slot to populate TRANSMOG_SLOTS by lookupKey and \
         indirectly populate SLOT_ID_TO_NAME"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_registers_static_popup_dialog() {
    let env = fresh_game_env();
    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let kind: String = env
        .eval("return type(StaticPopupDialogs['TRANSMOG_FAVORITE_WARNING'])")
        .expect("StaticPopupDialogs entry probe");
    assert_eq!(
        kind, "table",
        "StaticPopupDialogs['TRANSMOG_FAVORITE_WARNING'] must be \
         registered after load — declared at lua:1-12. OnAccept calls \
         TransmogUtil.ToggleFavorite(visualID, setFavorite=true, \
         itemsCollectionFrame, confirmed=true) so favoriting an \
         all-conditional appearance bypasses the warning recursion. \
         button1=OKAY, button2=CANCEL, hideOnEscape=1, timeout=0"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn explicit_load_emits_no_addon_specific_errors() {
    let env = fresh_game_env();
    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &transmog_shared_toc())
        .expect("Blizzard_TransmogShared must load via Rust loader");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_TransmogShared/") || e.contains("Blizzard_TransmogShared.lua")
        })
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Loading Blizzard_TransmogShared as the first addon must emit \
         zero addon-specific errors. The do-block at lua:613-647 \
         invokes InitializeSlotLocationInfo immediately, which calls \
         C_TransmogOutfitInfo.GetAllSlotLocationInfo — that stub must \
         tolerate an empty/nil return without raising. Found: \
         {addon_specific:?}"
    );
}
}
