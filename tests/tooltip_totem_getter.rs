//! WM94: original SetTotem accessor must show stored name, not spell composition.
#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::globals::real::totems::Totem;

#[test]
fn tooltip_totem_original_accessor_uses_stored_name_and_two_lines() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec("A_Admin.SetManualClock(100)").unwrap();
    env.state().borrow_mut().totem_slots = vec![
        Some(Totem {
            name: "Searing Totem".into(),
            start_time: 90.0,
            duration: 60.0,
            icon: 0,
            mod_rate: 1.0,
            spell_id: 19750,
        }),
        Some(Totem {
            name: "Healing Stream Totem".into(),
            start_time: 95.0,
            duration: 30.0,
            icon: 0,
            mod_rate: 2.0,
            spell_id: 19750,
        }),
    ];
    {
        let mut sim = env.state().borrow_mut();
        let occupied = sim.totem_slots[0].clone();
        sim.totem_slots.resize(9, None);
        sim.totem_slots[7] = occupied.clone();
        sim.totem_slots[8] = occupied;
    }
    env.exec(
        r#"
        for slot, name in ipairs({'Searing Totem', 'Healing Stream Totem'}) do
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetTotem(slot)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetTotem', 'totem accessor missing')
            assert(data and data.type == 18 and data.id == nil)
            assert(#data.lines == 2, 'totem must not include a spell description block')
            assert(data.lines[1].leftText == name, 'must use the stored name, not spell ID')
            assert(GameTooltip:GetLeftLine(1):GetText() == name)
            assert(type(data.lines[2].leftText) == 'string' and #data.lines[2].leftText > 0)
            assert(GameTooltip:GetLeftLine(2):GetText() == data.lines[2].leftText)
            assert(data.lines[2].leftColor and data.lines[2].leftColor.a == nil)
        end
        assert(select('#', C_TooltipInfo.GetTotem(0)) == 0)
        assert(select('#', C_TooltipInfo.GetTotem(3)) == 0)
        assert(C_TooltipInfo.GetTotem(8).lines[1].leftText == 'Searing Totem')
        assert(select('#', C_TooltipInfo.GetTotem(9)) == 0)
        assert(select('#', C_TooltipInfo.GetTotem(4294967295)) == 0)
        "#,
    )
    .unwrap();
}


fn assert_remaining(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    start: f64,
    duration: f64,
    rate: f64,
    key: &str,
    count: u32,
) {
    env.state().borrow_mut().totem_slots[0] = Some(Totem {
        name: "Searing Totem".into(),
        start_time: start,
        duration,
        icon: 0,
        mod_rate: rate,
        spell_id: 19750,
    });
    env.exec(&format!(
        r#"
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetTotem(1)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data and #data.lines == 2, 'occupied entry must retain both lines')
        local expected = format({key}, {count})
        assert(data.lines[2].leftText == expected,
            'remaining-time mismatch: ' .. tostring(data.lines[2].leftText) .. ' vs ' .. expected)
        assert(GameTooltip:GetLeftLine(2):GetText() == expected)
        "#
    ))
    .unwrap();
}

#[test]
fn tooltip_totem_original_accessor_preserves_rounding_units_expiry_and_clock_wrap() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec("A_Admin.SetManualClock(100)").unwrap();
    for (start, duration, rate, key, count) in [
        (90.0, 20.0, 2.0, "SPELL_TIME_REMAINING_SEC", 5),
        (100.0, 0.001, 2.0, "SPELL_TIME_REMAINING_SEC", 0),
        (100.0, 0.003, 2.0, "SPELL_TIME_REMAINING_SEC", 1),
        (100.0, 2.001, 2.0, "SPELL_TIME_REMAINING_SEC", 1),
        (100.0, 2.003, 2.0, "SPELL_TIME_REMAINING_SEC", 2),
        (100.0, 90.0, 1.0, "SPELL_TIME_REMAINING_SEC", 90),
        (100.0, 90.001, 1.0, "SPELL_TIME_REMAINING_MIN", 2),
        (100.0, 5400.0, 1.0, "SPELL_TIME_REMAINING_MIN", 90),
        (100.0, 5400.001, 1.0, "SPELL_TIME_REMAINING_HOURS", 2),
        (100.0, 129600.0, 1.0, "SPELL_TIME_REMAINING_HOURS", 36),
        // The f32 conversion precedes unit selection: this rounds back to the boundary.
        (100.0, 129600.001, 1.0, "SPELL_TIME_REMAINING_HOURS", 36),
        (100.0, 129600.008, 1.0, "SPELL_TIME_REMAINING_DAYS", 2),
        (0.0, 90.0, 1.0, "SPELL_TIME_REMAINING_SEC", 0),
        (0.0, 0.0, 1.0, "SPELL_TIME_REMAINING_SEC", 0),
    ] {
        assert_remaining(&env, start, duration, rate, key, count);
    }
    let wrapped = tooltip_full_env_helpers::load_full_env_without_post_load();
    wrapped.exec("A_Admin.SetManualClock(0.1)").unwrap();
    assert_remaining(&wrapped, 4294966.796, 1.0, 1.0, "SPELL_TIME_REMAINING_SEC", 1);
    // The occupied timer's wrapping end is zero, so native composition retains a zero line.
    assert_remaining(&wrapped, 4294966.296, 1.0, 1.0, "SPELL_TIME_REMAINING_SEC", 0);
    wrapped.state().borrow_mut().totem_slots[0].as_mut().unwrap().mod_rate = 0.0;
    assert!(wrapped.exec("C_TooltipInfo.GetTotem(1)").is_err());
    assert!(wrapped.exec("C_TooltipInfo.GetTotem(-1)").is_err());
}

