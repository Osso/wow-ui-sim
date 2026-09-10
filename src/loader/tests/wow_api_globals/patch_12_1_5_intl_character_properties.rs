use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_character_properties_return_unicode_data() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "GetCharacterProperties")) == "function")
        local cases = {
            {"A", 65, true, false, false, "Lu", "Latn", "Basic Latin"},
            {"7", 55, false, true, false, "Nd", "Zyyy", "Basic Latin"},
            {" ", 32, false, false, true, "Zs", "Zyyy", "Basic Latin"},
            {"٧", 1639, false, true, false, "Nd", "Arab", "Arabic"},
            {"́", 769, false, false, false, "Mn", "Zinh", "Combining Diacritical Marks"},
            {" ", 160, false, false, true, "Zs", "Zyyy", "Latin-1 Supplement"},
            {"😀", 128512, false, false, false, "So", "Zyyy", "Emoticons"},
            {"中", 20013, true, false, false, "Lo", "Hani", "CJK Unified Ideographs"},
            {"²", 178, false, false, false, "No", "Zyyy", "Latin-1 Supplement"}
        }
        local fields = {"codePoint", "isAlphabetic", "isDigit", "isWhitespace",
                        "generalCategory", "scriptCode", "blockCode"}
        for _, case in ipairs(cases) do
            local result = C_Intl.GetCharacterProperties(case[1])
            local count = 0
            for _ in pairs(result) do count = count + 1 end
            assert(count == 7)
            for i, field in ipairs(fields) do
                assert(result[field] == case[i + 1], field .. " for " .. case[1])
            end
            assert(select('#', C_Intl.GetCharacterProperties(case[1])) == 1)
        end
        -- U+2FE0 lies in the gap before CJK punctuation.
        local gap = C_Intl.GetCharacterProperties(string.char(226, 191, 160))
        assert(gap.codePoint == 12256 and gap.blockCode == "No_Block")
        assert(gap.generalCategory == "Cn" and gap.scriptCode == "Zzzz")
        assert(not gap.isAlphabetic and not gap.isDigit and not gap.isWhitespace)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_character_properties_validate_and_isolate_results() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "GetCharacterProperties")) == "function")
        assert(select('#', C_Intl.GetCharacterProperties("")) == 0)
        local first = C_Intl.GetCharacterProperties("A中")
        assert(first.codePoint == 65 and first.scriptCode == "Latn")
        first.codePoint = -1
        first.blockCode = "changed"
        local second = C_Intl.GetCharacterProperties("A")
        assert(first ~= second and second.codePoint == 65 and second.blockCode == "Basic Latin")
        for _, text in ipairs({string.char(255), string.char(192, 175),
                               string.char(237, 160, 128), "A" .. string.char(255)}) do
            local ok, err = pcall(C_Intl.GetCharacterProperties, text)
            assert(not ok and tostring(err):find("valid UTF%-8"))
        end
        for _, value in ipairs({false, 42, {}}) do
            assert(not pcall(C_Intl.GetCharacterProperties, value))
        end
        assert(not pcall(C_Intl.GetCharacterProperties))
        assert(CharacterProperties == nil)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_character_properties_preserve_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(CharacterProperties == nil)")
            .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
