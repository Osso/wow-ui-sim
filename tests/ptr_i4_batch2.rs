//! PTR 70493 original consumers; supplied inputs do not establish native policy.
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen_with_overrides, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn load_consumers(root_name: &str, screen: ScreenKind, hardcore: bool) -> WowLuaEnv {
    let root = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = match screen {
        ScreenKind::Game => crate::common::blizzard_addon_harness::new_blizzard_addon_env(&root),
        _ => crate::common::blizzard_addon_harness::new_blizzard_addon_glue_env(&root),
    };
    env.state().borrow_mut().game_rules.hardcore_active = hardcore;
    let selected =
        discover_blizzard_addon_closure_for_screen_with_overrides(&root, screen, &[root_name], &[]);
    assert!(selected.iter().any(|(name, _)| name == root_name));
    for (name, toc) in selected {
        if name == root_name {
            assert_selected_consumer(&toc, root_name);
        }
        let result = load_addon(&env.loader_env(), &toc).unwrap();
        if name == root_name {
            report_and_reject_target_gaps(&result.warnings);
        }
    }
    env
}

fn report_and_reject_target_gaps(warnings: &[String]) {
    for warning in warnings {
        eprintln!("original consumer warning: {warning}");
        let target_gap = warning.contains("GameDialogDefs.lua:3512:")
            || warning.contains("IsHardcoreActive")
            || warning.contains("RecentAlliesInteractionCategoryFilter");
        assert!(!target_gap, "{warning}");
    }
}

fn assert_selected_consumer(toc: &std::path::Path, name: &str) {
    let relative = match name {
        "Blizzard_StaticPopup_Game" => "GameDialogDefs.lua",
        "Blizzard_BarbershopUI" => "Mainline/Blizzard_BarberShopUI.lua",
        "Blizzard_CharacterCreate" => "Mainline/Blizzard_CharacterCreate.lua",
        "Blizzard_RecentAllies" => "Blizzard_RecentAlliesUtil.lua",
        "Blizzard_MicroMenu" => "Shared/MicroMenuContainer.lua",
        "Blizzard_UIErrorsFrame" => "Mainline/UIErrorsFrame.lua",
        _ => panic!("unexpected consumer root: {name}"),
    };
    let parsed = wow_ui_sim::toc::TocFile::from_file(toc).unwrap();
    assert!(
        parsed
            .files
            .iter()
            .any(|path| path == std::path::Path::new(relative))
    );
    eprintln!("runtime-selected original consumer: {name}/{relative}");
}

fn assert_boolean_query_shape(env: &WowLuaEnv, query: &str) {
    env.exec(&format!(
        "assert(type(C_GameRules.{query}()) == 'boolean'); assert(select('#', C_GameRules.{query}()) == 1)"
    ))
    .unwrap();
}

#[test]
fn hardcore_popup_consumes_changed_supplied_mode() {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    for active in [false, true] {
        let env = load_consumers("Blizzard_StaticPopup_Game", ScreenKind::Game, active);
        assert_boolean_query_shape(&env, "IsHardcoreActive");
        assert_eq!(
            env.eval::<bool>("return StaticPopupDialogs.HARDCORE_DEATH ~= nil")
                .unwrap(),
            active
        );
    }
}

#[test]
fn barbershop_visibility_consumes_toggle_capability() {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let env = load_consumers("Blizzard_BarbershopUI", ScreenKind::Game, false);
    env.exec("I4Toggle = CreateFrame('CheckButton'); I4Toggle:Hide()")
        .unwrap();
    for enabled in [true, false, true] {
        env.state().borrow_mut().game_rules.sd_hd_toggle_enabled = enabled;
        env.exec("BarberShopSDToggleMixin.OnLoad(I4Toggle)")
            .unwrap();
        assert_boolean_query_shape(&env, "IsSDHDToggleEnabled");
        assert_eq!(
            env.eval::<bool>("return I4Toggle:IsShown()").unwrap(),
            enabled
        );
    }
}

