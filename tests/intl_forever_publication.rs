//! Forever 1.60.1 publishes C_Intl: its pinned
//! `Blizzard_APIDocumentationGenerated/IntlDocumentation.lua` is byte-identical
//! to the Retail 12.1.5 file. No Forever Blizzard Lua calls C_Intl, so the test
//! uses the public API. ICU output is simulator behavior, not native evidence.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const CASING_AND_LENGTH: &str = r#"
    assert(C_Intl.ToUpper("abc") == "ABC")
    assert(C_Intl.ToUpper("straße") == "STRASSE")
    assert(C_Intl.ToLower("ÀB") == "àb")
    assert(C_Intl.Length("abc") == 3)
    assert(C_Intl.Length("日本") == 2)
    assert(C_Intl.GetCurrentLocale() == GetLocale())
"#;

const CONTEXT_FORMATTING: &str = r#"
    local en = C_Intl.CreateLocaleContext("en-US")
    local de = C_Intl.CreateLocaleContext("de-DE")
    assert(en:FormatNumber(1234.5, 0) == "1,234.5")
    assert(de:FormatNumber(1234.5, 0) == "1.234,5")
    assert(C_Intl.CompareStrings("a", "b", 2) < 0)
    assert(C_Intl.CompareStrings("b", "a", 2) > 0)
"#;

#[test]
fn forever_intl_operations_follow_their_inputs() {
    let env = WowLuaEnv::new().expect("Lua environment");
    for (label, chunk) in [
        ("casing and length", CASING_AND_LENGTH),
        ("context formatting", CONTEXT_FORMATTING),
    ] {
        env.exec(chunk)
            .unwrap_or_else(|error| panic!("{label}: {error}"));
    }
}
