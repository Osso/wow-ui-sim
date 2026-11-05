// Shared registration groups for tests consuming profile-owned vendor inputs.
// Root-name absence alone does not establish behavioral fixture absence.

/// Select the native action callback case without compiling Wrath-only inputs
/// into another profile's fixture. Modern test bodies remain unchanged.
#[macro_export]
macro_rules! wrath_action_case {
    ($($callback:ident)::+) => {
        #[cfg(feature = "client-wrath")]
        if $crate::common::action_bar_fixture::is_wrath() {
            return $crate::common::action_bar_fixture::assertions::$($callback)::+();
        }
    };
}

/// GroupFinder is not a Forever runtime consumer: pinned Forever
/// Blizzard_GroupFinder.toc:10 excludes camelot. Keep other profile cases,
/// including Wrath's separate native keybinding path, registered unchanged.
#[macro_export]
macro_rules! group_finder_ui_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wrath",
                feature = "client-mists",
                feature = "client-era",
                feature = "client-anniversary"
            ))]
            $item
        )*
    };
}

#[macro_export]
macro_rules! post_wrath_ui_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-mists",
                feature = "client-era",
                feature = "client-anniversary",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// Profiles whose pinned `TooltipInfoDocumentation.lua:1012` declares
/// `GetSocketedRelic`: Retail, PTR and Forever. Their tooltip getter exists, so
/// the artifact/socket selection paths can be exercised without Retail records.
#[macro_export]
macro_rules! socketed_relic_getter_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "profile-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// The WM97 DQ4 / WM104 relic joins (GemProperties, SpellItemEnchantment, curve
/// 1718, ItemBonusListLevelDelta) are pinned Retail 70391 and shared with PTR.
/// Other profiles publish the getter but must not apply these records.
#[macro_export]
macro_rules! retail_relic_record_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
            $item
        )*
    };
}

/// Forever declares `GetSocketedRelic` but has no relic record set here, so a
/// supplied socket resolves the shared catalog item only.
#[macro_export]
macro_rules! forever_catalog_relic_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "client-wowforever")]
            $item
        )*
    };
}

/// Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua` declare
/// `GetInventoryItemByID` (:407), `GetItemKey` (:484), `GetArtifactItem` (:43)
/// and `GetEquipmentSet` (:271). Other profiles do not declare them.
#[macro_export]
macro_rules! tooltip_selection_getters_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "profile-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

// classic-gating-g4

