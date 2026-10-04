//! Retail 12.1.0 page singles: small behavioral contracts.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn addon_create_frame_rejects_world_frame_type() {
    let env = WowLuaEnv::new().unwrap();
    let (ok, err): (bool, String) = env
        .eval(
            r#"
            local before = WorldFrame
            local ok, err = pcall(CreateFrame, "WorldFrame", "AddonWorldFrame", UIParent)
            assert(WorldFrame == before, "global WorldFrame replaced")
            assert(AddonWorldFrame == nil, "named instance created")
            return ok, tostring(err)
            "#,
        )
        .unwrap();
    assert!(!ok);
    assert!(err.contains("WorldFrame"), "{err}");
}

/// 12.1.0: formatting a secret number must not mark unrelated objects secret.
#[test]
fn format_number_with_secret_input_leaves_other_objects_plain() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local abbreviated = C_StringUtil.CreateAbbreviatedNumberFormatter()
        local rule = C_StringUtil.CreateNumericRuleFormatter()
        rule:SetBreakpoints({{threshold=0, format='%.1f'}})
        local other = UIParent:CreateFontString(nil, "ARTWORK")

        local text = abbreviated:FormatNumber(secretwrap(123456))
        assert(issecretvalue(text) and secretunwrap(text) == '123k')
        assert(rule:FormatNumber(secretwrap(12.5)) == '12.5')

        assert(not issecretvalue(abbreviated) and not issecretvalue(rule), 'formatter marked secret')
        assert(not issecretvalue(abbreviated:FormatNumber(1234)), 'later plain result secret')
        other:SetText("plain")
        assert(not other:HasSecretValues(), 'unrelated FontString marked secret')
        assert(not UIParent:HasSecretValues(), 'parent marked secret')
        "#,
    )
    .unwrap();
}
