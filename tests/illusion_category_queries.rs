//! Rows352/353: explicit illusion inputs; selector/filter policies are INFERRED.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::c_api::IllusionInfo;
use wow_ui_sim::lua_api::WowLuaEnv;

const SECRET_NAMES: [&str; 6] = [
    "ICNumber",
    "ICNil",
    "ICBool",
    "ICString",
    "ICFrameSecret",
    "ICTableSecret",
];

fn rows() -> Vec<IllusionInfo> {
    // Concrete TEST DATA, not a native catalog or native category partition.
    vec![
        IllusionInfo {
            category: 14,
            visual_id: 501,
            source_id: 601,
            icon: 132261,
            is_collected: false,
            is_usable: true,
            is_hide_visual: false,
        },
        IllusionInfo {
            category: 13,
            visual_id: 502,
            source_id: 602,
            icon: 132262,
            is_collected: true,
            is_usable: false,
            is_hide_visual: true,
        },
        IllusionInfo {
            category: 14,
            visual_id: 503,
            source_id: 603,
            icon: 132261,
            is_collected: true,
            is_usable: true,
            is_hide_visual: false,
        },
    ]
}

fn empty_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("illusion environment");
    env.exec(r#"
        assert(Enum.TransmogCollectionType.OneHAxe == 13)
        assert(Enum.TransmogCollectionType.OneHSword == 14)
        ICExpected = {
            {visualID=501, sourceID=601, icon=132261, isCollected=false, isUsable=true, isHideVisual=false},
            {visualID=502, sourceID=602, icon=132262, isCollected=true, isUsable=false, isHideVisual=true},
            {visualID=503, sourceID=603, icon=132261, isCollected=true, isUsable=true, isHideVisual=false},
        }
        function ICCheck(expected, ...)
            local function checkResult(...)
                assert(select('#', ...) == 1, 'exactly one result')
                local result = ...
                assert(type(result) == 'table' and not issecretvalue(result), 'public array')
                assert(#result == #expected, 'exact supplied row count')
                local count = 0
                for key, row in pairs(result) do
                    assert(type(key) == 'number' and key % 1 == 0, 'integer array index')
                    assert(key >= 1 and key <= #expected, 'dense array with no metadata')
                    assert(type(row) == 'table' and not issecretvalue(row), 'public row')
                    local wanted, fields = ICExpected[expected[key]], 0
                    for name, value in pairs(row) do
                        assert(wanted[name] ~= nil, 'only six documented fields, no category')
                        assert(type(value) == type(wanted[name]) and not issecretvalue(value), 'public typed field')
                        assert(value == wanted[name], 'exact supplied field value')
                        fields = fields + 1
                    end
                    assert(fields == 6, 'all six nonnullable fields')
                    count = count + 1
                end
                assert(count == #expected, 'no holes')
                return result
            end
            return checkResult(C_TransmogCollection.GetIllusions(...))
        end
        function ICReject(value)
            local ok, err = pcall(C_TransmogCollection.GetIllusions, value)
            assert(not ok, 'invalid or denied selector')
            assert(type(err) == 'string' and #err > 0, 'nonempty public error')
            assert(not string.find(err, 'PRIVATE-IllusionCategory', 1, true), 'no private payload in error')
            return err
        end
        function ICContexts(probe)
            assert(issecure())
            probe()
            assert(issecure(), 'secure caller preserved')
            local function addon()
                assert(debug.getstacktaint() == 'IllusionCategoryFixture')
                probe()
                assert(debug.getstacktaint() == 'IllusionCategoryFixture', 'taint preserved')
            end
            debug.setobjecttaint(addon, 'IllusionCategoryFixture')
            addon()
            assert(issecure(), 'secure caller restored')
        end
    "#).expect("assertion helpers; never replace query or callback");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = empty_env();
    env.state().borrow_mut().transmog_illusions = rows();
    env
}

fn publish(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let inserted = lua.set_global_val(name, value);
    lua.state_mut().pop();
    inserted.expect("globally root actual VM wrapper");
}

fn secret_env() -> WowLuaEnv {
    let env = seeded_env();
    env.exec("ICFrame = CreateFrame('Frame'); ICTable = {marker='PRIVATE-IllusionCategory'}")
        .unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    let number = wrap_host_secret_number(lua.state_mut(), 14.0);
    publish(&mut *lua, "ICNumber", number);
    let nil = wrap_secret(lua.state_mut(), Val::Nil).expect("secure VM supports secret nil");
    publish(&mut *lua, "ICNil", nil);
    let boolean = wrap_host_secret_bool(lua.state_mut(), false);
    publish(&mut *lua, "ICBool", boolean);
    let string = wrap_host_secret_string(lua.state_mut(), "PRIVATE-IllusionCategory");
    publish(&mut *lua, "ICString", string);
    for (original, name) in [("ICFrame", "ICFrameSecret"), ("ICTable", "ICTableSecret")] {
        let value = lua.get_global_val(original);
        let Val::Table(reference) = value else {
            panic!("actual frame/table required")
        };
        if original == "ICFrame" {
            assert!(
                lua.state_mut()
                    .gc
                    .tables
                    .get(reference)
                    .unwrap()
                    .backing()
                    .is_some(),
                "actual frame metadata"
            );
        }
        lua.state_mut().push(value);
        let wrapper = wrap_secret(lua.state_mut(), value).unwrap();
        publish(&mut *lua, name, wrapper);
        lua.state_mut().pop();
    }
    drop(lua);
    env.exec("ICSecrets = {ICNumber, ICNil, ICBool, ICString, ICFrameSecret, ICTableSecret}; for _, v in ipairs(ICSecrets) do assert(issecretvalue(v)) end").unwrap();
    env
}

fn wrapper_metadata(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    SECRET_NAMES
        .iter()
        .map(|name| {
            let value = lua.get_global_val(name);
            let Val::Userdata(reference) = value else {
                panic!("authentic wrapper missing: {name}")
            };
            assert!(
                rilua::table_security::is_secret_value(lua.state_mut(), value),
                "VM-owned secret metadata"
            );
            let wrapper = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .expect("rooted wrapper live");
            (value, wrapper.alloc_seq())
        })
        .collect()
}

fn secret_probe(env: &WowLuaEnv, script: &str) {
    let metadata = wrapper_metadata(env);
    let before = env.state().borrow().transmog_illusions.clone();
    env.exec(script)
        .expect("real query obeys authenticated boundary");
    assert_eq!(
        wrapper_metadata(env),
        metadata,
        "wrapper identity, allocation and VM secrecy preserved"
    );
    assert_eq!(
        env.state().borrow().transmog_illusions,
        before,
        "read-only input"
    );
}

#[test]
fn default_empty_returns_one_public_dense_array() {
    let env = empty_env();
    assert!(env.state().borrow().transmog_illusions.is_empty());
    env.exec("ICCheck({}); ICCheck({}, nil); ICCheck({}, 14)")
        .unwrap();
}

#[test]
fn omitted_and_nil_select_all_supplied_rows_in_order() {
    seeded_env()
        .exec("ICCheck({1,2,3}); ICCheck({1,2,3}, nil)")
        .unwrap();
}

#[test]
fn known_categories_select_exact_rows_using_existing_enum() {
    seeded_env().exec("ICCheck({1,3}, Enum.TransmogCollectionType.OneHSword); ICCheck({2}, Enum.TransmogCollectionType.OneHAxe)").unwrap();
}

#[test]
fn unknown_and_u32_boundary_categories_return_empty() {
    seeded_env()
        .exec("ICCheck({}, 0); ICCheck({}, 29); ICCheck({}, 4294967295)")
        .unwrap();
}

#[test]
fn source_order_and_exact_six_field_schema_preserve_all_flags() {
    let env = seeded_env();
    env.state().borrow_mut().transmog_illusions.reverse();
    env.exec("ICCheck({3,2,1}, nil); ICCheck({3,1}, 14); ICCheck({2}, 13)")
        .unwrap();
}

#[test]
fn repeated_queries_leave_host_vector_unchanged() {
    let env = seeded_env();
    let before = env.state().borrow().transmog_illusions.clone();
    env.exec("for i=1,4 do ICCheck({1,2,3}); ICCheck({1,3},14); ICCheck({},99) end")
        .unwrap();
    assert_eq!(env.state().borrow().transmog_illusions, before);
}

#[test]
fn results_and_rows_are_fresh_and_mutation_cannot_change_inputs() {
    let env = seeded_env();
    let before = env.state().borrow().transmog_illusions.clone();
    env.exec(
        r#"
        local a, b = ICCheck({1,3},14), ICCheck({1,3},14)
        assert(not rawequal(a,b) and not rawequal(a[1],b[1]) and not rawequal(a[1],a[2]))
        a[1].visualID = 999; a[1].isCollected = true; a[2] = nil; a.extra = true
        assert(b[1].visualID == 501 and b[1].isCollected == false and #b == 2)
        ICCheck({1,3},14)
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().transmog_illusions, before);
}

#[test]
fn live_replacement_and_clear_affect_queries_not_old_results() {
    let env = seeded_env();
    env.exec("ICOld = ICCheck({1,2,3})").unwrap();
    env.state().borrow_mut().transmog_illusions = vec![rows()[1].clone()];
    env.exec("ICCheck({2}); ICCheck({},14); ICCheck({2},13); assert(#ICOld == 3 and ICOld[1].visualID == 501)").unwrap();
    env.state().borrow_mut().transmog_illusions.clear();
    env.exec("ICCheck({}); ICCheck({},13); assert(#ICOld == 3)")
        .unwrap();
}

#[test]
fn environments_keep_explicit_vectors_isolated() {
    let first = seeded_env();
    let second = empty_env();
    first.exec("ICCheck({1,3},14)").unwrap();
    second.exec("ICCheck({})").unwrap();
    second.state().borrow_mut().transmog_illusions = vec![rows()[1].clone()];
    first.exec("ICCheck({1,2,3})").unwrap();
    second.exec("ICCheck({2})").unwrap();
}

#[test]
fn strict_public_invalid_categories_fail_in_both_contexts() {
    let env = seeded_env();
    let before = env.state().borrow().transmog_illusions.clone();
    env.exec(r#"
        local frame = CreateFrame('Frame')
        ICContexts(function()
            for _, value in ipairs({true,false,'14','',{},frame,-1,14.5,4294967296,0/0,math.huge,-math.huge}) do
                ICReject(value)
            end
            ICCheck({1,3},14)
        end)
    "#).unwrap();
    assert_eq!(env.state().borrow().transmog_illusions, before);
}

#[test]
fn ordinary_public_categories_work_without_clearing_caller_taint() {
    seeded_env().exec("ICContexts(function() ICCheck({1,3},14); ICCheck({2},13); ICCheck({1,2,3},nil); ICCheck({},99) end)").unwrap();
}

#[test]
fn authentic_secret_number_selects_in_secure_caller() {
    let env = secret_env();
    secret_probe(
        &env,
        "assert(issecure()); ICCheck({1,3},ICNumber); assert(issecure() and issecretvalue(ICNumber)); ICCheck({1,3},14)",
    );
}

#[test]
fn authentic_secret_nil_selects_all_secure_but_is_denied_tainted() {
    let env = secret_env();
    secret_probe(
        &env,
        r#"
        ICContexts(function()
            if issecure() then ICCheck({1,2,3}, ICNil) else ICReject(ICNil) end
            assert(issecretvalue(ICNil)); ICCheck({1,3},14)
        end)
    "#,
    );
}

#[test]
fn authentic_wrong_payloads_fail_secure_without_private_payload_errors() {
    let env = secret_env();
    secret_probe(
        &env,
        r#"
        assert(issecure())
        for _, value in ipairs({ICBool,ICString,ICFrameSecret,ICTableSecret}) do
            ICReject(value); assert(issecretvalue(value)); ICCheck({1,3},14)
        end
        assert(issecure())
    "#,
    );
}

#[test]
fn tainted_secrets_are_denied_before_payload_validation_and_recover_publicly() {
    let env = secret_env();
    secret_probe(
        &env,
        r#"
        local function addon()
            assert(debug.getstacktaint() == 'IllusionCategoryFixture')
            local denied = ICReject(ICNumber)
            for _, value in ipairs(ICSecrets) do
                assert(ICReject(value) == denied, 'same VM access gate before nil/type/model handling')
                assert(issecretvalue(value)); ICCheck({1,3},14)
                assert(debug.getstacktaint() == 'IllusionCategoryFixture')
            end
        end
        debug.setobjecttaint(addon, 'IllusionCategoryFixture'); addon(); assert(issecure())
    "#,
    );
}

#[test]
fn gc_retains_real_wrapper_metadata_and_public_query_recovery() {
    let env = secret_env();
    secret_probe(
        &env,
        r#"
        collectgarbage('collect')
        ICContexts(function()
            for _, value in ipairs(ICSecrets) do assert(issecretvalue(value)) end
            if issecure() then ICCheck({1,3},ICNumber) else ICReject(ICNumber) end
            ICCheck({1,3},14)
            local garbage = {}; for i=1,200 do garbage[i] = {i, tostring(i)} end
            collectgarbage('collect')
            if issecure() then ICCheck({1,2,3},ICNil) else ICReject(ICNil) end
            ICCheck({2},13)
        end)
        collectgarbage('collect'); ICCheck({1,2,3})
    "#,
    );
}
