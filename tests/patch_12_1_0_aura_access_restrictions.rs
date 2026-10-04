//! 12.1.0 AuraButton access restrictions over the live aura-secret state.
//!
//! The cached `Blizzard_AuraContainer` frame provider applies
//! `DenyTaintedAccessWhenAurasAreSecret` after `initializeFrame` (immediately
//! once logged in, else on PLAYER_ENTERING_WORLD). The simulator enforces that
//! restriction only while auras are secret and only for tainted callers.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{
    assert_no_lua_errors, aura, load_env, set_player_auras, settle,
};
use wow_ui_sim::lua_api::WowLuaEnv;

/// - `AuditInitWidths` / `AuditInitConstrained` record, inside the addon-owned
///   `initializeFrame`, the width it just set and `HasAccessConstraints()`.
/// - `AuditProbe(frame)` reports `SetSize`, `CanBeAccessedInContext`,
///   `HasAccessConstraints`, `IsForbidden` and the aura restriction bit as seen
///   by the calling context.
const HELPERS: &str = r#"
    AuditInitWidths = {}
    AuditInitConstrained = {}
    function AuditInitRecorded(frame)
        AuditInitIcon(frame)
        AuditInitWidths[#AuditInitWidths + 1] = AuditPlain(frame:GetWidth())
        AuditInitConstrained[#AuditInitConstrained + 1] = tostring(AuditPlain(frame:HasAccessConstraints()))
    end
    function AuditProbe(frame)
        local ok, err = pcall(frame.SetSize, frame, 24, 24)
        local size = ok and 'ok' or (tostring(err):find('forbidden object', 1, true) and 'forbidden' or tostring(err))
        local restriction = Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret
        return string.format('size=%s access=%s constrained=%s forbidden=%s restricted=%s',
            size,
            tostring(AuditPlain(frame:CanBeAccessedInContext())),
            tostring(AuditPlain(frame:HasAccessConstraints())),
            tostring(AuditPlain(frame:IsForbidden())),
            tostring(AuditPlain(frame:HasAnyAccessRestrictions(restriction))))
    end
    function AuditAddonProbe(frame)
        return AuditAddon(function() return AuditProbe(frame) end)
    end
"#;

const OPEN: &str = "size=ok access=true constrained=true forbidden=false restricted=true";
const FORBIDDEN: &str =
    "size=forbidden access=false constrained=true forbidden=false restricted=true";
const UNRESTRICTED: &str =
    "size=ok access=true constrained=false forbidden=false restricted=false";

fn set_auras_secret(env: &WowLuaEnv, secret: bool) {
    env.state().borrow_mut().unit_auras_restricted = secret;
}

/// Logged-in env whose addon container already spawned buttons for two auras.
fn container_env(logged_in: bool) -> WowLuaEnv {
    let env = load_env();
    env.exec(HELPERS).unwrap();
    env.set_logged_in(logged_in);
    env.exec(
        "AuditBox = AuditContainer({{'buffs', 'HELPFUL', {initializeFrame = AuditInitRecorded}}})",
    )
    .unwrap();
    set_player_auras(&env, vec![aura(501), aura(502)]);
    settle(&env);
    env.exec("AuditButton = AuditBox:GetAuraGroupFrame('buffs', 1)")
        .unwrap();
    env
}

fn probe(env: &WowLuaEnv, tainted: bool) -> String {
    let probe = if tainted {
        "AuditAddonProbe"
    } else {
        "AuditProbe"
    };
    env.eval::<String>(&format!("return {probe}(AuditButton)"))
        .unwrap()
}

#[test]
fn aura_buttons_are_forbidden_to_tainted_callers_only_while_auras_are_secret() {
    let env = container_env(true);
    // (auras secret, tainted caller, expected probe readout)
    let cases = [
        (false, false, OPEN),
        (false, true, OPEN),
        (true, false, OPEN),
        (true, true, FORBIDDEN),
        (false, true, OPEN),
        (true, true, FORBIDDEN),
        (false, true, OPEN),
    ];
    for (secret, tainted, expected) in cases {
        set_auras_secret(&env, secret);
        assert_eq!(
            probe(&env, tainted),
            expected,
            "auras secret={secret} tainted={tainted}"
        );
    }
    assert_no_lua_errors(&env);
}

#[test]
fn initialize_frame_runs_before_the_restriction_applies() {
    let env = container_env(true);
    let (created, width, constrained_during_init): (i32, f64, String) = env
        .eval("return #AuditInitWidths, AuditInitWidths[1], table.concat(AuditInitConstrained, ',')")
        .unwrap();
    assert!(created >= 2, "initializeFrame ran for every spawned button");
    assert_eq!(width, 20.0, "tainted initializeFrame resized the button");
    assert!(
        constrained_during_init.split(',').all(|value| value == "false"),
        "no restriction during initializeFrame: {constrained_during_init}"
    );
    set_auras_secret(&env, true);
    assert_eq!(probe(&env, true), FORBIDDEN, "restriction applied afterwards");
    assert_no_lua_errors(&env);
}

#[test]
fn pre_login_buttons_stay_accessible_through_player_login() {
    let env = container_env(false);
    set_auras_secret(&env, true);
    assert_eq!(probe(&env, true), UNRESTRICTED, "before PLAYER_LOGIN");

    env.exec(
        r#"
        AuditLoginProbe = nil
        local listener = CreateFrame('Frame')
        local function onLogin()
            AuditLoginProbe = AuditProbe(AuditButton)
        end
        debug.setobjecttaint(onLogin, AUDIT_ADDON)
        listener:SetScript('OnEvent', onLogin)
        listener:RegisterEvent('PLAYER_LOGIN')
        "#,
    )
    .unwrap();
    env.set_logged_in(true);
    env.fire_event("PLAYER_LOGIN").unwrap();
    let login_probe: String = env.eval("return AuditLoginProbe").unwrap();
    assert_eq!(login_probe, UNRESTRICTED, "addon PLAYER_LOGIN handler");
    assert_eq!(probe(&env, true), UNRESTRICTED, "after PLAYER_LOGIN");

    env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[rilua::Val::Bool(true), rilua::Val::Bool(false)],
    )
    .unwrap();
    assert_eq!(probe(&env, true), FORBIDDEN, "after PLAYER_ENTERING_WORLD");
    set_auras_secret(&env, false);
    assert_eq!(probe(&env, true), OPEN, "auras public again");
    assert_no_lua_errors(&env);
}
