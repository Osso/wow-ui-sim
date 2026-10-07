//! The publication probe must construct an animation, not a frame named VertexColor.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use std::collections::BTreeMap;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_10_2_5_vertex_color_probe_reaches_missing_endpoint_methods() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local texture = CreateFrame('Frame'):CreateTexture()
        local animation = texture:CreateAnimationGroup():CreateAnimation('VertexColor')
        assert(animation:GetObjectType() == 'VertexColor')
        "#,
    )
    .unwrap();
    let entry: sweep::Entry = serde_json::from_str(
        r#"{"id":"fixture","section":"widgets","direction":"added","symbol":"VertexColor:GetStartColor"}"#,
    )
    .unwrap();
    let (kind, detail, ok, _, _) = sweep::probe_entry(&env, &entry, false, &BTreeMap::new());
    assert_eq!(kind, "object-method");
    assert_eq!(detail, "VertexColor; lookup=nil");
    assert!(!ok, "Missing endpoints must remain an explicit publication gap");
}
