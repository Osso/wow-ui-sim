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
