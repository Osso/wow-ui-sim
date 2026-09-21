//! Profile isolation and the TOC rules used by the Forever source tree.

use std::path::{Path, PathBuf};
use wow_ui_sim::toc::TocFile;

#[cfg(feature = "client-wowforever")]
fn assert_ellesmere_specialization_guards(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        local guards = {
            reminders = function()
                if not GetSpecialization then return nil end
                local s = GetSpecialization(); if not s then return nil end
                return GetSpecializationInfo(s)
            end,
            nameplates = function()
                local specIndex = GetSpecialization and GetSpecialization() or 0
                return specIndex and specIndex > 0
                    and GetSpecializationInfo(specIndex) or nil
            end,
            profiles = function()
                local specIdx = GetSpecialization and GetSpecialization() or 0
                return specIdx and specIdx > 0
                    and GetSpecializationInfo(specIdx) or nil
            end,
        }
        for name, guard in pairs(guards) do
            local ok, value = pcall(guard)
            assert(ok, name .. ": " .. tostring(value))
            assert(value == nil, name .. " must skip legacy specialization")
        end
        assert(rawget(_G, "GetSpecialization") == nil)
        assert(rawget(_G, "GetSpecializationInfo") == nil)
        assert(GetSpecialization == nil and GetSpecializationInfo == nil)
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_specialization_visibility_preserves_ellesmere_guards() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    assert_ellesmere_specialization_guards(&env);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_specialization_visibility_keeps_namespace_state() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.class_index = 2;
        state.player.active_spec_index = 2;
    }
    env.exec(
        r#"
        assert(C_SpecializationInfo.GetSpecialization() == 2)
        local id, name, _, _, role = C_SpecializationInfo.GetSpecializationInfo(2)
        assert(id == 66 and name == "Protection" and role == "TANK")
        "#,
    )
    .unwrap();
    assert_ellesmere_specialization_guards(&env);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_specialization_visibility_survives_bootstrap_replay() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.class_index = 2;
        state.player.active_spec_index = 1;
    }
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let toc = TocFile::from_file(
        &ui.join("Blizzard_DeprecatedSpecialization/Blizzard_DeprecatedSpecialization.toc"),
    )
    .unwrap();
    assert!(toc.is_game_type_restricted());
    env.exec(r#"assert(GetCVarBool("loadDeprecationFallbacks"))"#)
        .unwrap();
    let query = r#"
        local index = C_SpecializationInfo.GetSpecialization()
        local id, name, _, _, role = C_SpecializationInfo.GetSpecializationInfo(index)
        return index, id, name, role
    "#;
    let expected = (1, 65, "Holy".to_owned(), "HEALER".to_owned());
    assert_eq!(
        env.eval::<(i32, i32, String, String)>(query).unwrap(),
        expected
    );
    for _ in 0..2 {
        env.loader_env().restore_post_cleanup_globals().unwrap();
        assert_ellesmere_specialization_guards(&env);
        assert_eq!(
            env.eval::<(i32, i32, String, String)>(query).unwrap(),
            expected
        );
    }
}

