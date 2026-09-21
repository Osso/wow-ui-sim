//! Integration tests for `src/lua_api/globals/cooldown_probes.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::SpellCooldownState;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {actual} to be close to {expected}"
    );
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn cooldown_set_paused_updates_is_paused() {
    env()
        .eval::<()>(
            r#"
        local cooldown = CreateFrame("Cooldown")
        cooldown:Resume()
        assert(cooldown:IsPaused() == false)
        cooldown:SetPaused(true)
        assert(cooldown:IsPaused() == true)
        cooldown:SetPaused(false)
        assert(cooldown:IsPaused() == false)
        cooldown:Pause()
        assert(cooldown:IsPaused() == true)
        cooldown:SetPaused(false)
        assert(cooldown:IsPaused() == false)
    "#,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn cooldown_set_paused_returns_zero_values() {
    env()
        .eval::<()>(
            r#"
        local cooldown = CreateFrame("Cooldown")
        assert(select('#', cooldown:SetPaused(true)) == 0)
        assert(select('#', cooldown:SetPaused(false)) == 0)
    "#,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn cooldown_set_paused_isolates_instances() {
    env()
        .eval::<()>(
            r#"
        local first, second = CreateFrame("Cooldown"), CreateFrame("Cooldown")
        first:Resume()
        second:Resume()
        first:SetPaused(true)
        assert(first:IsPaused() == true)
        assert(second:IsPaused() == false)
        second:SetPaused(true)
        first:SetPaused(false)
        assert(first:IsPaused() == false)
        assert(second:IsPaused() == true)
    "#,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn cooldown_set_paused_preserves_model_scene_pause_state() {
    env()
        .eval::<()>(
            r#"
        local scene = CreateFrame("ModelScene")
        local cooldown = CreateFrame("Cooldown")
        cooldown:Resume()
        scene:SetPaused(true)
        assert(scene:GetPaused() == true)
        assert(cooldown:IsPaused() == false)
        scene:SetPaused(false)
        assert(scene:GetPaused() == false)
        assert(cooldown:IsPaused() == false)

        cooldown:SetPaused(true)
        assert(cooldown:IsPaused() == true)
        assert(scene:GetPaused() == false)
        scene:SetPaused(true)
        assert(scene:GetPaused() == true)
        assert(cooldown:IsPaused() == true)
        cooldown:SetPaused(false)
        assert(cooldown:IsPaused() == false)
        assert(scene:GetPaused() == true)
        scene:SetPaused(false)
        assert(scene:GetPaused() == false)
        assert(cooldown:IsPaused() == false)
    "#,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn cooldown_set_paused_preserves_immediate_cooldown_times() {
    env()
        .eval::<()>(
            r#"
        local cooldown = CreateFrame("Cooldown")
        local start = GetTime()
        cooldown:SetCooldown(start, 60)
        local beforeStart, beforeDuration = cooldown:GetCooldownTimes()
        assert(beforeDuration > 0, 'active cooldown must have a duration')
        cooldown:SetPaused(true)
        local pausedStart, pausedDuration = cooldown:GetCooldownTimes()
        assert(pausedStart == beforeStart and pausedDuration == beforeDuration)
        cooldown:SetPaused(false)
        local resumedStart, resumedDuration = cooldown:GetCooldownTimes()
        assert(resumedStart == beforeStart and resumedDuration == beforeDuration)
    "#,
        )
        .unwrap();
}

// ── GetSpellCooldown ──────────────────────────────────────────────────────────

#[test]
fn get_spell_cooldown_zero_when_no_cooldown() {
    let env = env();
    let (start, duration, enable, mod_rate): (f64, f64, i32, f64) =
        env.eval("return GetSpellCooldown(12345)").unwrap();
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert_eq!(enable, 1);
    assert_eq!(mod_rate, 1.0);
}

#[test]
fn get_spell_cooldown_reads_spell_cooldowns_entry() {
    let env = env();
    let now = env.state().borrow().start_time.elapsed().as_secs_f64();
    env.state().borrow_mut().spell_cooldowns.insert(
        12345,
        SpellCooldownState {
            start: now,
            duration: 30.0,
        },
    );
    let (start, duration, _enable, _mod): (f64, f64, i32, f64) =
        env.eval("return GetSpellCooldown(12345)").unwrap();
    assert!(
        (start - now).abs() < 0.5,
        "start should match the seeded cooldown"
    );
    assert_close(duration, 30.0);
}

// ── C_Spell.GetSpellCooldownDuration ──────────────────────────────────────────

#[cfg(feature = "retail-12-0-0")]
#[test]
fn get_spell_cooldown_duration_reads_active_spell_and_runtime_clock() {
    let env = env();
    let start = {
        let mut state = env.state().borrow_mut();
        let start = state.start_time.elapsed().as_secs_f64() - 5.0;
        state.gcd = None;
        state.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start,
                duration: 30.0,
            },
        );
        start
    };
    let (actual_start, total, end, rate): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local duration = C_Spell.GetSpellCooldownDuration(12345)
            local info = C_Spell.GetSpellCooldown(12345)
            assert(duration ~= nil, 'active spell returns a duration object')
            assert(math.abs(duration:GetStartTime() - info.startTime) < 1e-9)
            assert(math.abs(duration:GetTotalDuration() - info.duration) < 1e-9)
            assert(duration:GetModRate() == info.modRate)
            local before = GetTime()
            local elapsed = duration:GetElapsedDuration()
            local remaining = duration:GetRemainingDuration()
            local after = GetTime()
            local start, finish = duration:GetStartTime(), duration:GetEndTime()
            local epsilon = 1e-6
            assert(elapsed >= math.min(30, math.max(0, before - start)) - epsilon)
            assert(elapsed <= math.min(30, math.max(0, after - start)) + epsilon)
            assert(remaining >= math.min(30, math.max(0, finish - after)) - epsilon)
            assert(remaining <= math.min(30, math.max(0, finish - before)) + epsilon)
            return start, duration:GetTotalDuration(), finish, duration:GetModRate()
            "#,
        )
        .expect("spell duration matches modeled cooldown state and runtime clock");
    assert_close(actual_start, start);
    assert_close(total, 30.0);
    assert_close(end, start + 30.0);
    assert_close(rate, 1.0);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn get_spell_cooldown_duration_resolves_alias_and_snapshots_state() {
    let env = env();
    let start = {
        let mut state = env.state().borrow_mut();
        let start = state.start_time.elapsed().as_secs_f64() - 5.0;
        state.gcd = None;
        state.spell_id_aliases.insert("test spell".into(), 12345);
        state.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start,
                duration: 30.0,
            },
        );
        start
    };
    env.exec(
        r#"
        -- Seeded alias policy, not a claim about real spell-name search.
        oldSpellDuration = C_Spell.GetSpellCooldownDuration('TeSt SpElL')
        local lower = C_Spell.GetSpellCooldownDuration('test spell')
        assert(oldSpellDuration:GetStartTime() == lower:GetStartTime())
        assert(oldSpellDuration:GetTotalDuration() == 30)
        assert(lower:GetTotalDuration() == 30)
        "#,
    )
    .expect("seeded spell alias resolves case variants");
    {
        let mut state = env.state().borrow_mut();
        state.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start,
                duration: 45.0,
            },
        );
        state.spell_id_aliases.insert("test spell".into(), 54321);
        state.spell_cooldowns.insert(
            54321,
            SpellCooldownState {
                start: start + 1.0,
                duration: 60.0,
            },
        );
    }
    let (old_start, old_total, new_start, new_total, numeric_total): (f64, f64, f64, f64, f64) =
        env.eval(
            r#"
            local current = C_Spell.GetSpellCooldownDuration('TEST SPELL')
            local numeric = C_Spell.GetSpellCooldownDuration(12345)
            return oldSpellDuration:GetStartTime(), oldSpellDuration:GetTotalDuration(),
                current:GetStartTime(), current:GetTotalDuration(), numeric:GetTotalDuration()
            "#,
        )
        .expect("new queries reflect changed alias and cooldown state");
    assert_close(old_start, start);
    assert_close(old_total, 30.0);
    assert_close(new_start, start + 1.0);
    assert_close(new_total, 60.0);
    assert_close(numeric_total, 45.0);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn get_spell_cooldown_duration_selects_later_ending_gcd() {
    let env = env();
    let gcd_start = {
        let mut state = env.state().borrow_mut();
        let now = state.start_time.elapsed().as_secs_f64();
        state.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start: now - 5.0,
                duration: 30.0,
            },
        );
        state.gcd = Some((now - 2.0, 60.0));
        now - 2.0
    };
    let (start, total, end, rate): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local duration = C_Spell.GetSpellCooldownDuration(12345)
            local info = C_Spell.GetSpellCooldown(12345)
            assert(math.abs(duration:GetStartTime() - info.startTime) < 1e-9)
            assert(math.abs(duration:GetTotalDuration() - info.duration) < 1e-9)
            return duration:GetStartTime(), duration:GetTotalDuration(),
                duration:GetEndTime(), duration:GetModRate()
            "#,
        )
        .expect("spell duration follows the modeled later-ending GCD");
    assert_close(start, gcd_start);
    assert_close(total, 60.0);
    assert_close(end, gcd_start + 60.0);
    assert_close(rate, 1.0);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn get_spell_cooldown_duration_inactive_numeric_and_unresolved_alias() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.gcd = None;
        state.spell_cooldowns.remove(&12345);
        state.spell_id_aliases.remove("unresolved test spell");
    }
    env.exec(
        r#"
        -- Explicit simulator policy, not native unknown-spell or return-arity proof.
        local duration = C_Spell.GetSpellCooldownDuration(12345)
        assert(duration ~= nil)
        assert(duration:GetStartTime() == 0)
        assert(duration:GetTotalDuration() == 0)
        assert(duration:GetEndTime() == 0)
        assert(duration:GetModRate() == 1)
        assert(duration:GetElapsedDuration() == 0)
        assert(duration:GetRemainingDuration() == 0)
        assert(C_Spell.GetSpellCooldownDuration('unresolved test spell') == nil)
        "#,
    )
    .expect("inactive numeric identifiers yield zero timing and unresolved aliases yield nil");
}