/// `Blizzard_TimerunningUtil/Blizzard_TimerunningUtil.toc:5` declares
/// `## AllowLoadGameType: mainline`. The files are listed on the classic
/// profiles (`data/blizzard-ui-files/mists.txt:2544`,
/// `era.txt:2545`, `anniversary.txt:2545`) but `toc::active_game_types`
/// only matches `mainline`/`standard` for Retail and PTR, and `camelot`
/// for WowForever, so the panel fixtures cannot select the addon there.
#[macro_export]
macro_rules! timerunning_util_panel_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// `Blizzard_ClickBindingUI` has no entry in the classic manifests
/// (`data/blizzard-ui-files/{mists,era,anniversary}.txt` grep count 0), so
/// `Blizzard_ClickBindingUI_Bootstrap.lua` cannot be read there.
#[macro_export]
macro_rules! click_binding_ui_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// Classic `Blizzard_GameTooltip_Classic.toc` selects
/// `Classic/GameTooltip.lua`, which defines neither `GameTooltipDataMixin`
/// (`Mainline/GameTooltip.lua:951` in Retail/WowForever) nor the data
/// accessors `ProcessInfo`, `GetPrimaryTooltipInfo`, `SetMinimapMouseover`,
/// `SetOwnedItemByID`, `SetRecipeResultItem`, `SetUpgradeItem`,
/// `SetWorldLootObject` or `SetWorldCursor`. The shared
/// `Tooltip/TooltipComparisonManager.lua` is also
/// `[AllowLoadGameType mainline]` in `Blizzard_SharedXMLGame.toc:11`.
#[macro_export]
macro_rules! retail_game_tooltip_data_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// The pinned classic `Blizzard_QuestTimer.toc:4` declares
/// `## Dependencies: Blizzard_UIParent, Blizzard_GameTooltip`, while Retail,
/// PTR and WowForever declare `Blizzard_GameTooltip, Blizzard_ManagedFrameSystem`.
/// The pinned classic `Blizzard_SharedXMLGame.toc` marks
/// `DressUpModelFrameMixin.lua` (`:13`) and `Mainline/LootSpecialization.lua`
/// `[AllowLoadGameType mainline]`, so the Retail body assertions hold only on
/// the mainline profiles.
#[macro_export]
macro_rules! retail_toc_metadata_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// `CharacterSelectUtil.SetTooltipForCharacterInfo` is defined only in
/// `Blizzard_GlueXML/Mainline/CharacterSelect/CharacterSelectUtil.lua:121`
/// (WowForever also defines `Camelot/CharacterSelect/CharacterSelectUtil.lua:7`).
/// The classic GlueXML addon has no such function.
#[macro_export]
macro_rules! character_select_tooltip_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// Same `Blizzard_TimerunningUtil.toc:5` `## AllowLoadGameType: mainline`
/// fixture as `timerunning_util_panel_fixture`, plus Wrath: the action-bar
/// tooltip probe takes `wrath_action_case!` before the shared panel fixture
/// loads, and Wrath already passed that path in the baseline.
#[macro_export]
macro_rules! timerunning_util_panel_or_wrath_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wrath",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// `Blizzard_POIButton` has no entry in `data/blizzard-ui-files/mists.txt`,
/// `era.txt`, `anniversary.txt` or `wrath.txt` (grep count 0), so the panel
/// fixtures that require the addon root cannot load it on a classic profile.
/// Use this form for a post-Wrath probe that must stay excluded on Wrath, and
/// nest it under `post_wrath_ui_fixture!` so both reasons are explicit.
#[macro_export]
macro_rules! poi_button_panel_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// `Blizzard_POIButton` has no entry in `data/blizzard-ui-files/mists.txt`,
/// `era.txt`, `anniversary.txt` or `wrath.txt` (grep count 0), so the panel
/// fixtures that require the addon root cannot load it on a classic profile.
/// Wrath is included, so use this only where the probe already ran on Wrath,
/// or where an outer `post_wrath_ui_fixture!` still excludes it. A probe that
/// was Wrath-excluded on master and is not nested must use
/// `poi_button_panel_fixture!` instead.
#[macro_export]
macro_rules! poi_button_panel_or_wrath_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wrath",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

// classic-tooltip-residue

/// The paragon reputation tooltip is Retail and WoW Forever content:
/// `Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua:686` defines
/// `ReputationParagonWatchBar_OnEnter` there (WowForever `Camelot/ReputationFrame.lua:717`,
/// also `Mainline/ReputationFrame.lua:707`). The classic TOCs select
/// `Classic\ReputationFrame.lua [AllowLoadGameType wrath, cata, mists]`
/// (`mists|era|anniversary/…/Blizzard_UIPanels_Game_Classic.toc:22`) or
/// `Vanilla\ReputationFrame.lua [AllowLoadGameType vanilla, tbc]` (`:23`), and both
/// files define the handler as an empty stub:
/// `mists/…/Classic/ReputationFrame.lua:287`,
/// `era|anniversary/…/Vanilla/ReputationFrame.lua:150`, each commented
/// "No Paragon reputations for Classic!". The seeded-paragon assertions therefore
/// describe a consumer those profiles do not ship.
#[macro_export]
macro_rules! paragon_reputation_tooltip_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

// ptr-forever-pvp-residue

/// WowForever's pinned `Blizzard_PVPUI/Blizzard_PVPUI.toc:8` declares
/// `ExcludeLoadGameType: camelot`, and `wowforever` is a camelot client
/// (`src/toc/mod.rs::active_game_types`), so the pinned client excludes the addon:
/// `Blizzard_PVPUI` is not runtime-selected on Forever. Its TOC-body assertions
/// (variant TOCs, Retail body text) therefore do not describe that profile.
#[macro_export]
macro_rules! pvp_ui_toc_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-mists",
                feature = "client-era",
                feature = "client-anniversary"
            ))]
            $item
        )*
    };
}

/// Only the Retail-era caches list `Blizzard_PVPUI_Mists.toc`
/// (`data/blizzard-ui-files/retail.txt:2617`). PTR 12.1.7 and WowForever pin one
/// bare TOC and no flavor variants (`data/blizzard-ui-files/ptr.txt:2931`,
/// `data/blizzard-ui-files/wowforever.txt:2901`), so the Mists-variant cases are
/// inapplicable there.
#[macro_export]
macro_rules! pvp_ui_variant_toc_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-mists",
                feature = "client-era",
                feature = "client-anniversary"
            ))]
            $item
        )*
    };
}
