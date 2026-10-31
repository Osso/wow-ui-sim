use super::*;
use clap::Parser;
use wow_ui_sim::config::SimConfig;
use wow_ui_sim::lua_api::state::RACE_DATA;

fn player_class_and_race(env: &WowLuaEnv) -> (String, String) {
    env.eval(
        "local _, class = UnitClass('player'); \
         local _, race = UnitRace('player'); return class, race",
    )
    .unwrap()
}

fn mage_orc_config() -> SimConfig {
    let mut config = SimConfig::default();
    config.player_class = "Mage".to_string();
    config.player_race = "Orc".to_string();
    config
}

fn load_initial_player_probe(env: &WowLuaEnv) -> (String, String) {
    let fixture = tempfile::tempdir().unwrap();
    let addon = fixture.path().join("InitialPlayerProbe");
    std::fs::create_dir(&addon).unwrap();
    std::fs::write(
        addon.join("InitialPlayerProbe.toc"),
        format!(
            "## Interface: {}\nCore.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION,
        ),
    )
    .unwrap();
    std::fs::write(
        addon.join("Core.lua"),
        "local _, class = UnitClass('player'); local _, race = UnitRace('player'); \
         INITIAL_PLAYER_CLASS = class; INITIAL_PLAYER_RACE = race; \
         if C_SpecializationInfo and C_SpecializationInfo.GetSpecialization then \
             INITIAL_PLAYER_SPEC = C_SpecializationInfo.GetSpecialization() \
         end",
    )
    .unwrap();
    let loaded =
        wow_ui_sim::loader::load_addon(&env.loader_env(), &addon.join("InitialPlayerProbe.toc"))
            .unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.lua_files, 1);
    env.eval("return INITIAL_PLAYER_CLASS, INITIAL_PLAYER_RACE")
        .unwrap()
}

fn observe_selection_events(env: &WowLuaEnv) {
    env.exec(
        "selection_events = {}; world_events = 0; local frame = CreateFrame('Frame'); \
         frame:RegisterEvent('PLAYER_SPECIALIZATION_CHANGED'); \
         frame:RegisterEvent('UNIT_PORTRAIT_UPDATE'); \
         frame:RegisterEvent('PLAYER_ENTERING_WORLD'); \
         frame:SetScript('OnEvent', function(_, event, unit) \
             if event == 'PLAYER_ENTERING_WORLD' then \
                 world_events = world_events + 1 \
             else \
                 table.insert(selection_events, event .. ':' .. tostring(unit) .. ':' .. \
                     select(2, UnitClass('player')) .. ':' .. select(2, UnitRace('player'))) \
             end \
         end)",
    )
    .unwrap();
}

#[test]
fn saved_and_default_class_and_race_reach_addon_initialization() {
    for (config, expected) in [
        (SimConfig::default(), ("PALADIN".into(), "Human".into())),
        (mage_orc_config(), ("MAGE".into(), "Orc".into())),
    ] {
        let env = WowLuaEnv::new().unwrap();
        apply_initial_player_settings(&env, &config, None).unwrap();
        assert_eq!(load_initial_player_probe(&env), expected);
    }
}

#[test]
fn cli_class_and_race_seed_addons_and_late_hook_preserves_final_selection() {
    let mut env = WowLuaEnv::new().unwrap();
    let args = Args::try_parse_from([
        "wow-sim",
        "--class",
        "Druid",
        "--race",
        "NightElf",
        "dump-tree",
    ])
    .unwrap();
    let selection = crate::class_spec::parse_selection(
        args.class.as_deref(),
        args.spec.as_deref(),
        args.race.as_deref(),
    )
    .unwrap()
    .unwrap();
    apply_initial_player_settings(&env, &mage_orc_config(), Some(&selection)).unwrap();
    assert_eq!(
        load_initial_player_probe(&env),
        ("DRUID".into(), "NightElf".into())
    );
    observe_selection_events(&env);
    env.exec(
        "local frame = CreateFrame('Frame'); frame:RegisterEvent('PLAYER_ENTERING_WORLD'); \
         frame:SetScript('OnEvent', function() \
             A_Admin.SetPlayerClass(8); A_Admin.SetPlayerRace(1) \
         end)",
    )
    .unwrap();
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    assert_eq!(player_class_and_race(&env), ("MAGE".into(), "Orc".into()));
    crate::class_spec::schedule_selection(&env, selection);
    assert_eq!(env.eval::<i32>("return #selection_events").unwrap(), 0);

    env.exec_maybe_secure("", false).unwrap();
    assert_eq!(
        player_class_and_race(&env),
        ("DRUID".into(), "NightElf".into())
    );
    assert_eq!(
        env.eval::<String>("return table.concat(selection_events, ',')")
            .unwrap(),
        "PLAYER_SPECIALIZATION_CHANGED:player:DRUID:NightElf,UNIT_PORTRAIT_UPDATE:player:DRUID:NightElf"
    );
    env.exec_maybe_secure("", false).unwrap();
    assert_eq!(env.eval::<i32>("return #selection_events").unwrap(), 2);
    assert_eq!(env.eval::<i32>("return world_events").unwrap(), 1);

    let troll = RACE_DATA
        .iter()
        .position(|(name, _, _)| *name == "Troll")
        .unwrap();
    env.exec(&format!(
        "A_Admin.SetPlayerClass(4); A_Admin.SetPlayerRace({troll})"
    ))
    .unwrap();
    let observed = Rc::new(RefCell::new(None));
    let on_reload = Rc::clone(&observed);
    env.set_reload_loader(move |env, _| {
        *on_reload.borrow_mut() = Some(player_class_and_race(env));
        Ok(())
    });
    env.exec("ReloadUI()").unwrap();
    assert!(env.service_reload(&mut None).unwrap());
    let expected = ("ROGUE".into(), "Troll".into());
    assert_eq!(*observed.borrow(), Some(expected.clone()));
    assert_eq!(player_class_and_race(&env), expected);
}

