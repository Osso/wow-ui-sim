//! Tests for ROT (damage over time) reducing player health to zero and marking dead.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_player_health_decreases_with_rot_damage() {
    let env = env();

    let initial_hp: i32 = env.eval("return UnitHealth('player')").unwrap();
    assert_eq!(initial_hp, 100_000);

    // Apply 10,000 damage to the player via state.
    {
        let mut state = env.state().borrow_mut();
        state.player_health -= 10_000;
    }

    let hp: i32 = env.eval("return UnitHealth('player')").unwrap();
    assert_eq!(hp, 90_000);
}

#[test]
fn test_rot_damage_ticks_health_to_zero() {
    let env = env();

    // Set player to low health so ROT kills quickly.
    {
        let mut state = env.state().borrow_mut();
        state.player_health = 500;
    }

    // Apply 5 ticks of 100 damage each.
    for tick in 0..5 {
        let expected = 500 - (tick + 1) * 100;
        {
            let mut state = env.state().borrow_mut();
            state.player_health = (state.player_health - 100).max(0);
        }
        let hp: i32 = env.eval("return UnitHealth('player')").unwrap();
        assert_eq!(hp, expected, "health after tick {}", tick + 1);
    }

    let final_hp: i32 = env.eval("return UnitHealth('player')").unwrap();
    assert_eq!(final_hp, 0);
}

#[test]
fn test_player_marked_dead_at_zero_health() {
    let env = env();

    // Player starts alive.
    let alive: bool = env.eval("return not UnitIsDead('player')").unwrap();
    assert!(alive, "player should start alive");

    // Set health to zero.
    {
        let mut state = env.state().borrow_mut();
        state.player_health = 0;
    }

    let dead: bool = env.eval("return UnitIsDead('player')").unwrap();
    assert!(dead, "player should be dead at 0 hp");

    let dead_or_ghost: bool = env.eval("return UnitIsDeadOrGhost('player')").unwrap();
    assert!(dead_or_ghost, "UnitIsDeadOrGhost should also be true");
}

#[test]
fn test_unit_health_event_fires_on_rot_tick() {
    let env = env();

    // Register a frame to listen for UNIT_HEALTH on the player.
    env.exec(
        r#"
        _G.healthEventCount = 0
        _G.lastHealthUnit = nil
        local f = CreateFrame("Frame")
        f:RegisterEvent("UNIT_HEALTH")
        f:SetScript("OnEvent", function(self, event, unit)
            if unit == "player" then
                _G.healthEventCount = _G.healthEventCount + 1
                _G.lastHealthUnit = unit
            end
        end)
        "#,
    )
    .unwrap();

    // Apply damage and fire the event.
    {
        let mut state = env.state().borrow_mut();
        state.player_health -= 5_000;
    }
    let lua = env.lua();
    env.fire_event_with_args(
        "UNIT_HEALTH",
        &[mlua::Value::String(lua.create_string("player").unwrap())],
    )
    .unwrap();

    let count: i32 = env.eval("return _G.healthEventCount").unwrap();
    assert_eq!(count, 1);

    let unit: String = env.eval("return _G.lastHealthUnit").unwrap();
    assert_eq!(unit, "player");

    let hp: i32 = env.eval("return UnitHealth('player')").unwrap();
    assert_eq!(hp, 95_000);
}

#[test]
fn test_rot_damage_full_sequence_to_death() {
    let env = env();

    // Track events via Lua.
    env.exec(
        r#"
        _G.healthUpdates = {}
        _G.playerDied = false
        local f = CreateFrame("Frame")
        f:RegisterEvent("UNIT_HEALTH")
        f:SetScript("OnEvent", function(self, event, unit)
            if unit == "player" then
                local hp = UnitHealth("player")
                table.insert(_G.healthUpdates, hp)
                if UnitIsDead("player") then
                    _G.playerDied = true
                end
            end
        end)
        "#,
    )
    .unwrap();

    let tick_damage = 20_000;
    let lua = env.lua();

    // Tick 5 times: 100k -> 80k -> 60k -> 40k -> 20k -> 0
    for _ in 0..5 {
        {
            let mut state = env.state().borrow_mut();
            state.player_health = (state.player_health - tick_damage).max(0);
        }
        env.fire_event_with_args(
            "UNIT_HEALTH",
            &[mlua::Value::String(lua.create_string("player").unwrap())],
        )
        .unwrap();
    }

    // Verify health went down in order.
    let (h1, h2, h3, h4, h5): (i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            return _G.healthUpdates[1], _G.healthUpdates[2],
                   _G.healthUpdates[3], _G.healthUpdates[4],
                   _G.healthUpdates[5]
            "#,
        )
        .unwrap();
    assert_eq!([h1, h2, h3, h4, h5], [80_000, 60_000, 40_000, 20_000, 0]);

    // Verify player is dead.
    let died: bool = env.eval("return _G.playerDied").unwrap();
    assert!(died, "OnEvent handler should have detected death");

    let dead: bool = env.eval("return UnitIsDead('player')").unwrap();
    assert!(dead, "UnitIsDead should return true at 0 hp");
}
