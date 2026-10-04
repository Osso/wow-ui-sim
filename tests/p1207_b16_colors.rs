#![cfg(feature = "retail-12-0-7")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn warning_rgba_overrides_are_live_detached_and_environment_local() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local ids = C_EncounterEvents.GetEventList()
        assert(#ids >= 2, 'two concrete distinct catalog keys required')
        FirstColorID, SecondColorID = ids[1], ids[2]
        assert(FirstColorID ~= SecondColorID)
        local first = {r=0.125, g=0.25, b=0.5, a=0.375}
        local second = {r=0.75, g=0.625, b=0.25, a=0.875}
        C_EncounterEvents.SetEventColor(FirstColorID, first)
        C_EncounterEvents.SetEventColor(SecondColorID, second)
        local initial = C_EncounterEvents.GetEventColor(FirstColorID)
        assert(initial.r == 0.125 and initial.g == 0.25 and initial.b == 0.5)
        assert(initial.a == 0.375 and initial ~= first)
        first.a = 1
        initial.r = 1
        local function probe()
            assert(not issecure())
            local unchanged = C_EncounterEvents.GetEventColor(FirstColorID)
            assert(unchanged.r == 0.125 and unchanged.a == 0.375)
            assert(not issecretvalue(unchanged.r) and not issecretvalue(unchanged.a))
            local other = C_EncounterEvents.GetEventColor(SecondColorID)
            assert(other.r == 0.75 and other.g == 0.625 and other.b == 0.25)
            assert(other.a == 0.875)
            C_EncounterEvents.SetEventColor(FirstColorID,
                {r=0.5, g=0.375, b=0.25, a=0.125})
            local current = C_EncounterEvents.GetEventColor(FirstColorID)
            assert(current.r == 0.5 and current.g == 0.375 and current.b == 0.25)
            assert(current.a == 0.125)
            assert(C_EncounterEvents.GetEventColor(SecondColorID).a == 0.875)
        end
        debug.setobjecttaint(probe, 'ColorProbe')
        probe()
        assert(issecure())
        "#,
    )
    .expect("existing two-argument warning-color producer copies live RGBA overrides");
    let other = WowLuaEnv::new().unwrap();
    let id: f64 = env.eval("return FirstColorID").unwrap();
    let empty: bool = other
        .eval(&format!(
            "return C_EncounterEvents.GetEventColor({id}) == nil"
        ))
        .unwrap();
    assert!(
        empty,
        "warning overrides must not leak between environments"
    );
}

// This is the existing 12.0.7 bridge's tuple contract, not the later cached
// ColorMixin/trigger contract. Do not promote it as current-cache conformance.
#[cfg(not(feature = "retail-12-1-5"))]
#[test]
fn legacy_timeline_bridge_reflects_warning_override_alpha_changes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local id = C_EncounterEvents.GetEventList()[1]
        C_EncounterEvents.SetEventColor(id, {r=0.125, g=0.25, b=0.5, a=0.375})
        local function check(r, g, b, a, ...)
            assert(select('#', ...) == 0)
            assert(r == 0.125 and g == 0.25 and b == 0.5 and a == 0.375)
        end
        check(C_EncounterTimeline.GetEventColor(id))
        C_EncounterEvents.SetEventColor(id, {r=0.75, g=0.625, b=0.25, a=0.875})
        local r, g, b, a = C_EncounterTimeline.GetEventColor(id)
        assert(r == 0.75 and g == 0.625 and b == 0.25 and a == 0.875)
        "#,
    )
    .expect("legacy four-component bridge consumes current override, not fixed white");
}
