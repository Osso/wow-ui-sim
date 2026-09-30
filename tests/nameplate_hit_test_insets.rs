//! Configuration-only NamePlateManager contract; native defaults/security are inferred.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn nameplate_hit_test_insets_round_trip_isolation_and_replacement() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local get = C_NamePlateManager.GetNamePlateHitTestInsets
        local set = C_NamePlateManager.SetNamePlateHitTestInsets
        local types = {Enum.NamePlateType.Friendly, Enum.NamePlateType.Enemy}
        assert(types[1] == 0 and types[2] == 1)
        local function expect(kind, a, b, c, d)
            assert(select('#', get(kind)) == 4)
            local left, right, top, bottom = get(kind)
            assert(left == a and right == b and top == c and bottom == d)
        end
        for _, kind in ipairs(types) do expect(kind, 0, 0, 0, 0) end
        for i, kind in ipairs(types) do
            assert(select('#', set(kind, 2.5, -1.25, 3, 4)) == 0)
            expect(kind, 2.5, -1.25, 3, 4)
            local other = types[3-i]
            if i == 1 then expect(other, 0, 0, 0, 0)
            else expect(other, 2.5, -1.25, 3, 4) end
            set(kind, -8, 0.5, -2, 9)
            expect(kind, -8, 0.5, -2, 9)
            set(kind, 2.5, -1.25, 3, 4)
        end
        assert(C_NamePlate.GetNamePlateForUnit('nameplate1') == nil)
        assert(next(C_NamePlate.GetNamePlates()) == nil)
        assert(C_NamePlate.GetNamePlateForUnit('target') == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn nameplate_hit_test_insets_errors_preserve_configuration() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local get = C_NamePlateManager.GetNamePlateHitTestInsets
        local set = C_NamePlateManager.SetNamePlateHitTestInsets
        set(0, 2.5, -1.25, 3, 4)
        set(1, -7, 6, 5, -4)
        local function unchanged()
            local a,b,c,d = get(0)
            assert(a == 2.5 and b == -1.25 and c == 3 and d == 4)
            a,b,c,d = get(1)
            assert(a == -7 and b == 6 and c == 5 and d == -4)
        end
        for _, kind in ipairs({-1, 2, 0.5, 0/0, math.huge, -math.huge, '0', false, {}}) do
            assert(not pcall(get, kind))
            assert(not pcall(set, kind, 11, 12, 13, 14))
            unchanged()
        end
        assert(not pcall(get))
        assert(not pcall(set, nil, 11, 12, 13, 14))
        for position = 1, 4 do
            for _, value in ipairs({0/0, math.huge, -math.huge, '2', false, {}}) do
                local args = {0, 11, 12, 13, 14}
                args[position+1] = value
                assert(not pcall(set, unpack(args)))
                unchanged()
            end
            local args = {0, 11, 12, 13, 14}
            args[position+1] = nil
            assert(not pcall(set, unpack(args, 1, 5)))
            unchanged()
        end
        unchanged()
        "#,
    )
    .unwrap();
}

#[test]
fn nameplate_hit_test_insets_secret_and_restricted_callers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local get = C_NamePlateManager.GetNamePlateHitTestInsets
        local set = C_NamePlateManager.SetNamePlateHitTestInsets
        set(0, 2.5, -1.25, 3, 4)
        local secretType = secretwrap(0)
        local a,b,c,d = get(secretType)
        assert(a == 2.5 and b == -1.25 and c == 3 and d == 4)
        for position = 1, 5 do
            local args = {0, 11, 12, 13, 14}
            args[position] = secretwrap(args[position])
            assert(not pcall(set, unpack(args)))
        end
        local function tainted()
            assert(not issecure())
            local ok, err = pcall(set, 0, 11, 12, 13, 14)
            assert(not ok and type(err) == 'string')
            assert(not issecure())
            assert(not pcall(get, secretType))
            local left,right,top,bottom = get(0)
            assert(left == 2.5 and right == -1.25 and top == 3 and bottom == 4)
            assert(not issecretvalue(left))
        end
        debug.setobjecttaint(tainted, 'NamePlateInsetsProbe')
        tainted()
        a,b,c,d = get(0)
        assert(a == 2.5 and b == -1.25 and c == 3 and d == 4)
        a,b,c,d = get(1)
        assert(a == 0 and b == 0 and c == 0 and d == 0)
        "#,
    )
    .unwrap();
}
