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

// classic-profiles-baseline

/// `Blizzard_AccountStore` ships only on Retail/PTR/WowForever: their manifests
/// contain `Blizzard_AccountStore/Blizzard_AccountStore.lua`, while
/// `data/blizzard-ui-files/{mists,era,anniversary,wrath}.txt` contain no
/// `Blizzard_AccountStore/` entry.
#[macro_export]
macro_rules! account_store_ui_fixture {
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

/// `Blizzard_CharacterCreate` ships only on Retail/PTR/WowForever
/// (`Blizzard_CharacterCreate/Blizzard_CharacterCreate.toc` in their manifests);
/// the pinned Wrath/Mists/Era/Anniversary manifests contain no
/// `Blizzard_CharacterCreate` entry, so their glue addon set has no such addon.
#[macro_export]
macro_rules! character_create_ui_fixture {
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

/// Wrath's manifest is the pinned 3.3.5 FrameXML/AddOns snapshot
/// (`data/blizzard-ui-builds/wrath.json`), synced by
/// `src/blizzard_ui_sync.rs::sync_wrath_ui_to` -> `sync_from_snapshot`; it never
/// resolves through the CASC listfile. Every other profile's manifest is a CASC
/// file list, so only those can assert listfile resolution.
#[macro_export]
macro_rules! casc_listfile_manifest_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "profile-retail",
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

/// Modern-engine client surface that the pinned 3.3.5 Wrath tree does not
/// provide: `C_*` namespaces (its UI source references none), fileDataID texture
/// identity (3.3.5 `SetTexture` takes paths), and addons absent from the Wrath
/// manifest (for example `Blizzard_Dispatcher`). Retail, PTR and every classic
/// profile except Wrath load the modern engine surface.
#[macro_export]
macro_rules! modern_engine_namespace_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "profile-retail",
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

/// `C_AuraContainerUtil`, `Enum.ForbiddenAspect` and `C_FrameManager` are
/// declared only by Retail/PTR/WowForever pinned
/// `Blizzard_APIDocumentationGenerated` (`AuraContainerUtilDocumentation.lua`,
/// `ForbiddenAspectConstantsDocumentation.lua`,
/// `UIFrameManagerDocumentation.lua`). The Mists/Era/Anniversary doc sets
/// contain none of the three namespaced files.
#[macro_export]
macro_rules! retail_namespace_fixture {
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

/// `C_CampaignInfo.GetCampaignInfo` and
/// `C_CooldownViewer.GetGroupBuffItems` are declared only by
/// Retail/PTR/WowForever pinned docs (`WarCampaignDocumentation.lua`,
/// `CooldownViewerDocumentation.lua`); the Mists/Era/Anniversary doc sets
/// declare neither function.
#[macro_export]
macro_rules! retail_only_namespace_member_fixture {
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

/// Secret values exist only where `ClientProfile::uses_secret_values()` is true
/// (`src/client_profile.rs:86-91`: Retail, PTR, WowForever). Classic builds
/// disable the secret system (`docs/specs/classic-secret-policy.md`: 12.0.5
/// source lines 213-215), so `secretwrap`, `scrub` and secret formatting keep
/// ordinary values there.
#[macro_export]
macro_rules! classic_secret_values_fixture {
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

/// Retail-family clients removed the legacy talent grid and skill window, which
/// `lua_api::workarounds::temporary::legacy_talent_skill_defaults` fills with
/// empty defaults (module doc). Classic profiles still ship those globals: the
/// pinned Wrath `FrameXML/BuffFrame.lua` consumer and talent tabs exist, and the
/// Mists/Era/Anniversary caches keep `GetSkillLineInfo` data, so their real
/// values win over the empty stand-ins.
#[macro_export]
macro_rules! retail_removed_legacy_globals_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
            $item
        )*
    };
}
