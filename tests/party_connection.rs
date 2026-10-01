//! Shared party connection input contract; no native execution claim.

use wow_ui_sim::lua_api::WowLuaEnv;

fn listen_for_party_connection() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.eval::<()>(
        r#"
        assert(type(A_Admin.SetPartyMemberConnected) == "function",
            "A_Admin.SetPartyMemberConnected must be implemented")
        A_Admin.SetPartySize(2)
        connectionEvents = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("UNIT_CONNECTION")
        listener:SetScript("OnEvent", function(_, event, ...)
            local unitTarget, isConnected = ...
            connectionEvents[#connectionEvents + 1] = {
                event = event,
                argumentCount = select('#', ...),
                unitTarget = unitTarget,
                isConnected = isConnected,
                queriedConnected = UnitIsConnected(unitTarget),
                hasOfflineMember = GroupHasOfflineMember(),
            }
        end)
        "#,
    )
    .expect("register actual UNIT_CONNECTION listener");
    env
}

#[test]
fn both_edges_publish_exact_payload_and_post_state_synchronously() {
    let env = listen_for_party_connection();
    env.eval::<()>(
        r#"
        assert(UnitIsConnected("party1") == true)
        assert(UnitIsConnected("party2") == true)
        assert(GroupHasOfflineMember() == false)

        A_Admin.SetPartyMemberConnected(1, false)
        assert(#connectionEvents == 1, "disconnect must dispatch before returning")
        local disconnected = connectionEvents[1]
        assert(disconnected.event == "UNIT_CONNECTION")
        assert(disconnected.argumentCount == 2)
        assert(disconnected.unitTarget == "party1")
        assert(disconnected.isConnected == false)
        assert(disconnected.queriedConnected == false)
        assert(disconnected.hasOfflineMember == true)
        assert(UnitIsConnected("party1") == false)
        assert(GroupHasOfflineMember() == true)

        A_Admin.SetPartyMemberConnected(1, false)
        assert(#connectionEvents == 1, "repeat disconnect must not dispatch")
        A_Admin.SetPartyMemberConnected(1, true)
        assert(#connectionEvents == 2, "reconnect must dispatch before returning")
        local reconnected = connectionEvents[2]
        assert(reconnected.event == "UNIT_CONNECTION")
        assert(reconnected.argumentCount == 2)
        assert(reconnected.unitTarget == "party1")
        assert(reconnected.isConnected == true)
        assert(reconnected.queriedConnected == true)
        assert(reconnected.hasOfflineMember == false)
        assert(UnitIsConnected("party1") == true)
        assert(GroupHasOfflineMember() == false)

        A_Admin.SetPartyMemberConnected(1, true)
        assert(#connectionEvents == 2, "repeat reconnect must not dispatch")
        "#,
    )
    .expect("both connection edges and unchanged input");
}

#[test]
fn absent_valid_party_index_is_a_no_op() {
    let env = listen_for_party_connection();
    env.eval::<()>(
        r#"
        assert(UnitExists("party3") == false)
        A_Admin.SetPartyMemberConnected(3, false)
        A_Admin.SetPartyMemberConnected(3, true)
        assert(#connectionEvents == 0)
        assert(UnitExists("party3") == false)
        assert(GetNumSubgroupMembers() == 2)
        assert(UnitIsConnected("party1") == true)
        assert(UnitIsConnected("party2") == true)
        assert(GroupHasOfflineMember() == false)
        "#,
    )
    .expect("absent member input succeeds without mutation or event");
}

#[test]
fn reconnect_keeps_aggregate_offline_while_another_member_is_disconnected() {
    let env = listen_for_party_connection();
    env.eval::<()>(
        r#"
        A_Admin.SetPartyMemberConnected(1, false)
        assert(#connectionEvents == 1)
        A_Admin.SetPartyMemberConnected(2, false)
        assert(#connectionEvents == 2)
        local second = connectionEvents[2]
        assert(second.argumentCount == 2)
        assert(second.unitTarget == "party2")
        assert(second.isConnected == false)
        assert(second.queriedConnected == false)
        assert(second.hasOfflineMember == true)

        A_Admin.SetPartyMemberConnected(1, true)
        assert(#connectionEvents == 3)
        local firstReconnected = connectionEvents[3]
        assert(firstReconnected.unitTarget == "party1")
        assert(firstReconnected.isConnected == true)
        assert(firstReconnected.queriedConnected == true)
        assert(firstReconnected.hasOfflineMember == true)
        assert(UnitIsConnected("party1") == true)
        assert(UnitIsConnected("party2") == false)
        assert(GroupHasOfflineMember() == true)
        "#,
    )
    .expect("aggregate queries include every current party member");
}

#[test]
fn shrinking_discards_removed_member_connection_state() {
    let env = listen_for_party_connection();
    env.eval::<()>(
        r#"
        A_Admin.SetPartyMemberConnected(2, false)
        assert(UnitIsConnected("party2") == false)
        assert(GroupHasOfflineMember() == true)
        A_Admin.SetPartySize(1)
        assert(UnitExists("party2") == false)
        assert(UnitIsConnected("party1") == true)
        assert(GroupHasOfflineMember() == false)
        A_Admin.SetPartySize(2)
        assert(UnitExists("party2") == true)
        assert(UnitIsConnected("party2") == true)
        assert(GroupHasOfflineMember() == false)
        "#,
    )
    .expect("shrunk roster loses offline state and regrows connected");
}

#[test]
fn removing_party_discards_offline_state_before_regrowth() {
    let env = listen_for_party_connection();
    env.eval::<()>(
        r#"
        A_Admin.SetPartyMemberConnected(1, false)
        A_Admin.SetPartyMemberConnected(2, false)
        assert(GroupHasOfflineMember() == true)
        A_Admin.SetPartySize(0)
        assert(IsInGroup() == false)
        assert(GroupHasOfflineMember() == false)
        A_Admin.SetPartySize(2)
        assert(UnitIsConnected("party1") == true)
        assert(UnitIsConnected("party2") == true)
        assert(GroupHasOfflineMember() == false)
        "#,
    )
    .expect("party removal loses all connection state");
}

#[test]
fn inactive_retained_members_are_not_connection_inputs() {
    let env = listen_for_party_connection();
    env.state().borrow_mut().party_group_active = false;
    env.eval::<()>(
        r#"
        A_Admin.SetPartyMemberConnected(1, false)
        assert(#connectionEvents == 0)
        assert(UnitIsConnected("party1") == false)
        assert(GroupHasOfflineMember() == false)
        AcceptGroup()
        assert(UnitIsConnected("party1") == true)
        assert(GroupHasOfflineMember() == false)
        "#,
    )
    .expect("inactive retained members remain connected without dispatch");
}

#[test]
fn party_disconnect_preserves_non_party_connection_queries() {
    let env = listen_for_party_connection();
    env.eval::<()>(
        r#"
        A_Admin.SetTarget("Target", 80, 1, true)
        A_Admin.SetFocus("Focus", 80, 1, false)
        A_Admin.SetPartyMemberConnected(1, false)
        assert(UnitIsConnected("party1") == false)
        for _, unit in ipairs({"player", "pet", "vehicle", "target", "focus", "raid1"}) do
            assert(UnitIsConnected(unit) == true, unit)
        end
        assert(UnitIsConnected("unknown") == false)
        A_Admin.ClearTarget()
        A_Admin.ClearFocus()
        assert(UnitIsConnected("target") == false)
        assert(UnitIsConnected("focus") == false)
        "#,
    )
    .expect("party connectivity leaves existing other-token semantics unchanged");
}

#[test]
fn connection_state_and_dispatch_are_isolated_per_environment() {
    let first = listen_for_party_connection();
    let second = listen_for_party_connection();
    first
        .eval::<()>(
            r#"
            A_Admin.SetPartyMemberConnected(1, false)
            assert(#connectionEvents == 1)
            assert(UnitIsConnected("party1") == false)
            assert(GroupHasOfflineMember() == true)
            "#,
        )
        .expect("disconnect first environment");
    second
        .eval::<()>(
            r#"
            assert(#connectionEvents == 0)
            assert(UnitIsConnected("party1") == true)
            assert(GroupHasOfflineMember() == false)
            A_Admin.SetPartyMemberConnected(2, false)
            assert(#connectionEvents == 1)
            assert(connectionEvents[1].unitTarget == "party2")
            assert(UnitIsConnected("party1") == true)
            assert(UnitIsConnected("party2") == false)
            assert(GroupHasOfflineMember() == true)
            "#,
        )
        .expect("second environment has independent input and listener");
    first
        .eval::<()>(
            r#"
            assert(#connectionEvents == 1)
            assert(UnitIsConnected("party1") == false)
            assert(UnitIsConnected("party2") == true)
            assert(GroupHasOfflineMember() == true)
            "#,
        )
        .expect("second input does not mutate first environment");
}
