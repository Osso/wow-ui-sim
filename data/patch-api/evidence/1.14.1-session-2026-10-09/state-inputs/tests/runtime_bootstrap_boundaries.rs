use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn c_macro_namespace_is_not_generic_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_Macro = C_Macro or __wow_namespace()"),
        "C_Macro must be registered by Rust or the explicit macro workaround boundary, not generic runtime bootstrap"
    );
}

#[test]
fn state_backed_namespaces_are_not_generic_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for namespace in ["C_PaperDollInfo", "C_StorePublic", "C_Widget"] {
        let fallback = format!("{namespace} = {namespace} or __wow_namespace()");
        assert!(
            !bootstrap.contains(&fallback),
            "{namespace} must be registered by Rust or an explicit workaround boundary, not generic runtime bootstrap"
        );
    }
}

#[test]
fn profession_spec_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_ProfSpecs"),
        "C_ProfSpecs defaults must live in the explicit temporary professions workaround boundary"
    );
}

#[test]
fn legacy_spell_wrappers_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("IsPressHoldReleaseSpell"),
        "legacy spell wrappers must live in the explicit temporary legacy spell workaround boundary"
    );
    assert!(
        !bootstrap.contains("IsSelectedSpellBookItem"),
        "legacy spellbook selection fallback must live in the explicit temporary legacy spell workaround boundary"
    );
}

#[test]
fn game_time_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("GameTime_GetTime"),
        "GameTime defaults must live in the explicit temporary GameTime/calendar workaround boundary"
    );
    assert!(
        !bootstrap.contains("function GetGameTime"),
        "GetGameTime default must live in the explicit temporary GameTime/calendar workaround boundary"
    );
    assert!(
        !bootstrap.contains("function GetTime"),
        "GetTime must be registered by the Rust global surface, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("__wow_normalize_time_table"),
        "time() normalization helper must live in the explicit temporary GameTime/calendar workaround boundary"
    );
    assert!(
        !bootstrap.contains("function time(dateTable)"),
        "time() default must live in the explicit temporary GameTime/calendar workaround boundary"
    );
    assert!(
        !bootstrap.contains("function time()"),
        "time() default must live in the explicit temporary GameTime/calendar workaround boundary"
    );
}

#[test]
fn audio_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_CombatAudioAlert"),
        "combat audio defaults must live in the explicit temporary sound workaround boundary"
    );
    assert!(
        !bootstrap.contains("C_Sound"),
        "C_Sound defaults must live in the explicit temporary sound workaround boundary"
    );
    for symbol in ["MuteSoundFile", "UnmuteSoundFile"] {
        assert!(
            !bootstrap.contains(&format!("function {symbol}")),
            "{symbol} default must live in the explicit temporary sound workaround boundary"
        );
    }
}

#[test]
fn ui_frame_manager_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("UIFrameManager"),
        "UIFrameManager defaults must live in the explicit temporary UI workaround boundary"
    );
}

#[test]
fn recruit_a_friend_surface_is_not_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_RecruitAFriend"),
        "C_RecruitAFriend must be registered by its Rust missing-surface module, not runtime bootstrap"
    );
}

#[test]
fn prototype_dialog_surface_is_not_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_PrototypeDialog"),
        "C_PrototypeDialog compatibility state must live in the explicit temporary Lua workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn transmog_sets_surface_is_not_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_TransmogSets"),
        "C_TransmogSets defaults must live in the explicit temporary Lua workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn duration_util_surface_is_not_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_DurationUtil"),
        "C_DurationUtil must be registered by its Rust Lua duration object surface, not runtime bootstrap"
    );
}

#[test]
fn club_surface_is_not_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_Club"),
        "C_Club must be registered by its Rust guild/club surface and temporary notification shim, not runtime bootstrap"
    );
}

#[test]
fn event_utils_surface_is_not_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_EventUtils"),
        "C_EventUtils must be registered by the Rust event registry surface, not runtime bootstrap"
    );
}

