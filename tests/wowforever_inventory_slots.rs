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
    load_source(&env, "Blizzard_FrameXML/Camelot/EquipmentFlyout.lua");
    load_source(&env, "Blizzard_ItemButton/Shared/ItemButtonTemplate.lua");
    load_source(&env, "Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua");
    load_source(&env, "Blizzard_UIPanels_Game/Camelot/PaperDollFrame.lua");
    env.state().borrow_mut().forever_startup.ammo_required = Some(true);
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
fn forever_ammo_visibility_uses_character_requirement() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().forever_startup.ammo_required = Some(false);
    load_source(&env, "Blizzard_FrameXML/Camelot/EquipmentFlyout.lua");
    load_source(&env, "Blizzard_ItemButton/Shared/ItemButtonTemplate.lua");
    load_source(&env, "Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua");
    load_source(&env, "Blizzard_UIPanels_Game/Camelot/PaperDollFrame.lua");
    env.exec(
        r#"
        CharacterFrame = CreateFrame("Frame")
        CharacterFrame.LeftPaneHost = CreateFrame("Frame", nil, CharacterFrame)
        local button = CreateFrame("Button", "CharacterAmmoSlot")
        button.icon = button:CreateTexture()
        PaperDollItemSlotButton_OnLoad(button)
        assert(not button:IsShown(), "character without ammo requirement hides slot")
        "#,
    )
    .unwrap();
    env.state().borrow_mut().forever_startup.ammo_required = Some(true);
    env.exec(
        r#"
        CharacterAmmoSlot:Show()
        PaperDollItemSlotButton_OnLoad(CharacterAmmoSlot)
        assert(CharacterAmmoSlot:IsShown(), "supplied ammo requirement keeps slot")
        assert(CharacterAmmoSlot:GetID() == 0)
        assert(CharacterAmmoSlot.icon:GetTexture() == 136510)
        assert(select('#', C_PaperDollInfo.AmmoNeeded()) == 1)
        assert(type(C_PaperDollInfo.AmmoNeeded()) == "boolean")
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

#[test]
fn forever_character_panel_uses_supplied_title_and_equipment_textures() {
    let env = WowLuaEnv::new().unwrap();
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env.exec(
        r#"
        A_Admin.EquipItem(1, 211993)
        A_Admin.EquipItem(16, 231268)
        ToggleCharacter("PaperDollFrame", true)
        assert(CharacterFrame:IsShown() and PaperDollFrame:IsShown())
        A_Admin.SetPlayerName("R88gc Lane Character")
        assert(CharacterFrame.TitleContainer.TitleText:GetText() == UnitPVPName("player"))
        assert(UnitPVPName("player") == "R88gc Lane Character")
        local head = GetInventoryItemTexture("player", CharacterHeadSlot:GetID())
        local mainHand = GetInventoryItemTexture("player", CharacterMainHandSlot:GetID())
        assert(head ~= nil and mainHand ~= nil and head ~= mainHand)
        assert(CharacterHeadSlot.icon:GetTexture() == head)
        assert(CharacterMainHandSlot.icon:GetTexture() == mainHand)
        "#,
    )
    .unwrap();
}
