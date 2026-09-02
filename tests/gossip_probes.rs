//! Integration tests for `src/lua_api/globals/real/gossip_probes.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── Default (no active dialog) ────────────────────────────────────────────────

#[test]
fn gossip_probes_default_to_zero() {
    let env = env();
    let (options, available, active): (i32, i32, i32) = env
        .eval(
            "return GetGossipNumOptions(), GetGossipNumAvailableQuests(), GetGossipNumActiveQuests()",
        )
        .unwrap();
    assert_eq!(options, 0);
    assert_eq!(available, 0);
    assert_eq!(active, 0);
}

// ── Seeded counts ─────────────────────────────────────────────────────────────

#[test]
fn gossip_probes_read_state_counts() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.gossip.active = true;
        st.gossip.num_options = 3;
        st.gossip.num_available_quests = 2;
        st.gossip.num_active_quests = 1;
    }
    let (options, available, active): (i32, i32, i32) = env
        .eval(
            "return GetGossipNumOptions(), GetGossipNumAvailableQuests(), GetGossipNumActiveQuests()",
        )
        .unwrap();
    assert_eq!(options, 3);
    assert_eq!(available, 2);
    assert_eq!(active, 1);
}

// ── Event dispatch ────────────────────────────────────────────────────────────

#[test]
fn gossip_show_event_reaches_listeners() {
    let env = env();
    env.exec(
        r#"
        __gossip_show_fired = false
        local f = CreateFrame("Frame")
        f:RegisterEvent("GOSSIP_SHOW")
        f:SetScript("OnEvent", function(_, event)
            if event == "GOSSIP_SHOW" then
                __gossip_show_fired = true
            end
        end)
        "#,
    )
    .unwrap();
    env.fire_event("GOSSIP_SHOW").unwrap();
    let fired: bool = env.eval("return __gossip_show_fired").unwrap();
    assert!(fired);
}

#[test]
fn gossip_closed_event_reaches_listeners() {
    let env = env();
    env.exec(
        r#"
        __gossip_closed_fired = false
        local f = CreateFrame("Frame")
        f:RegisterEvent("GOSSIP_CLOSED")
        f:SetScript("OnEvent", function(_, event)
            if event == "GOSSIP_CLOSED" then
                __gossip_closed_fired = true
            end
        end)
        "#,
    )
    .unwrap();
    env.fire_event("GOSSIP_CLOSED").unwrap();
    let fired: bool = env.eval("return __gossip_closed_fired").unwrap();
    assert!(fired);
}

#[test]
fn gossip_counts_are_available_through_the_lua_boundary() {
    let env = env();
    env.state().borrow_mut().gossip.num_options = 4;

    let options: i32 = env.eval("return GetGossipNumOptions()").unwrap();

    assert_eq!(options, 4, "the Lua gossip probe should expose the seeded count");
}