#[test]
fn barbershop_checked_state_consumes_account_sd() {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let env = load_consumers("Blizzard_BarbershopUI", ScreenKind::Game, false);
    env.exec("I4Toggle = CreateFrame('CheckButton')").unwrap();
    for enabled in [true, false, true] {
        env.state().borrow_mut().game_rules.account_has_sd_enabled = enabled;
        env.exec("BarberShopSDToggleMixin.OnShow(I4Toggle)")
            .unwrap();
        assert_boolean_query_shape(&env, "AccountHasSDEnabled");
        assert_eq!(
            env.eval::<bool>("return not not I4Toggle:GetChecked()")
                .unwrap(),
            !enabled
        );
    }
}

#[test]
fn barbershop_click_updates_supplied_sd_selection() {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let env = load_consumers("Blizzard_BarbershopUI", ScreenKind::Game, false);
    env.exec("I4Toggle = CreateFrame('CheckButton')").unwrap();
    for checked in [false, true, false] {
        env.exec(&format!(
            "I4Toggle:SetChecked({checked}); BarberShopSDToggleMixin.OnClick(I4Toggle)"
        ))
        .unwrap();
        {
            let state = env.state().borrow();
            assert_eq!(state.game_rules.account_has_sd_enabled, !checked);
            // 70493 GameRulesDocumentation: BarberShop = 2.
            assert_eq!(state.game_rules.sd_toggle_usage_location, Some(2));
        }
        // Check arity only after checking the original consumer's mutation.
        env.exec("assert(select('#', C_GameRules.SetSDHDToggleValue(not I4Toggle:GetChecked(), Enum.SdToggleUsageLocationUI.BarberShop)) == 0)").unwrap();
    }
    let other = WowLuaEnv::new().unwrap();
    assert!(!other.state().borrow().game_rules.account_has_sd_enabled);
    assert_eq!(
        other.state().borrow().game_rules.sd_toggle_usage_location,
        None
    );
}

#[test]
fn character_create_consumes_self_found_capability() {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let env = load_consumers(
        "Blizzard_CharacterCreate",
        ScreenKind::CharacterSelect,
        false,
    );
    for allowed in [true, false, true] {
        env.state().borrow_mut().game_rules.self_found_allowed = allowed;
        env.exec("CharacterCreateSelfFoundButtonMixin.CheckSelfFoundButton(CharacterCreateSelfFoundButton)").unwrap();
        assert_boolean_query_shape(&env, "IsSelfFoundAllowed");
        assert_eq!(
            env.eval::<bool>("return CharacterCreateSelfFoundButton:IsShown()")
                .unwrap(),
            allowed
        );
    }
}

