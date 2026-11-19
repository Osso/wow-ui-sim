//! `bit` library results, captured from the former Lua fallback.
//!
//! Every expected value below was printed by the Lua implementation that
//! `compat_overrides.rs` installed before the native library replaced it
//! (handoff `questie-login-perf/PROOF/bit-golden-base.stdout`). The cases cover
//! nil holes in variadic calls, numeric strings, non-number arguments, NaN,
//! infinities, negative and fractional input, values past 2^32 and 2^53, and
//! `bit.mod` divisors that truncate to zero.

use wow_ui_sim::lua_api::WowLuaEnv;

const CASES_LUA: &str = r#"
local cases = {
    {"bit.band(12, 10)", "8"}, {"bit.band(4294967295, 2^31 + 5)", "2147483653"},
    {"bit.band(12, 10, 6)", "0"}, {"bit.band()", "4294967295"}, {"bit.band(12)", "12"},
    {"bit.band(12, nil, 10)", "0"}, {"bit.band(12, 10, nil)", "8"},
    {"bit.band(nil, nil, 7)", "0"}, {"bit.band(-1, 255)", "255"},
    {"bit.band(-2^32 - 1, 255)", "255"}, {"bit.band(3.999, 7)", "3"},
    {"bit.band(-1.5, 3)", "2"}, {"bit.band(2^32 + 5, 7)", "5"},
    {"bit.band(2^60 + 4096, 2^32 - 1)", "4096"}, {"bit.band('0x10', 31)", "16"},
    {"bit.band(' 12 ', '10')", "8"}, {"bit.band('abc', 7)", "0"}, {"bit.band(true, 7)", "0"},
    {"bit.band({}, 7)", "0"}, {"bit.band(0/0, 7)", "0"}, {"bit.band(1/0, 7)", "0"},
    {"bit.band(-1/0, 7)", "0"},
    {"bit.bor()", "0"}, {"bit.bor(12, 10)", "14"}, {"bit.bor(1, 2, 4, 8)", "15"},
    {"bit.bor(0/0, 5)", "5"}, {"bit.bor(5, nil)", "5"}, {"bit.bor(-1, 0)", "4294967295"},
    {"bit.bor('1e3', 1)", "1001"},
    {"bit.bxor(12, 10)", "6"}, {"bit.bxor(0/0, 5)", "7"}, {"bit.bxor(5, 0/0)", "7"},
    {"bit.bxor(0/0, 0/0)", "0"}, {"bit.bxor(-1, 1)", "4294967294"}, {"bit.bxor(nil, 9)", "9"},
    {"bit.bnot(0)", "4294967295"}, {"bit.bnot(-1)", "0"}, {"bit.bnot(1/0)", "nan"},
    {"bit.bnot('5')", "4294967290"}, {"bit.bnot(2^32)", "4294967295"},
    {"bit.lshift(1, 3)", "8"}, {"bit.lshift(1, 31)", "2147483648"}, {"bit.lshift(1, 32)", "0"},
    {"bit.lshift(3, 31)", "2147483648"}, {"bit.lshift(1, 1024)", "nan"},
    {"bit.lshift(0, 1024)", "nan"}, {"bit.lshift(5, -1)", "nan"}, {"bit.lshift(5, 0/0)", "nan"},
    {"bit.rshift(16, 2)", "4"}, {"bit.rshift(-1, 28)", "15"}, {"bit.rshift(1, 1024)", "0"},
    {"bit.rshift(5, 0/0)", "nan"}, {"bit.arshift(16, 2)", "4"}, {"bit.arshift(-1, 31)", "1"},
    {"bit.mod(17, 5)", "2"}, {"bit.mod(5, 0.5)", "2"}, {"bit.mod(5, 0)", "inf"},
    {"bit.mod(5, -0.5)", "-2"}, {"bit.mod(5, '-0')", "-inf"}, {"bit.mod(-3, 5)", "3"},
    {"bit.mod(17, -5)", "17"}, {"bit.mod(5, nil)", "nan"}, {"bit.mod(5, 'x')", "nan"},
    {"bit.mod(0/0, 3)", "nan"},
    {"bit.arshift == bit.rshift", "true"}, {"select('#', bit.band(1, 1))", "1"},
}
local function show(value)
    if type(value) ~= "number" then return tostring(value) end
    if value ~= value then return "nan" end
    return string.format("%.17g", value)
end
local mismatches = {}
for _, case in ipairs(cases) do
    local actual = show(loadstring("return " .. case[1])())
    if actual ~= case[2] then
        mismatches[#mismatches + 1] = case[1] .. " = " .. actual .. ", expected " .. case[2]
    end
end
return #cases .. ":" .. table.concat(mismatches, "; ")
"#;

#[test]
fn bit_library_matches_former_lua_fallback_results() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env.eval(CASES_LUA).unwrap();
    assert_eq!(result, "66:");
}
