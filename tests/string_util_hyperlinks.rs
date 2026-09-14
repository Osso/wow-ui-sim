#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_cases(cases: &str) {
    WowLuaEnv::new()
        .expect("Lua environment")
        .exec(&format!(
            r##"
            local function check(expected, ...)
                assert(select("#", ...) == 1, "expected one result")
                local actual = ...
                assert(type(actual) == "string", "expected string result")
                assert(actual == expected,
                    string.format("expected %q, got %q", expected, actual))
            end
            local strip = C_StringUtil.StripHyperlinks
            {cases}
            "##,
        ))
        .expect("StripHyperlinks output contract");
}

#[test]
fn strip_hyperlinks_defaults_remove_markup_preserving_utf8_text() {
    assert_cases(
        r#"
        check("avant Épée 雪 après",
            strip("avant |cFF00FF00|Hitem:12345|h[Épée 雪]|h|r après"))
        check("leftmiddleright", strip("left|Aatlas-name:16:16|amiddle|TInterface\\Icons\\Spell:16|tright"))
        check("", strip(""))
        check("café 雪 (x) {y} <z> + / : !\nend", strip("café 雪 (x) {y} <z> + / : !\nend"))
        "#,
    );
}

#[test]
fn strip_hyperlinks_color_and_bracket_flags_are_independent() {
    assert_cases(
        r#"
        local text = "|cFF00FF00|Hitem:12345|h[Épée]|h|r [plain]"
        check("|cFF00FF00Épée|r plain", strip(text, true, false, false, false, false))
        check("[Épée] [plain]", strip(text, false, true, false, false, false))
        check("|cFF00FF00[Épée]|r [plain]", strip(text, true, true, false, false, false))
        check("abc", strip("[a]b[c]", false, false, false, false, false))
        check("[a]b[c]", strip("[a]b[c]", false, true, false, false, false))
        "#,
    );
}

#[test]
fn strip_hyperlinks_atlas_and_texture_flags_are_independent() {
    assert_cases(
        r#"
        local atlas = "|Aatlas-name:16:16|a"
        local texture = "|TInterface\\Icons\\Spell:16|t"
        local text = "L" .. atlas .. "M" .. texture .. "R"
        check("L" .. atlas .. "MR", strip(text, false, false, false, true, false))
        check("LM" .. texture .. "R", strip(text, false, false, false, false, true))
        check(text, strip(text, false, false, false, true, true))
        check("LMR", strip(text, false, false, false, false, false))
        "#,
    );
}

#[test]
fn strip_hyperlinks_talent_consumer_flag_combination() {
    assert_cases(
        r#"
        local atlas = "|Aatlas-name:16:16|a"
        local texture = "|TInterface\\Icons\\Spell:16|t"
        local text = "|cFF00FF00|Hspell:42|h[Élan]|h|r " .. atlas .. texture
        check("[Élan] " .. atlas .. texture, strip(text, false, true, false, true, true))
        "#,
    );
}

#[test]
fn strip_hyperlinks_newline_false_preserves_literal_sequence_policy() {
    // Literal |n preservation is simulator policy, not native-output evidence.
    assert_cases(
        r#"
        local text = "first|n雪\nlast|n"
        check(text, strip(text))
        check(text, strip(text, false, false, false, false, false))
        check("first雪\nlast", strip(text, false, false, true, false, false))
        check("[first]雪", strip("[first]|n雪", true, true, true, true, true))
        "#,
    );
}
