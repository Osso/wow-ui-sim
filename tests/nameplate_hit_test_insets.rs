//! Retail hit-test inset parsing and the simulator's limited read/write state.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn nameplate_hit_test_insets_store_scaled_values_per_type() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local get = C_NamePlateManager.GetNamePlateHitTestInsets
        local set = C_NamePlateManager.SetNamePlateHitTestInsets
        local types = {Enum.NamePlateType.Friendly, Enum.NamePlateType.Enemy}
        assert(types[1] == 0 and types[2] == 1)
        local function stored(value)
            return (value / 1024) * 0.8
        end
        local function expect(kind, left, right, top, bottom)
            assert(select('#', get(kind)) == 4)
            local actualLeft, actualRight, actualTop, actualBottom = get(kind)
            assert(math.abs(actualLeft - stored(left)) < 0.000001)
            assert(math.abs(actualRight - stored(right)) < 0.000001)
            assert(math.abs(actualTop - stored(top)) < 0.000001)
            assert(math.abs(actualBottom - stored(bottom)) < 0.000001)
        end
        for _, kind in ipairs(types) do expect(kind, 0, 0, 0, 0) end
        for i, kind in ipairs(types) do
            set(kind, 2.5, -1.25, 3, 4)
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
        local function stored(value)
            return (value / 1024) * 0.8
        end
        set(0, 2.5, -1.25, 3, 4)
        set(1, -7, 6, 5, -4)
        local function unchanged()
            local a,b,c,d = get(0)
            assert(math.abs(a - stored(2.5)) < 0.000001)
            assert(math.abs(b - stored(-1.25)) < 0.000001)
            assert(math.abs(c - stored(3)) < 0.000001)
            assert(math.abs(d - stored(4)) < 0.000001)
            a,b,c,d = get(1)
            assert(math.abs(a - stored(-7)) < 0.000001)
            assert(math.abs(b - stored(6)) < 0.000001)
            assert(math.abs(c - stored(5)) < 0.000001)
            assert(math.abs(d - stored(-4)) < 0.000001)
        end
        for _, kind in ipairs({-1, 2, 0/0, math.huge, -math.huge, 258, 2147483648}) do
            assert(not pcall(set, kind, 11, 12, 13, 14))
            unchanged()
        end
        assert(not pcall(set, nil, 11, 12, 13, 14))
        for position = 1, 4 do
            for _, value in ipairs({'not-number', false, {}}) do
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

#[cfg(feature = "client-retail")]
#[test]
fn retail_tainted_nameplate_hit_test_insets_succeeds_without_restriction_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local set = C_NamePlateManager.SetNamePlateHitTestInsets
        local function tainted()
            assert(not issecure())
            set(Enum.NamePlateType.Enemy, -10000, -10000, -10000, -10000)
        end
        debug.setobjecttaint(tainted, 'EllesmereUINameplates')
        tainted()
        "#,
    )
    .expect("an ordinary tainted nameplate setup without restriction state must not error");
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_nameplate_hit_test_insets_matches_native_input_parsing() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local set = C_NamePlateManager.SetNamePlateHitTestInsets
        local get = C_NamePlateManager.GetNamePlateHitTestInsets
        local function assertStored(typeId, left)
            local actual = get(typeId)
            assert(math.abs(actual - (left / 1024 * 0.8)) < 0.000001)
        end

        set(256, 11, 12, 13, 14)
        assertStored(0, 11)
        set(257, 21, 22, 23, 24)
        assertStored(1, 21)
        set('not a number', 31, 32, 33, 34)
        assertStored(0, 31)
        set(false, 41, 42, 43, 44)
        assertStored(0, 41)
        set('1', 51, 52, 53, 54)
        assertStored(1, 51)

        assert(not pcall(set, nil, 1, 2, 3, 4))
        assert(not pcall(set, 2147483648, 1, 2, 3, 4))
        assert(not pcall(set, 0, 'not-number', 2, 3, 4))
        assert(pcall(set, 0, '2', '2.5', '-3', '4'))
        "#,
    )
    .expect("Retail type and inset arguments must use the native numeric conversions");
}
