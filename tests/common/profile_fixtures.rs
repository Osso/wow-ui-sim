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

/// Mists installs its own client identity in `src/mists/compat_bootstrap.lua:12-33`
/// (MoP expansion level, 5.5.4/68042/Jun 2 2026/50504) and
/// `tests/mists_compat_bootstrap.rs:91-110` covers it. The shared temporary
/// client-info defaults therefore describe every profile except Mists.
#[macro_export]
macro_rules! non_mists_client_identity_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "client-mists"))]
            $item
        )*
    };
}

/// `reputation_state::apply_bootstrap` seeds the Retail faction fixture only
/// outside Era/Anniversary (src/lua_api/workarounds/temporary/reputation_state.rs:242-247,
/// "Era/Anniversary have no selected Retail faction fixture"), so the seeded
/// faction-count case describes the profiles that receive that fixture.
#[macro_export]
macro_rules! retail_faction_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(not(any(feature = "client-era", feature = "client-anniversary")))]
            $item
        )*
    };
}

// classic-pvp-residue

/// Era and Anniversary pin `Blizzard_PVPUI/Blizzard_PVPUI_Mainline.toc`
/// (`## AllowLoadGameType: mainline`, `data/blizzard-ui-files/era.txt:1880`,
/// `anniversary.txt:1880`) and `Blizzard_PVPUI_Mists.toc`
/// (`## AllowLoadGameType: mists`, `era.txt:1881`, `anniversary.txt:1881`) with no
/// bare TOC. `find_toc_file` (`src/loader/mod.rs:296`) tries
/// `active_profile_toc_suffixes` (`:227-238`) `_Vanilla`, `""`, `_Classic` for
/// those profiles, so no TOC resolves; `discover_all_blizzard_addons`
/// (`src/loader/mod.rs:574`) then drops the addon. Their game types are
/// `vanilla | classic_era | classic` and `vanilla | classic_anniversary |
/// classic` (`src/toc/mod.rs:88-95`), which also match neither TOC's
/// `AllowLoadGameType`. The PVPUI TOC-body fixtures therefore describe no file
/// those profiles can load.
#[macro_export]
macro_rules! classic_pvp_residue_pvp_ui_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-mists"
            ))]
            $item
        )*
    };
}

/// Only the Retail-era caches pin `Blizzard_PVPUI_Mists.toc`
/// (`data/blizzard-ui-files/retail.txt:2617`, `mists.txt:1880`). PTR 12.1.7 ships
/// one bare TOC (`ptr.txt:2931`), so the `_Mists` variant cases cannot resolve
/// there.
#[macro_export]
macro_rules! classic_pvp_residue_pvp_ui_variant_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "client-retail", feature = "client-mists"))]
            $item
        )*
    };
}

/// `discover_all_blizzard_addons` (`src/loader/mod.rs:574`) lists an addon only
/// when `find_toc_file` resolves one of its TOCs. Retail, PTR, Mists and
/// WowForever pin a resolvable `Blizzard_PVPUI` TOC; Era and Anniversary pin
/// only `_Mainline`/`_Mists` TOCs that their suffix order never selects.
#[macro_export]
macro_rules! classic_pvp_residue_pvp_ui_inventory_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-retail",
                feature = "client-ptr",
                feature = "client-mists",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// `Blizzard_PVPMatch` has no entry in the classic manifests
/// (`data/blizzard-ui-files/{mists,era,anniversary}.txt` grep count 0), so its
/// TOC, eager-discovery and inventory fixtures cannot read or discover it there.
#[macro_export]
macro_rules! classic_pvp_residue_pvp_match_fixture {
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

/// `Blizzard_PlayerSpells` has no entry in the classic manifests
/// (`data/blizzard-ui-files/{mists,era,anniversary}.txt` grep count 0). The
/// PvP-talent and Warmode mixins live only in that addon's
/// `PvPTalents/Blizzard_PvPTalentListTemplates.lua` and
/// `PvPTalents/Blizzard_WarmodeButtonTemplate.lua`, so
/// `load_full_game_ui_with_player_spells`
/// (`tests/blizzard_player_spells_loads.rs:197`) cannot load it there.
#[macro_export]
macro_rules! classic_pvp_residue_player_spells_pvp_fixture {
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

/// Mists' pinned `Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua`
/// (907 lines) declares neither `GetSpecialEventDetails` nor
/// `GetSpecialEventInfo`, and no shipped Mists addon calls them (`rg
/// GetSpecialEvent` under the Mists cache matches only `GetSpecialEventBrawlInfo`,
/// e.g. `Blizzard_PVPUI/Mainline/Blizzard_PVPUI.lua:755`).
/// `src/c_api/patch_retired_members.rs:254-255` retires those two names for
/// Retail 9.2.0. The "legacy classic lookup" this fixture requires has no
/// declaration on any pinned profile.
#[macro_export]
macro_rules! classic_pvp_residue_retired_pvp_lookup_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any())]
            $item
        )*
    };
}

