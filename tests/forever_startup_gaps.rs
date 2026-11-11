#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::forever_startup::{ExternalEvent, GroupBuffAlert};
use wow_ui_sim::lua_api::WowLuaEnv;

fn startup() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    crate::common::load_profile_framexml(&env);
    let addons = wow_ui_sim::loader::discover_blizzard_addons(&ui);
    for (name, toc) in addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    env
}

#[test]
fn forever_original_consumers_register_70532_events() {
    let env = startup();
    env.exec(
        r#"
        assert(SecureTransferDialog:IsEventRegistered("SECURE_TRANSFER_CONFIRM_TAKE_MAIL_MONEY"))
        assert(ShardTransferImminentFrame:IsEventRegistered("SHARD_TRANSFER_CANCELLED"))
        assert(ShardTransferImminentMinimizeButton:IsEventRegistered("SHARD_TRANSFER_CANCELLED"))
        local probe = CreateFrame("Frame")
        for _, event in ipairs({
            "EXTERNAL_EVENT_LAUNCH_URL_RESULT",
            "SOCIAL_UI_SOCIAL_QUEUE_SYSTEM_STATUS_UPDATED",
        }) do
            probe:RegisterEvent(event)
            assert(probe:IsEventRegistered(event), event)
        end
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    for event in [
        "SECURE_TRANSFER_CONFIRM_TAKE_MAIL_MONEY",
        "SHARD_TRANSFER_CANCELLED",
        "EXTERNAL_EVENT_LAUNCH_URL_RESULT",
        "SOCIAL_UI_SOCIAL_QUEUE_SYSTEM_STATUS_UPDATED",
    ] {
        assert!(
            !state.lua_errors.iter().any(|error| error.contains(event)),
            "{event}"
        );
    }
}

#[test]
fn forever_original_group_buff_consumer_projects_supplied_alerts() {
    let env = startup();
    env.state().borrow_mut().forever_startup.group_buff_alerts = vec![
        GroupBuffAlert {
            spell_id: 21562,
            visual_value: 1,
        },
        GroupBuffAlert {
            spell_id: 1459,
            visual_value: 2,
        },
    ];
    env.exec(
        r#"
        local consumer = CreateFromMixins(GroupBuffMixin)
        consumer:RefreshGroupBuffVisualAlerts()
        assert(consumer:GetGroupBuffVisualAlert(21562) == 1)
        assert(consumer:GetGroupBuffVisualAlert(1459) == 2)
        assert(consumer:GetGroupBuffVisualAlert(999999) == nil)
        local rows = C_UnitAuras.GetGroupBuffVisualAlerts()
        assert(#rows == 2 and rows[1].spellID == 21562 and rows[2].visualValue == 2)
        assert(Enum.VisualAlertType.MarchingAnts == 1)
        assert(Enum.VisualAlertType.MarchingAntsCyan == 2)
        assert(select('#', C_UnitAuras.GetGroupBuffVisualAlerts()) == 1)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .forever_startup
        .group_buff_alerts
        .clear();
    env.exec(
        r#"
        local consumer = CreateFromMixins(GroupBuffMixin)
        consumer:RefreshGroupBuffVisualAlerts()
        assert(consumer:GetGroupBuffVisualAlert(21562) == nil)
        assert(#C_UnitAuras.GetGroupBuffVisualAlerts() == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_original_external_event_consumer_projects_url_state() {
    let env = startup();
    env.exec(
        r#"
        urlProbe = CreateFrame("Button", nil, UIParent, "ExternalEventURLMinimapButtonTemplate")
        urlProbe:Refresh()
        assert(not urlProbe:IsShown())
        assert(not C_ExternalEventURL.CanShow())
        assert(not C_ExternalEventURL.HasURL())
        assert(not C_ExternalEventURL.IsNew())
        "#,
    )
    .unwrap();
    env.state().borrow_mut().forever_startup.external_event = Some(ExternalEvent {
        url: "https://example.invalid/event".to_owned(),
        can_show: true,
        is_new: true,
    });
    env.exec(
        r#"
        urlProbe:Refresh()
        assert(urlProbe:IsShown())
        assert(urlProbe.SpellEffectModelScene:IsShown())
        assert(C_ExternalEventURL.CanShow() and C_ExternalEventURL.HasURL() and C_ExternalEventURL.IsNew())
        assert(select('#', C_ExternalEventURL.HasURL()) == 1)
        "#,
    ).unwrap();
    env.state()
        .borrow_mut()
        .forever_startup
        .external_event
        .as_mut()
        .unwrap()
        .is_new = false;
    env.exec("urlProbe:Refresh(); assert(not urlProbe.SpellEffectModelScene:IsShown())")
        .unwrap();
    env.state()
        .borrow_mut()
        .forever_startup
        .external_event
        .as_mut()
        .unwrap()
        .url
        .clear();
    env.exec("urlProbe:Refresh(); assert(not urlProbe:IsShown()); assert(not C_ExternalEventURL.HasURL())")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        let event = state.forever_startup.external_event.as_mut().unwrap();
        event.url = "https://example.invalid/second".to_owned();
        event.can_show = false;
    }
    env.exec("urlProbe:Refresh(); assert(not urlProbe:IsShown())")
        .unwrap();
}

#[test]
fn forever_original_gamepad_reticle_projects_cursor_visibility() {
    let env = startup();
    for visible in [true, false] {
        env.state().borrow_mut().forever_startup.cursor_visible = visible;
        env.exec("GamepadReticle:OnUpdate()").unwrap();
        assert_eq!(
            env.eval::<bool>("return GamepadReticle.Icons:IsShown()")
                .unwrap(),
            !visible
        );
        assert_eq!(
            env.eval::<bool>("return GamepadReticle.SwingTimer.enabledCursorVis")
                .unwrap(),
            !visible
        );
        assert_eq!(
            env.eval::<bool>("return IsCursorVisible()").unwrap(),
            visible
        );
    }
}

#[test]
fn forever_original_bank_consumer_uses_bank_type_permissions() {
    let env = startup();
    env.exec(
        r#"
        bankProbe = CreateFromMixins(BankPanelMixin)
        bankProbe.bankType = Enum.BankType.Character
        bankProbe.PurchaseTab = CreateFrame("CheckButton")
        bankProbe.PurchaseTab:SetChecked(true)
        bankProbe.PurchaseTab.IsSelected = bankProbe.PurchaseTab.GetChecked
        assert(bankProbe:ShouldShowPurchasePrompt() == false)
        "#,
    )
    .unwrap();
    let bank_type = env.eval::<i32>("return Enum.BankType.Character").unwrap();
    env.state()
        .borrow_mut()
        .forever_startup
        .purchasable_bank_types
        .insert(bank_type);
    env.exec(
        r#"
        assert(bankProbe:ShouldShowPurchasePrompt() == true)
        bankProbe.bankType = Enum.BankType.Account
        assert(bankProbe:ShouldShowPurchasePrompt() == false)
        assert(select('#', C_Bank.CanPurchaseBankTab(Enum.BankType.Character)) == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_original_pvp_consumer_uses_completion_state() {
    let env = startup();
    env.state().borrow_mut().pvp_match_active = false;
    env.exec(
        r#"
        pvpProbe = CreateFrame("Frame")
        PVPMatchResultsMixin.OnEvent(pvpProbe, "PLAYER_ENTERING_WORLD")
        assert(not pvpProbe:IsEventRegistered("PVP_MATCH_COMPLETE"))
        "#,
    )
    .unwrap();
    env.state().borrow_mut().forever_startup.match_complete = true;
    env.exec(
        r#"
        PVPMatchResultsMixin.OnEvent(pvpProbe, "PLAYER_ENTERING_WORLD")
        assert(pvpProbe:IsEventRegistered("PVP_MATCH_COMPLETE"))
        assert(C_PvP.IsMatchComplete() == true)
        assert(select('#', C_PvP.IsMatchComplete()) == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_stock_target_spell_reaches_original_item_context() {
    let env = startup();
    env.exec(
        r#"
        local info = debug.getinfo(ItemButtonUtil.GetItemContext, 'S')
        assert(info.source:match('Blizzard_FrameXMLUtil/ItemUtil.lua$'))
        local ok, result = xpcall(ItemButtonUtil.GetItemContext, debug.traceback)
        assert(ok, result)
        assert(result == nil and C_Spell.GetTargetSpellID() == 0)
        assert(C_Spell.GetTargetSpellID() ~= FEED_PET_SPELL_ID)
        "#,
    )
    .expect("stock target must reach original item-context consumer");
}

#[test]
fn forever_stock_localities_reach_original_dropdown() {
    let env = startup();
    env.exec(
        r#"
        local init = CommunitiesGuildPreferredPlaySettingsFrameMixin.InitializeDatacenterDropdown
        local info = debug.getinfo(init, 'S')
        assert(info.source:match('Blizzard_Communities/GuildPreferredPlaySettings.lua$'))
        local menu
        local dropdown = { SetupMenu = function(_, callback) menu = callback end }
        local frame = { DatacenterDropdown = dropdown }
        init(frame)
        local entries = 0
        menu(dropdown, { CreateButton = function() entries = entries + 1 end })
        local ids, names = C_AccountServices.GetPreferredDatacenterLocalities()
        assert(type(ids) == 'table' and type(names) == 'table')
        assert(#ids == 0 and #names == 0 and entries == 0)
    "#,
    )
    .expect("stock localities must reach original dropdown consumer");
}

#[test]
fn forever_original_item_context_uses_supplied_target_spell() {
    let env = startup();
    env.state().borrow_mut().forever_startup.target_spell_id = Some(6991);
    env.exec("assert(ItemButtonUtil.GetItemContext() == ItemButtonUtil.ItemContextEnum.FeedPet)")
        .unwrap();
    env.state().borrow_mut().forever_startup.target_spell_id = Some(19750);
    env.exec("assert(ItemButtonUtil.GetItemContext() ~= ItemButtonUtil.ItemContextEnum.FeedPet)")
        .unwrap();
    env.state().borrow_mut().forever_startup.target_spell_id = None;
    env.exec("assert(C_Spell.GetTargetSpellID() == nil)")
        .unwrap();
}