#[cfg(not(feature = "retail-12-0-0"))]
#[test]
fn get_spell_cooldown_duration_keeps_pre_retail_12_0_0_nil_result() {
    let env = env();
    let now = env.state().borrow().start_time.elapsed().as_secs_f64();
    env.state().borrow_mut().spell_cooldowns.insert(
        12345,
        SpellCooldownState {
            start: now,
            duration: 30.0,
        },
    );
    let unchanged: bool = env
        .eval("return C_Spell.GetSpellCooldownDuration(12345) == nil")
        .unwrap();
    assert!(unchanged, "preserve the earlier-profile nil result");
}

// ── GetActionCooldown ─────────────────────────────────────────────────────────

#[test]
fn get_action_cooldown_resolves_bar_slot_through_spell() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.action_bars.insert(1, 12345);
        let now = st.start_time.elapsed().as_secs_f64();
        st.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start: now,
                duration: 15.0,
            },
        );
    }
    let (_start, duration, enable, mod_rate): (f64, f64, i32, f64) =
        env.eval("return GetActionCooldown(1)").unwrap();
    assert_close(duration, 15.0);
    assert_eq!(enable, 1);
    assert_eq!(mod_rate, 1.0);
}

#[test]
fn get_action_cooldown_empty_slot_returns_zero() {
    let env = env();
    let (start, duration, enable, _mod): (f64, f64, i32, f64) =
        env.eval("return GetActionCooldown(99)").unwrap();
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert_eq!(enable, 1);
}

