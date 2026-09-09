//! Byte-oriented simulator contract; native locale/security semantics unverified.

use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_string_extensions_matching() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'contains', 'startswith', 'endswith'}) do
            local f = string[name]
            assert(type(f) == 'function', name .. ' missing')
            assert(f('', '') == true)
            assert(f('', 'x') == false)
            assert(f('abc', '') == true)
            assert(f('abc', 'abc') == true)
            assert(f('abc', 'abcd') == false)
            assert(f('abc', 'ABC') == false)
            assert(select('#', f('abc', 'a')) == 1)
        end
        assert(string.contains('abc', 'b'))
        assert(not string.contains('abc', 'z'))
        assert(string.contains('ababababac', 'ababac'))
        assert(not string.contains('aaaaaaaaab', 'aaaaac'))
        assert(string.startswith('abc', 'ab'))
        assert(not string.startswith('abc', 'bc'))
        assert(string.endswith('abc', 'bc'))
        assert(not string.endswith('abc', 'ab'))
        assert(string.contains('x.*%[]y', '.*%[]'))
        assert(not string.contains('abc', '.'))
        local bytes = string.char(255, 0, 128)
        assert(string.contains(bytes, string.char(0, 128)))
        assert(string.startswith(bytes, string.char(255, 0)))
        assert(string.endswith(bytes, string.char(0, 128)))
        assert(not string.contains(bytes, string.char(254)))
        assert(bytes:startswith(string.char(255)))
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_string_extensions_trimming() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(string.ltrim) == 'function', 'ltrim missing')
        assert(type(string.rtrim) == 'function', 'rtrim missing')
        local ws = ' \r\n\t'
        assert(string.ltrim(ws .. 'abc' .. ws) == 'abc' .. ws)
        assert(string.rtrim(ws .. 'abc' .. ws) == ws .. 'abc')
        for _, f in ipairs({string.ltrim, string.rtrim}) do
            assert(f('') == '')
            assert(f(ws) == '')
            assert(f('abc') == 'abc')
            assert(f(' a ', '') == ' a ')
            assert(f('aba', 'ab') == '')
            assert(f(string.char(11, 12)) == string.char(11, 12))
            assert(select('#', f('abc')) == 1)
        end
        assert(string.ltrim('abbaXaba', 'ab') == 'Xaba')
        assert(string.rtrim('abbaXaba', 'ab') == 'abbaX')
        assert(string.ltrim('.*%[]X.*', '.*%[]') == 'X.*')
        assert(string.rtrim('.*X.*%[]', '.*%[]') == '.*X')
        local bytes = string.char(255, 0, 128)
        assert(string.ltrim(' ' .. bytes .. ' ') == bytes .. ' ')
        assert(string.rtrim(' ' .. bytes .. ' ') == ' ' .. bytes)
        assert(string.ltrim(bytes, string.char(255, 0)) == string.char(128))
        assert(string.rtrim(bytes, string.char(0, 128)) == string.char(255))
        assert((' x '):ltrim() == 'x ')
    "#,
    )
    .unwrap();
}

#[test]
fn patch_12_1_5_string_extensions_preserves_legacy_trim() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(string.trim(' \tfoo\r\n ') == 'foo')
        assert(strtrim(' \tfoo\r\n ') == 'foo')
        assert(strtrim('xxfooxx', 'x') == 'foo')
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_string_extensions_absent_on_retail() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'contains', 'startswith', 'endswith', 'ltrim', 'rtrim'}) do
            assert(string[name] == nil, name .. ' leaked into retail')
        end
    "#,
    )
    .unwrap();
}
