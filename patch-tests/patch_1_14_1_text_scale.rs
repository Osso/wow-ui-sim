//! Current headless Era getter state proof, not a native 1.14.1 signature/default.
//! The source's FontStringSetTextScale spelling does not authorize an alias.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn existing_fontstring_getter_reads_independent_updated_scale_state() {
    let env = WowLuaEnv::new().expect("headless Era environment");
    let scales: (f64, f64, f64) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame")
            local first = parent:CreateFontString()
            local second = parent:CreateFontString()
            first:SetTextScale(1.5)
            second:SetTextScale(0.75)
            local before = first:GetTextScale()
            first:SetTextScale(2.25)
            return before, first:GetTextScale(), second:GetTextScale()
            "#,
        )
        .expect("existing SetTextScale state updates and GetTextScale reads");
    assert_eq!(scales, (1.5, 2.25, 0.75));
}