fn assert_action_cooldown_active(env: &WowLuaEnv, active: bool) {
    use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};

    let has_active_field = match ACTIVE {
        ClientProfile::WowForever => true,
        ClientProfile::Retail | ClientProfile::Ptr => ACTIVE_INTERFACE_VERSION >= 120100,
        _ => false,
    };
    let expected = if has_active_field {
        active.to_string()
    } else {
        "nil".to_owned()
    };
    env.exec(&format!(
        "assert(C_ActionBar.GetActionCooldown(1).isActive == {expected}, \
         'action cooldown active field must match the profile contract')"
    ))
    .unwrap();
}

#[test]
fn get_action_cooldown_active_tracks_admin_spell_assignment_and_clear() {
    let env = env();
    env.state().borrow_mut().gcd = None;
    env.exec(
        r#"
        A_Admin.SetActionSlot(1, 19750)
        A_Admin.SetSpellCooldown(19750, 5)
        local kind, spellID = GetActionInfo(1)
        assert(kind == 'spell' and spellID == 19750)
        local action = C_ActionBar.GetActionCooldown(1)
        local spell = C_Spell.GetSpellCooldown(19750)
        assert(action.startTime > 0 and action.startTime == spell.startTime)
        assert(action.duration == 5 and action.duration == spell.duration)
        assert(action.isEnabled == true and action.modRate == 1)
        assert(spell.isActive == true)
        "#,
    )
    .unwrap();
    assert_action_cooldown_active(&env, true);

    env.exec(
        r#"
        A_Admin.SetSpellCooldown(19750, 0)
        local action = C_ActionBar.GetActionCooldown(1)
        assert(action.startTime == 0 and action.duration == 0)
        assert(action.isEnabled == true and action.modRate == 1)
        assert(C_Spell.GetSpellCooldown(19750).isActive == false)
        local kind, spellID = GetActionInfo(1)
        assert(kind == 'spell' and spellID == 19750)
        "#,
    )
    .unwrap();
    assert_action_cooldown_active(&env, false);
}

