#![cfg(feature = "client-wowforever")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn item_button_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_Colors",
            "Blizzard_FrameXMLUtil",
            "Blizzard_ItemButton",
        ],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_ItemButton"));
    env
}

#[test]
fn original_item_button_reset_hides_count_and_clears_texture() {
    item_button_env()
        .exec(
            r#"
            local button = CreateFrame("ItemButton", "ForeverItemButtonResetProbe", UIParent)
            button:SetItemButtonCount(7)
            button:SetItemButtonTexture(134400)
            assert(button.Count:IsShown() and tonumber(button.Count:GetText()) == 7)
            assert(button.icon:IsShown() and button.icon:GetTexture() == 134400)

            local source = debug.getinfo(button.Reset, "S").source:gsub("\\", "/")
            assert(source:find("Blizzard_ItemButton/Shared/ItemButtonTemplate.lua", 1, true), source)
            button:Reset()
            assert(button:GetItemButtonCount() == 0 and not button.Count:IsShown())
            assert(button.icon:GetTexture() == nil and not button.icon:IsShown())
            "#,
        )
        .expect("original Forever ItemButton reset updates its count and texture");
}

#[test]
fn original_item_button_helpers_update_texture_count_and_colors() {
    item_button_env()
        .exec(
            r#"
            local helpers = {SetItemButtonTexture, SetItemButtonCount,
                SetItemButtonTextureVertexColor, SetItemButtonNormalTextureVertexColor}
            for i = 1, 4 do
                local helper = helpers[i]
                assert(type(helper) == "function")
                local source = debug.getinfo(helper, "S").source:gsub("\\", "/")
                assert(source:find("Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua", 1, true), source)
            end
            local button = CreateFrame("ItemButton", "ForeverItemButtonHelperProbe", UIParent)
            SetItemButtonTexture(button, "Interface\\Icons\\INV_Misc_QuestionMark")
            SetItemButtonCount(button, 7)
            SetItemButtonTextureVertexColor(button, 0.1, 0.2, 0.3)
            SetItemButtonNormalTextureVertexColor(button, 0.4, 0.5, 0.6)
            local function close(a, b) return math.abs(a - b) < 0.0001 end
            local ir, ig, ib = button.icon:GetVertexColor()
            local nr, ng, nb = button:GetNormalTexture():GetVertexColor()
            assert(button.icon:GetTexture() == 134400)
            assert(button.Count:IsShown() and tonumber(button.Count:GetText()) == 7)
            assert(close(ir, 0.1) and close(ig, 0.2) and close(ib, 0.3))
            assert(close(nr, 0.4) and close(ng, 0.5) and close(nb, 0.6))
            "#,
        )
        .expect("the original Forever helpers own and update the item-button surface");
}