#[test]
#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
fn wowforever_profile_exclusion_annotation_filters_files_and_dependencies() {
    let toc = TocFile::parse(
        Path::new("/addons/Example"),
        "## Dep: Common\n## Dep: OtherGame [ExcludeLoadGameType mainline]\nCommon.lua\nOther.lua [ExcludeLoadGameType mainline]\n",
    );

    assert_eq!(toc.dependencies(), vec!["Common"]);
    assert_eq!(toc.files, vec![PathBuf::from("Common.lua")]);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_identity_and_manifest_are_distinct() {
    use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};

    assert_eq!(ACTIVE, ClientProfile::WowForever);
    assert_eq!(ACTIVE.subdir(), "WowForever");
    assert_eq!(ACTIVE.cache_subdir(), "wowforever");
    assert_eq!(ACTIVE.interface_version(), 16001);
    assert_eq!(ACTIVE_INTERFACE_VERSION, 16001);
    assert!(!cfg!(feature = "retail-12-0-0"));
    assert!(!cfg!(feature = "client-anniversary"));
    let entries: Vec<_> = wow_ui_sim::blizzard_ui_sync::manifest_entries().collect();
    assert_eq!(entries.len(), 4398);
    assert!(entries.contains(&"Blizzard_FrameXML/Camelot/StackSplitFrame.xml"));
    assert!(entries.contains(&"Blizzard_FrameXMLBase/Blizzard_FrameXMLBase.toc"));
    assert!(
        wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .ends_with("blizzard-ui/wowforever/AddOns")
    );
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_reports_build_identity_and_finite_event_validation() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let identity: (String, String, i32) = env
        .eval(
            "local version, build, _, interface = GetBuildInfo(); return version, build, interface",
        )
        .unwrap();
    assert_eq!(identity, ("1.60.1".to_owned(), "69913".to_owned(), 16001));
    assert!(wow_ui_sim::event::is_registerable_event("PLAYER_LOGIN"));
    for event in [
        "CONFIRM_BATTLE_NET_FRIEND_INVITE_SHOW",
        "DIEL_CYCLE_CHANGED",
        "DISCORD_LINK_UPDATE",
        "EXTERNAL_EVENT_LAUNCH_URL_FAILED",
        "GROUP_BUFF_VISUAL_ALERTS_CHANGED",
        "GUILD_RANKS_UPDATE_ACTIVE_PLAYER",
        "INPUT_DEVICE_INTERFACE_TRANSITION",
        "LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY",
        "SOCIAL_UI_SYSTEM_STATUS_UPDATED",
        "UNIT_HAPPINESS",
        "UNIT_PET_TRAINING_POINTS",
        "UNIT_PING_PIN_ADDED",
        "UNIT_PING_PIN_REMOVED",
        "DISCORD_STATUS_UPDATE",
    ] {
        assert!(wow_ui_sim::event::is_registerable_event(event), "{event}");
    }
    assert!(!wow_ui_sim::event::is_registerable_event(
        "WOWFOREVER_INVENTED_EVENT"
    ));
    assert!(!wow_ui_sim::event::is_registerable_event(""));
    let retail_accessor: String = env
        .eval("return type(GetSecurePendingButtonCallback)")
        .unwrap();
    assert_eq!(retail_accessor, "nil");
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_loads_camelot_mainline_toc_entries() {
    // Forever 1.60.1.69913 Blizzard_FrameXML and FrameXMLBase TOC excerpts.
    let toc = TocFile::parse(
        Path::new("/addons/Blizzard_FrameXML"),
        "## Dep: Blizzard_UnitPopup [AllowLoadGameType classic]\n\
         ## Dep: Blizzard_UIParentPanelManager [AllowLoadGameType mainline]\n\
         [Family]\\StackSplitFrame.lua\n\
         [Family]\\StackSplitFrame.xml [ExcludeLoadGameType camelot]\n\
         [Game]\\StackSplitFrame.xml\t[AllowLoadGameType camelot]\n\
         Vanilla\\Constants.lua [AllowLoadGameType vanilla]\n",
    );
    assert_eq!(toc.dependencies(), vec!["Blizzard_UIParentPanelManager"]);
    assert_eq!(
        toc.files,
        vec![
            PathBuf::from("Mainline/StackSplitFrame.lua"),
            PathBuf::from("Camelot/StackSplitFrame.xml"),
        ]
    );
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_applies_header_filters_without_vanilla_alias() {
    for tag in ["camelot", "mainline"] {
        let toc = TocFile::parse(
            Path::new("/addons/Test"),
            &format!("## AllowLoadGameType: {tag}\n"),
        );
        assert!(!toc.is_game_type_restricted(), "{tag}");
    }
    for header in [
        "## AllowLoadGameType: vanilla",
        "## AllowLoadGameType: classic",
        "## AllowLoadGameType: standard",
        "## ExcludeLoadGameType: camelot",
        "## ExcludeLoadGameType: mainline",
    ] {
        assert!(
            TocFile::parse(Path::new("/addons/Test"), header).is_game_type_restricted(),
            "{header}"
        );
    }
    assert!(!TocFile::parse(Path::new("/addons/Test"), "## OnlyBetaAndPTR: 1").is_ptr_only());
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_selects_camelot_then_generic_then_mainline_toc() {
    let root = tempfile::tempdir().unwrap();
    let addon = root.path().join("Example");
    std::fs::create_dir(&addon).unwrap();
    for suffix in [
        "_Vanilla",
        "_Mainline",
        "_Classic",
        "_Mists",
        "_Wrath",
        "_TBC",
        "_Cata",
        "",
    ] {
        std::fs::write(addon.join(format!("Example{suffix}.toc")), "Core.lua\n").unwrap();
    }
    assert_eq!(
        wow_ui_sim::loader::find_toc_file(&addon),
        Some(addon.join("Example.toc"))
    );
    std::fs::write(addon.join("Example_Camelot.toc"), "Core.lua\n").unwrap();
    assert_eq!(
        wow_ui_sim::loader::find_toc_file(&addon),
        Some(addon.join("Example_Camelot.toc"))
    );
    std::fs::remove_file(addon.join("Example_Camelot.toc")).unwrap();
    std::fs::remove_file(addon.join("Example.toc")).unwrap();
    assert_eq!(
        wow_ui_sim::loader::find_toc_file(&addon),
        Some(addon.join("Example_Mainline.toc"))
    );
    std::fs::remove_file(addon.join("Example_Mainline.toc")).unwrap();
    std::fs::write(addon.join("Example-Mainline.toc"), "Core.lua\n").unwrap();
    assert_eq!(
        wow_ui_sim::loader::find_toc_file(&addon),
        Some(addon.join("Example-Mainline.toc"))
    );
    std::fs::remove_file(addon.join("Example-Mainline.toc")).unwrap();
    assert_eq!(wow_ui_sim::loader::find_toc_file(&addon), None);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_loads_carbonite_dash_provider_before_dependents() {
    use wow_ui_sim::{loader, lua_api::WowLuaEnv, screen::ScreenKind};
    let root = tempfile::tempdir().unwrap();
    let provider = root.path().join("Carbonite");
    std::fs::create_dir(&provider).unwrap();
    // Cached Carbonite package shape: incompatible Retail generic beside Camelot dash TOC.
    std::fs::write(
        provider.join("Carbonite.toc"),
        "## Interface: 120007, 120100\nRetail.lua\n",
    )
    .unwrap();
    std::fs::write(
        provider.join("Carbonite-Camelot.toc"),
        "## Interface: 16001\n## X-Camelot-Toc: dash\n## LoadOnDemand: 0\nCamelot.lua\n",
    )
    .unwrap();
    std::fs::write(
        provider.join("Retail.lua"),
        "error('wrong Retail provider selected')",
    )
    .unwrap();
    std::fs::write(provider.join("Camelot.lua"), "CarboniteFlavor = 'camelot'").unwrap();
    let consumers = ["Carbonite.Info", "Carbonite.Notes", "Carbonite.Warehouse"];
    for name in consumers {
        let dir = root.path().join(name);
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{name}.toc")),
            "## Interface: 16001\n## Dependencies: Carbonite\nCore.lua\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("Core.lua"),
            "assert(CarboniteFlavor == 'camelot'); CarboniteConsumers = (CarboniteConsumers or 0) + 1",
        ).unwrap();
    }
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![root.path().to_path_buf()];
    let discovered = loader::discover_blizzard_addon_closure_for_screen(
        root.path(),
        ScreenKind::Game,
        &consumers,
    );
    for (_, path) in discovered {
        let toc = TocFile::from_file(&path).unwrap();
        // The real third-party startup loader applies this interface gate before loading.
        if toc.supports_interface_version(wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION) {
            let result = loader::load_addon(&env.loader_env(), &path).unwrap();
            assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        }
    }
    assert_eq!(env.eval::<i32>("return CarboniteConsumers").unwrap(), 3);
    assert_eq!(
        env.eval::<String>("return CarboniteFlavor").unwrap(),
        "camelot"
    );
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_loads_pinned_world_map_filename() {
    let root = tempfile::tempdir().unwrap();
    let addon = root.path().join("Blizzard_WorldMap");
    std::fs::create_dir(&addon).unwrap();
    let toc_path = addon.join("Blizzard_WorldMap_Mainline.toc");
    std::fs::write(
        &toc_path,
        "[Game]/Blizzard_WorldMapConstants.lua [AllowLoadGameType camelot]\n",
    )
    .unwrap();
    assert_eq!(
        wow_ui_sim::loader::find_toc_file(&addon),
        Some(toc_path.clone())
    );
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert_eq!(
        toc.files,
        vec![PathBuf::from("Camelot/Blizzard_WorldMapConstants.lua")]
    );
}

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_profile_discovers_mainline_required_provider_before_consumer() {
    use wow_ui_sim::{loader, lua_api::WowLuaEnv, screen::ScreenKind};
    let root = tempfile::tempdir().unwrap();
    for (name, metadata, source) in [
        (
            "Blizzard_SharedMapDataProviders",
            "## LoadOnDemand: 1\n",
            "MapExplorationDataProviderMixin = { OnAdded = function(self, map) self.map = map end }",
        ),
        (
            "Blizzard_WorldMap",
            "## RequiredDep: Blizzard_SharedMapDataProviders\n",
            "local provider = CreateFromMixins(MapExplorationDataProviderMixin); provider:OnAdded(37); ProviderMap = provider.map",
        ),
    ] {
        let dir = root.path().join(name);
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{name}_Mainline.toc")),
            format!("## AllowLoad: game\n## AllowLoadGameType: mainline\n{metadata}Core.lua\n"),
        )
        .unwrap();
        std::fs::write(dir.join("Core.lua"), source).unwrap();
    }
    let addons = loader::discover_blizzard_addon_closure_for_screen(
        root.path(),
        ScreenKind::Game,
        &["Blizzard_WorldMap"],
    );
    assert_eq!(
        addons
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["Blizzard_SharedMapDataProviders", "Blizzard_WorldMap"]
    );
    let env = WowLuaEnv::new().unwrap();
    for (_, toc) in addons {
        let result = loader::load_addon(&env.loader_env(), &toc).unwrap();
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    }
    assert_eq!(env.eval::<i32>("return ProviderMap").unwrap(), 37);
}
