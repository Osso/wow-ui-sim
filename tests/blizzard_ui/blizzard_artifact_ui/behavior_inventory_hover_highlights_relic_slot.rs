//! Inventory relic-hover behavior for `Blizzard_ArtifactUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use wow_ui_sim::lua_api::state::{ArtifactInfo, BagItem, RelicSlotInfo};

const ROOT: &str = "Blizzard_ArtifactUI";
const ARTIFACT_ICON: &str = "Interface/Icons/inv_sword_2h_artifactashbringer_d_01";
const RELIC_ITEM_ID: u32 = 1_234;
const RELIC_ITEM_LINK: &str = "|Hitem:1234::::::::80:70:::::::::|h[Test Relic]|h";

// ArtifactUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[test]
fn inventory_hover_highlights_matching_relic_slot_and_leave_clears_it() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);
        seed_viewed_artifact_with_relic_item(env);

        let mismatches: Vec<String> = env
            .eval(INVENTORY_RELIC_HOVER_PROBE)
            .expect("ArtifactUI inventory relic-hover probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must route relic inventory hover and leave to PerksTab highlights; \
             mismatches: {mismatches:?}"
        );
    });
}
}

fn seed_viewed_artifact_with_relic_item(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.viewed_artifact.info = Some(sample_artifact());
    state.viewed_artifact.total_purchased_ranks = 3;
    state.viewed_artifact.is_at_forge = true;
    state.viewed_artifact.relic_slots.push(sample_relic_slot());
    state.artifact_relic_items.insert(RELIC_ITEM_ID as i32);
    state.bag_items.insert(
        (0, 1),
        BagItem {
            item_id: RELIC_ITEM_ID,
            stack_count: 1,
            hyperlink: Some(RELIC_ITEM_LINK.to_string()),
            durability: None,
            applied_transmog: None,
            owned_identity: None,
        },
    );
}

fn sample_artifact() -> ArtifactInfo {
    ArtifactInfo {
        item_id: 128_910,
        alt_item_id: 128_911,
        name: "Ashbringer".to_string(),
        icon: ARTIFACT_ICON.to_string(),
        total_xp: 12_500,
        points_spent: 3,
        quality: 6,
        artifact_appearance_id: 41,
        appearance_mod_id: 0,
        item_appearance_id: 0,
        alt_item_appearance_id: 0,
        alt_on_top: false,
        tier: 1,
        maxed: false,
        disabled: false,
        category: 1,
    }
}

fn sample_relic_slot() -> RelicSlotInfo {
    RelicSlotInfo {
        slot_type: "Iron".to_string(),
        locked_reason: None,
        name: "Iron Relic".to_string(),
        icon: "Interface/Icons/inv_relics_idolofferocity".to_string(),
        link: "item:1234".to_string(),
        socket: None,
    }
}

fn load_artifact_ui(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (loaded, error): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
        .expect("C_AddOns.LoadAddOn probe should run cleanly");
    assert!(
        loaded,
        "`{ROOT}` must load before inventory relic-hover probe; error={error:?}"
    );
}

const INVENTORY_RELIC_HOVER_PROBE: &str = r#"
local mismatches = {}

local function expect(condition, message)
    if not condition then
        table.insert(mismatches, message)
    end
end

local originalPerksOnUIOpened = ArtifactFrame.PerksTab.OnUIOpened
local originalShowHighlight = ArtifactFrame.PerksTab.ShowHighlightForRelicItemID
local originalHideHighlight = ArtifactFrame.PerksTab.HideHighlightForRelicItemID
local originalRefreshRelicHighlights = ArtifactFrame.PerksTab.TitleContainer.RefreshRelicHighlights
local calls = {}

ArtifactFrame.PerksTab.OnUIOpened = function() end
ArtifactFrame.PerksTab.ShowHighlightForRelicItemID = function(self, itemID, itemLink)
    table.insert(calls, { name = "show", itemID = itemID, itemLink = itemLink })
end
ArtifactFrame.PerksTab.HideHighlightForRelicItemID = function(self, itemID, itemLink)
    table.insert(calls, { name = "hide", itemID = itemID, itemLink = itemLink })
end
ArtifactFrame.PerksTab.TitleContainer.RefreshRelicHighlights = function(self, itemID, itemLink)
    table.insert(calls, { name = "refresh", itemID = itemID, itemLink = itemLink })
end

local expectedInfo = C_Container.GetContainerItemInfo(0, 1)
local ok, errorMessage = pcall(function()
    ShowUIPanel(ArtifactFrame)
    ArtifactFrame:OnInventoryItemMouseEnter(0, 1)
    ArtifactFrame:OnInventoryItemMouseLeave(0, 1)
end)

ArtifactFrame.PerksTab.OnUIOpened = originalPerksOnUIOpened
ArtifactFrame.PerksTab.ShowHighlightForRelicItemID = originalShowHighlight
ArtifactFrame.PerksTab.HideHighlightForRelicItemID = originalHideHighlight
ArtifactFrame.PerksTab.TitleContainer.RefreshRelicHighlights = originalRefreshRelicHighlights

expect(ok, "inventory relic hover error:" .. tostring(errorMessage))
expect(expectedInfo and expectedInfo.itemID == 1234, "seeded itemID:" .. tostring(expectedInfo and expectedInfo.itemID))
expect(expectedInfo and expectedInfo.hyperlink == "|Hitem:1234::::::::80:70:::::::::|h[Test Relic]|h", "seeded hyperlink:" .. tostring(expectedInfo and expectedInfo.hyperlink))
expect(calls[1] and calls[1].name == "show", "first call:" .. tostring(calls[1] and calls[1].name))
expect(calls[1] and calls[1].itemID == 1234, "show itemID:" .. tostring(calls[1] and calls[1].itemID))
expect(calls[1] and calls[1].itemLink == expectedInfo.hyperlink, "show link:" .. tostring(calls[1] and calls[1].itemLink))
expect(calls[2] and calls[2].name == "refresh", "second call:" .. tostring(calls[2] and calls[2].name))
expect(calls[2] and calls[2].itemID == 1234, "enter refresh itemID:" .. tostring(calls[2] and calls[2].itemID))
expect(calls[2] and calls[2].itemLink == expectedInfo.hyperlink, "enter refresh link:" .. tostring(calls[2] and calls[2].itemLink))
expect(calls[3] and calls[3].name == "hide", "third call:" .. tostring(calls[3] and calls[3].name))
expect(calls[3] and calls[3].itemID == 1234, "hide itemID:" .. tostring(calls[3] and calls[3].itemID))
expect(calls[3] and calls[3].itemLink == expectedInfo.hyperlink, "hide link:" .. tostring(calls[3] and calls[3].itemLink))
expect(calls[4] and calls[4].name == "refresh", "fourth call:" .. tostring(calls[4] and calls[4].name))
expect(calls[4] and calls[4].itemID == nil, "leave refresh itemID:" .. tostring(calls[4] and calls[4].itemID))
expect(calls[4] and calls[4].itemLink == nil, "leave refresh link:" .. tostring(calls[4] and calls[4].itemLink))
expect(#calls == 4, "call count:" .. tostring(#calls))

return mismatches
"#;