#[test]
fn invalid_cli_class_or_race_does_not_change_player_state() {
    let env = WowLuaEnv::new().unwrap();
    let original = player_class_and_race(&env);
    for (class, race) in [(Some("Unknown Class"), None), (None, Some("NotARace"))] {
        assert!(crate::class_spec::parse_selection(class, None, race).is_err());
        assert_eq!(player_class_and_race(&env), original);
    }
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-wowforever"
))]
mod modern_specs {
    use super::*;

    #[test]
    fn cli_class_race_and_spec_are_visible_during_addon_initialization() {
        let env = WowLuaEnv::new().unwrap();
        observe_selection_events(&env);
        let args = Args::try_parse_from([
            "wow-sim",
            "--class",
            "Mage",
            "--spec",
            "Frost",
            "--race",
            "Night Elf",
            "dump-tree",
        ])
        .unwrap();
        let selection = crate::class_spec::parse_selection(
            args.class.as_deref(),
            args.spec.as_deref(),
            args.race.as_deref(),
        )
        .unwrap()
        .unwrap();
        apply_initial_player_settings(&env, &SimConfig::default(), Some(&selection)).unwrap();
        assert_eq!(
            load_initial_player_probe(&env),
            ("MAGE".into(), "NightElf".into())
        );
        assert_eq!(env.eval::<i32>("return INITIAL_PLAYER_SPEC").unwrap(), 3);
        assert_eq!(env.eval::<i32>("return #selection_events").unwrap(), 0);
    }

    #[test]
    fn spec_only_uses_initial_class_then_resolves_again_after_startup() {
        let env = WowLuaEnv::new().unwrap();
        let selection = crate::class_spec::parse_selection(None, Some("Holy"), None)
            .unwrap()
            .unwrap();
        apply_initial_player_settings(&env, &SimConfig::default(), Some(&selection)).unwrap();
        assert_eq!(
            load_initial_player_probe(&env),
            ("PALADIN".into(), "Human".into())
        );
        assert_eq!(env.eval::<i32>("return INITIAL_PLAYER_SPEC").unwrap(), 1);
        observe_selection_events(&env);
        env.exec(
            "local frame = CreateFrame('Frame'); frame:RegisterEvent('PLAYER_ENTERING_WORLD'); \
             frame:SetScript('OnEvent', function() A_Admin.SetPlayerClass(5) end)",
        )
        .unwrap();
        env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
        crate::class_spec::schedule_selection(&env, selection);
        env.exec_maybe_secure("", false).unwrap();
        assert_eq!(player_class_and_race(&env).0, "PRIEST");
        assert_eq!(
            env.eval::<i32>("return C_SpecializationInfo.GetSpecialization()")
                .unwrap(),
            2
        );
        assert_eq!(
            env.eval::<String>("return table.concat(selection_events, ',')")
                .unwrap(),
            "PLAYER_SPECIALIZATION_CHANGED:player:PRIEST:Human"
        );
        assert_eq!(env.eval::<i32>("return world_events").unwrap(), 1);
    }

    #[test]
    fn invalid_initial_spec_rejects_all_settings_before_mutation() {
        let env = WowLuaEnv::new().unwrap();
        let original = player_class_and_race(&env);
        let original_spec = env.state().borrow().player.active_spec_index;
        let selection = crate::class_spec::parse_selection(None, Some("Holy"), Some("NightElf"))
            .unwrap()
            .unwrap();
        let error =
            apply_initial_player_settings(&env, &mage_orc_config(), Some(&selection)).unwrap_err();
        assert!(error.to_string().contains("valid specializations: Arcane"));
        assert_eq!(player_class_and_race(&env), original);
        assert_eq!(env.state().borrow().player.active_spec_index, original_spec);
    }
}
