//! Integration tests for the 6 targeting globals migrated off GLOBAL_NIL_STUBS.
//!
//! Covers: TargetUnit, FocusUnit, ClearTarget, TargetLastTarget,
//! TargetNearestFriend, TargetNearestEnemy.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ── TargetUnit ────────────────────────────────────────────────────────────────

#[test]
fn target_unit_target_token_round_trips_via_unit_exists() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            A_Admin.SetTarget("Boss", 63, 1, true)
            TargetUnit("target")
            return UnitExists("target")
            "#,
        )
        .unwrap();
    assert!(
        exists,
        "UnitExists('target') should be true after TargetUnit"
    );
}

#[test]
fn target_unit_player_token_sets_player_as_target() {
    let env = env();
    let is_player: bool = env
        .eval(
            r#"
            TargetUnit("player")
            return UnitExists("target")
            "#,
        )
        .unwrap();
    assert!(is_player);
}

#[test]
fn target_unit_unknown_token_is_silent_noop() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            TargetUnit("nonexistent_unit_xyz")
            return UnitExists("target")
            "#,
        )
        .unwrap();
    assert!(!exists);
}

#[test]
fn target_unit_enemy1_falls_back_to_default_enemy() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            TargetUnit("enemy1")
            return UnitName("target")
            "#,
        )
        .unwrap();
    assert_eq!(name, "Hogger");
}

#[test]
fn target_unit_does_not_crash_and_target_exists() {
    // Smoke test: TargetUnit against an existing target token doesn't crash
    // and the target is still set afterwards.
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            A_Admin.SetTarget("Boss", 63, 1, true)
            TargetUnit("target")
            return UnitExists("target")
            "#,
        )
        .unwrap();
    assert!(exists);
}

// ── FocusUnit ─────────────────────────────────────────────────────────────────

#[test]
fn focus_unit_focus_token_round_trips_via_unit_exists() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            A_Admin.SetFocus("Tank", 80, 2, false)
            FocusUnit("focus")
            return UnitExists("focus")
            "#,
        )
        .unwrap();
    assert!(exists, "UnitExists('focus') should be true after FocusUnit");
}

#[test]
fn focus_unit_player_token_sets_player_as_focus() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            FocusUnit("player")
            return UnitExists("focus")
            "#,
        )
        .unwrap();
    assert!(exists);
}

#[test]
fn focus_unit_unknown_token_leaves_focus_empty() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            FocusUnit("nonexistent_unit_xyz")
            return UnitExists("focus")
            "#,
        )
        .unwrap();
    assert!(!exists);
}

// ── ClearTarget ───────────────────────────────────────────────────────────────

#[test]
fn clear_target_removes_current_target() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            A_Admin.SetTarget("Boss", 63, 1, true)
            ClearTarget()
            return UnitExists("target")
            "#,
        )
        .unwrap();
    assert!(!exists, "ClearTarget should remove target");
}

#[test]
fn clear_target_returns_true_when_existing_target_is_cleared() {
    let env = env();
    let cleared: bool = env
        .eval(
            r#"
            A_Admin.SetTarget("Boss", 63, 1, true)
            return ClearTarget()
            "#,
        )
        .unwrap();
    assert!(cleared);
}

#[test]
fn clear_target_returns_false_when_no_target_exists() {
    let env = env();
    let cleared: bool = env.eval("return ClearTarget()").unwrap();
    assert!(!cleared);
}

// ── TargetLastTarget ──────────────────────────────────────────────────────────

#[test]
fn target_last_target_swaps_after_clear_target() {
    let env = env();
    let (before, after): (bool, bool) = env
        .eval(
            r#"
            A_Admin.SetTarget("Boss", 63, 1, true)
            -- Current: Boss. No previous yet.
            ClearTarget()
            -- Current: none. Previous: Boss.
            local before = UnitExists("target")  -- false
            TargetLastTarget()
            -- Current: Boss again. Previous: none.
            local after = UnitExists("target")   -- true
            return before, after
            "#,
        )
        .unwrap();
    assert!(!before, "target should be gone after ClearTarget");
    assert!(after, "TargetLastTarget should restore previous target");
}