#[test]
fn proxy_object_factories_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_CurveUtil"),
        "C_CurveUtil proxy factories must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("C_FunctionContainers"),
        "C_FunctionContainers proxy factories must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("CreateAbbreviateConfig"),
        "AbbreviateConfig proxy factory must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("CreateUnitHealPredictionCalculator"),
        "UnitHealPrediction proxy factory must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("ProxyUtil"),
        "ProxyUtil defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("ProxyConvertableMixin"),
        "ProxyConvertableMixin defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn callback_registry_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("CallbackRegistryMixin"),
        "CallbackRegistryMixin defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("CallbackRegistrantMixin"),
        "CallbackRegistrantMixin defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("CVarCallbackRegistry"),
        "CVarCallbackRegistry defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("EventRegistry"),
        "EventRegistry defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn pool_constructor_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("function CreateFramePool"),
        "CreateFramePool fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("CreateTexturePool ="),
        "CreateTexturePool fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("CreateFontStringPool ="),
        "CreateFontStringPool fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("function CreateFramePoolCollection"),
        "CreateFramePoolCollection fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("function CreateFrameFactory"),
        "CreateFrameFactory fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn debug_environment_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for symbol in [
        "GetGlobalEnvironment",
        "GetButtonMetatable",
        "GetEditBoxMetatable",
        "secretwrap",
        "GetCallstackHeight",
        "GetErrorCallstackHeight",
        "SetErrorCallstackHeight",
        "debugstack",
        "debuglocals",
        "AddSourceLocationExclude",
    ] {
        assert!(
            !bootstrap.contains(&format!("function {symbol}")),
            "{symbol} fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn client_info_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for symbol in [
        "GetBuildInfo",
        "GetRealmName",
        "GetNormalizedRealmName",
        "GetRealmID",
        "GetExpansionLevel",
        "GetUpgradeExpansionLevel",
        "IsExpansionTrial",
        "GetExpansionTrialInfo",
        "IsMacClient",
        "IsWindowsClient",
        "GetGraphicsAPIs",
        "RequestTimePlayed",
        "GetClientDisplayExpansionLevel",
        "GetAccountExpansionLevel",
        "GetMaxLevelForExpansionLevel",
        "GetMaxLevelForPlayerExpansion",
        "GetExpansionDisplayInfo",
        "PartialPlayTime",
        "NoPlayTime",
        "GetBillingTimeRested",
        "GetReleaseTimeRemaining",
    ] {
        assert!(
            !bootstrap.contains(&format!("function {symbol}")),
            "{symbol} fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn formatting_utility_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for symbol in [
        "GetText",
        "BreakUpLargeNumbers",
        "CalculateStringEditDistance",
        "strsplittable",
        "MergeTable",
        "tFilter",
        "mapvalues",
        "tAppendAll",
        "GetScreenDPIScale",
        "FindInTableIf",
        "GetMoneyString",
        "GetColorForCurrencyReward",
        "ConsoleGetColorFromType",
        "ConsoleGetFontHeight",
        "ConsoleSetFontHeight",
        "AbbreviateLargeNumbers",
    ] {
        assert!(
            !bootstrap.contains(&format!("function {symbol}")),
            "{symbol} fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
        );
    }
    assert!(
        !bootstrap.contains("string.K_ReplaceVars"),
        "K_ReplaceVars fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("string.K_AddDefaultValueText"),
        "K_AddDefaultValueText fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("stringIndex:split"),
        "string split fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    for text_global in [
        "BACK = BACK",
        "NEXT = NEXT",
        "PREVIEW = PREVIEW",
        "CUSTOMIZE = CUSTOMIZE",
        "FINISH = FINISH",
    ] {
        assert!(
            !bootstrap.contains(text_global),
            "{text_global} fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
        );
    }
    assert!(
        !bootstrap.contains("difftime = os.difftime"),
        "difftime fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn performance_metric_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for symbol in [
        "GetFramerate",
        "UpdateAddOnMemoryUsage",
        "UpdateAddOnCPUUsage",
        "ResetCPUUsage",
        "GetAddOnMemoryUsage",
        "GetAddOnCPUUsage",
        "GetFrameCPUUsage",
    ] {
        assert!(
            !bootstrap.contains(&format!("function {symbol}")),
            "{symbol} fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn display_scale_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");
    let shared_bootstrap = include_str!("../src/lua_api/env_init/shared_bootstrap.lua");

    for symbol in ["GetDefaultScale", "GetMinRenderScale", "GetMaxRenderScale"] {
        assert!(
            !bootstrap.contains(&format!("function {symbol}")),
            "{symbol} fallback must live in the explicit temporary workaround boundary, not runtime bootstrap"
        );
        assert!(
            !shared_bootstrap.contains(&format!("function {symbol}")),
            "{symbol} fallback must live in the explicit temporary workaround boundary, not shared bootstrap"
        );
    }
}

#[test]
fn action_button_util_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("ActionButtonUtil"),
        "ActionButtonUtil defaults must live in the explicit temporary action-bar workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn c_cvar_surface_is_not_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_CVar"),
        "C_CVar must be registered by the Rust SimState cvar surface, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("__wow_cvars"),
        "CVar storage must stay in SimState.cvars, not a Lua-only runtime bootstrap table"
    );
}

