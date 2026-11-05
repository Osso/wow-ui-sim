//! Producer gaps on the classic Collections path, exercised through the
//! pinned Blizzard consumers.
//!
//! * `Blizzard_Collections/Classic/Blizzard_HeirloomCollection.lua:703-706`
//!   (`HeirloomsMixin:SetClassAndSpecFilters`) calls
//!   `C_Heirloom.SetClassAndSpecFilters`, which the pinned classic source uses
//!   at lines 26, 32, 545 and 557 and reads back at lines 22, 277, 539, 694 and
//!   699. `UNSPECIFIED_CLASS_FILTER`/`UNSPECIFIED_SPEC_FILTER` are both 0
//!   (`Blizzard_ClassMenu/Blizzard_ClassMenu.lua:1-2`).
//! * `Blizzard_Transmog/Blizzard_Transmog.lua:199`
//!   (`TransmogFrameMixin:RefreshOutfits`) calls
//!   `C_TransmogOutfitInfo.GetOutfitsInfo` and consumes the returned records.
//!
//! The profiles in scope are the modern-engine classic clients whose pinned
//! `Blizzard_APIDocumentationGenerated` declares the surface:
//! `TransmogOutfitInfoDocumentation.lua` declares `C_TransmogOutfitInfo` on
//! Mists (66 functions), Era and Anniversary (65; no `IsTransmogEnabled`).
//! The legacy `C_Heirloom` namespace has no generated documentation; its
//! consumer call sites are the evidence.

#![cfg(any(
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]

use crate::common;
use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const HEIRLOOM_CONSUMER_LUA: &str = r#"
-- HeirloomsMixin:SetClassAndSpecFilters touches paging and the class menu;
-- every other statement is the pinned consumer body verbatim.
local heirlooms = {
    PagingFrame = { SetCurrentPage = function() end },
    ClassDropdown = { GenerateMenu = function() end },
    FullRefreshIfVisible = function() end,
}

HeirloomsMixin.SetClassAndSpecFilters(heirlooms, 2, 3)
if HeirloomsMixin.GetClassFilter(heirlooms) ~= 2 then return "class_roundtrip" end
if HeirloomsMixin.GetSpecFilter(heirlooms) ~= 3 then return "spec_roundtrip" end

-- A second, distinct pair must replace the first instead of accumulating.
HeirloomsMixin.SetClassAndSpecFilters(heirlooms, 5, 0)
local classFilter, specFilter = C_Heirloom.GetClassAndSpecFilters()
if classFilter ~= 5 then return "class_replace" end
if specFilter ~= 0 then return "spec_replace" end

-- The filter pair decides the view mode at line 276-283.
HeirloomsMixin.SetClassAndSpecFilters(heirlooms, 0, 0)
if C_Heirloom.GetClassAndSpecFilters() ~= 0 then return "class_reset" end
return "ok"
"#;

const OUTFIT_CONSUMER_LUA: &str = r#"
-- TransmogFrameMixin:RefreshOutfits only reads the outfit list and hands a
-- data provider to the outfit collection; both sides are stubbed here.
local refreshed = nil
local frame = {
    OutfitCollection = {
        Refresh = function(_, provider, selectActiveOutfit)
            refreshed = { provider = provider, selectActiveOutfit = selectActiveOutfit }
        end,
    },
}

TransmogFrameMixin.RefreshOutfits(frame, false)
if not refreshed then return "first_refresh_missing" end
if refreshed.provider:GetSize() ~= 0 then return "default_catalog_not_empty" end

-- Two distinct supplied records through the producer's own creation path.
C_TransmogOutfitInfo.AddNewOutfit("Copper", 123)
C_TransmogOutfitInfo.AddNewOutfit("Silver", 456)
TransmogFrameMixin.RefreshOutfits(frame, true)
if not refreshed then return "second_refresh_missing" end
if refreshed.selectActiveOutfit ~= true then return "refresh_argument" end

local collection = refreshed.provider:GetCollection()
if #collection ~= 2 then return "count=" .. #collection end
if collection[1].name ~= "Copper" or collection[1].icon ~= 123 then return "first_record" end
if collection[2].name ~= "Silver" or collection[2].icon ~= 456 then return "second_record" end
if type(collection[1].onClickCallback) ~= "function" then return "click_callback" end
if type(collection[1].onEditCallback) ~= "function" then return "edit_callback" end
return "ok"
"#;

#[test]
fn heirloom_filters_round_trip_through_collections_consumer() {
    common::with_timeout(240, || {
        with_blizzard_addon_smoke_shape(&["Blizzard_Collections"], &[], |env, loaded| {
            assert!(
                loaded.iter().any(|name| name == "Blizzard_Collections"),
                "Blizzard_Collections must load for the pinned Heirloom consumer; \
                 loaded set: {loaded:?}"
            );
            let result: String = env
                .eval(HEIRLOOM_CONSUMER_LUA)
                .expect("HeirloomsMixin filter consumer should run");
            assert_eq!(result, "ok");
        });
    });
}

#[test]
fn transmog_outfit_consumer_refreshes_from_published_outfits() {
    common::with_timeout(240, || {
        with_blizzard_addon_smoke_shape(&["Blizzard_Transmog"], &[], |env, loaded| {
            assert!(
                loaded.iter().any(|name| name == "Blizzard_Transmog"),
                "Blizzard_Transmog must load for the pinned RefreshOutfits consumer; \
                 loaded set: {loaded:?}"
            );
            let result: String = env
                .eval(OUTFIT_CONSUMER_LUA)
                .expect("TransmogFrameMixin:RefreshOutfits should run");
            assert_eq!(result, "ok");
        });
    });
}