#[test]
fn target_last_target_no_previous_is_noop() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            TargetLastTarget()
            return UnitExists("target")
            "#,
        )
        .unwrap();
    assert!(!exists);
}

#[test]
fn target_unit_snapshots_previous_target() {
    let env = env();
    let name_after_swap: String = env
        .eval(
            r#"
            A_Admin.SetTarget("Alpha", 63, 1, true)
            A_Admin.SetTarget("Beta", 60, 1, true)
            -- Now target both sequentially via TargetUnit
            TargetUnit("target")   -- current=Beta, previous=Beta (no-op replay)
            ClearTarget()          -- current=nil, previous=Beta
            TargetLastTarget()     -- current=Beta
            return UnitName("target")
            "#,
        )
        .unwrap();
    assert_eq!(name_after_swap, "Beta");
}

// ── TargetNearestFriend ───────────────────────────────────────────────────────

#[test]
fn target_nearest_friend_picks_first_party_member() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            A_Admin.SetPartySize(2)
            A_Admin.SetPartyMember(1, "Healer", 5, 80)
            A_Admin.SetPartyMember(2, "Tank", 2, 80)
            TargetNearestFriend()
            return UnitName("target")
            "#,
        )
        .unwrap();
    assert_eq!(name, "Healer", "should pick first party member");
}

#[test]
fn target_nearest_friend_noop_when_no_party() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            TargetNearestFriend()
            return UnitExists("target")
            "#,
        )
        .unwrap();
    assert!(!exists);
}

// ── TargetNearestEnemy ────────────────────────────────────────────────────────

#[test]
fn target_nearest_enemy_noop_on_empty_pool() {
    let env = env();
    let exists: bool = env
        .eval(
            r#"
            TargetNearestEnemy()
            return UnitExists("target")
            "#,
        )
        .unwrap();
    assert!(!exists, "empty enemy pool should be a no-op");
}

#[test]
fn target_nearest_enemy_picks_first_after_admin_seeding() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            A_Admin.SetEnemyPool(
                {name="Kyveza", level=63, class_index=1},
                {name="Minion",  level=60, class_index=1}
            )
            TargetNearestEnemy()
            return UnitName("target")
            "#,
        )
        .unwrap();
    assert_eq!(name, "Kyveza", "should pick first enemy pool entry");
}