#[test]
fn tooltip_totem_slot_reader_truncates_after_subtracting_one() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec("A_Admin.SetManualClock(100)").unwrap();
    env.state().borrow_mut().totem_slots = vec![
        Some(Totem {
            name: "Searing Totem".into(),
            start_time: 90.0,
            duration: 60.0,
            icon: 0,
            mod_rate: 1.0,
            spell_id: 19750,
        }),
        Some(Totem {
            name: "Healing Stream Totem".into(),
            start_time: 95.0,
            duration: 30.0,
            icon: 0,
            mod_rate: 2.0,
            spell_id: 19750,
        }),
    ];
    {
        let mut sim = env.state().borrow_mut();
        let occupied = sim.totem_slots[0].clone();
        sim.totem_slots.resize(9, None);
        sim.totem_slots[7] = occupied;
    }
    env.exec(
        r#"
        -- Round 6(a): the one-based index reader subtracts 1.0 and truncates; it has
        -- no integral test, so a fractional slot selects the slot it truncates to.
        assert(C_TooltipInfo.GetTotem(1.9).lines[1].leftText == 'Searing Totem')
        assert(C_TooltipInfo.GetTotem(2.9).lines[1].leftText == 'Healing Stream Totem')
        assert(C_TooltipInfo.GetTotem(8.9).lines[1].leftText == 'Searing Totem')
        -- INFERRED (same truncation rule applied below 1): 0.5 becomes internal slot 0.
        assert(C_TooltipInfo.GetTotem(0.5).lines[1].leftText == 'Searing Totem')
        -- Out-of-range values keep taking the miss arm, never a diagnostic.
        assert(select('#', C_TooltipInfo.GetTotem(9)) == 0)
        assert(select('#', C_TooltipInfo.GetTotem(9.9)) == 0)
        -- Round 6(a): a missing, non-number, negative, out-of-domain or non-finite
        -- argument raises instead of returning nothing.
        assert(not pcall(C_TooltipInfo.GetTotem))
        assert(not pcall(C_TooltipInfo.GetTotem, nil))
        assert(not pcall(C_TooltipInfo.GetTotem, 'searing'))
        assert(not pcall(C_TooltipInfo.GetTotem, -1))
        assert(not pcall(C_TooltipInfo.GetTotem, 4294967296))
        assert(not pcall(C_TooltipInfo.GetTotem, 0 / 0))
        "#,
    )
    .unwrap();
}
