use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn socket_info_queries_reflect_configured_state() {
    let env = env();
    let (
        num_sockets,
        socket_type_three,
        has_existing_info,
        existing_icon_ok,
        existing_match_ok,
        existing_link_ok,
        has_new_info,
        new_icon_ok,
        new_match_ok,
        new_link_ok,
    ): (
        i32,
        String,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            C_ItemSocketInfo._state.numSockets = 3
            C_ItemSocketInfo._state.socketTypes = {
                [1] = "Red",
                [2] = "Yellow",
                [3] = "Blue",
            }
            C_ItemSocketInfo._state.existingSockets = {
                [1] = { name = "Ruby", icon = 111, gemMatchesSocket = true, link = "item:111" },
            }
            C_ItemSocketInfo._state.newSockets = {
                [2] = { name = "Sapphire", icon = 222, gemMatchesSocket = false, link = "item:222" },
            }

            local existingName, existingIcon, existingMatch = C_ItemSocketInfo.GetExistingSocketInfo(1)
            local newName, newIcon, newMatch = C_ItemSocketInfo.GetNewSocketInfo(2)
            return C_ItemSocketInfo.GetNumSockets(),
                   C_ItemSocketInfo.GetSocketTypes(3),
                   existingName == "Ruby",
                   existingIcon == 111,
                   existingMatch == true,
                   C_ItemSocketInfo.GetExistingSocketLink(1) == "item:111",
                   newName == "Sapphire",
                   newIcon == 222,
                   newMatch == false,
                   C_ItemSocketInfo.GetNewSocketLink(2) == "item:222"
            "#,
        )
        .unwrap();

    assert_eq!(num_sockets, 3, "number of sockets should come from state");
    assert_eq!(
        socket_type_three, "Blue",
        "socket type should come from configured socketTypes"
    );
    assert!(has_existing_info, "existing socket name should be returned");
    assert!(
        existing_icon_ok,
        "existing socket icon should come from configured state"
    );
    assert!(
        existing_match_ok,
        "existing gemMatchesSocket should be returned"
    );
    assert!(existing_link_ok, "existing socket link should be returned");
    assert!(has_new_info, "new socket name should be returned");
    assert!(
        new_icon_ok,
        "new socket icon should come from configured state"
    );
    assert!(new_match_ok, "new gemMatchesSocket should be returned");
    assert!(new_link_ok, "new socket link should be returned");
}

#[test]
fn click_socket_button_applies_configured_proposal() {
    let env = env();
    let (clicked, selected_index_ok, has_new_socket, has_bound_gem, link_ok): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            C_ItemSocketInfo._state.numSockets = 2
            C_ItemSocketInfo._state.clickProposals = {
                [2] = {
                    name = "Bound Sapphire",
                    icon = 777,
                    gemMatchesSocket = true,
                    link = "item:777",
                    isBound = true,
                }
            }
            C_ItemSocketInfo._state.newSockets = {}
            C_ItemSocketInfo._state.hasBoundGemProposed = false

            local clicked = C_ItemSocketInfo.ClickSocketButton(2)
            local name, icon, gemMatchesSocket = C_ItemSocketInfo.GetNewSocketInfo(2)
            return clicked,
                   C_ItemSocketInfo._state.selectedSocketIndex == 2,
                   name == "Bound Sapphire" and icon == 777 and gemMatchesSocket == true,
                   C_ItemSocketInfo.HasBoundGemProposed(),
                   C_ItemSocketInfo.GetNewSocketLink(2) == "item:777"
            "#,
        )
        .unwrap();

    assert!(clicked, "click should succeed for a valid socket index");
    assert!(
        selected_index_ok,
        "click should track selected socket index in state"
    );
    assert!(
        has_new_socket,
        "click proposal should be copied into new socket info"
    );
    assert!(
        has_bound_gem,
        "bound click proposal should update HasBoundGemProposed"
    );
    assert!(
        link_ok,
        "click proposal link should be exposed via GetNewSocketLink"
    );
}

