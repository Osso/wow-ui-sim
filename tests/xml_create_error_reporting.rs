//! Nested template creation failures must remain visible to addon loading.

use std::fs;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::clear_templates;

#[test]
fn partially_created_parent_key_and_array_child_reports_original_nested_failure() {
    clear_templates();
    let directory = tempfile::tempdir().unwrap();
    let addon = directory.path().join("NestedCreateError");
    fs::create_dir(&addon).unwrap();
    fs::write(
        addon.join("NestedCreateError.lua"),
        r#"
        FailingChildMixin = {}
        function FailingChildMixin:OnLoad()
            error('Missing FileName or FilePath')
        end
        "#,
    )
    .unwrap();
    fs::write(
        addon.join("NestedCreateError.xml"),
        r#"<Ui>
            <Frame name="NestedErrorTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="NestedChild" mixin="FailingChildMixin">
                        <Scripts><OnLoad method="OnLoad"/></Scripts>
                    </Frame>
                </Frames>
            </Frame>
            <Frame name="NestedErrorRoot" parent="UIParent">
                <Frames>
                    <Frame parentKey="Broken" parentArray="BrokenChildren"
                        inherits="NestedErrorTemplate"/>
                </Frames>
            </Frame>
        </Ui>"#,
    )
    .unwrap();
    let toc = addon.join("NestedCreateError.toc");
    fs::write(
        &toc,
        "## Title: Nested create error probe\nNestedCreateError.lua\nNestedCreateError.xml\n",
    )
    .unwrap();

    let env = WowLuaEnv::new().unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(
        loaded
            .warnings
            .iter()
            .any(|warning| warning.contains("Missing FileName or FilePath")),
        "nested creation failed, but addon reported healthy loading: {:?}",
        loaded.warnings
    );
    let state = env.state();
    let state = state.borrow();
    assert_eq!(state.lua_errors.len(), 1, "{:?}", state.lua_errors);
    assert!(state.lua_errors[0].contains("Missing FileName or FilePath"));
}