// classic-aura-residue

/// The `TargetFrame.Auras` container and its `AuraButton` children are the
/// mainline aura engine: `Blizzard_UnitFrame/Blizzard_UnitFrame_Mainline.toc:27-28`
/// lists `Shared/TargetFrameAuraContainer.lua|.xml` and
/// `Mainline/TargetFrame.xml` declares `parentKey="Auras"` (Retail, PTR and
/// WowForever pin the same pair at `:48-49`). The classic TOC lists only
/// `Shared/TargetFrame.lua`, `Classic/TargetFrame.lua` and
/// `Classic/TargetFrame.xml` (`mists/AddOns/Blizzard_UnitFrame/Blizzard_UnitFrame_Classic.toc:24-27`),
/// and `Classic/TargetFrame.lua:451-500` creates `TargetFrameBuff<N>`
/// `TargetBuffFrameTemplate` buttons instead, so no private-partition
/// `AuraButton.Count` exists to inspect on a classic profile.
#[macro_export]
macro_rules! target_frame_aura_container_fixture {
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

/// Counterpart to `target_frame_aura_container_fixture`: the classic UnitFrame
/// TOC selects `Classic/TargetFrame.xml`, whose `TargetFrameMixin:OnEvent`
/// reads `C_PlayerInteractionManager.IsReplacingUnit()` on every
/// `PLAYER_TARGET_CHANGED`
/// (`mists|era|anniversary/AddOns/Blizzard_UnitFrame/Classic/TargetFrame.lua:168`)
/// and whose `UpdateAuras` builds `TargetFrameBuff<N>` buttons (:451-500).
#[macro_export]
macro_rules! classic_target_frame_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "client-mists",
                feature = "client-era",
                feature = "client-anniversary"
            ))]
            $item
        )*
    };
}

// classic-secur-residue

/// `Blizzard_MapCanvasSecureUtil`, `Blizzard_PingUI` and
/// `Blizzard_SubscriptionInterstitialUI` ship only to the Mainline cache
/// (`data/blizzard-ui-files/retail.txt:2381-2382`, `:2672-2674`, `:3567-3569`;
/// PTR and WowForever list them too). Mists, Era, Anniversary and Wrath list
/// none of the three (grep count 0) and the directories are absent from those
/// caches, so no TOC or frame of theirs exists to inspect there.
#[macro_export]
macro_rules! classic_secur_residue_mainline_cache_fixture {
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

/// The map-lane secure-util cases keep their Wrath native FrameXML branch
/// (`tests/blizzard_map_lane.rs`), so Wrath stays enabled here. The addon
/// assertions below it describe `Blizzard_MapCanvasSecureUtil`, which has no
/// entry in the Mists/Era/Anniversary manifests
/// (`data/blizzard-ui-files/{mists,era,anniversary}.txt` grep count 0).
#[macro_export]
macro_rules! classic_secur_residue_map_lane_secure_util_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "profile-retail",
                feature = "client-ptr",
                feature = "client-wrath",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}

/// Pinned `Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua`
/// declares `SetForbidden` with `HasRestrictions = true` on Retail/PTR/WowForever
/// (`retail.txt` cache `:288-292`) but without it on Mists, Era and Anniversary
/// (same file `:116-121`). Only the Mainline-family clients treat an insecure
/// `SetForbidden` as a no-op. Wrath has no pinned documentation file, so its
/// existing case stays enabled.
#[macro_export]
macro_rules! classic_secur_residue_set_forbidden_restriction_fixture {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "profile-retail",
                feature = "client-ptr",
                feature = "client-wrath",
                feature = "client-wowforever"
            ))]
            $item
        )*
    };
}
