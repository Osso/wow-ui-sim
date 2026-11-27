//! C_StringUtil helpers that Forever 70707 declares with the Retail signatures
//! (StringUtilDocumentation.lua:10/:57/:73/:105/:120/:137/:172/:187/:203).
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const ESCAPES_AND_WRAP: &str = r#"
    assert(C_StringUtil.EscapeLuaFormatString("50%") == "50%%")
    assert(C_StringUtil.EscapeLuaFormatString("a%b%c") == "a%%b%%c")
    assert(C_StringUtil.EscapeLuaPatterns("1+1") == "1%+1")
    assert(C_StringUtil.EscapeLuaPatterns("[x].") == "%[x%]%.")
    assert(C_StringUtil.WrapString("mid", "<", ">") == "<mid>")
    assert(C_StringUtil.WrapString("x", "(", ")") == "(x)")
    assert(C_StringUtil.WrapString("", "(", ")") == "")
"#;

const NUMBERS_AND_SPACES: &str = r#"
    assert(C_StringUtil.RemoveContiguousSpaces("a    b", 1) == "a b")
    assert(C_StringUtil.RemoveContiguousSpaces("a    b  c", 2) == "a  b  c")
    assert(C_StringUtil.TruncateWhenZero(0) == "")
    assert(C_StringUtil.TruncateWhenZero(7.9) == "7")
    assert(C_StringUtil.FloorToNearestString(2.7) == "2")
    assert(C_StringUtil.FloorToNearestString(-2.2) == "-3")
    assert(C_StringUtil.RoundToNearestString(2.5) == "3")
    assert(C_StringUtil.RoundToNearestString(2.4) == "2")
"#;

const MARKUP_AND_FORMATTER: &str = r#"
    local loose = "a|TInterface\\Icons\\Foo:16|tb"
    assert(C_StringUtil.StripTextureMarkupForLooseFiles(loose) == "ab")
    local byID = "a|T134400:16|tb"
    assert(C_StringUtil.StripTextureMarkupForLooseFiles(byID) == byID)
    local formatter = C_StringUtil.CreateAbbreviatedNumberFormatter()
    assert(formatter:FormatNumber(123456) == "123k")
    assert(formatter:FormatNumber(-1234) == "-1.2k")
    assert(formatter:FormatNumber(42) == "42")
"#;

#[test]
fn string_util_helpers_follow_their_inputs() {
    let env = WowLuaEnv::new().expect("Lua environment");
    for (label, chunk) in [
        ("escapes and wrap", ESCAPES_AND_WRAP),
        ("numbers and spaces", NUMBERS_AND_SPACES),
        ("markup and formatter", MARKUP_AND_FORMATTER),
    ] {
        env.exec(chunk)
            .unwrap_or_else(|error| panic!("{label}: {error}"));
    }
}
