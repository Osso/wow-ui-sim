#![cfg(feature = "retail-12-0-5")]

use std::time::{Duration, Instant};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::SpellCooldownState;

fn seed_cooldowns(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.start_time = Instant::now() - Duration::from_secs(35);
    state.action_bars.insert(17, 19750);
    state.spell_cooldowns.insert(
        19750,
        SpellCooldownState {
            start: 12.0,
            duration: 3700.0,
        },
    );
    state.gcd = Some((30.0, 6000.0));
}

#[test]
fn unit_permissions_compare_distinct_guids_despite_identical_names() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("TargetUnit('player'); FocusUnit('target')")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        let target = state.current_target.as_mut().unwrap();
        target.name = "Shared Name".into();
        target.guid = "Creature-0-0-0-0-448-000002".into();
        state.current_focus.as_mut().unwrap().name = "Shared Name".into();
    }
    env.exec(
        r#"
        assert(UnitName('target') == UnitName('focus'))
        assert(UnitGUID('target') ~= UnitGUID('focus'))
        assert(UnitIsUnit('target', 'focus') == false)
        assert(UnitIsUnit('focus', 'target') == false)
        FocusUnit('target')
        assert(UnitGUID('target') == UnitGUID('focus'))
        assert(UnitIsUnit('target', 'focus') == true)
        assert(UnitIsUnit('focus', 'target') == true)
        "#,
    )
    .unwrap();
}

#[test]
fn ignore_gcd_selected_objects_drive_cooldown_widgets_after_state_changes() {
    let env = WowLuaEnv::new().unwrap();
    seed_cooldowns(&env);
    env.exec(
        r#"
        queries = {
            function(ignore) return C_ActionBar.GetActionCooldownDuration(17, ignore) end,
            function(ignore) return C_Spell.GetSpellCooldownDuration(19750, ignore) end,
            function(ignore) return C_SpellBook.GetSpellBookItemCooldownDuration(5, 0, ignore) end,
        }
        assert(C_SpellBook.GetSpellBookItemInfo(5, 0).spellID == 19750)
        snapshots = {}
        cooldowns = {}
        for index, query in ipairs(queries) do
            local cooldown = CreateFrame('Cooldown')
            cooldowns[index] = cooldown
            cooldown:SetCooldownFromDurationObject(query(false))
            local start, total = cooldown:GetCooldownTimes()
            assert(start == 30000 and total == 6000000)
            snapshots[index] = query(true)
            cooldown:SetCooldownFromDurationObject(snapshots[index])
            start, total = cooldown:GetCooldownTimes()
            assert(start == 12000 and total == 3700000)
            assert(cooldown:GetCooldownDisplayDuration() == 3700000)
        end
        "#,
    )
    .unwrap();
    env.state().borrow_mut().spell_cooldowns.remove(&19750);
    env.exec(
        r#"
        for index, query in ipairs(queries) do
            local cooldown = cooldowns[index]
            cooldown:SetCooldownFromDurationObject(query(true))
            local start, total = cooldown:GetCooldownTimes()
            assert(start == 0 and total == 0)
            cooldown:SetCooldownFromDurationObject(query(false))
            start, total = cooldown:GetCooldownTimes()
            assert(start == 30000 and total == 6000000)
            cooldown:SetCooldownFromDurationObject(snapshots[index])
            start, total = cooldown:GetCooldownTimes()
            assert(start == 12000 and total == 3700000)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn ignore_gcd_secure_secret_flags_select_action_and_book_intervals() {
    let env = WowLuaEnv::new().unwrap();
    seed_cooldowns(&env);
    env.exec(
        r#"
        assert(issecure())
        local queries = {
            function(ignore) return C_ActionBar.GetActionCooldownDuration(17, ignore) end,
            function(ignore) return C_SpellBook.GetSpellBookItemCooldownDuration(5, 0, ignore) end,
        }
        local include = secretwrap(false)
        local exclude = secretwrap(true)
        for _, query in ipairs(queries) do
            local individual = query(exclude)
            assert(individual:GetStartTime() == 12)
            assert(individual:GetTotalDuration() == 3700)
            local withGCD = query(include)
            assert(withGCD:GetStartTime() == 30)
            assert(withGCD:GetTotalDuration() == 6000)
        end
        assert(issecretvalue(include) and issecretvalue(exclude))
        assert(issecure())
        "#,
    )
    .unwrap();
}

#[test]
fn ignore_gcd_tainted_secret_flags_are_denied_without_losing_taint() {
    let env = WowLuaEnv::new().unwrap();
    seed_cooldowns(&env);
    env.exec(
        r#"
        local include = secretwrap(false)
        local exclude = secretwrap(true)
        local function addon()
            assert(not issecure())
            for _, flag in ipairs({include, exclude}) do
                assert(not pcall(C_ActionBar.GetActionCooldownDuration, 17, flag))
                assert(not pcall(C_SpellBook.GetSpellBookItemCooldownDuration, 5, 0, flag))
            end
            assert(C_ActionBar.GetActionCooldownDuration(17, true):GetStartTime() == 12)
            assert(C_SpellBook.GetSpellBookItemCooldownDuration(5, 0, true):GetStartTime() == 12)
            assert(debug.getstacktaint() == 'IgnoreGcdProbe')
        end
        debug.setobjecttaint(addon, 'IgnoreGcdProbe')
        addon()
        assert(issecure())
        assert(C_ActionBar.GetActionCooldownDuration(17, exclude):GetStartTime() == 12)
        assert(C_SpellBook.GetSpellBookItemCooldownDuration(5, 0, exclude):GetStartTime() == 12)
        "#,
    )
    .unwrap();
}

#[test]
fn ignore_gcd_book_authenticates_flag_before_missing_entry_return() {
    let env = WowLuaEnv::new().unwrap();
    seed_cooldowns(&env);
    env.exec(
        r#"
        local flag = secretwrap(true)
        local function addon()
            for _, slot in ipairs({0, 19750}) do
                assert(C_SpellBook.GetSpellBookItemCooldownDuration(slot, 0, true) == nil)
                assert(not pcall(C_SpellBook.GetSpellBookItemCooldownDuration, slot, 0, flag))
            end
            assert(debug.getstacktaint() == 'IgnoreGcdOrderingProbe')
        end
        debug.setobjecttaint(addon, 'IgnoreGcdOrderingProbe')
        addon()
        assert(issecure())
        assert(C_SpellBook.GetSpellBookItemCooldownDuration(19750, 0, flag) == nil)
        "#,
    )
    .unwrap();
}