#[test]
fn target_nearest_enemy_enemy_pool_is_replaceable() {
    let env = env();
    let (first, second): (String, String) = env
        .eval(
            r#"
            A_Admin.SetEnemyPool({name="Alpha", level=63, class_index=1})
            TargetNearestEnemy()
            local first = UnitName("target")

            ClearTarget()
            A_Admin.SetEnemyPool({name="Beta", level=60, class_index=1})
            TargetNearestEnemy()
            local second = UnitName("target")
            return first, second
            "#,
        )
        .unwrap();
    assert_eq!(first, "Alpha");
    assert_eq!(second, "Beta");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn raid_target_icons_update_real_blizzard_target_frame_consumer() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape(
            &["Blizzard_UnitFrame"],
            &[],
            |env, _loaded| {
                env.exec(
                    r#"
                    assert(type(TargetFrameMixin.OnEvent) == "function")
                    assert(type(TargetFrameMixin.UpdateRaidTargetIcon) == "function")
                    A_Admin.SetEnemyPool({name="IconAlpha"}, {name="IconBeta"})
                    TargetUnit("enemy1")
                    local firstGuid = UnitGUID("target")

                    local frame = CreateFrame("Frame", nil, UIParent)
                    Mixin(frame, TargetFrameMixin)
                    frame.unit = "target"
                    frame.TargetFrameContent = CreateFrame("Frame", nil, frame)
                    local content = frame.TargetFrameContent
                    content.TargetFrameContentContextual = CreateFrame("Frame", nil, content)
                    local contextual = content.TargetFrameContentContextual
                    contextual.RaidTargetIcon = contextual:CreateTexture(nil, "OVERLAY")
                    local icon = contextual.RaidTargetIcon
                    icon:SetTexture("Interface\\TargetingFrame\\UI-RaidTargetingIcons")
                    frame:SetScript("OnEvent", TargetFrameMixin.OnEvent)
                    frame:RegisterEvent("RAID_TARGET_UPDATE")

                    local observed = {}
                    frame:HookScript("OnEvent", function(self, event)
                        assert(event == "RAID_TARGET_UPDATE")
                        observed[#observed + 1] = {
                            index = GetRaidTargetIndex(self.unit),
                            shown = icon:IsShown(),
                            coords = {icon:GetTexCoord()},
                        }
                    end)
                    local function assert_icon(index, eventCount)
                        assert(#observed == eventCount, "missing or duplicate icon event")
                        local last = observed[eventCount]
                        assert(last.index == index, "event saw stale unit state")
                        assert(last.shown == (index ~= nil), "vendor icon visibility")
                        assert(GetRaidTargetIndex("target") == index)
                        assert(icon:IsShown() == (index ~= nil))
                        if index then
                            local expected = index == 1
                                and {0, 0, 0, 0.25, 0.25, 0, 0.25, 0.25}
                                or {0.75, 0.25, 0.75, 0.5, 1, 0.25, 1, 0.5}
                            assert(#last.coords == 8)
                            for i, value in ipairs(expected) do
                                assert(last.coords[i] == value, "vendor sprite coordinate " .. i)
                            end
                        end
                    end

                    SetRaidTarget("target", 1)
                    assert_icon(1, 1)
                    SetRaidTarget("target", 8)
                    assert_icon(8, 2)
                    SetRaidTarget("target", 0)
                    assert_icon(nil, 3)

                    SetRaidTargetIcon("target", 1)
                    assert_icon(1, 4)
                    SetRaidTargetIcon("target", 1)
                    assert_icon(nil, 5)
                    SetRaidTarget("target", 8)
                    assert_icon(8, 6)

                    TargetUnit("enemy2")
                    assert(UnitGUID("target") ~= firstGuid, "fixture needs distinct identities")
                    frame:UpdateRaidTargetIcon()
                    assert(GetRaidTargetIndex("target") == nil and not icon:IsShown())
                    SetRaidTarget("target", 1)
                    assert_icon(1, 7)
                    TargetUnit("enemy1")
                    assert(UnitGUID("target") == firstGuid)
                    frame:UpdateRaidTargetIcon()
                    assert(GetRaidTargetIndex("target") == 8 and icon:IsShown())
                    local coords = {icon:GetTexCoord()}
                    assert(coords[1] == 0.75 and coords[2] == 0.25)
                    assert(coords[7] == 1 and coords[8] == 0.5)
                    frame:UnregisterAllEvents()
                    "#,
                )
                .expect("real TargetFrame event handler consumes assigned unit raid icons");
            },
        );
    });
}