#[test]
fn accept_sockets_promotes_new_gems_and_clears_proposed_state() {
    let env = env();
    let (
        accepted,
        existing_one_ok,
        existing_two_ok,
        new_socket_cleared,
        no_bound_gem,
        accept_count_ok,
    ): (bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            C_ItemSocketInfo._state.numSockets = 2
            C_ItemSocketInfo._state.existingSockets = {
                [1] = { name = "Old Gem", icon = 10, gemMatchesSocket = false, link = "item:10" },
            }
            C_ItemSocketInfo._state.newSockets = {
                [1] = { name = "New Gem", icon = 20, gemMatchesSocket = true, link = "item:20", isBound = true },
                [2] = { name = "Second Gem", icon = 30, gemMatchesSocket = false, link = "item:30", isBound = false },
            }
            C_ItemSocketInfo._state.hasBoundGemProposed = true
            C_ItemSocketInfo._state.acceptCount = 0

            local accepted = C_ItemSocketInfo.AcceptSockets()
            local oneName, oneIcon, oneMatch = C_ItemSocketInfo.GetExistingSocketInfo(1)
            local twoName, twoIcon, twoMatch = C_ItemSocketInfo.GetExistingSocketInfo(2)
            local newName = C_ItemSocketInfo.GetNewSocketInfo(1)
            return accepted,
                   oneName == "New Gem" and oneIcon == 20 and oneMatch == true,
                   twoName == "Second Gem" and twoIcon == 30 and twoMatch == false,
                   newName == nil,
                   not C_ItemSocketInfo.HasBoundGemProposed(),
                   C_ItemSocketInfo._state.acceptCount == 1
            "#,
        )
        .unwrap();

    assert!(accepted, "AcceptSockets should report success");
    assert!(
        existing_one_ok,
        "AcceptSockets should replace existing socket info with proposed gem"
    );
    assert!(
        existing_two_ok,
        "AcceptSockets should move all proposed gems into existing sockets"
    );
    assert!(
        new_socket_cleared,
        "AcceptSockets should clear proposed new sockets"
    );
    assert!(
        no_bound_gem,
        "AcceptSockets should clear HasBoundGemProposed after apply"
    );
    assert!(accept_count_ok, "AcceptSockets should track action count");
}

#[test]
fn close_socket_info_marks_closed_and_clears_pending_gems() {
    let env = env();
    let (first_close, second_close, is_closed, pending_cleared, close_count_ok): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            C_ItemSocketInfo._state.isOpen = true
            C_ItemSocketInfo._state.newSockets = {
                [1] = { name = "Pending Gem", icon = 55, gemMatchesSocket = false, isBound = true },
            }
            C_ItemSocketInfo._state.hasBoundGemProposed = true
            C_ItemSocketInfo._state.closeCount = 0

            local firstClose = C_ItemSocketInfo.CloseSocketInfo()
            local secondClose = C_ItemSocketInfo.CloseSocketInfo()
            local pendingName = C_ItemSocketInfo.GetNewSocketInfo(1)
            return firstClose,
                   secondClose,
                   C_ItemSocketInfo._state.isOpen == false,
                   pendingName == nil and not C_ItemSocketInfo.HasBoundGemProposed(),
                   C_ItemSocketInfo._state.closeCount == 2
            "#,
        )
        .unwrap();

    assert!(
        first_close,
        "first close should report previously open state"
    );
    assert!(
        !second_close,
        "second close should report that socket UI was already closed"
    );
    assert!(is_closed, "close should mark socket UI as closed");
    assert!(
        pending_cleared,
        "close should clear pending gems and bound-gem flag"
    );
    assert!(close_count_ok, "close should track action count");
}

#[test]
fn artifact_relic_detection_supports_item_ids_and_links() {
    let env = env();
    let (number_match, link_match, table_match, non_match, global_match): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            C_ItemSocketInfo._state.artifactRelicItemIDs = {
                [12345] = true,
            }
            return C_ItemSocketInfo.IsArtifactRelicItem(12345),
                   C_ItemSocketInfo.IsArtifactRelicItem("item:12345::::::::70:::::::"),
                   C_ItemSocketInfo.IsArtifactRelicItem({ itemID = 12345 }),
                   not C_ItemSocketInfo.IsArtifactRelicItem(99999),
                   C_ItemSocketInfo.IsArtifactRelicItem("|Hitem:12345::::::::70:::::::|h[Test Relic]|h")
            "#,
        )
        .unwrap();

    assert!(number_match, "numeric item IDs should be supported");
    assert!(link_match, "item links should be parsed for item ID");
    assert!(
        table_match,
        "table payloads with itemID should be supported"
    );
    assert!(non_match, "unknown item IDs should return false");
    assert!(global_match, "legacy global should share the Rust implementation");
}

#[test]
fn socket_item_flags_and_ui_type_are_state_backed() {
    let env = env();
    let (ui_type, item_name, item_icon, quality, refundable, bound_tradeable): (
        i32,
        String,
        i32,
        i32,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            C_ItemSocketInfo._state.uiType = 42
            C_ItemSocketInfo._state.itemInfo = {
                name = "Socketed Helm",
                icon = 901,
                quality = 4,
                isRefundable = true,
                isBoundTradeable = true,
            }
            local name, icon, quality = C_ItemSocketInfo.GetSocketItemInfo()
            return C_ItemSocketInfo.GetCurrUIType(),
                   name,
                   icon,
                   quality,
                   C_ItemSocketInfo.GetSocketItemRefundable(),
                   C_ItemSocketInfo.GetSocketItemBoundTradeable()
            "#,
        )
        .unwrap();

    assert_eq!(ui_type, 42, "current UI type should come from state");
    assert_eq!(
        item_name, "Socketed Helm",
        "item name should come from state"
    );
    assert_eq!(item_icon, 901, "item icon should come from state");
    assert_eq!(quality, 4, "item quality should come from state");
    assert!(refundable, "refundable flag should come from state");
    assert!(
        bound_tradeable,
        "bound-tradeable flag should come from state"
    );
}