#[test]
fn get_action_cooldown_active_is_false_after_expiration() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.start_time = std::time::Instant::now() - std::time::Duration::from_secs(30);
        state.gcd = None;
        state.action_bars.insert(1, 19750);
        state.spell_cooldowns.insert(
            19750,
            SpellCooldownState {
                start: 20.0,
                duration: 5.0,
            },
        );
    }
    env.exec(
        r#"
        local action = C_ActionBar.GetActionCooldown(1)
        assert(action.startTime == 0 and action.duration == 0)
        assert(action.isEnabled == true and action.modRate == 1)
        "#,
    )
    .unwrap();
    assert_action_cooldown_active(&env, false);
}

#[test]
fn get_action_cooldown_active_is_false_for_zero_start() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.start_time = std::time::Instant::now() - std::time::Duration::from_secs(1);
        state.gcd = None;
        state.action_bars.insert(1, 19750);
        state.spell_cooldowns.insert(
            19750,
            SpellCooldownState {
                start: 0.0,
                duration: 5.0,
            },
        );
    }
    env.exec(
        r#"
        local action = C_ActionBar.GetActionCooldown(1)
        assert(action.startTime == 0 and action.duration == 5)
        assert(action.isEnabled == true and action.modRate == 1)
        "#,
    )
    .unwrap();
    assert_action_cooldown_active(&env, false);
}

// ── C_ActionBar.GetActionCooldownDuration ──────────────────────────────────────

