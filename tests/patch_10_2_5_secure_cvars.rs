//! Accepted secure-CVar combat policy; behavioral RED not yet observed.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

const HOTFIX_SECURE_CVARS: [&str; 7] = [
    "autoStand",
    "autoUnshift",
    "SoftTargetEnemy",
    "SoftTargetEnemyRange",
    "SoftTargetForce",
    "SoftTargetInteract",
    "SoftTargetInteractRange",
];

fn isolated_env() -> (WowLuaEnv, tempfile::TempDir) {
    let session = tempfile::tempdir().expect("create secure-CVar session directory");
    let env = WowLuaEnv::with_session_wtf_path(session.path().to_path_buf())
        .expect("create bare Lua API test environment");
    (env, session)
}

const WRITE_ASSERTIONS: &str = r#"
local count, observedName, observedValue, observedRead = 0
local listener = CreateFrame('Frame')
listener:RegisterEvent('CVAR_UPDATE')
listener:SetScript('OnEvent', function(_, event, cvar, value)
    count = count + 1
    observedName, observedValue = cvar, value
    observedRead = GetCVar(cvar)
end)
local reached, callerTaint = false, nil
local invoke = function()
    reached, callerTaint = true, debug.getstacktaint()
    return invokeSetter(writeName, '1')
end
if expectTaint then
    debug.setobjecttaint(invoke, 'Patch1025SecureCVarDraft')
end
local ok, result = pcall(invoke)
listener:UnregisterAllEvents()
-- Check the tainted closure really ran; a failed taint assertion inside
-- pcall alone would masquerade as a correctly denied setter.
assert(reached, name .. ': closure did not run')
if expectTaint then
    assert(callerTaint == 'Patch1025SecureCVarDraft', name .. ': genuine addon taint required')
else
    assert(callerTaint == nil, name .. ': secure control must be untainted')
end
assert(ok == expectAccepted, name .. ': unexpected permission result')
if expectAccepted then
    assert(result == true, name .. ': successful write must return true')
    assert(GetCVar(name) == '1', name .. ': global read after accepted write')
    assert(C_CVar.GetCVar(name) == '1', name .. ': namespace read after accepted write')
    assert(GetCVar(writeName) == '1', name .. ': case-normalized read after accepted write')
    assert(count == 1, name .. ': accepted write must notify once')
    assert(observedName == writeName and observedValue == '1', name .. ': accepted event payload')
    assert(observedRead == '1', name .. ': event must observe committed value')
else
    assert(type(result) == 'string', name .. ': denial must report an error string')
    assert(result:find('protected from insecure code in combat', 1, true),
        name .. ': modeled combat protection required: ' .. result)
    assert(GetCVar(name) == '0', name .. ': denied write changed global read')
    assert(C_CVar.GetCVar(name) == '0', name .. ': denied write changed namespace read')
    assert(GetCVar(writeName) == '0', name .. ': denied write changed case-normalized read')
    assert(count == 0, name .. ': denied write emitted CVAR_UPDATE')
end
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteContext {
    InsecureCombat,
    SecureCombat,
    InsecureOutOfCombat,
}

fn seed_old_value(env: &WowLuaEnv, name: &str) {
    // Secure setup establishes a concrete old value regardless of factory
    // defaults. This is a permission test, not default-publication evidence.
    env.state().borrow_mut().player.in_combat = false;
    env.exec(&format!(
        "assert(issecure()); assert(SetCVar('{name}', '0')); \
         assert(GetCVar('{name}') == '0'); assert(C_CVar.GetCVar('{name}') == '0')"
    ))
    .expect("seed isolated old value before listener registration");
}

fn assert_write_case(name: &str, setter: &str, uppercase: bool, context: WriteContext) {
    let (env, _session) = isolated_env();
    seed_old_value(&env, name);
    let in_combat = context != WriteContext::InsecureOutOfCombat;
    let tainted = context != WriteContext::SecureCombat;
    let accepted = context != WriteContext::InsecureCombat;
    env.state().borrow_mut().player.in_combat = in_combat;
    let write_name = if uppercase {
        name.to_ascii_uppercase()
    } else {
        name.to_owned()
    };
    let code = format!(
        "local name, writeName = '{name}', '{write_name}'\n\
         local expectAccepted, expectTaint = {accepted}, {tainted}\n\
         local invokeSetter = {setter}\n{WRITE_ASSERTIONS}"
    );
    env.exec(&code).unwrap_or_else(|error| {
        panic!("{name}, {setter}, uppercase={uppercase}, context={context:?}: {error}")
    });
}

fn assert_seven_cvar_matrix(context: WriteContext) {
    for name in HOTFIX_SECURE_CVARS {
        for setter in ["SetCVar", "C_CVar.SetCVar"] {
            for uppercase in [false, true] {
                assert_write_case(name, setter, uppercase, context);
            }
        }
    }
}

#[test]
fn patch_10_2_5_seven_secure_cvars_deny_insecure_combat_writes() {
    assert_seven_cvar_matrix(WriteContext::InsecureCombat);
}

#[test]
fn patch_10_2_5_seven_secure_cvars_accept_secure_combat_writes() {
    assert_seven_cvar_matrix(WriteContext::SecureCombat);
}

#[test]
fn patch_10_2_5_seven_secure_cvars_accept_insecure_out_of_combat_writes() {
    assert_seven_cvar_matrix(WriteContext::InsecureOutOfCombat);
}

#[test]
fn patch_10_2_5_extension_preserves_existing_nameplate_combat_protection() {
    // Representative historical regression; does not replace the full 5.4.8 list/suite.
    for setter in ["SetCVar", "C_CVar.SetCVar"] {
        for uppercase in [false, true] {
            assert_write_case(
                "nameplateShowEnemies",
                setter,
                uppercase,
                WriteContext::InsecureCombat,
            );
        }
    }
}
