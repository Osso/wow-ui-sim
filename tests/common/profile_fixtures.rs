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
