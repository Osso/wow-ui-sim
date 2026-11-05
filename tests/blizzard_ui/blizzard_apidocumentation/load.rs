//! Load probes for `Blizzard_APIDocumentation`.

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, with_blizzard_addon_startup_shape,
};
use crate::common::panel_fixtures::{
    blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors,
};

const ROOT: &str = "Blizzard_APIDocumentation";

// APIDocumentation vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn api_documentation_loads_cleanly_and_expands_default_chat_history() {
    with_blizzard_addon_startup_shape(&[], &[], |env, _loaded| {
        load_api_documentation(env);

        let (api_documentation_type, object_type, default_chat_max_lines): (String, String, i64) =
            env.eval(
                r#"
                assert(DEFAULT_CHAT_FRAME ~= nil, "DEFAULT_CHAT_FRAME must exist")
                return type(APIDocumentation),
                       DEFAULT_CHAT_FRAME:GetObjectType(),
                       DEFAULT_CHAT_FRAME:GetMaxLines()
                "#,
            )
            .expect("APIDocumentation load probe must run cleanly");

        assert_eq!(
            api_documentation_type, "table",
            "APIDocumentation singleton must be created by addon load"
        );
        assert!(
            default_chat_max_lines >= 2000,
            "APIDocumentation:OnLoad must raise DEFAULT_CHAT_FRAME ({object_type}) max lines to at least 2000; got {default_chat_max_lines}"
        );
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn api_documentation_loads_expected_mixin_tables() {
    with_blizzard_addon_startup_shape(&[], &[], |env, _loaded| {
        load_api_documentation(env);

        let all_are_tables: bool = env
            .eval(
                r#"
                local globals = {
                    "BaseAPIMixin",
                    "SystemsAPIMixin",
                    "FunctionsAPIMixin",
                    "EventsAPIMixin",
                    "FieldsAPIMixin",
                    "TablesAPIMixin",
                    "APIDocumentationMixin",
                    "APIDocumentation",
                }

                for _, globalName in ipairs(globals) do
                    if type(_G[globalName]) ~= "table" then
                        return false
                    end
                end

                return true
                "#,
            )
            .expect("APIDocumentation mixin table probe must run cleanly");

        assert!(
            all_are_tables,
            "APIDocumentation load must publish every documented mixin global as a table"
        );
    });
}
}

fn load_api_documentation(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    clear_recorded_lua_errors(env);
    let ui_dir = blizzard_ui_dir();
    let loaded = load_blizzard_addon_closure_into_env(env, &ui_dir, &[ROOT], &[]);

    assert!(
        loaded.iter().any(|addon| addon == ROOT),
        "{ROOT} must be included in the loaded addon closure; loaded={loaded:?}"
    );

    let errors = recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "{ROOT} must settle without recorded Lua errors:\n  {}",
        errors.join("\n  ")
    );
}
