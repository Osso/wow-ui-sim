//! Forever's native constants consumed by the unmodified gamepad action bar.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
#[cfg(feature = "client-wowforever")]
fn action_bar_initializes_both_documented_button_groups() {
    let env = WowLuaEnv::new().unwrap();
    let source = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_GamepadActionBars/ActionBar.lua");
    env.exec(&std::fs::read_to_string(source).unwrap()).unwrap();
    env.exec(r#"
        local bar = CreateFrame("Frame")
        bar.Left = {}; bar.Right = {}
        for _, group in ipairs({bar.Left, bar.Right}) do
            for i = 1, 4 do
                local button = CreateFrame("Button", nil, bar)
                button.HotKey = button:CreateFontString()
                button.ButtonIcon = button:CreateTexture()
                function button:SetButtonIndexOnActionBar(index) self.index = index end
                function button:SetActionBarParent(parent) self.actionBar = parent end
                group["ActionButton" .. i] = button
            end
        end
        GamepadActionBarMixin.InitActionButtons(bar)
        assert(#bar.actionButtons == 8)
        for i, button in ipairs(bar.actionButtons) do
            assert(button == (i <= 4 and bar.Left["ActionButton" .. i] or bar.Right["ActionButton" .. (i - 4)]))
            assert(button.index == i and button.actionBar == bar)
            assert(button:GetAttribute("showgrid") == 1 and button:IsShown())
            assert(not button.HotKey:IsShown())
            assert(button.ButtonIcon:GetWidth() == 20 and button.ButtonIcon:GetHeight() == 20)
        end
    "#).unwrap();
}

#[test]
fn gamepad_constants_are_profile_scoped() {
    let env = WowLuaEnv::new().unwrap();
    let published: bool = env
        .eval("return rawget(Constants, 'GamepadActionBarConstants') ~= nil")
        .unwrap();
    assert_eq!(published, cfg!(feature = "client-wowforever"));
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_gamepad_cvar_defaults_drive_vendor_mapping_and_events() {
    let env = WowLuaEnv::new().unwrap();
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.exec(
        &std::fs::read_to_string(cache.join("Blizzard_GamepadActionBars/ActionBar.lua")).unwrap(),
    )
    .unwrap();
    env.exec(
        &std::fs::read_to_string(
            cache.join("Blizzard_GamepadActionBars/GamepadOverrideBarMixin.lua"),
        )
        .unwrap(),
    )
    .unwrap();
    env.exec(r#"
        for _, entry in ipairs({
            {"GamepadPossessBarOverride", "4", "GAMEPAD_POSSESS_BAR_OVERRIDE_CHANGED"},
            {"GamepadStanceBarOverride", "3", "GAMEPAD_STANCE_BAR_OVERRIDE_CHANGED"},
        }) do
            local name, default, event = unpack(entry)
            assert(GetCVar(name) == default, name .. " current")
            assert(C_CVar.GetCVarDefault(name) == default, name .. " default")
            local anchor = CreateFrame("Frame")
            local alternate = CreateFrame("Frame")
            local bar = CreateFrame("Frame")
            Mixin(bar, GamepadOverrideBarMixin)
            bar.overrideCVar = name
            bar.overrideCVarChangedEvent = event
            bar.overrideMap = {}
            bar.isOverrideBarActive = false
            bar.pagingUnitOwner = anchor
            bar:SetOverrideMapping(tonumber(default), function(owner) assert(owner == anchor); return anchor end, 1)
            bar:SetOverrideMapping(2, function() return alternate end, 2)
            bar:ApplyInitialOverridePositioning()
            assert(bar:GetParent() == anchor)
            local events = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent(event)
            listener:RegisterEvent("CVAR_UPDATE")
            listener:SetScript("OnEvent", function(_, received, a, b)
                events[#events + 1] = {received, a, b}
                if received == event then
                    assert(a == tonumber(default) and b == 2)
                    bar:OnOverrideCVarChanged(a, b)
                end
            end)
            assert(C_CVar.SetCVar(name, "2"))
            assert(GetCVar(string.lower(name)) == "2")
            assert(GetCVarDefault(name) == default)
            assert(bar:GetParent() == alternate)
            local updates, overrides = 0, 0
            for _, delivered in ipairs(events) do
                if delivered[1] == event then overrides = overrides + 1 end
                if delivered[1] == "CVAR_UPDATE" then
                    assert(delivered[2] == name and delivered[3] == "2")
                    updates = updates + 1
                end
            end
            assert(updates == 1 and overrides == 1)
            listener:UnregisterAllEvents()
            local changes = 0
            listener:RegisterEvent(event)
            listener:SetScript("OnEvent", function() changes = changes + 1 end)
            assert(SetCVar(name, "2"))
            assert(changes == 0)
            assert(not pcall(SetCVar, name, "not-a-number"))
            assert(GetCVar(name) == "2" and GetCVarDefault(name) == default)
            assert(changes == 0)
            listener:UnregisterAllEvents()
        end
    "#).unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_possess_default_selects_vendor_page_one_bottom_bar() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        GamepadActionBarMixin = {}
        GamepadActionBarPageUnitMixin = {
            GetTopAnchorFrame = function(self) return self.top end,
            GetLeftAnchorFrame = function(self) return self.left end,
            GetRightAnchorFrame = function(self) return self.right end,
            GetBottomAnchorFrame = function(self) return self.bottom end,
        }
    "#,
    )
    .unwrap();
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    for file in ["GamepadOverrideBarMixin.lua", "PossessBar.lua"] {
        env.exec(
            &std::fs::read_to_string(cache.join("Blizzard_GamepadActionBars").join(file)).unwrap(),
        )
        .unwrap();
    }
    env.exec(
        r#"
        local page = CreateFrame("Frame")
        for _, name in ipairs({"top", "left", "right", "bottom"}) do
            page[name] = CreateFrame("Frame", nil, page)
            page[name].Bar = CreateFrame("Frame", nil, page[name])
        end
        local bar = CreateFrame("Frame", nil, page)
        Mixin(bar, GamepadPossessBarMixin)
        bar.overrideMap = {}
        bar.overrideCVar = "GamepadPossessBarOverride"
        bar.isOverrideBarActive = false
        bar.pagingUnitOwner = page
        page.actionBars = {possessBar = bar}
        function page:RefreshPageTrackerSpecialPageSlotVisibility()
            self.specialPageVisible = bar:IsPossessBarOverrideOnSpecialPage()
        end
        function page:HandleSpecialPageActiveStateChange()
            self.specialPageActive = bar:IsPossessBarOverrideOnSpecialPage()
        end
        GamepadActionBarPageUnitMixin.InitializePossessBar(page)
        assert(bar:GetParent() == page.bottom, "native default selects Page1BottomBar")
        assert(bar:GetActionBarLinkedWithOverrideBar() == page.bottom.Bar)
        assert(bar:GetLinkedOverrideBarPage(GetCVar(bar.overrideCVar)) == 1)
        assert(SetCVar(bar.overrideCVar, "1"))
        bar:ApplyInitialOverridePositioning()
        assert(bar:GetParent() == page.top)
        assert(bar:GetLinkedOverrideBarPage("1") == 4)
        assert(GetCVarDefault(bar.overrideCVar) == "4")
    "#,
    )
    .unwrap();
}

#[test]
fn forever_gamepad_cvar_defaults_are_profile_scoped() {
    let env = WowLuaEnv::new().unwrap();
    let published: bool = env.eval("return GetCVarDefault('GamepadPossessBarOverride') ~= nil and GetCVarDefault('GamepadStanceBarOverride') ~= nil").unwrap();
    assert_eq!(published, cfg!(feature = "client-wowforever"));
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_gamepad_pet_storage_uses_existing_pet_slots() {
    use wow_ui_sim::lua_api::state::PetActionSlot;
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        for i in 0..8 {
            state.pet_actions[i] = PetActionSlot {
                has_action: true,
                name: Some(format!("Pet{}", i + 1)),
                spell_id: Some(16827 + i as u32),
                ..PetActionSlot::default()
            };
        }
        state.action_bars.insert(1, 6603);
    }
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.exec(&std::fs::read_to_string(cache.join("Blizzard_GamepadActionBars/ActionBarButton.lua")).unwrap()).unwrap();
    env.exec(r#"
        local first = C_GamepadUI.GetFirstGamepadPetActionStorageSlotIndex()
        assert(first == 1, "logical pet slot base")
        for i = 1, 8 do
            local button = CreateFrame("Button")
            button:SetID(first + i - 1)
            assert(GamepadActionBarPetButtonMixin.HasAction(button) == "Pet" .. i)
            local start, duration, enabled = GetPetActionCooldown(button:GetID())
            assert(start == 0 and duration == 0 and enabled == 1)
        end
        local kind, id = GetActionInfo(1)
        assert(kind == "spell" and id == 6603)
    "#).unwrap();
    env.state().borrow_mut().pet_actions[0] = PetActionSlot::default();
    env.exec("assert(GetPetActionInfo(C_GamepadUI.GetFirstGamepadPetActionStorageSlotIndex()) == nil); local _, id = GetActionInfo(1); assert(id == 6603)").unwrap();
}
