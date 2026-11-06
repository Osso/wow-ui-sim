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
//!
//! The addon closure is loaded directly instead of through the Retail panel
//! baseline: `tests/common/panel_fixtures.rs` requires `Blizzard_TimerunningUtil`,
//! whose pinned TOC is `AllowLoadGameType mainline`, so that baseline cannot run
//! on a classic profile.

#![cfg(any(
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]

use crate::common;
use common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-mists")]
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

fn load_consumer(root: &str) -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &[root], &[]);
    assert!(
        loaded.iter().any(|name| name == root),
        "{root} must load for its pinned consumer; loaded set: {loaded:?}"
    );
    env.apply_post_load_workarounds();
    env
}

#[test]
fn classic_outfit_namespace_matches_pinned_collections_documentation() {
    // Pinned declaration source for the active profile cache.
    let docs = common::blizzard_addon_harness::blizzard_ui_dir()
        .join("Blizzard_APIDocumentationGenerated")
        .join("TransmogOutfitInfoDocumentation.lua");
    let text = std::fs::read_to_string(&docs)
        .unwrap_or_else(|err| panic!("read {}: {err}", docs.display()));
    let lines: Vec<&str> = text.lines().collect();
    let mut declared = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("Name = \"") else {
            continue;
        };
        let Some(name) = rest.strip_suffix("\",") else {
            continue;
        };
        let next = lines.get(index + 1).map(|line| line.trim());
        if next == Some("Type = \"Function\",") {
            declared.push(name);
        }
    }
    assert!(
        declared.len() >= 60,
        "pinned {} must declare the outfit surface; found {}",
        docs.display(),
        declared.len()
    );

    let names = declared
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(",");
    let env = WowLuaEnv::new().expect("create headless environment");
    let result: String = env
        .eval(&format!(
            r#"
            local declared = {{ {names} }}
            local seen = {{}}
            for _, name in ipairs(declared) do
                if type(C_TransmogOutfitInfo[name]) ~= "function" then
                    return "missing:" .. name
                end
                seen[name] = true
            end
            for name, value in pairs(C_TransmogOutfitInfo) do
                if type(value) == "function" and not seen[name] and name:sub(1, 2) ~= "__" then
                    return "extra:" .. name
                end
            end
            return "ok"
            "#
        ))
        .expect("declared outfit surface probe");
    assert_eq!(result, "ok");
}

// The pinned Era and Anniversary caches ship only `Blizzard_Collections_Cata.toc`,
// `_Mists.toc` and `_Wrath.toc`; `find_toc_file` accepts `_Vanilla`, the bare
// name, or `_Classic` for those profiles, so the original Collections addon
// cannot load there. The C_Heirloom producer is still compiled and registered on
// every profile; Mists is the profile whose pinned tests exercise this consumer.
#[cfg(feature = "client-mists")]
#[test]
fn heirloom_filters_round_trip_through_collections_consumer() {
    common::with_timeout(240, || {
        let env = load_consumer("Blizzard_Collections");
        let result: String = env
            .eval(HEIRLOOM_CONSUMER_LUA)
            .expect("HeirloomsMixin filter consumer should run");
        assert_eq!(result, "ok");
    });
}

#[test]
fn transmog_outfit_consumer_refreshes_from_published_outfits() {
    common::with_timeout(240, || {
        let env = load_consumer("Blizzard_Transmog");
        let result: String = env
            .eval(OUTFIT_CONSUMER_LUA)
            .expect("TransmogFrameMixin:RefreshOutfits should run");
        assert_eq!(result, "ok");
    });
}
