//! Existing bare Era CVar storage only; no native 11302 effects/default/signature credit.
use serde_json::json;
use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn current_era_cvar_reads_follow_explicit_storage_transitions() {
    assert_eq!(ACTIVE, ClientProfile::Era);
    assert_eq!(ACTIVE_INTERFACE_VERSION, 11507);
    let env = WowLuaEnv::new().expect("own bare Era env without loaded UI/cache");
    let types: (String, String, String) = env
        .eval("return type(rawget(_G,'SetCVar')), type(rawget(_G,'GetCVar')), type(rawget(_G,'GetCVarBool'))")
        .expect("real registration, not name factory");
    assert_eq!(
        types,
        ("function".into(), "function".into(), "function".into())
    );
    let mut observations = Vec::new();
    for (name, value, boolean) in [
        ("alwaysShowTargetNameplate", "0", false),
        ("alwaysShowTargetNameplate", "1", true),
        ("alwaysShowTargetNameplate", "0", false),
        ("instantQuestText", "1", true),
        ("instantQuestText", "0", false),
        ("instantQuestText", "1", true),
    ] {
        let code = format!(
            "local accepted = SetCVar('{name}','{value}'); return accepted, GetCVar('{name}'), GetCVarBool('{name}')"
        );
        let observed: (bool, String, bool) = env.eval(&code).expect("mutate/read current store");
        assert_eq!(observed, (true, value.into(), boolean));
        let stored = env.state().borrow().cvars.get(name);
        assert_eq!(stored.as_deref(), Some(value));
        observations
            .push(json!({"name":name,"input":value,"observed":observed,"rust_store":stored}));
    }
    let output = std::env::var_os("WOW_SIM_P1132_CVAR_OUT").expect("own result path required");
    assert!(std::path::Path::new(&output).is_absolute());
    let proof = json!({"scope":"existing bare Era generic CVar storage across six explicit transitions",
        "configured_interface":ACTIVE_INTERFACE_VERSION,"source_interface":11302,
        "raw_types":types,"observations":observations,"runtime_changes":0,"native_credit":false,
        "source_default_credit":false,"historical_signature_credit":false,"cvar_effect_credit":false,
        "source_ledger_mutated":false});
    std::fs::write(output, serde_json::to_vec_pretty(&proof).unwrap()).expect("own observations");
    eprintln!("P1132_CURRENT_CVAR_STORAGE {proof}");
}
