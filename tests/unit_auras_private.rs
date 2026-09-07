use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(feature = "retail-12-1-0")]
fn blocked_event_env() -> WowLuaEnv {
    let env = env();
    env.exec(
        r#"
        blockedAll, blockedPlayer = {}, {}
        local all = CreateFrame('Frame')
        all:RegisterEvent('UNIT_AURA_BLOCKED')
        all:SetScript('OnEvent', function(_, event, unit, auraID)
            blockedAll[#blockedAll + 1] = { event, unit, auraID }
        end)
        local player = CreateFrame('Frame')
        player:RegisterUnitEvent('UNIT_AURA_BLOCKED', 'player')
        player:SetScript('OnEvent', function(_, event, unit, auraID)
            blockedPlayer[#blockedPlayer + 1] = { event, unit, auraID }
        end)
        assert(not pcall(all.RegisterEvent, all, 'WOWLESS_NOPE'))
    "#,
    )
    .expect("register the documented aura-blocked event");
    env
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn unit_aura_blocked_supports_registered_and_unit_filtered_dispatch() {
    let env = blocked_event_env();
    for (unit, aura_id) in [("player", 92345.0), ("target", 92346.0)] {
        env.fire_event_with_args(
            "UNIT_AURA_BLOCKED",
            &[env.lua_string(unit), rilua::Val::Num(aura_id)],
        )
        .expect("dispatch aura-blocked payload");
    }
    assert_blocked_event_delivery(&env);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn native_unit_event_dispatch_respects_unit_filter() {
    let env = env();
    env.exec(
        r#"
        local allEvents, playerEvents, targetEvents = {}, {}, {}
        local function record(events)
            return function(_, event, unit, spellID)
                events[#events + 1] = { event, unit, spellID }
            end
        end
        local all = CreateFrame('Frame')
        all:RegisterEvent('UNIT_SPELLCAST_START')
        all:SetScript('OnEvent', record(allEvents))
        local player = CreateFrame('Frame')
        player:RegisterUnitEvent('UNIT_SPELLCAST_START', 'player')
        player:SetScript('OnEvent', record(playerEvents))
        local target = CreateFrame('Frame')
        target:RegisterUnitEvent('UNIT_SPELLCAST_START', 'target')
        target:SetScript('OnEvent', record(targetEvents))
        local current = C_SpecializationInfo.GetSpecialization()
        assert(C_SpecializationInfo.SetSpecialization(current == 1 and 2 or 1))
        assert(#allEvents == 1 and #playerEvents == 1 and #targetEvents == 0)
        for _, events in ipairs({ allEvents, playerEvents }) do
            assert(events[1][1] == 'UNIT_SPELLCAST_START')
            assert(events[1][2] == 'player' and events[1][3] == 200749)
        end
    "#,
    )
    .expect("native specialization cast preserves payload and unit filtering");
}

#[cfg(feature = "retail-12-1-0")]
fn assert_blocked_event_delivery(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(#blockedAll == 2 and #blockedPlayer == 1)
        assert(blockedPlayer[1][1] == 'UNIT_AURA_BLOCKED')
        assert(blockedPlayer[1][2] == 'player' and blockedPlayer[1][3] == 92345)
        assert(blockedAll[2][2] == 'target' and blockedAll[2][3] == 92346)
    "#,
    )
    .expect("preserve payload and unit filtering");
}

#[test]
fn private_aura_anchor_callbacks_and_state_are_tracked() {
    let env = env();
    let (
        ids_increment,
        added_callback_ok,
        anchor_list_ok,
        filtered_list_ok,
        removed_callback_ok,
        anchors_after_remove_ok,
    ): (bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local added = {}
            local removed = {}
            C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(anchorInfo)
                added[#added + 1] = anchorInfo
            end)
            C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(anchorID)
                removed[#removed + 1] = anchorID
            end)

            local anchorID1 = C_UnitAurasPrivate._AddPrivateAuraAnchorForTest({
                unitToken = "player",
                auraIndex = 1,
                isBuff = true,
                maxAuras = 5,
            })
            local anchorID2 = C_UnitAurasPrivate._AddPrivateAuraAnchorForTest({
                unitToken = "target",
                auraIndex = 2,
                isBuff = false,
                maxAuras = 3,
            })

            local allAnchors = C_UnitAurasPrivate.GetPrivateAuraAnchors()
            local playerAnchors = C_UnitAurasPrivate.GetPrivateAuraAnchors("player")

            local removedOk = C_UnitAurasPrivate._RemovePrivateAuraAnchorForTest(anchorID1)
            local anchorsAfterRemove = C_UnitAurasPrivate.GetPrivateAuraAnchors()

            return anchorID1 > 0 and anchorID2 > anchorID1,
                   #added == 2 and added[1].anchorID == anchorID1 and added[1].unitToken == "player",
                   #allAnchors == 2 and allAnchors[2].anchorID == anchorID2,
                   #playerAnchors == 1 and playerAnchors[1].anchorID == anchorID1,
                   removedOk and #removed == 1 and removed[1] == anchorID1,
                   #anchorsAfterRemove == 1 and anchorsAfterRemove[1].anchorID == anchorID2
            "#,
        )
        .unwrap();

    assert!(
        ids_increment,
        "anchor IDs should be generated incrementally"
    );
    assert!(
        added_callback_ok,
        "added callback should receive copied anchor info"
    );
    assert!(
        anchor_list_ok,
        "GetPrivateAuraAnchors should return all tracked anchors"
    );
    assert!(
        filtered_list_ok,
        "GetPrivateAuraAnchors(unit) should filter by unit token"
    );
    assert!(
        removed_callback_ok,
        "remove callback should be fired with removed anchor ID"
    );
    assert!(
        anchors_after_remove_ok,
        "removed anchors should no longer appear in tracked state"
    );
}

#[test]
fn private_warning_and_show_dispel_callbacks_are_state_backed() {
    let env = env();
    let (warning_frame_stored, dispel_callback_ok, last_show_type_recorded): (bool, bool, bool) =
        env.eval(
            r#"
            local warningFrame = CreateFrame("Frame", "UnitAurasPrivateWarningFrame")
            C_UnitAurasPrivate.SetPrivateWarningTextFrame(warningFrame)

            local dispelCallbackOk = true
            local lastShowTypeRecorded = true
            if rawget(C_UnitAuras, "TriggerPrivateAuraShowDispelType") ~= nil then
                local dispelCalls = {}
                C_UnitAurasPrivate.SetShowDispelTypeCallback(function(showDispelType)
                    dispelCalls[#dispelCalls + 1] = showDispelType
                end)

                C_UnitAuras.TriggerPrivateAuraShowDispelType(true)
                C_UnitAuras.TriggerPrivateAuraShowDispelType(false)

                dispelCallbackOk = #dispelCalls == 2
                    and dispelCalls[1] == true
                    and dispelCalls[2] == false
                lastShowTypeRecorded = C_UnitAurasPrivate._state.lastShowDispelType == false
            end

            return C_UnitAurasPrivate._state.warningTextFrame == warningFrame,
                   dispelCallbackOk,
                   lastShowTypeRecorded
            "#,
        )
        .unwrap();

    assert!(
        warning_frame_stored,
        "SetPrivateWarningTextFrame should persist the warning frame reference"
    );
    assert!(
        dispel_callback_ok,
        "available TriggerPrivateAuraShowDispelType should invoke registered callback"
    );
    assert!(
        last_show_type_recorded,
        "available show-dispel trigger should update private aura state"
    );
}

#[test]
fn private_aura_update_callbacks_and_lookup_queries_are_stateful() {
    let env = env();
    let (update_callback_fired, lookup_data_ok, read_is_copy): (bool, bool, bool) = env
        .eval(
            r#"
            C_UnitAurasPrivate._state.privateAurasByUnit["player"] = {
                { auraInstanceID = 11, spellId = 1001, applications = 1 },
                { auraInstanceID = 22, spellId = 1002, applications = 2 },
            }
            C_UnitAurasPrivate._state.auraDataByUnit["player"] = {
                [11] = { auraInstanceID = 11, spellId = 1001, applications = 1 },
                [22] = { auraInstanceID = 22, spellId = 1002, applications = 2 },
            }

            local callbackCalls = 0
            local lastPrivateSource = nil
            local sawFullUpdate = false
            C_UnitAurasPrivate.AddPrivateAuraUpdateCallback("player", function(privateSource, updateInfo)
                callbackCalls = callbackCalls + 1
                lastPrivateSource = privateSource
                sawFullUpdate = updateInfo and updateInfo.isFullUpdate == true
            end)

            local fired = C_UnitAurasPrivate._TriggerPrivateAuraUpdate("player", "Private", { isFullUpdate = true })
            local allAuras = C_UnitAurasPrivate.GetAllPrivateAuras("player")
            local auraByID = C_UnitAurasPrivate.GetAuraDataByAuraInstanceIDPrivate("player", 11)

            allAuras[1].spellId = 9999
            local secondRead = C_UnitAurasPrivate.GetAllPrivateAuras("player")

            return fired == 1 and callbackCalls == 1 and lastPrivateSource == "Private" and sawFullUpdate,
                   #allAuras == 2 and auraByID and auraByID.spellId == 1001,
                   secondRead[1].spellId == 1001
            "#,
        )
        .unwrap();

    assert!(
        update_callback_fired,
        "registered update callbacks should fire for matching unit"
    );
    assert!(
        lookup_data_ok,
        "private aura query methods should read configured unit aura state"
    );
    assert!(
        read_is_copy,
        "GetAllPrivateAuras should return a copy, not mutable internal state"
    );
}
