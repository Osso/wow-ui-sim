#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_source(env: &WowLuaEnv, relative: &str) {
    let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join(relative);
    env.exec(&std::fs::read_to_string(path).unwrap()).unwrap();
}

#[test]
fn forever_paper_doll_initializes_real_equipment_buttons() {
    let env = WowLuaEnv::new().unwrap();
    load_source(&env, "Blizzard_ItemButton/Shared/ItemButtonTemplate.lua");
    load_source(&env, "Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua");
    load_source(&env, "Blizzard_UIPanels_Game/Camelot/PaperDollFrame.lua");
    env.exec(
        r#"
        CharacterFrame = CreateFrame("Frame")
        CharacterFrame.LeftPaneHost = CreateFrame("Frame", nil, CharacterFrame)
        for _, case in ipairs({
            {"HeadSlot", 1, 136516},
            {"MainHandSlot", 16, 136518},
            {"AmmoSlot", 0, 136510},
        }) do
            local button = CreateFrame("Button", "Character" .. case[1])
            button.icon = button:CreateTexture()
            PaperDollItemSlotButton_OnLoad(button)
            assert(button:GetID() == case[2], case[1] .. " ID")
            assert(button.icon:GetTexture() == case[3], case[1] .. " icon")
            assert(button.backgroundTextureName == case[3])
            assert(button.checkRelic == false)
            assert(type(button.GetItemLocationCallback) == "function")
        end
        "#,
    )
    .unwrap();
}

#[test]
fn forever_inventory_namespace_reuses_existing_slot_contract() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        for _, method in ipairs({"GetInventorySlotInfo", "GetInventorySlotInfoForInvSlot"}) do
            local query = C_PaperDollInfo[method]
            for _, name in ipairs({"HeadSlot", "MainHandSlot", "AmmoSlot"}) do
                local id, texture, relic = query(name)
                local expectedId, expectedTexture, expectedRelic = GetInventorySlotInfo(name)
                assert(id == expectedId and texture == expectedTexture and relic == expectedRelic)
                assert(select("#", query(name)) == 3)
            end
            assert(query("UnknownSlot") == nil)
        end
        "##,
    )
    .unwrap();
}
