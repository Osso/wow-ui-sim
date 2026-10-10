//! UnitHasPowerType capability queries; grouped in the integration target.

#[cfg(feature = "retail-12-0-5")]
mod retail {
    use wow_ui_sim::lua_api::WowLuaEnv;
    use wow_ui_sim::lua_api::state::{PartyMember, SecondaryPowerState, TargetInfo};

    fn env() -> WowLuaEnv {
        WowLuaEnv::new().expect("Lua environment")
    }

    #[test]
    fn unit_has_power_type_primary_does_not_require_resources() {
        let env = env();
        {
            let mut sim = env.state().borrow_mut();
            sim.player.power_type = 3;
            sim.player.power = 0;
            sim.player.power_max = 0;
            sim.player.secondary_powers.clear();
        }
        env.eval::<()>(
            r#"
            for _, unit in ipairs({"player", "self", "pet", "vehicle"}) do
                assert(UnitHasPowerType(unit, 3), unit .. " primary")
                assert(not UnitHasPowerType(unit, 0), unit .. " unrelated")
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn unit_has_power_type_secondary_requires_explicit_player_capability() {
        let env = env();
        {
            let mut sim = env.state().borrow_mut();
            sim.player.power_type = 0;
            sim.player.secondary_powers.clear();
            sim.player
                .secondary_powers
                .insert(9, SecondaryPowerState { current: 0, max: 5 });
            sim.player
                .secondary_powers
                .insert(42, SecondaryPowerState { current: 0, max: 0 });
        }
        env.eval::<()>(
            r#"
            for _, unit in ipairs({"player", "self"}) do
                assert(UnitHasPowerType(unit, 9), unit .. " empty holy power")
                assert(UnitHasPowerType(unit, 42), unit .. " explicitly modeled pool")
                assert(not UnitHasPowerType(unit, 4), unit .. " no invented combo points")
                assert(not UnitHasPowerType(unit, -1), unit .. " unmodeled integer")
                assert(not UnitHasPowerType(unit, 2147483647))
            end
            for _, unit in ipairs({"pet", "vehicle"}) do
                assert(not UnitHasPowerType(unit, 9), unit .. " no player secondary inheritance")
                assert(not UnitHasPowerType(unit, 42))
            end
            "#,
        )
        .unwrap();
        env.state().borrow_mut().player.secondary_powers.remove(&9);
        assert!(
            !env.eval::<bool>(r#"return UnitHasPowerType("player", 9)"#)
                .unwrap()
        );
    }

    #[test]
    fn unit_has_power_type_target_and_group_use_present_primary_only() {
        let env = env();
        {
            let mut sim = env.state().borrow_mut();
            sim.current_target = Some(TargetInfo {
                unit_id: "target".into(),
                name: "Empty Rage Target".into(),
                class_index: 1,
                level: 60,
                health: 100,
                health_max: 100,
                power: 0,
                power_max: 0,
                power_type: 1,
                power_type_name: "RAGE".into(),
                is_player: false,
                is_enemy: true,
                guid: "Creature-0-0-0-0-123-1".into(),
                classification: "normal".into(),
                creature_type: "Humanoid".into(),
                reaction: 1,
                interaction: Default::default(),
            });
            sim.party_group_active = true;
            sim.party_members = vec![PartyMember {
                map_position: None,
                name: "Empty Focus Member".into(),
                name_cached: true,
                connected: true,
                class_index: 3,
                level: 60,
                health: 100,
                health_max: 100,
                power: 0,
                power_max: 0,
                power_type: 2,
                power_type_name: "FOCUS".into(),
                is_leader: false,
                dead_since: None,
                buffs: vec![],
                debuffs: vec![],
            }];
            sim.player
                .secondary_powers
                .insert(9, SecondaryPowerState { current: 0, max: 5 });
        }
        env.eval::<()>(
            r#"
            assert(UnitHasPowerType("target", 1))
            assert(not UnitHasPowerType("target", 9))
            for _, unit in ipairs({"party1", "raid1"}) do
                assert(UnitHasPowerType(unit, 2), unit .. " primary")
                assert(not UnitHasPowerType(unit, 9), unit .. " no secondary")
            end
            "#,
        )
        .unwrap();
        {
            let mut sim = env.state().borrow_mut();
            sim.current_target = None;
            sim.party_group_active = false;
        }
        env.eval::<()>(
            r#"
            for _, unit in ipairs({"target", "focus", "party1", "raid1", "party2", "unknown", ""}) do
                for _, power in ipairs({0, 1, 2, 9}) do
                    assert(UnitHasPowerType(unit, power) == false, unit .. " absent")
                end
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn unit_has_power_type_returns_one_non_secret_boolean() {
        let env = env();
        env.state().borrow_mut().player.power_type = 0;
        env.eval::<()>(
            r#"
            for _, power in ipairs({0, 2147483647}) do
                local result = UnitHasPowerType("player", power)
                assert(select('#', UnitHasPowerType("player", power)) == 1)
                assert(type(result) == "boolean")
                assert(not issecretvalue(result))
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn unit_has_power_type_rejects_malformed_arguments() {
        env()
            .eval::<()>(
                r#"
            assert(type(UnitHasPowerType) == "function", "API must exist before argument probes")
            local function rejects(...)
                local ok, message = pcall(UnitHasPowerType, ...)
                assert(not ok, "malformed arguments must fail")
                assert(type(message) == "string")
            end
            rejects()
            rejects(nil, 0)
            rejects(12, 0)
            rejects(true, 0)
            rejects({}, 0)
            rejects("player")
            rejects("player", nil)
            rejects("player", "0")
            rejects("player", false)
            rejects("player", {})
            rejects("player", 0.5)
            rejects("player", 0/0)
            rejects("player", math.huge)
            rejects("player", -math.huge)
            rejects("player", 2147483648)
            rejects("player", -2147483649)
            rejects("unknown", "0")
            "#,
            )
            .unwrap();
    }
}

#[cfg(not(feature = "retail-12-0-5"))]
#[test]
fn unit_has_power_type_is_not_published_before_epoch() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("Lua environment");
    assert!(env.eval::<bool>("return UnitHasPowerType == nil").unwrap());
}