#[test]
fn recent_allies_original_labels_consume_documented_enum() {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let env = load_consumers("Blizzard_RecentAllies", ScreenKind::Game, false);
    env.exec(r#"
        local names = {'Professions', 'PvP', 'Raiding', 'Dungeons', 'Delves', 'Questing'}
        local labels = {
            SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_PROFESSIONS,
            SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_PVP,
            SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_RAIDING,
            SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_DUNGEONS,
            SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_DELVE,
            SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_QUESTING,
        }
        for index, name in ipairs(names) do
            assert(RecentAlliesUtil.GetLabelForInteractionCategoryFilter(index - 1) == labels[index])
            assert(Enum.RecentAlliesInteractionCategoryFilter[name] == index - 1)
        end
        assert(RecentAlliesUtil.GetLabelForInteractionCategoryFilter(77) == '')
        local meta = Enum.RecentAlliesInteractionCategoryFilterMeta
        assert(meta.MinValue == 0 and meta.MaxValue == 5 and meta.NumValues == 6)
    "#).unwrap();
}

#[test]
fn micro_menu_onload_consumes_supplied_wowhack_mode() {
    if !matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::Retail) {
        return;
    }
    let env = load_consumers("Blizzard_MicroMenu", ScreenKind::Game, false);
    for mode in ["Standard", "WoWHack", "Plunderstorm", "WoWHack"] {
        let mode_id = env
            .eval::<i32>(&format!("return Enum.GameMode.{mode}"))
            .unwrap();
        env.state().borrow_mut().game_rules.active_game_mode = mode_id;
        // Reset the observed button placement before the original OnLoad runs again.
        env.exec(
            "for _, name in ipairs({'PlayerSpellsMicroButton', 'SpellbookMicroButton', \
             'TalentMicroButton', 'ScribeMicroButton'}) do \
             local button = _G[name]; if button then button:SetParent(UIParent); \
             button.layoutIndex = nil end end; MicroMenu.numButtons = 0; MicroMenu:OnLoad()",
        )
        .unwrap();
        assert_boolean_query_shape(&env, "IsWoWHack");
        let wowhack = mode == "WoWHack";
        assert_eq!(env.eval::<bool>("return C_GameRules.IsWoWHack()").unwrap(), wowhack);
        // The pinned Retail override always uses PlayerSpells; PTR selects separate buttons.
        let separated = ACTIVE == ClientProfile::Ptr && wowhack;
        assert_eq!(
            env.eval::<bool>("return PlayerSpellsMicroButton:GetParent() == MicroMenu and type(PlayerSpellsMicroButton.layoutIndex) == 'number'")
                .unwrap(),
            !separated,
            "{mode}: PlayerSpellsMicroButton"
        );
        if ACTIVE == ClientProfile::Ptr {
            for button in ["SpellbookMicroButton", "TalentMicroButton", "ScribeMicroButton"] {
                assert_eq!(
                    env.eval::<bool>(&format!(
                        "return {button}:GetParent() == MicroMenu and type({button}.layoutIndex) == 'number'"
                    ))
                    .unwrap(),
                    separated,
                    "{mode}: {button}"
                );
            }
        }
    }
    let other = WowLuaEnv::new().unwrap();
    assert!(
        !other
            .eval::<bool>("return C_GameRules.IsWoWHack()")
            .unwrap()
    );
}

#[test]
fn ui_errors_onload_consumes_supplied_wowhack_mode() {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let env = load_consumers("Blizzard_UIErrorsFrame", ScreenKind::Game, false);
    for mode in ["Standard", "WoWHack", "Plunderstorm", "WoWHack"] {
        let mode_id = env
            .eval::<i32>(&format!("return Enum.GameMode.{mode}"))
            .unwrap();
        env.state().borrow_mut().game_rules.active_game_mode = mode_id;
        env.exec("UIErrorsFrame:ClearAllPoints(); UIErrorsFrame:SetPoint('TOP', UIParent, 'TOP', 0, -122); UIErrorsFrame:SetMessageTypeEnabled(LE_GAME_ERR_OUT_OF_ENERGY, false)").unwrap();
        // Original UIErrorsFrame.lua:12 must run before the assertions.
        env.exec("UIErrorsFrame:OnLoad()").unwrap();
        assert_boolean_query_shape(&env, "IsWoWHack");
        let wowhack = mode == "WoWHack";
        assert_eq!(
            env.eval::<bool>("return UIErrorsFrame:ShouldDisplayMessageType(LE_GAME_ERR_OUT_OF_ENERGY, 'I4 energy')")
                .unwrap(),
            wowhack
        );
        assert_eq!(
            env.eval::<f64>("local _, _, _, _, y = UIErrorsFrame:GetPoint(1); return y")
                .unwrap(),
            if wowhack { -250.0 } else { -122.0 }
        );
        assert!(
            env.eval::<bool>("return type(UIErrorsFrame.flashingFontStrings) == 'table'")
                .unwrap()
        );
    }
}

#[test]
fn non_ptr_publication_is_unchanged() {
    if ACTIVE == ClientProfile::Ptr {
        return;
    }
    let env = WowLuaEnv::new().unwrap();
    if ACTIVE == ClientProfile::Retail {
        env.exec(
            r#"
            for _, name in ipairs({'IsHardcoreActive', 'IsSDHDToggleEnabled', 'AccountHasSDEnabled',
                'SetSDHDToggleValue', 'IsSelfFoundAllowed', 'GetForeverExperiencePreset'}) do
                assert(rawget(C_GameRules, name) == nil, name)
            end
            assert(rawget(Enum, 'RecentAlliesInteractionCategoryFilter') == nil)
        "#,
        )
        .unwrap();
    }
}