#[test]
fn raid_target_icons_assign_move_replace_and_clear() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetPartySize(2)
        assert(GetRaidTargetIndex('player') == nil)
        assert(select('#', GetRaidTargetIndex('player')) == 1)
        assert(select('#', SetRaidTarget('player', 8)) == 0)
        assert(GetRaidTargetIndex('self') == 8)
        SetRaidTarget('party1', 7)
        assert(GetRaidTargetIndex('player') == 8)
        assert(GetRaidTargetIndex('party1') == 7)
        SetRaidTargetIcon('party1', 8)
        assert(GetRaidTargetIndex('player') == nil)
        assert(GetRaidTargetIndex('party1') == 8)
        SetRaidTarget('party2', 7)
        assert(GetRaidTargetIndex('party2') == 7)
        SetRaidTarget('party1', 0)
        assert(GetRaidTargetIndex('party1') == nil)
        assert(GetRaidTargetIndex('party2') == 7)
        for index = 1, 8 do
            SetRaidTarget('player', index)
            assert(GetRaidTargetIndex('player') == index)
        end
        SetRaidTarget('player', 0)
        assert(GetRaidTargetIndex('player') == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn raid_target_icons_follow_guid_aliases_not_target_slots() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetPartySize(2)
        TargetUnit('player')
        assert(UnitGUID('target') == UnitGUID('player'), 'player snapshot GUID')
        SetRaidTarget('target', 1)
        TargetUnit('party1')
        assert(UnitGUID('target') == UnitGUID('party1'), 'party1 snapshot GUID')
        assert(UnitGUID('target') ~= UnitGUID('player'), 'distinct player and party GUIDs')
        FocusUnit('target')
        SetRaidTarget('FOCUS', 2)
        assert(GetRaidTargetIndex('party1') == 2)
        assert(GetRaidTargetIndex('target') == 2)
        assert(GetRaidTargetIndex('player') == 1)
        TargetUnit('party2')
        assert(UnitGUID('target') == UnitGUID('party2'), 'party2 snapshot GUID')
        assert(GetRaidTargetIndex('target') == nil)
        assert(GetRaidTargetIndex('focus') == 2)
        SetRaidTarget('target', 3)
        TargetNearestFriend()
        assert(UnitGUID('target') == UnitGUID('party1'), 'nearest friend snapshot GUID')
        assert(GetRaidTargetIndex('target') == 2)
        TargetUnit('enemy1')
        SetRaidTarget('target', 4)
        ClearTarget()
        assert(GetRaidTargetIndex('target') == nil)
        TargetUnit('enemy1')
        assert(GetRaidTargetIndex('target') == 4)
        assert(GetRaidTargetIndex('party2') == 3)
    "#,
    )
    .unwrap();
}

#[test]
fn raid_target_icons_reject_invalid_indices_without_side_effects() {
    let env = env();
    env.exec(r#"
        SetRaidTarget('player', 5)
        raidIconEvents = 0
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('RAID_TARGET_UPDATE')
        frame:SetScript('OnEvent', function() raidIconEvents = raidIconEvents + 1 end)
        local invalid = {false, true, '2', 'bad', {}, function() end, -1, 9, 1.5, 0/0, math.huge, -math.huge}
        for _, value in ipairs(invalid) do
            assert(not pcall(SetRaidTarget, 'player', value), 'invalid index accepted')
            assert(GetRaidTargetIndex('player') == 5, 'invalid input changed icon')
        end
        assert(not pcall(SetRaidTarget, 'player'))
        assert(not pcall(SetRaidTarget, 'player', nil))
        assert(raidIconEvents == 0)
    "#).unwrap();
    assert!(
        !env.state()
            .borrow()
            .events
            .pending()
            .iter()
            .any(|event| event.name == "RAID_TARGET_UPDATE")
    );
}

#[test]
fn raid_target_icons_notify_once_after_mutation_without_queued_duplicate() {
    let env = env();
    env.exec(
        r#"
        raidIconEvents = 0
        raidIconObserved = {}
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('RAID_TARGET_UPDATE')
        frame:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'RAID_TARGET_UPDATE' and select('#', ...) == 0)
            raidIconEvents = raidIconEvents + 1
            raidIconObserved[raidIconEvents] = GetRaidTargetIndex('player') or 0
        end)
        SetRaidTarget('player', 6)
        assert(raidIconEvents == 1 and raidIconObserved[1] == 6)
        SetRaidTarget('player', 6)
        assert(raidIconEvents == 2 and raidIconObserved[2] == 6)
        SetRaidTarget('player', 0)
        assert(raidIconEvents == 3 and raidIconObserved[3] == 0)
        assert(select('#', SetRaidTarget(nil)) == 0)
        assert(select('#', SetRaidTarget('unknown', {})) == 0)
        assert(select('#', SetRaidTarget('target')) == 0)
        assert(GetRaidTargetIndex(nil) == nil and GetRaidTargetIndex('unknown') == nil)
        assert(select('#', GetRaidTargetIndex(nil)) == 1)
        assert(raidIconEvents == 3)
    "#,
    )
    .unwrap();
    assert!(
        !env.state()
            .borrow()
            .events
            .pending()
            .iter()
            .any(|event| event.name == "RAID_TARGET_UPDATE")
    );
}
