use super::*;
use wow_ui_sim::client_profile::{self, ClientProfile};

fn load_profile_framexml() -> (WowLuaEnv, Vec<String>) {
    if client_profile::ACTIVE != ClientProfile::Wrath {
        return load_single_blizzard_addon("Blizzard_FrameXML");
    }

    // Wrath's game foundations live in the flat FrameXML TOC, not a split addon.
    let toc_path = client_profile::blizzard_ui_framexml_toc()
        .expect("Wrath's manifest-backed FrameXML.toc should exist");
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Login);
    let warnings = load_blizzard_addon_warnings(&env, "FrameXML", &toc_path);
    (env, warnings)
}

#[test]
fn test_secure_action_button_template_is_protected_tuple_after_blizzard_framexml_load() {
    test_timeout! {
        let (env, warnings) = load_profile_framexml();
        let (is_protected, is_protected_explicitly): (bool, bool) = env
            .eval(
                r#"
                local button = CreateFrame(
                    "Button",
                    "SecureActionButtonTemplateIsProtectedProbe",
                    UIParent,
                    "SecureActionButtonTemplate"
                )
                return button:IsProtected()
                "#,
            )
            .expect("SecureActionButtonTemplate IsProtected probe should be callable");

        assert!(is_protected, "warnings:\n  {}", warnings.join("\n  "));
        assert!(is_protected_explicitly);
    }
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn test_c_addons_load_addon_preserves_account_store_mixin_methods() {
    test_timeout! {
        let env = load_all_addons();

        let (
            ok,
            loaded,
            frame_ty,
            mixin_ty,
            mixin_set_storefront_ty,
            on_load_ty,
            set_storefront_ty,
            set_fullscreen_ty,
            err,
        ): (
            bool,
            bool,
            String,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
        ) = env
            .eval(
                r#"
                local ok, err = pcall(function()
                    C_AddOns.LoadAddOn("Blizzard_AccountStore")
                end)
                return ok,
                    C_AddOns.IsAddOnLoaded("Blizzard_AccountStore"),
                    type(AccountStoreFrame),
                    type(AccountStoreMixin),
                    type(AccountStoreMixin and AccountStoreMixin.SetStoreFrontID),
                    type(AccountStoreFrame and AccountStoreFrame.OnLoad),
                    type(AccountStoreFrame and AccountStoreFrame.SetStoreFrontID),
                    type(AccountStoreFrame and AccountStoreFrame.SetFullscreenMode),
                    ok and nil or tostring(err)
                "#,
            )
            .expect("C_AddOns.LoadAddOn inspection should be callable");

        assert!(ok, "C_AddOns.LoadAddOn should not error: {:?}", err);
        assert!(loaded, "Blizzard_AccountStore should be marked loaded");
        assert_eq!(
            frame_ty, "table",
            "AccountStoreFrame should exist after runtime load"
        );
        assert_eq!(
            mixin_ty, "table",
            "AccountStoreMixin should exist after runtime load"
        );
        assert_eq!(
            mixin_set_storefront_ty, "function",
            "AccountStoreMixin.SetStoreFrontID should exist after runtime load"
        );
        assert_eq!(
            on_load_ty, "function",
            "AccountStoreFrame.OnLoad should exist after runtime load"
        );
        assert_eq!(
            set_storefront_ty, "function",
            "AccountStoreFrame.SetStoreFrontID should exist after runtime load"
        );
        assert_eq!(
            set_fullscreen_ty, "function",
            "AccountStoreFrame.SetFullscreenMode should exist after runtime load"
        );
    }
}

// Native 1.60.1.70707 activates {mainline, camelot}: `standard` is recognised but inactive and
// `wowlabs` is an unknown token, so `## AllowLoadGameType: standard, wowlabs` fails closed and
// Blizzard_AccountStore must not load (evidence: native-evidence-70707/TOC_AllowLoadGameType.md).
#[cfg(feature = "client-wowforever")]
#[test]
fn test_wowforever_game_type_restriction_rejects_account_store_load_addon() {
    test_timeout! {
        let env = load_all_addons();

        let account_store_toc =
            blizzard_ui_dir().join("Blizzard_AccountStore/Blizzard_AccountStore.toc");
        let rejected_peer_toc =
            blizzard_ui_dir().join("Blizzard_HousingMarketCart/Blizzard_HousingMarketCart.toc");
        assert!(
            TocFile::from_file(&account_store_toc)
                .expect("Blizzard_AccountStore TOC should parse")
                .is_game_type_restricted(),
            "`standard` is not active on Forever, so the AccountStore game-type gate rejects it"
        );
        assert!(
            TocFile::from_file(&rejected_peer_toc)
                .expect("Blizzard_HousingMarketCart TOC should parse")
                .is_game_type_restricted(),
            "Blizzard_HousingMarketCart is the reference standard-only addon for this profile"
        );

        let (
            ok,
            load_ok,
            load_reason,
            loaded_or_loading,
            loaded,
            peer_load_ok,
            peer_load_reason,
            frame_ty,
            mixin_ty,
            err,
        ): (
            bool,
            bool,
            Option<String>,
            bool,
            bool,
            bool,
            Option<String>,
            String,
            String,
            Option<String>,
        ) = env
            .eval(
                r#"
                local ok, err = pcall(function()
                    C_AddOns.LoadAddOn("Blizzard_AccountStore")
                end)
                local load_ok, load_reason = C_AddOns.LoadAddOn("Blizzard_AccountStore")
                local loaded_or_loading, loaded = C_AddOns.IsAddOnLoaded("Blizzard_AccountStore")
                local peer_load_ok, peer_load_reason = C_AddOns.LoadAddOn("Blizzard_HousingMarketCart")
                return ok,
                    load_ok,
                    load_reason,
                    loaded_or_loading,
                    loaded,
                    peer_load_ok,
                    peer_load_reason,
                    type(AccountStoreFrame),
                    type(AccountStoreMixin),
                    ok and nil or tostring(err)
                "#,
            )
            .expect("C_AddOns.LoadAddOn inspection should be callable");

        assert!(ok, "C_AddOns.LoadAddOn should not error: {:?}", err);
        assert!(!load_ok, "Blizzard_AccountStore must not load on Forever");
        assert!(
            !loaded_or_loading && !loaded,
            "C_AddOns.IsAddOnLoaded must report Blizzard_AccountStore as not loaded"
        );
        assert_eq!(frame_ty, "nil", "AccountStoreFrame must not exist on Forever");
        assert_eq!(mixin_ty, "nil", "AccountStoreMixin must not exist on Forever");
        assert!(!peer_load_ok, "the standard-only peer addon must not load either");
        assert_eq!(
            load_reason, peer_load_reason,
            "a game-type-rejected addon must report the same load reason as its peer"
        );
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn test_rust_load_addon_after_base_load_preserves_account_store_mixin_methods() {
    test_timeout! {
        let env = load_all_addons();

        let ui = blizzard_ui_dir();
        let addons = discover_all_blizzard_addons(&ui);
        let (_, toc_path) = addons
            .into_iter()
            .find(|(name, _)| name == "Blizzard_AccountStore")
            .expect("Blizzard_AccountStore should exist");
        let _result = load_addon(&env.loader_env(), &toc_path).expect("late Rust load should succeed");

        let (
            mixin_fn_ty,
            scratch_on_load_ty,
            frame_ty,
            mixin_ty,
            on_load_ty,
            set_storefront_ty,
            _get_object_type_ty,
            _set_point_ty,
        ): (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ) =
            env.eval(
                r#"
                local scratch = {}
                if type(Mixin) == "function" and type(AccountStoreMixin) == "table" then
                    Mixin(scratch, AccountStoreMixin)
                end
                return type(Mixin),
                    type(scratch.OnLoad),
                    type(AccountStoreFrame),
                    type(AccountStoreMixin),
                    type(AccountStoreFrame and AccountStoreFrame.OnLoad),
                    type(AccountStoreFrame and AccountStoreFrame.SetStoreFrontID),
                    type(AccountStoreFrame and AccountStoreFrame.GetObjectType),
                    type(AccountStoreFrame and AccountStoreFrame.SetPoint)
                "#,
            )
            .expect("late Rust load inspection should be callable");
        assert_eq!(mixin_fn_ty, "function");
        assert_eq!(scratch_on_load_ty, "function");
        assert_eq!(frame_ty, "table");
        assert_eq!(mixin_ty, "table");
        assert_eq!(on_load_ty, "function");
        assert_eq!(set_storefront_ty, "function");
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn test_low_health_frame_animation_bound_after_load() {
    test_timeout! {
        let env = load_all_addons();

        let (frame_ty, group_ty, alpha_ty): (String, String, String) = env
            .eval(
                r#"
                return type(LowHealthFrame),
                    type(LowHealthFrame and LowHealthFrame.pulseAnim),
                    type(LowHealthFrame and LowHealthFrame.pulseAnim and LowHealthFrame.pulseAnim.AlphaAnim)
                "#,
            )
            .expect("LowHealthFrame inspection should be callable");

        assert_eq!(frame_ty, "table", "LowHealthFrame should exist after addon load");
        assert_eq!(group_ty, "table", "LowHealthFrame.pulseAnim should exist after addon load");
        assert_eq!(
            alpha_ty, "table",
            "LowHealthFrame.pulseAnim.AlphaAnim should exist after addon load"
        );
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn test_low_health_frame_animation_bound_after_blizzard_framexml_load() {
    test_timeout! {
        let (env, warnings) = load_single_blizzard_addon("Blizzard_FrameXML");

        let (frame_ty, group_ty, alpha_ty): (String, String, String) = env
            .eval(
                r#"
                return type(LowHealthFrame),
                    type(LowHealthFrame and LowHealthFrame.pulseAnim),
                    type(LowHealthFrame and LowHealthFrame.pulseAnim and LowHealthFrame.pulseAnim.AlphaAnim)
                "#,
            )
            .expect("LowHealthFrame inspection should be callable");

        assert_eq!(frame_ty, "table", "LowHealthFrame should exist after Blizzard_FrameXML load");
        assert_eq!(
            group_ty, "table",
            "LowHealthFrame.pulseAnim should exist after Blizzard_FrameXML load; warnings:\n  {}",
            warnings.join("\n  ")
        );
        assert_eq!(
            alpha_ty, "table",
            "LowHealthFrame.pulseAnim.AlphaAnim should exist after Blizzard_FrameXML load"
        );
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn test_blizzard_framexml_load_registers_boss_banner_cvar_without_warning() {
    test_timeout! {
        let (env, warnings) = load_single_blizzard_addon("Blizzard_FrameXML");

        let register_cvar_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("RegisterCVar")
                    || warning.contains("CvarUtil.lua:2")
                    || warning.contains("PraiseTheSun")
            })
            .cloned()
            .collect();

        assert!(
            register_cvar_warnings.is_empty(),
            "Blizzard_FrameXML should not warn on BossBanner RegisterCVar:\n  {}",
            register_cvar_warnings.join("\n  ")
        );

        let (value, default_value): (String, String) = env
            .eval(r#"return GetCVar("PraiseTheSun"), GetCVarDefault("PraiseTheSun")"#)
            .expect("BossBanner cvar should be readable after Blizzard_FrameXML load");
        assert!(
            value == "0" || value == "1",
            "BossBanner cvar should be readable after registration, got {value:?}"
        );
        assert_eq!(default_value, "0");
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn test_blizzard_framexml_loads_role_poll_without_role_icon_warning() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);

        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);
        let mut warnings = Vec::new();

        for (name, toc_path) in &addons {
            let result = load_addon(&env.loader_env(), toc_path)
                .unwrap_or_else(|error| panic!("{name} should load: {error}"));
            for warning in result.warnings {
                warnings.push(format!("[load {name}] {warning}"));
            }
            if name == "Blizzard_FrameXML" {
                break;
            }
        }

        let role_icon_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("GetIconForRoleEnum")
                    || warning.contains("RolePollPopupRoleButton")
            })
            .cloned()
            .collect();

        assert!(
            role_icon_warnings.is_empty(),
            "Blizzard_FrameXML should not warn on RolePoll role icons:\n  {}",
            role_icon_warnings.join("\n  ")
        );

        let (tank_role, healer_role, damage_role): (i32, i32, i32) = env
            .eval(
                r#"
                return RolePollPopupRoleButtonTank.role or -1,
                       RolePollPopupRoleButtonHealer.role or -1,
                       RolePollPopupRoleButtonDPS.role or -1
                "#,
            )
            .expect("RolePoll role buttons should stay readable after FrameXML load");
        assert_eq!(tank_role, 0);
        assert_eq!(healer_role, 1);
        assert_eq!(damage_role, 2);
    }
}

#[test]
fn test_blizzard_framexml_loads_zone_text_without_fading_frame_warning() {
    test_timeout! {
        let (env, warnings) = load_profile_framexml();

        let fading_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("FadingFrame_OnLoad")
                    || warning.contains("FadingFrame_Show")
                    || warning.contains("ZoneText.lua:72")
                    || warning.contains("ZoneText.lua:124")
            })
            .cloned()
            .collect();

        assert!(
            fading_warnings.is_empty(),
            "Profile FrameXML should not warn on fading-frame helpers:\n  {}",
            fading_warnings.join("\n  ")
        );

        let (zone_hidden, subzone_hidden, fade_in, hold, fade_out): (bool, bool, f64, f64, f64) = env
            .eval(
                r#"
                return not ZoneTextFrame:IsShown(),
                       not SubZoneTextFrame:IsShown(),
                       ZoneTextFrame.fadeInTime,
                       ZoneTextFrame.holdTime,
                       ZoneTextFrame.fadeOutTime
                "#,
            )
            .expect("zone text fading-frame state should be readable");
        assert!(zone_hidden);
        assert!(subzone_hidden);
        assert_eq!(fade_in, 0.5);
        assert_eq!(hold, 1.0);
        assert_eq!(fade_out, 2.0);
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn test_blizzard_framexml_loads_without_eventutil_warning() {
    test_timeout! {
        let (_env, warnings) = load_single_blizzard_addon("Blizzard_FrameXML");

        let eventutil_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("EventUtil")
                    || warning.contains("MotionSickness.lua:23")
                    || warning.contains("AlertFrames.lua:281")
            })
            .cloned()
            .collect();

        assert!(
            eventutil_warnings.is_empty(),
            "Blizzard_FrameXML should not warn on EventUtil helpers:\n  {}",
            eventutil_warnings.join("\n  ")
        );
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn test_blizzard_framexml_loads_without_setup_localization_warning() {
    test_timeout! {
        let (_env, warnings) = load_single_blizzard_addon("Blizzard_FrameXML");

        let localization_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("SetupLocalization")
                    || warning.contains("Shared/Localization.lua:55")
                    || warning.contains("Mainline/Localization.lua:48")
            })
            .cloned()
            .collect();

        assert!(
            localization_warnings.is_empty(),
            "Blizzard_FrameXML should not warn on SetupLocalization:\n  {}",
            localization_warnings.join("\n  ")
        );
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn test_blizzard_framexml_loads_without_frameutil_warning() {
    test_timeout! {
        let (_env, warnings) = load_single_blizzard_addon("Blizzard_FrameXML");

        let frameutil_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("FrameUtil")
                    || warning.contains("UIErrorsFrame.lua:8")
                    || warning.contains("LootHistory.lua:307")
                    || warning.contains("QuestSession.lua:831")
            })
            .cloned()
            .collect();

        assert!(
            frameutil_warnings.is_empty(),
            "Blizzard_FrameXML should not warn on FrameUtil helpers:\n  {}",
            frameutil_warnings.join("\n  ")
        );
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn test_blizzard_commentator_loads_without_cooldown_frame_warning() {
    test_timeout! {
        let (_env, warnings) = load_blizzard_addon_by_folder("Blizzard_Commentator");

        let cooldown_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("CooldownFrame_Set")
                    || warning.contains("CooldownFrame_Clear")
                    || warning.contains("Blizzard_CommentatorSpell.lua:74")
                    || warning.contains("Blizzard_CommentatorSpell.lua:87")
                    || warning.contains("Blizzard_CommentatorSpell.lua:88")
                    || warning.contains("Blizzard_CommentatorSpell.lua:105")
            })
            .cloned()
            .collect();

        assert!(
            cooldown_warnings.is_empty(),
            "Blizzard_Commentator should not warn on cooldown-frame helpers:\n  {}",
            cooldown_warnings.join("\n  ")
        );
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn test_ui_parent_panel_manager_loads_with_minimap_cluster_stub() {
    test_timeout! {
        let (env, warnings) = load_single_blizzard_addon("Blizzard_UIParentPanelManager");

        let minimap_cluster_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("MinimapCluster")
                    || warning.contains("UIParentPanelManager.lua:784")
            })
            .cloned()
            .collect();

        assert!(
            minimap_cluster_warnings.is_empty(),
            "Blizzard_UIParentPanelManager should not warn on MinimapCluster:\n  {}",
            minimap_cluster_warnings.join("\n  ")
        );

        let (cluster_type, minimap_is_child, cluster_height): (String, bool, f64) = env
            .eval(
                r#"
                return type(MinimapCluster),
                       Minimap:GetParent() == MinimapCluster,
                       MinimapCluster:GetHeight()
                "#,
            )
            .expect("startup MinimapCluster stub should be queryable");

        assert_eq!(cluster_type, "table");
        assert!(minimap_is_child, "startup Minimap should hang off MinimapCluster");
        assert!(cluster_height > 0.0, "startup MinimapCluster should have a usable size");
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn test_housing_tutorials_load_without_cvar_bitfield_warning() {
    test_timeout! {
        let (env, warnings) = load_blizzard_addon_by_folder("Blizzard_HousingTutorials");

        let bitfield_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("GetCVarBitfield")
                    || warning.contains("CvarUtil.lua:30")
            })
            .cloned()
            .collect();

        assert!(
            bitfield_warnings.is_empty(),
            "Blizzard_HousingTutorials should not warn on CVar bitfield helpers:\n  {}",
            bitfield_warnings.join("\n  ")
        );

        let tutorial_seen: bool = env
            .eval(
                r#"
                return C_CVar.GetCVarBitfield(
                    "closedInfoFramesAccountWide",
                    Enum.FrameTutorialAccount.HousingItemAcquisition
                )
                "#,
            )
            .expect("housing tutorial bitfield read should be callable after addon load");
        let _ = tutorial_seen;
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn test_new_player_experience_loads_without_minimap_cluster_warning() {
    test_timeout! {
        let (env, warnings) = load_blizzard_addon_by_folder("Blizzard_NewPlayerExperience");

        let minimap_cluster_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("MinimapCluster")
                    || warning.contains("Blizzard_TutorialTutorials.lua:687")
                    || warning.contains("Blizzard_TutorialTutorials.lua:692")
                    || warning.contains("Blizzard_TutorialTutorials.lua:709")
            })
            .cloned()
            .collect();

        assert!(
            minimap_cluster_warnings.is_empty(),
            "Blizzard_NewPlayerExperience should not warn on MinimapCluster startup access:\n  {}",
            minimap_cluster_warnings.join("\n  ")
        );

        let (exists, parent_name): (bool, String) = env
            .eval(
                r#"
                return MinimapCluster ~= nil, MinimapCluster:GetParent():GetName()
                "#,
            )
            .expect("MinimapCluster should be available to startup addons");
        assert!(exists);
        assert_eq!(parent_name, "UIParent");
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn test_battlefield_map_startup_uses_maputil_displayable_map_helper() {
    test_timeout! {
        let (env, warnings) = load_blizzard_addon_by_folder("Blizzard_BattlefieldMap");

        let load_maputil_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("MapUtil")
                    || warning.contains("GetDisplayableMapForPlayer")
            })
            .cloned()
            .collect();

        assert!(
            load_maputil_warnings.is_empty(),
            "Blizzard_BattlefieldMap should not warn on MapUtil during load:\n  {}",
            load_maputil_warnings.join("\n  ")
        );

        install_test_error_handler(&env);
        env.exec(
            r#"
            RegisterCVar("showBattlefieldMinimap", "1")
            SetCVar("showBattlefieldMinimap", "1")
            "#,
        )
        .expect("battlefield map cvar should be writable");

        let mut startup_warnings = Vec::new();
        startup_warnings.extend(fire(
            &env,
            "ADDON_LOADED",
            &[env.lua_string("Blizzard_BattlefieldMap")],
        ));
        startup_warnings.extend(fire(
            &env,
            "PLAYER_ENTERING_WORLD",
            &[rilua::Val::Bool(true), rilua::Val::Bool(false)],
        ));

        let maputil_warnings: Vec<String> = startup_warnings
            .iter()
            .filter(|warning| {
                warning.contains("MapUtil")
                    || warning.contains("GetDisplayableMapForPlayer")
                    || warning.contains("Blizzard_BattlefieldMap.lua:154")
                    || warning.contains("Blizzard_BattlefieldMap.lua:189")
            })
            .cloned()
            .collect();

        assert!(
            maputil_warnings.is_empty(),
            "Blizzard_BattlefieldMap startup should not warn on MapUtil:\n  {}",
            maputil_warnings.join("\n  ")
        );

        let (maputil_type, helper_type, displayable_map_id): (String, String, i32) = env
            .eval(
                r#"
                return type(MapUtil),
                       type(MapUtil.GetDisplayableMapForPlayer),
                       MapUtil.GetDisplayableMapForPlayer()
                "#,
            )
            .expect("battlefield map startup should leave MapUtil displayable-map helpers callable");
        assert_eq!(maputil_type, "table");
        assert_eq!(helper_type, "function");
        assert!(displayable_map_id > 0);
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn test_world_map_loads_without_maputil_warning() {
    test_timeout! {
        let (env, warnings) = load_blizzard_addon_by_folder("Blizzard_WorldMap");

        let maputil_warnings: Vec<String> = warnings
            .iter()
            .filter(|warning| {
                warning.contains("MapUtil")
                    || warning.contains("Blizzard_WorldMap.lua")
                    || warning.contains("Blizzard_WorldMapTemplates.lua")
            })
            .cloned()
            .collect();

        assert!(
            maputil_warnings.is_empty(),
            "Blizzard_WorldMap should not warn on MapUtil startup access:\n  {}",
            maputil_warnings.join("\n  ")
        );

        let (has_displayable_map, has_parent_info): (bool, bool) = env
            .eval(
                r#"
                local mapID = MapUtil.GetDisplayableMapForPlayer()
                return type(mapID) == "number",
                       pcall(function() return MapUtil.GetMapParentInfo(1, Enum.UIMapType.Zone) end)
                "#,
            )
            .expect("MapUtil startup helpers should be available after world map load");
        assert!(has_displayable_map);
        assert!(has_parent_info);
    }
}