#[test]
fn get_action_cooldown_duration_reads_active_slot_and_runtime_clock() {
    let env = env();
    let start = {
        let mut state = env.state().borrow_mut();
        let start = state.start_time.elapsed().as_secs_f64() - 5.0;
        state.gcd = None;
        state.action_bars.insert(1, 12345);
        state.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start,
                duration: 30.0,
            },
        );
        start
    };
    let (actual_start, total, end, rate): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local duration = C_ActionBar.GetActionCooldownDuration(1)
            local info = C_ActionBar.GetActionCooldown(1)
            assert(duration ~= nil, 'active slot returns a duration object')
            assert(math.abs(duration:GetStartTime() - info.startTime) < 1e-9,
                'duration start must match the action cooldown')
            assert(math.abs(duration:GetTotalDuration() - info.duration) < 1e-9,
                'duration total must match the action cooldown')
            assert(duration:GetModRate() == info.modRate)
            local before = GetTime()
            local elapsed = duration:GetElapsedDuration()
            local remaining = duration:GetRemainingDuration()
            local after = GetTime()
            local start, finish = duration:GetStartTime(), duration:GetEndTime()
            local epsilon = 1e-6
            assert(elapsed >= math.min(30, math.max(0, before - start)) - epsilon)
            assert(elapsed <= math.min(30, math.max(0, after - start)) + epsilon)
            assert(remaining >= math.min(30, math.max(0, finish - after)) - epsilon)
            assert(remaining <= math.min(30, math.max(0, finish - before)) + epsilon)
            return start, duration:GetTotalDuration(), finish, duration:GetModRate()
            "#,
        )
        .expect("active action duration matches cooldown state and the runtime clock");
    assert_close(actual_start, start);
    assert_close(total, 30.0);
    assert_close(end, start + 30.0);
    assert_close(rate, 1.0);
}

#[test]
fn get_action_cooldown_duration_snapshots_slot_and_spell_state() {
    let env = env();
    let start = {
        let mut state = env.state().borrow_mut();
        let start = state.start_time.elapsed().as_secs_f64() - 5.0;
        state.gcd = None;
        state.action_bars.insert(1, 12345);
        state.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start,
                duration: 30.0,
            },
        );
        start
    };
    env.exec("oldActionDuration = C_ActionBar.GetActionCooldownDuration(1)")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start,
                duration: 45.0,
            },
        );
        state.action_bars.insert(1, 54321);
        state.spell_cooldowns.insert(
            54321,
            SpellCooldownState {
                start: start + 1.0,
                duration: 60.0,
            },
        );
    }
    let (old_start, old_total, new_start, new_total): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local current = C_ActionBar.GetActionCooldownDuration(1)
            return oldActionDuration:GetStartTime(), oldActionDuration:GetTotalDuration(),
                current:GetStartTime(), current:GetTotalDuration()
            "#,
        )
        .unwrap();
    assert_close(old_start, start);
    assert_close(old_total, 30.0);
    assert_close(new_start, start + 1.0);
    assert_close(new_total, 60.0);
}

#[test]
fn get_action_cooldown_duration_selects_later_ending_gcd() {
    let env = env();
    let gcd_start = {
        let mut state = env.state().borrow_mut();
        let now = state.start_time.elapsed().as_secs_f64();
        state.action_bars.insert(1, 12345);
        state.spell_cooldowns.insert(
            12345,
            SpellCooldownState {
                start: now - 5.0,
                duration: 30.0,
            },
        );
        state.gcd = Some((now - 2.0, 60.0));
        now - 2.0
    };
    let (start, total, end, rate): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local duration = C_ActionBar.GetActionCooldownDuration(1)
            local info = C_ActionBar.GetActionCooldown(1)
            assert(math.abs(duration:GetStartTime() - info.startTime) < 1e-9,
                'duration and cooldown select the same GCD start')
            assert(math.abs(duration:GetTotalDuration() - info.duration) < 1e-9,
                'duration and cooldown select the same GCD span')
            return duration:GetStartTime(), duration:GetTotalDuration(),
                duration:GetEndTime(), duration:GetModRate()
            "#,
        )
        .expect("the later-ending GCD determines the modeled action cooldown duration");
    assert_close(start, gcd_start);
    assert_close(total, 60.0);
    assert_close(end, gcd_start + 60.0);
    assert_close(rate, 1.0);
}