#[test]
fn color_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_UIColor"),
        "C_UIColor defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("C_ColorUtil"),
        "C_ColorUtil defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("function CreateColor"),
        "CreateColor default must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("__wow_make_color"),
        "color construction helper must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("QuestDifficultyColors"),
        "Quest difficulty color defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn merchant_and_raid_lock_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_MerchantFrame"),
        "C_MerchantFrame defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("C_RaidLocks"),
        "C_RaidLocks defaults must live in the explicit temporary workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn state_backed_traits_and_xml_util_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("C_Traits ="),
        "C_Traits must be registered by the Rust trait surface, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("C_XMLUtil ="),
        "C_XMLUtil must be registered by the Rust XML utility surface, not runtime bootstrap"
    );
}

#[test]
fn container_portrait_texture_default_is_not_runtime_bootstrap_fallback() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("SetBagPortraitTexture"),
        "C_Container.SetBagPortraitTexture must live in the explicit temporary container workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn locale_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("function GetLocale"),
        "GetLocale default must live in the explicit temporary legacy/global locale workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn request_load_callbacks_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for callback in [
        "RequestLoadItemDataByID",
        "RequestLoadSpellData",
        "RequestLoadQuestByID",
    ] {
        assert!(
            !bootstrap.contains(callback),
            "{callback} must live in its C API owner or explicit temporary ObjectAPI workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn quest_objective_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for fallback in ["GetTasksTable", "SpellCanTargetQuest", "GetNumAutoQuestPopUps", "GetAutoQuestPopUp"] {
        let owner = "temporary quest objective workaround";
        assert!(
            !bootstrap.contains(&format!("function {fallback}")),
            "{fallback} must live in the explicit {owner} boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn tail_inert_globals_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for fallback in [
        "IsPlayerInWorld",
        "GetItemLevelColor",
        "ClearCursorHoveredItem",
        "SetCursorHoveredItem",
        "SetCursorHoveredItemTradeItem",
        "UnitInSubgroup",
        "GetNumGuildPerks",
        "RequestGuildRewards",
        "GetGuildRenameRequired",
        "GetAvailableBandwidth",
    ] {
        assert!(
            !bootstrap.contains(fallback),
            "{fallback} must live in the explicit temporary inert-global workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn weapon_enchant_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("function GetWeaponEnchantInfo"),
        "GetWeaponEnchantInfo must live in the explicit temporary weapon-enchant workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn catalog_shop_soundkit_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for fallback in [
        "CATALOG_SHOP_SELECT_NAV_MENU",
        "CATALOG_SHOP_SELECT_GENERIC_UI_BUTTON",
    ] {
        assert!(
            !bootstrap.contains(fallback),
            "{fallback} must live in the explicit temporary Catalog Shop workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn transmog_util_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for fallback in [
        "__wow_make_transmog_location",
        "TransmogUtil",
        "GetTransmogLocation",
        "CreateTransmogLocation",
        "GetBestItemModifiedAppearanceID",
    ] {
        assert!(
            !bootstrap.contains(fallback),
            "{fallback} must live in the explicit temporary TransmogUtil workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn assisted_combat_manager_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("AssistedCombatManager"),
        "AssistedCombatManager defaults must live in the explicit temporary assisted combat workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn combat_log_and_chat_onupdate_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("CombatLogInbound"),
        "CombatLogInbound defaults must live in the explicit temporary combat-log workaround boundary, not runtime bootstrap"
    );
    assert!(
        !bootstrap.contains("FCF_OnUpdate == nil"),
        "FCF_OnUpdate default must live in the explicit temporary UIParent OnUpdate workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn seconds_formatter_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for fallback in ["SecondsFormatter", "SecondsFormatterMixin"] {
        assert!(
            !bootstrap.contains(fallback),
            "{fallback} defaults must live in the explicit temporary seconds formatter workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn shared_xml_utility_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for fallback in [
        "CreateAnchor", "GetFinalNameFromTextureKit", "SetClampedTextureRotation", "CopyValuesAsKeys",
        "GetMicroIconForRole", "PingSystemInitializer", "EnumUtil.MakeEnum", "CreateCounter",
        "GetOrCreateTableEntry", "GenerateClosure",
    ] {
        assert!(
            !bootstrap.contains(fallback),
            "{fallback} defaults must live in the explicit temporary SharedXML utility workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn top_level_parent_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for fallback in [
        "GetAppropriateTopLevelParent",
        "SetAlternateTopLevelParent",
        "ClearAlternateTopLevelParent",
        "SetAppropriateTopLevelParent",
        "GetAppropriateTooltip",
    ] {
        assert!(
            !bootstrap.contains(fallback),
            "{fallback} defaults must live in the explicit temporary top-level parent workaround boundary, not runtime bootstrap"
        );
    }
}

#[test]
fn base_nine_slice_dialog_defaults_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("BaseNineSliceDialogMixin"),
        "BaseNineSliceDialogMixin defaults must live in the explicit temporary dialog workaround boundary, not runtime bootstrap"
    );
}

#[test]
fn c_macro_namespace_still_has_rust_backed_macro_text() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let result: String = env
        .eval(
            r#"
            if type(C_Macro) ~= "table" then return "missing_namespace" end
            if type(C_Macro.RunMacroText) ~= "function" then return "missing_run_macro_text" end
            return "ok"
            "#,
        )
        .expect("C_Macro probe should run");

    assert_eq!(result, "ok");
}

#[test]
fn state_backed_namespaces_still_have_registered_members() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let result: String = env
        .eval(
            r#"
            if type(C_PaperDollInfo) ~= "table" then return "missing_paper_doll" end
            if type(C_PaperDollInfo.GetArmorEffectiveness) ~= "function" then return "missing_armor" end
            if type(C_Widget) ~= "table" then return "missing_widget" end
            if type(C_Widget.IsFrameWidget) ~= "function" then return "missing_widget_fn" end
            if C_Widget.IsFrameWidget({}) ~= false then return "widget_table" end
            return "ok"
            "#,
        )
        .expect("state-backed namespace probe should run");

    assert_eq!(result, "ok");
}

#[test]
fn economy_inventory_queries_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for symbol in [
        "GetMoney",
        "GetAverageItemLevel",
        "GetCursorMoney",
        "GetInventoryItemLink",
        "GetInventoryItemsForSlot",
    ] {
        assert!(
            !bootstrap.contains(&format!("function {symbol}")),
            "{symbol} must be registered by the Rust state/inventory API, not runtime bootstrap"
        );
    }
}

#[test]
fn channel_queries_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    assert!(
        !bootstrap.contains("function GetChannelList"),
        "GetChannelList must be registered by the Rust channel API, not runtime bootstrap"
    );
}

#[test]
fn achievement_category_queries_are_not_runtime_bootstrap_fallbacks() {
    let bootstrap = include_str!("../src/lua_api/env_init/runtime_surface_bootstrap.lua");

    for symbol in [
        "GetCategoryList",
        "GetGuildCategoryList",
        "GetStatisticsCategoryList",
    ] {
        assert!(
            !bootstrap.contains(&format!("function {symbol}")),
            "{symbol} must be registered by the Rust achievement API, not runtime bootstrap"
        );
    }
}
