use std::fs;
use std::path::PathBuf;

fn source_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

#[test]
fn settings_initializer_hook_is_temporary_workaround_not_runtime_bootstrap() {
    let runtime = fs::read_to_string(source_path("src/lua_api/env_init/runtime_surface_bootstrap.lua"))
        .expect("read runtime bootstrap source");
    let settings_workaround = fs::read_to_string(source_path(
        "src/lua_api/workarounds/temporary/settings_surface_defaults.rs",
    ))
    .expect("read Settings workaround source");

    for runtime_only_forbidden in [
        "PingSoundsInitializer",
        "SettingsRegistrar",
        "SetParentInitializer",
        "SetSearchIgnoredInLayout",
        "SetKioskProtected",
    ] {
        assert!(
            !runtime.contains(runtime_only_forbidden),
            "`{runtime_only_forbidden}` belongs in the temporary Settings workaround, not the generic runtime bootstrap"
        );
        assert!(
            settings_workaround.contains(runtime_only_forbidden),
            "`{runtime_only_forbidden}` should be owned by settings_surface_defaults"
        );
    }

    assert!(
        runtime.contains("__wow_prepare_temporary_global_assignment"),
        "runtime bootstrap should keep only the generic temporary assignment extension point"
    );
}
