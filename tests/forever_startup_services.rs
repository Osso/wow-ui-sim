#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::c_housing::HousingArea;
use wow_ui_sim::c_api::forever_startup::PurchasableBankTab;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_startup(env: &WowLuaEnv) {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    crate::common::load_profile_framexml(env);
    for (name, toc) in wow_ui_sim::loader::discover_blizzard_addons(&ui) {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

fn startup() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    load_startup(&env);
    env
}

const STACK_SPLIT_BINDING_SETUP: &str = r#"
    SetBinding("KEY_NAME", "OPENALLBAGS")
    ToggleBackpack()
    local frame = ContainerFrame1
    if not frame or not frame:IsShown() then frame = ContainerFrameCombinedBags end
    assert(frame and frame:IsShown(), "ToggleBackpack showed no bag frame")
    StackSplitConsumerButton = nil
    for _, button in frame:EnumerateValidItems() do
        if button:GetBagID() == 0 and button:GetID() == 1 then
            StackSplitConsumerButton = button
        end
    end
    assert(StackSplitConsumerButton, "bag slot 1 has no item button")
    SetModifiedClick("SPLITSTACK", "SHIFT")
"#;

const DISCORD_GUILD_CONTROL_MENU: &str = r#"
    local function pick_data(description, data)
        for _, entry in description:EnumerateElementDescriptions() do
            if entry:GetData() == data then
                assert(entry:Pick(MenuInputContext.MouseButton, "LeftButton"))
                return
            end
        end
        error("menu description missing data " .. data)
    end
    A_Admin.SetGuildInfo("Test Guild", "Guild Master", 1)
    SetCVar("discordClientEnabled", "1")
    A_Admin.SetDiscordServers({
        { name = "North", channels = { { name = "general" }, { name = "raids" } } },
        { name = "South", channels = { { name = "guild" } } },
    })
    ShowUIPanel(GuildControlUI)
    GuildControlUI.dropdown:Click()
    pick_data(GuildControlUI.dropdown:GetMenuDescription(), 4)
    assert(GuildControlUI.selectedTab == 4)
    DiscordUnlinkFrame.button:Click()
    assert(C_Discord.IsUserOAuthed())
    local serverDropdown = GuildControlUI.discordFrame.serverDropdown
    serverDropdown:Click()
    pick_data(serverDropdown:GetMenuDescription(), 1)
    assert(GuildControlUI.discordFrame.channelListTitle:GetText():find("North", 1, true))
    local channelDropdown = GuildControlUI.discordFrame.channelDropdown
    channelDropdown:Click()
    local channels = channelDropdown:GetMenuDescription()
    assert(channels:HasElements())
    pick_data(channels, 2)
    assert(GuildControlUI.selectedDiscordChannel == 2)
    serverDropdown:Click()
    pick_data(serverDropdown:GetMenuDescription(), 2)
    assert(GuildControlUI.discordFrame.channelListTitle:GetText():find("South", 1, true))
    channelDropdown:Click()
    pick_data(channelDropdown:GetMenuDescription(), 1)
    assert(GuildControlUI.selectedDiscordChannel == 1)
"#;

#[test]
fn forever_stack_split_key_handlers_run_bound_actions() {
    let env = startup();
    env.exec(
        r#"
        A_Admin.ClearBags()
        A_Admin.AddBagItem(0, 1, 6948, 5)
        "#,
    )
    .expect("supply a stackable bag item");
    for key in ["K", "L"] {
        env.exec(&STACK_SPLIT_BINDING_SETUP.replace("KEY_NAME", key))
            .expect("open backpack and bind the key");
        env.send_key_down("SHIFT", None).expect("press split modifier");
        env.exec("StackSplitConsumerButton:Click('LeftButton')")
            .expect("open StackSplitFrame through its bag item button");
        env.send_key_up("SHIFT");
        assert!(env.eval::<bool>("return StackSplitFrame:IsShown()").unwrap());
        env.send_key_press(key, None)
            .unwrap_or_else(|error| panic!("dispatch {key} to StackSplitFrame: {error}"));
        let bag_open = env
            .eval::<bool>("return (ContainerFrame1 and ContainerFrame1:IsShown()) or (ContainerFrameCombinedBags and ContainerFrameCombinedBags:IsShown())")
            .unwrap();
        assert!(!bag_open, "RunBinding did not close the bags for key {key}");
    }
}

#[test]
fn forever_gamepad_core_binding_context_tracks_interface_events() {
    let env = startup();
    env.exec(
        r#"
        local context = Enum.BindingContext.GamepadModeInGameCore
        assert(C_KeyBindings.IsBindingContextActive(context) == false)
        A_Admin.SetUsingGamepad(true)
        A_Admin.FireEvent("INPUT_DEVICE_INTERFACE_TRANSITION", Enum.InputDeviceInterfaceType.Gamepad, Enum.InputDeviceInterfaceType.Mkb)
        assert(C_KeyBindings.IsBindingContextActive(context) == true)
        A_Admin.SetUsingGamepad(false)
        A_Admin.FireEvent("INPUT_DEVICE_INTERFACE_TRANSITION", Enum.InputDeviceInterfaceType.Mkb, Enum.InputDeviceInterfaceType.Gamepad)
        assert(C_KeyBindings.IsBindingContextActive(context) == false)
        "#,
    )
    .expect("Gamepad Core activates and deactivates its binding context");
}

#[test]
fn forever_guild_control_menus_follow_supplied_discord_servers() {
    let env = startup();
    env.exec(DISCORD_GUILD_CONTROL_MENU)
        .expect("Guild Control menus show supplied Discord server and channel choices");
}

#[test]
fn forever_original_guild_menu_projects_discord_cvar() {
    let env = startup();
    env.state()
        .borrow()
        .cvars
        .register("discordClientEnabled", Some("0"));
    for enabled in [false, true] {
        assert!(
            env.state()
                .borrow()
                .cvars
                .set("discordClientEnabled", if enabled { "1" } else { "0" })
        );
        let found = env
            .eval::<bool>(
                r#"
            GuildControlUI_SetupSelected(1)
            GuildControlUI.dropdown:GenerateMenu()
            local description = GuildControlUI.dropdown:GetMenuDescription()
            for _, entry in description:EnumerateElementDescriptions() do
                if entry:GetData() == 4 then return true end
            end
            return false
            "#,
            )
            .unwrap();
        assert_eq!(found, enabled);
        assert_eq!(
            env.eval::<bool>("return C_Discord.IsEnabled()").unwrap(),
            enabled
        );
    }
}

#[test]
fn forever_original_raid_manager_projects_ping_availability() {
    let env = startup();
    env.exec(
        r#"
        CompactRaidFrameManager_OnLoad(CompactRaidFrameManager)
        local menu = CompactRaidFrameManager.displayFrame.RestrictPingsDropdown:GetMenuDescription()
        assert(not menu or not menu:HasElements())
        "#,
    )
    .unwrap();
    env.state().borrow_mut().forever_startup.ping_enabled = true;
    env.exec(
        r#"
        CompactRaidFrameManager_OnLoad(CompactRaidFrameManager)
        CompactRaidFrameManager.displayFrame.RestrictPingsDropdown:GenerateMenu()
        local menu = CompactRaidFrameManager.displayFrame.RestrictPingsDropdown:GetMenuDescription()
        assert(menu:HasElements())
        assert(C_Ping.IsPingSystemEnabled() == true)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_original_bank_lock_consumer_projects_nullable_reason() {
    let env = startup();
    env.exec(
        r#"
        lockProbe = CreateFromMixins(BankPanelMixin)
        lockProbe.bankType = Enum.BankType.Character
        assert(lockProbe:IsBankTypeLocked() == false)
        assert(C_Bank.FetchBankLockedReason(Enum.BankType.Character) == nil)
        assert(select('#', C_Bank.FetchBankLockedReason(Enum.BankType.Character)) == 1)
        "#,
    )
    .unwrap();
    let bank_type = env.eval::<i32>("return Enum.BankType.Character").unwrap();
    let reason = env
        .eval::<i32>("return Enum.BankLockedReason.BankConversionFailed")
        .unwrap();
    env.state()
        .borrow_mut()
        .forever_startup
        .bank_locked_reasons
        .insert(bank_type, reason);
    env.exec(
        r#"
        assert(lockProbe:IsBankTypeLocked() == true)
        assert(C_Bank.FetchBankLockedReason(Enum.BankType.Character) == Enum.BankLockedReason.BankConversionFailed)
        assert(C_Bank.FetchBankLockedReason(Enum.BankType.Account) == nil)
        "#,
    ).unwrap();
}

#[test]
fn forever_original_tts_consumer_marks_character_settings_saved() {
    let env = startup();
    env.state()
        .borrow_mut()
        .forever_startup
        .character_tts_settings_saved = false;
    env.exec(
        r#"
        assert(C_TTSSettings.GetCharacterSettingsSaved() == false)
        TextToSpeechFrame.loaded = false
        TextToSpeechFrame.loadedEvents = {
            PLAYER_ENTERING_WORLD = true,
            VARIABLES_LOADED = true,
            ADDON_LOADED = true,
        }
        TextToSpeechFrame_CheckLoad(TextToSpeechFrame)
        assert(C_TTSSettings.GetCharacterSettingsSaved() == true)
        assert(select('#', C_TTSSettings.MarkCharacterSettingsSaved()) == 0)
        "#,
    )
    .unwrap();
    assert!(
        env.state()
            .borrow()
            .forever_startup
            .character_tts_settings_saved
    );
}

#[test]
fn forever_original_game_housing_consumer_reads_spatial_state() {
    let env = startup();
    for area in [
        None,
        Some(HousingArea::HouseInterior),
        Some(HousingArea::PlotExterior),
    ] {
        env.state().borrow_mut().housing.current_housing_area = area;
        assert_eq!(
            env.eval::<bool>("return C_Housing.IsInsideHouseOrPlot()")
                .unwrap(),
            area.is_some()
        );
        if area.is_none() {
            env.exec("assert(GameEvent.HandlePlayerEnteringWorld(nil, 'PLAYER_ENTERING_WORLD', false, false) == true)").unwrap();
        } else {
            // 70532 Blizzard_HousingControls.toc: ## AllowLoadGameType: standard.
            // The sole HousingControls_LoadUI publisher is excluded from camelot.
            let error = env.eval::<String>(r#"
                assert(HousingControls_LoadUI == nil)
                local ok, error = pcall(GameEvent.HandlePlayerEnteringWorld, nil, 'PLAYER_ENTERING_WORLD', false, false)
                assert(not ok)
                return error
            "#).unwrap();
            assert_eq!(
                error,
                "...ce/AddOns/Blizzard_Game/Mainline/EventImplementation.lua:852: attempt to call a nil value"
            );
        }
    }
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let toc = wow_ui_sim::toc::TocFile::from_file(
        &ui.join("Blizzard_HousingControls/Blizzard_HousingControls.toc"),
    )
    .unwrap();
    assert!(toc.is_game_type_restricted());
    assert!(
        !env.eval::<bool>("return C_AddOns.IsAddOnLoaded('Blizzard_HousingControls')")
            .unwrap()
    );
}

#[test]
fn forever_original_gamepad_core_removes_temporary_cvars() {
    let env = WowLuaEnv::new().unwrap();
    assert!(env.state().borrow().cvars.set("SoftTargetEnemy", "2"));
    env.exec("C_CVar.SetTempCVar('SoftTargetEnemy', '7')")
        .unwrap();
    load_startup(&env);
    assert_eq!(
        env.state().borrow().cvars.get("SoftTargetEnemy").as_deref(),
        Some("2")
    );
    assert!(env.state().borrow().cvars.set("SoftTargetEnemy", "3"));
    env.exec("assert(GetCVar('SoftTargetEnemy') == '3'); assert(C_CVar.GetCVar('SoftTargetEnemy') == '3')").unwrap();
    assert_eq!(
        env.state().borrow().cvars.get("SoftTargetEnemy").as_deref(),
        Some("3")
    );
    let saved = env.state().borrow().cvars.get("SoftTargetEnemy");
    env.exec("C_CVar.SetTempCVar('SoftTargetEnemy', '9')")
        .unwrap();
    assert_eq!(
        env.eval::<String>("return GetCVar('SoftTargetEnemy')")
            .unwrap(),
        "9"
    );
    assert_eq!(
        env.eval::<String>("return C_CVar.GetCVar('SoftTargetEnemy')")
            .unwrap(),
        "9"
    );
    let core = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_Gamepad/Core.lua");
    env.exec(&std::fs::read_to_string(core).unwrap()).unwrap();
    assert_eq!(
        env.eval::<Option<String>>("return GetCVar('SoftTargetEnemy')")
            .unwrap(),
        saved
    );
    assert_eq!(env.state().borrow().cvars.get("SoftTargetEnemy"), saved);
    assert!(env.state().borrow().cvars.set("SoftTargetEnemy", "4"));
    env.exec("assert(GetCVar('SoftTargetEnemy') == '4'); assert(C_CVar.GetCVar('SoftTargetEnemy') == '4')").unwrap();
    assert_eq!(
        env.state().borrow().cvars.get("SoftTargetEnemy").as_deref(),
        Some("4")
    );
}

#[test]
fn forever_original_bank_tabs_project_maximum_state() {
    let env = startup();
    let bank_type = env.eval::<i32>("return BankPanel.bankType").unwrap();
    env.exec(
        r#"
        BankPanel:RefreshBankTabs()
        assert(BankPanel.PurchaseTab:IsShown())
        assert(C_Bank.HasMaxBankTabs(BankPanel.bankType) == false)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .forever_startup
        .max_bank_tab_types
        .insert(bank_type);
    env.exec(
        r#"
        BankPanel:RefreshBankTabs()
        assert(not BankPanel.PurchaseTab:IsShown())
        assert(C_Bank.HasMaxBankTabs(BankPanel.bankType) == true)
        assert(select('#', C_Bank.HasMaxBankTabs(BankPanel.bankType)) == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_original_tts_checkboxes_project_bool_settings() {
    let env = startup();
    env.exec(
        r#"
        TextToSpeechFrame_Update(TextToSpeechFrame)
        assert(not TextToSpeechFrame.PanelContainer.AddCharacterNameToSpeechCheckButton:GetChecked())
        assert(not TextToSpeechFrame.PanelContainer.NarrateMyMessagesCheckButton:GetChecked())
        "#,
    ).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.forever_startup.enabled_tts_settings.extend([1, 4]);
    }
    env.exec(
        r#"
        assert(Enum.TtsBoolSetting.AddCharacterNameToSpeech == 1)
        assert(Enum.TtsBoolSetting.NarrateMyMessages == 4)
        TextToSpeechFrame_Update(TextToSpeechFrame)
        assert(TextToSpeechFrame.PanelContainer.AddCharacterNameToSpeechCheckButton:GetChecked())
        assert(TextToSpeechFrame.PanelContainer.NarrateMyMessagesCheckButton:GetChecked())
        assert(not TextToSpeechFrame.PanelContainer.UseAlternateVoiceForSystemMessagesCheckButton:GetChecked())
        assert(C_TTSSettings.GetSetting(Enum.TtsBoolSetting.AddCharacterNameToSpeech) == true)
        assert(C_TTSSettings.GetSetting(Enum.TtsBoolSetting.AlternateSystemVoice) == false)
        assert(select('#', C_TTSSettings.GetSetting(Enum.TtsBoolSetting.AlternateSystemVoice)) == 1)
        "#,
    ).unwrap();
}

#[test]
fn forever_original_ping_menu_projects_party_restriction() {
    let env = startup();
    env.state().borrow_mut().forever_startup.ping_enabled = true;
    for restriction in [0, 3] {
        env.state()
            .borrow_mut()
            .forever_startup
            .party_ping_restriction = restriction;
        let selected = env
            .eval::<i32>(
                r#"
            CompactRaidFrameManager_OnLoad(CompactRaidFrameManager)
            local dropdown = CompactRaidFrameManager.displayFrame.RestrictPingsDropdown
            dropdown:GenerateMenu()
            local description = dropdown:GetMenuDescription()
            local selected, count = nil, 0
            for _, entry in description:EnumerateElementDescriptions() do
                if entry:IsSelected() then
                    selected, count = entry:GetData(), count + 1
                end
            end
            assert(count == 1)
            assert(Enum.RestrictPingsTo.None == 0 and Enum.RestrictPingsTo.TankHealer == 3)
            return selected
            "#,
            )
            .unwrap();
        assert_eq!(selected, i32::from(restriction));
    }
}

#[test]
fn forever_original_bank_first_slot_reads_optional_offer() {
    let env = startup();
    env.exec(
        r#"
        BankFrame:PurchaseFirstSlot()
        assert(C_Bank.FetchNextPurchasableBankTabData(Enum.BankType.Character) == nil)
        assert(select('#', C_Bank.FetchNextPurchasableBankTabData(Enum.BankType.Character)) == 1)
        "#,
    )
    .unwrap();
    let bank_type = env.eval::<i32>("return Enum.BankType.Character").unwrap();
    env.state()
        .borrow_mut()
        .forever_startup
        .next_bank_tabs
        .insert(
            bank_type,
            PurchasableBankTab {
                tab_cost: 4200,
                can_afford: false,
                purchase_prompt_title: "Bank tab offer".to_owned(),
                purchase_prompt_body: "A supplied paid tab".to_owned(),
                purchase_prompt_confirmation: "Confirm tab offer".to_owned(),
            },
        );
    env.exec(
        r#"
        BankFrame:PurchaseFirstSlot()
        local data = C_Bank.FetchNextPurchasableBankTabData(Enum.BankType.Character)
        assert(data.tabCost == 4200 and data.canAfford == false)
        assert(data.purchasePromptTitle == "Bank tab offer")
        assert(data.purchasePromptBody == "A supplied paid tab")
        assert(data.purchasePromptConfirmation == "Confirm tab offer")
        assert(C_Bank.FetchNextPurchasableBankTabData(Enum.BankType.Account) == nil)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .forever_startup
        .next_bank_tabs
        .clear();
    env.exec("BankFrame:PurchaseFirstSlot(); assert(C_Bank.FetchNextPurchasableBankTabData(Enum.BankType.Character) == nil)").unwrap();
}

#[test]
fn forever_original_social_consumer_accepts_raid_status_event() {
    let env = startup();
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('SOCIAL_UI_RAID_LIST_SYSTEM_STATUS_UPDATED')
        assert(frame:IsEventRegistered('SOCIAL_UI_RAID_LIST_SYSTEM_STATUS_UPDATED'))
        "#,
    )
    .unwrap();
    assert!(
        !env.state()
            .borrow()
            .lua_errors
            .iter()
            .any(|error| error.contains("SOCIAL_UI_RAID_LIST_SYSTEM_STATUS_UPDATED"))
    );
}