#[test]
fn get_action_cooldown_duration_empty_inactive_and_expired_are_zero() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        let now = state.start_time.elapsed().as_secs_f64();
        state.gcd = None;
        state.action_bars.remove(&99);
        state.action_bars.insert(1, 12345);
        state.spell_cooldowns.remove(&12345);
        state.action_bars.insert(2, 54321);
        state.spell_cooldowns.insert(
            54321,
            SpellCooldownState {
                start: now - 40.0,
                duration: 30.0,
            },
        );
    }
    env.exec(
        r#"
        -- Valid-slot zero-duration policy, not native invalid-slot semantics.
        for _, slot in ipairs({99, 1, 2}) do
            local duration = C_ActionBar.GetActionCooldownDuration(slot)
            assert(duration ~= nil, 'valid inactive slots return a duration object')
            assert(duration:GetStartTime() == 0)
            assert(duration:GetTotalDuration() == 0)
            assert(duration:GetEndTime() == 0)
            assert(duration:GetModRate() == 1)
            assert(duration:GetElapsedDuration() == 0)
            assert(duration:GetRemainingDuration() == 0)
        end
        "#,
    )
    .expect("empty, inactive and expired action cooldowns have zero duration");
}

// ── GetInventoryItemCooldown ──────────────────────────────────────────────────

#[test]
fn get_inventory_item_cooldown_zero_when_no_cooldown() {
    let env = env();
    let (start, duration, enable): (f64, f64, i32) = env
        .eval(r#"return GetInventoryItemCooldown("player", 13)"#)
        .unwrap();
    assert_eq!(start, 0.0);
    assert_eq!(duration, 0.0);
    assert_eq!(enable, 1);
}

#[test]
fn get_inventory_item_cooldown_reads_state_entry() {
    let env = env();
    let now = env.state().borrow().start_time.elapsed().as_secs_f64();
    env.state().borrow_mut().inventory_item_cooldowns.insert(
        13,
        SpellCooldownState {
            start: now,
            duration: 120.0,
        },
    );
    let (start, duration, enable): (f64, f64, i32) = env
        .eval(r#"return GetInventoryItemCooldown("player", 13)"#)
        .unwrap();
    assert!((start - now).abs() < 0.5);
    assert_eq!(duration, 120.0);
    assert_eq!(enable, 1);
}

// ── GetSpellBonusDamage / GetSpellBonusHealing ────────────────────────────────

#[test]
fn spell_bonus_damage_and_healing_share_intellect_bucket() {
    let env = env();
    env.state().borrow_mut().player.stats.intellect = 2500.0;
    let (damage, healing): (f64, f64) = env
        .eval("return GetSpellBonusDamage(1), GetSpellBonusHealing()")
        .unwrap();
    assert_eq!(damage, 2500.0);
    assert_eq!(healing, 2500.0);
}

// ── GetSpellAutocast ──────────────────────────────────────────────────────────

#[test]
fn get_spell_autocast_always_false() {
    let env = env();
    let (castable, casting): (bool, bool) = env.eval("return GetSpellAutocast(12345)").unwrap();
    assert!(!castable);
    assert!(!casting);
}

// ── GetSpellLevelLearned ──────────────────────────────────────────────────────

#[test]
fn get_spell_level_learned_zero_for_unknown_spell() {
    let env = env();
    let level: i32 = env.eval("return GetSpellLevelLearned(99999)").unwrap();
    assert_eq!(level, 0);
}

#[test]
fn get_spell_level_learned_one_for_known_spell() {
    let env = env();
    env.state().borrow_mut().known_spells.insert(12345);
    let level: i32 = env.eval("return GetSpellLevelLearned(12345)").unwrap();
    assert_eq!(level, 1);
}
