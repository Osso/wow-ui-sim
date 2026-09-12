//! Tests for A_Admin encounter & loot roll simulation.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// SimulateBossKill
// ============================================================================

#[test]
fn test_boss_kill_fires_encounter_end() {
    let env = env();
    let got: bool = env
        .eval(
            r#"
            local fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("ENCOUNTER_END")
            f:SetScript("OnEvent", function(self, event, eid, ename, diff, size, success)
                if event == "ENCOUNTER_END" and eid == 2902 and ename == "Ky'veza"
                   and diff == 16 and size == 20 and success == 1 then
                    fired = true
                end
            end)
            A_Admin.SimulateBossKill(2902, "Ky'veza", 16, 20)
            return fired
            "#,
        )
        .unwrap();
    assert!(got);
}

#[test]
fn test_boss_kill_fires_boss_kill_event() {
    let env = env();
    let got: bool = env
        .eval(
            r#"
            local fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("BOSS_KILL")
            f:SetScript("OnEvent", function(self, event, eid, ename)
                if event == "BOSS_KILL" and eid == 2902 and ename == "Ky'veza" then
                    fired = true
                end
            end)
            A_Admin.SimulateBossKill(2902, "Ky'veza", 16, 20)
            return fired
            "#,
        )
        .unwrap();
    assert!(got);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn test_boss_kill_copies_ordered_encounter_status_before_dispatch() {
    let env = env();
    env.exec(
        r#"
        local supplied = {
            {creatureID = 218370, creatureName = "First boss", remainingHealthPercent = 0},
            {creatureID = 218371, creatureName = "Second boss", remainingHealthPercent = 37.5},
        }
        local events = {}
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("ENCOUNTER_END")
        frame:RegisterEvent("BOSS_KILL")
        frame:SetScript("OnEvent", function(_, event, ...)
            local args = {...}
            events[#events + 1] = {event = event, count = select('#', ...), args = args}
            if event == "ENCOUNTER_END" then
                supplied[1].creatureName = "Changed during callback"
                supplied[2].remainingHealthPercent = 100
                supplied[3] = supplied[1]
            end
        end)
        A_Admin.SimulateBossKill(2902, "Ky'veza", 16, 20, supplied)
        assert(#events == 2)
        assert(events[1].event == "ENCOUNTER_END" and events[1].count == 6)
        local args = events[1].args
        assert(args[1] == 2902 and args[2] == "Ky'veza" and args[3] == 16)
        assert(args[4] == 20 and args[5] == 1)
        local status = args[6]
        assert(type(status) == "table" and status ~= supplied and #status == 2)
        assert(status[1] ~= supplied[1] and status[2] ~= supplied[2])
        assert(status[1].creatureID == 218370 and status[1].creatureName == "First boss")
        assert(status[1].remainingHealthPercent == 0)
        assert(status[2].creatureID == 218371 and status[2].creatureName == "Second boss")
        assert(status[2].remainingHealthPercent == 37.5)
        assert(events[2].event == "BOSS_KILL" and events[2].count == 2)
        assert(events[2].args[1] == 2902 and events[2].args[2] == "Ky'veza")
        "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn test_boss_kill_omitted_and_nil_status_are_fresh_empty_lists() {
    let env = env();
    env.exec(
        r#"
        local events = {}
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("ENCOUNTER_END")
        frame:RegisterEvent("BOSS_KILL")
        frame:SetScript("OnEvent", function(_, event, ...)
            events[#events + 1] = {event = event, count = select('#', ...), args = {...}}
        end)
        A_Admin.SimulateBossKill(2902, "Ky'veza", 16, 20)
        A_Admin.SimulateBossKill(2902, "Ky'veza", 16, 20, nil)
        assert(#events == 4)
        for index = 1, 3, 2 do
            assert(events[index].event == "ENCOUNTER_END" and events[index].count == 6)
            local status = events[index].args[6]
            assert(type(status) == "table" and next(status) == nil)
            assert(events[index + 1].event == "BOSS_KILL" and events[index + 1].count == 2)
        end
        assert(events[1].args[6] ~= events[3].args[6])
        "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn test_boss_kill_rejects_malformed_status_before_any_event() {
    let env = env();
    env.exec(
        r#"
        local valid = {creatureID = 218370, creatureName = "First boss", remainingHealthPercent = 0}
        local invalid = {
            false,
            "not a list",
            {boss = valid},
            {[1] = valid, [3] = valid},
            {valid, false},
            {valid, {creatureID = 218371, remainingHealthPercent = 50}},
            {valid, {creatureID = 0, creatureName = "Boss", remainingHealthPercent = 50}},
            {valid, {creatureID = 1.5, creatureName = "Boss", remainingHealthPercent = 50}},
            {valid, {creatureID = math.huge, creatureName = "Boss", remainingHealthPercent = 50}},
            {valid, {creatureID = 218371, creatureName = 12, remainingHealthPercent = 50}},
            {valid, {creatureID = 218371, creatureName = "Boss", remainingHealthPercent = -1}},
            {valid, {creatureID = 218371, creatureName = "Boss", remainingHealthPercent = 101}},
            {valid, {creatureID = 218371, creatureName = "Boss", remainingHealthPercent = math.huge}},
            {valid, {creatureID = 218371, creatureName = "Boss", remainingHealthPercent = 0/0}},
        }
        local events = 0
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("ENCOUNTER_END")
        frame:RegisterEvent("BOSS_KILL")
        frame:SetScript("OnEvent", function() events = events + 1 end)
        for index, status in ipairs(invalid) do
            local ok = pcall(A_Admin.SimulateBossKill, 2902, "Ky'veza", 16, 20, status)
            assert(not ok, "malformed status accepted: " .. index)
            assert(events == 0, "event dispatched before rejection: " .. index)
        end
        "#,
    )
    .unwrap();
}

#[cfg(not(feature = "retail-12-0-7"))]
#[test]
fn test_boss_kill_earlier_profile_preserves_five_argument_payload() {
    let env = env();
    env.exec(
        r#"
        local events = {}
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("ENCOUNTER_END")
        frame:RegisterEvent("BOSS_KILL")
        frame:SetScript("OnEvent", function(_, event, ...)
            events[#events + 1] = {event = event, count = select('#', ...), args = {...}}
        end)
        A_Admin.SimulateBossKill(2902, "Ky'veza", 16, 20)
        assert(#events == 2)
        assert(events[1].event == "ENCOUNTER_END" and events[1].count == 5)
        local args = events[1].args
        assert(args[1] == 2902 and args[2] == "Ky'veza" and args[3] == 16)
        assert(args[4] == 20 and args[5] == 1)
        assert(events[2].event == "BOSS_KILL" and events[2].count == 2)
        assert(events[2].args[1] == 2902 and events[2].args[2] == "Ky'veza")
        "#,
    )
    .unwrap();
}

// ============================================================================
// StartLootRoll / EndLootRoll
// ============================================================================

#[test]
fn test_start_loot_roll_fires_event() {
    let env = env();
    let (roll_id, roll_time): (i32, f64) = env
        .eval(
            r#"
            local rid, rtime
            local f = CreateFrame("Frame")
            f:RegisterEvent("START_LOOT_ROLL")
            f:SetScript("OnEvent", function(self, event, id, time)
                rid = id; rtime = time
            end)
            A_Admin.StartLootRoll(42, 30, "Heroic Sword", "Interface\\Icons\\inv_sword", 4, 639)
            return rid, rtime
            "#,
        )
        .unwrap();
    assert_eq!(roll_id, 42);
    assert!((roll_time - 30.0).abs() < 0.001);
}

#[test]
fn test_get_loot_roll_item_info_returns_data() {
    let env = env();
    let (texture, name, quality, ilvl): (String, String, i32, i32) = env
        .eval(
            r#"
            A_Admin.StartLootRoll(1, 30, "Heroic Sword", "Interface\\Icons\\inv_sword", 4, 639)
            local tex, n, count, q, bop, need, greed, de, deLvl, il = GetLootRollItemInfo(1)
            return tex, n, q, il
            "#,
        )
        .unwrap();
    assert_eq!(texture, "Interface\\Icons\\inv_sword");
    assert_eq!(name, "Heroic Sword");
    assert_eq!(quality, 4);
    assert_eq!(ilvl, 639);
}

#[test]
fn test_get_loot_roll_item_info_unknown_returns_nil() {
    let env = env();
    let is_nil: bool = env.eval("return GetLootRollItemInfo(999) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_loot_roll_item_link() {
    let env = env();
    let link: String = env
        .eval(
            r#"
            A_Admin.StartLootRoll(1, 30, "Sword", "tex", 4, 600, "|cffff8000|Hitem:12345|h[Sword]|h|r")
            return GetLootRollItemLink(1)
            "#,
        )
        .unwrap();
    assert!(link.contains("12345"));
}

#[test]
fn test_get_loot_roll_item_link_nil_when_missing() {
    let env = env();
    let is_nil: bool = env.eval("return GetLootRollItemLink(999) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_loot_roll_time_left() {
    let env = env();
    let time: f64 = env
        .eval(
            r#"
            A_Admin.StartLootRoll(1, 25, "Sword", "tex", 4, 600)
            return GetLootRollTimeLeft(1)
            "#,
        )
        .unwrap();
    assert!((time - 25.0).abs() < 0.001);
}

#[test]
fn test_get_active_loot_roll_ids() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            A_Admin.StartLootRoll(10, 30, "Sword", "tex", 4, 600)
            A_Admin.StartLootRoll(20, 30, "Shield", "tex2", 3, 600)
            return #GetActiveLootRollIDs()
            "#,
        )
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_end_loot_roll_removes_and_fires_event() {
    let env = env();
    let (fired, count): (bool, i32) = env
        .eval(
            r#"
            local fired = false
            local f = CreateFrame("Frame")
            f:RegisterEvent("LOOT_ROLLS_COMPLETE")
            f:SetScript("OnEvent", function(self, event, handle)
                if event == "LOOT_ROLLS_COMPLETE" and handle == 1 then fired = true end
            end)
            A_Admin.StartLootRoll(1, 30, "Sword", "tex", 4, 600)
            A_Admin.EndLootRoll(1)
            return fired, #GetActiveLootRollIDs()
            "#,
        )
        .unwrap();
    assert!(fired);
    assert_eq!(count, 0);
}

#[test]
fn test_get_active_loot_roll_ids_empty_by_default() {
    let env = env();
    let count: i32 = env.eval("return #GetActiveLootRollIDs()").unwrap();
    assert_eq!(count, 0);
}
