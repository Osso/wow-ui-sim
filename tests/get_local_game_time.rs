use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn game_clock_contract() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        assert(type(GetLocalGameTime) == "function", "GetLocalGameTime is missing")
        local function check(hour, minute)
            local function verify(...)
                assert(select("#", ...) == 2)
                local h, m = ...
                assert(type(h) == "number" and type(m) == "number")
                assert(h == hour and m == minute)
            end
            verify(GetGameTime())
            verify(GetLocalGameTime())
        end
        check(12, 0)
        A_Admin.SetGameTime(3, 14)
        check(3, 14)
        A_Admin.SetGameTime(23, 59)
        check(23, 59)
        A_Admin.SetGameTime(nil)
        check(-1, -1)
        "##,
    )
    .unwrap();
    for args in [
        "-1, 0", "24, 0", "0, -1", "0, 60", "1.5, 0", "0, 1.5", "1", "nil, 1",
    ] {
        assert!(
            env.exec(&format!("A_Admin.SetGameTime({args})")).is_err(),
            "accepted {args}"
        );
    }
}
