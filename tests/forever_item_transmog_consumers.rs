#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

const TRANSMOG_POPUP_LUA: &str = r#"
IconSelectorPopupFrameTemplateMixin.OkayButton_OnClick = function() end
local ids = {}
local invalidDialog
local listener = CreateFrame("Frame")
listener:RegisterEvent("TRANSMOG_OUTFITS_CHANGED")
listener:SetScript("OnEvent", function(_, _, id) ids[#ids + 1] = id end)
StaticPopup_Show = function(name) invalidDialog = name end
local name, icon = "", 0
local popup = { mode = IconSelectorPopupFrameModes.New, BorderBox = {
    SelectedIconArea = { SelectedIconButton = { GetIconTexture = function() return icon end } },
    IconSelectorEditBox = { GetText = function() return name end },
} }
TransmogOutfitPopupMixin.OkayButton_OnClick(popup)
if invalidDialog ~= "TRANSMOG_OUTFIT_INVALID_NAME" then return "invalid_name" end
name, icon = "Copper", 123
TransmogOutfitPopupMixin.OkayButton_OnClick(popup)
name, icon = "Silver", 456
TransmogOutfitPopupMixin.OkayButton_OnClick(popup)
if #ids ~= 2 or ids[1] ~= 1 or ids[2] ~= 2 then return "events" end
return "ok"
"#;

const TRANSMOG_REFRESH_LUA: &str = r#"
local refreshed
local frame = { OutfitCollection = { Refresh = function(_, provider, selectActive)
    refreshed = { provider = provider, selectActive = selectActive }
end } }
TransmogFrameMixin.RefreshOutfits(frame, true)
local collection = refreshed.provider:GetCollection()
if refreshed.selectActive ~= true or #collection ~= 2 then return "count" end
if collection[1].name ~= "Copper" or collection[1].icon ~= 123 then return "first" end
if collection[2].name ~= "Silver" or collection[2].icon ~= 456 then return "second" end
return "ok"
"#;

#[test]
fn transmog_outfit_popup_adds_two_outfits_through_consumer() {
    common::with_timeout(240, || {
        let env = load_transmog_consumer();
        let result: String = env.eval(TRANSMOG_POPUP_LUA).expect("popup consumer");
        assert_eq!(result, "ok");
        let result: String = env.eval(TRANSMOG_REFRESH_LUA).expect("outfit list consumer");
        assert_eq!(result, "ok");
    });
}

fn load_transmog_consumer() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_Transmog"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_Transmog"));
    env.apply_post_load_workarounds();
    env
}

fn owned(item_id: u32, identity: u64) -> BagItem {
    BagItem {
        owned_identity: Some(identity),
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}

#[test]
fn secure_target_consumer_drops_two_items_into_trade() {
    common::with_timeout(240, || {
        let ui = blizzard_ui_dir();
        let env = new_blizzard_addon_env(&ui);
        let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_FrameXML"], &[]);
        assert!(loaded.iter().any(|name| name == "Blizzard_FrameXML"));
        env.apply_post_load_workarounds();
        env.state().borrow_mut().bag_items.extend([
            ((0, 1), owned(6949, 101)),
            ((0, 2), owned(6950, 102)),
        ]);
        env.exec(r#"
            local button = CreateFrame("Button", nil, UIParent)
            button:SetAttribute("type", "target")
            button:SetAttribute("unit", "player")
            InitiateTrade("player")
            for slot = 1, 2 do
                C_Container.PickupContainerItem(0, slot)
                SecureActionButton_OnClick(button, "LeftButton", false, false, true)
                assert(not CursorHasItem())
            end
        "#).unwrap();
        assert_eq!(
            env.state()
                .borrow()
                .active_trade
                .as_ref()
                .unwrap()
                .player_slots[..2],
            [6949, 6950]
        );
    });
}
