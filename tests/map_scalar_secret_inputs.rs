//! Scalar map selectors: current secret acceptance and ordinary compatibility.
//! Known-map values come from simulator defaults, not native catalog claims.

use wow_ui_sim::lua_api::WowLuaEnv;

const ART_RESULT: &str = "assert(type(value) == 'number' and value == 89, 'generated art ID')";
const INFO_RESULT: &str = r#"
    assert(type(value) == 'table', 'map details table')
    assert(value.mapID == 84, 'known UI map ID')
    assert(value.name == 'Stormwind City', 'seeded map name')
"#;

fn seed_scalar_map_env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create real Lua environment")
}

#[cfg(feature = "retail-12-1-0")]
fn seed_scalar_map_secret_env() -> WowLuaEnv {
    use rilua::LuaApiMut;
    use rilua::table_security::wrap_host_secret_number;

    let env = seed_scalar_map_env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua)
            .expect("register authentic VM security helpers");
        let secret = wrap_host_secret_number(lua.state_mut(), 84.0);
        lua.state_mut().push(secret);
        let inserted = lua.set_global_val("ScalarMapSecret84", secret);
        lua.state_mut().pop();
        inserted.expect("root authentic host secret number in Lua");
    }
    env
}

#[cfg(feature = "retail-12-1-0")]
fn assert_untainted_secret_query(api: &str, result_assertion: &str) {
    let env = seed_scalar_map_secret_env();
    env.exec(&format!(
        r#"
        assert(issecure(), 'untainted caller')
        local parent_taint = debug.getstacktaint()
        local secret = ScalarMapSecret84
        assert(issecretvalue(secret), 'authentic host secret')
        local function capture(...)
            assert(select('#', ...) == 1, 'exactly one result')
            local value = ...
            {result_assertion}
        end
        capture(C_Map.{api}(secret))
        assert(rawequal(secret, ScalarMapSecret84), 'same rooted wrapper')
        assert(issecretvalue(secret), 'wrapper remains secret')
        assert(debug.getstacktaint() == parent_taint, 'caller taint preserved')
        assert(issecure(), 'caller remains secure')
        "#,
    ))
    .expect("untainted scalar query accepts authentic secret map 84");
}

#[cfg(feature = "retail-12-1-0")]
fn assert_tainted_scalar_query(api: &str, result_assertion: &str) {
    let env = seed_scalar_map_secret_env();
    env.exec(&format!(
        r#"
        assert(issecure(), 'secure parent')
        local parent_taint = debug.getstacktaint()
        local secret = ScalarMapSecret84
        assert(issecretvalue(secret), 'authentic host secret')
        local function addon()
            assert(debug.getstacktaint() == 'ScalarMapAddon', 'addon taint active')
            local function capture(...)
                assert(select('#', ...) == 1, 'ordinary ID returns exactly one result')
                local value = ...
                {result_assertion}
            end
            capture(C_Map.{api}(84))
            assert(debug.getstacktaint() == 'ScalarMapAddon', 'ordinary query preserves taint')
            local ok = pcall(C_Map.{api}, secret)
            assert(not ok, 'tainted secret query must reject')
            assert(debug.getstacktaint() == 'ScalarMapAddon', 'rejection preserves taint')
            assert(rawequal(secret, ScalarMapSecret84), 'same rooted wrapper in addon')
            assert(issecretvalue(secret), 'wrapper remains secret in addon')
        end
        debug.setobjecttaint(addon, 'ScalarMapAddon')
        addon()
        assert(debug.getstacktaint() == parent_taint, 'parent taint restored')
        assert(issecure(), 'secure parent restored')
        assert(rawequal(secret, ScalarMapSecret84), 'same rooted wrapper in parent')
        assert(issecretvalue(secret), 'wrapper remains secret in parent')
        "#,
    ))
    .expect("tainted addon accepts ordinary selector but rejects authentic secret");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn get_map_art_id_untainted_secret_returns_one_fixture_number() {
    assert_untainted_secret_query("GetMapArtID", ART_RESULT);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn get_map_info_untainted_secret_returns_one_fixture_details_table() {
    assert_untainted_secret_query("GetMapInfo", INFO_RESULT);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn get_map_art_id_tainted_addon_accepts_ordinary_and_rejects_secret() {
    assert_tainted_scalar_query("GetMapArtID", ART_RESULT);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn get_map_info_tainted_addon_accepts_ordinary_and_rejects_secret() {
    assert_tainted_scalar_query("GetMapInfo", INFO_RESULT);
}

fn assert_ordinary_selector_rejections(env: &WowLuaEnv, api: &str) {
    env.exec(&format!(
        r#"
        local query = C_Map.{api}
        local function rejects(diagnostic, ...)
            local ok, err = pcall(query, ...)
            assert(not ok, 'invalid ordinary selector must reject')
            assert(type(err) == 'string', 'string diagnostic')
            assert(string.find(err, diagnostic, 1, true), err)
        end
        rejects('expected integer, got non-integer number at argument 1', 84.5)
        rejects('expected number, got string at argument 1', '84')
        rejects('expected number, got nil at argument 1')
        rejects('expected number, got boolean at argument 1', true)
        rejects('expected number, got table at argument 1', {{}})
        rejects('expected i32, value 2147483648 out of range at argument 1', 2147483648)
        rejects('expected i32, value -2147483649 out of range at argument 1', -2147483649)
        "#,
    ))
    .expect("ordinary scalar selector parsing and diagnostics remain unchanged");
}

#[test]
fn get_map_art_id_ordinary_fixture_absence_and_selector_compatibility() {
    let env = seed_scalar_map_env();
    env.exec(&format!(
        r#"
        local function capture(...)
            assert(select('#', ...) == 1, 'exactly one ordinary result')
            local value = ...
            {ART_RESULT}
        end
        capture(C_Map.GetMapArtID(84))
        assert(select('#', C_Map.GetMapArtID(9999)) == 0, 'absent art returns zero values')
        assert(select('#', C_Map.GetMapArtID(999999)) == 0, 'outside ID returns zero values')
        "#,
    ))
    .expect("ordinary art lookup preserves fixture and absent-map return shape");
    assert_ordinary_selector_rejections(&env, "GetMapArtID");
}

#[test]
fn get_map_info_ordinary_fixture_generic_and_selector_compatibility() {
    let env = seed_scalar_map_env();
    env.exec(&format!(
        r#"
        local function capture(...)
            assert(select('#', ...) == 1, 'exactly one ordinary result')
            local value = ...
            {INFO_RESULT}
        end
        capture(C_Map.GetMapInfo(84))
        local function capture_generic(...)
            assert(select('#', ...) == 1, 'exactly one generic result')
            local value = ...
            assert(type(value) == 'table', 'generic details table')
            assert(value.mapID == 9999 and value.name == 'Map 9999', 'existing generic identity')
            assert(value.mapType == 3, 'existing generic map type')
            assert(value.parentMapID == 0 and value.flags == 0, 'existing generic metadata')
        end
        local unknown = C_Map.GetMapInfo(9999)
        if unknown then capture_generic(unknown) end
        assert(select('#', C_Map.GetMapInfo(999999)) == 0, 'outside ID returns zero values')
        "#,
    ))
    .expect("ordinary info lookup preserves fixture and generic-map return shape");
    assert_ordinary_selector_rejections(&env, "GetMapInfo");
}
