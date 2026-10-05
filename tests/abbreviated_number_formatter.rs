#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

const SETUP: &str = r#"
    local clock = C_DurationUtil.CreateManualClock(0)
    function duration(number)
        local d = C_DurationUtil.CreateDuration()
        d:SetClock(clock)
        d:SetTimeFromStart(0, number)
        return d
    end
    function row(threshold, suffix, significand, fraction)
        return {breakpoint=threshold, abbreviation=suffix,
            significandDivisor=significand, fractionDivisor=fraction,
            abbreviationIsGlobal=false}
    end
    formatter = C_StringUtil.CreateAbbreviatedNumberFormatter()
"#;

fn execute(code: &str) {
    let env = WowLuaEnv::new().unwrap();
    env.exec(SETUP).expect("abbreviated formatter factory");
    env.exec(code).expect("abbreviated formatter behavior");
}

#[test]
fn documented_default_and_duration_format_methods() {
    execute(
        r#"
        assert(type(formatter) == 'userdata')
        assert(formatter:FormatNumber(123456) == '123k', 'documented common NumericFormatter method')
        assert(formatter:FormatNumber(-1234) == '-1.2k')
        assert(not pcall(formatter.FormatNumber, newproxy(), 123456))
        assert(not pcall(formatter.FormatNumber, formatter, math.huge))
        assert(not pcall(formatter.FormatNumber, formatter, secretwrap('123456')))
        local d = duration(123456)
        assert(d:FormatTotalDuration(formatter) == '123k')
        assert(d:FormatRemainingDuration(formatter) == '123k')
        d:GetClock():SetTime(12345)
        assert(d:FormatElapsedDuration(formatter) == '12k')
        assert(duration(1234):FormatTotalDuration(formatter) == '1.2k')
        assert(duration(42):FormatTotalDuration(formatter) == '42')
        d:SetTimeFromStart(0, 246912, 2)
        assert(d:FormatTotalDuration(formatter) == '123k')
        assert(d:FormatTotalDuration(formatter, 1) == '246k')
        assert(not pcall(d.FormatTotalDuration, d, {}))
        assert(not pcall(d.FormatTotalDuration, d, newproxy()))
        assert(not pcall(d.FormatTotalDuration, d, formatter, 2))
    "#,
    );
}

#[test]
fn custom_breakpoints_global_lookup_and_private_copies() {
    execute(
        r#"
        CUSTOM_ABBREVIATION = ' chips'
        local config = {row(10000, 'K', 1000, 1), row(1000, 'K', 100, 10)}
        formatter:SetBreakpoints(config)
        config[2].abbreviation = 'changed'
        assert(duration(1234):FormatTotalDuration(formatter) == '1.2K')
        assert(duration(12345):FormatTotalDuration(formatter) == '12K')
        local readback = formatter:GetBreakpoints()
        assert(readback[1].breakpoint == 1000 and readback[1].fractionDivisor == 10)
        readback[1].abbreviation = 'changed'
        assert(duration(1234):FormatTotalDuration(formatter) == '1.2K')
        local copied = formatter:Copy()
        assert(copied ~= formatter and type(copied) == 'userdata' and not issecretvalue(copied))
        formatter:AddBreakpoint({breakpoint=1000000, abbreviation='CUSTOM_ABBREVIATION',
            significandDivisor=100000, fractionDivisor=10})
        assert(duration(1234567):FormatTotalDuration(formatter) == '1.2 chips')
        assert(duration(1234567):FormatTotalDuration(copied) == '1234K')
        formatter:SetBreakpoints({{breakpoint=1000, abbreviation='FIRST_NUMBER_CAP_NO_SPACE',
            significandDivisor=100, fractionDivisor=10}})
        assert(duration(1234):FormatTotalDuration(formatter) == '1.2K')
        formatter:ClearBreakpoints()
        assert(#formatter:GetBreakpoints() == 0)
        assert(duration(123456):FormatTotalDuration(formatter) == '123456')
        assert(duration(1234):FormatTotalDuration(copied) == '1.2K')
        formatter:ResetBreakpoints()
        assert(duration(123456):FormatTotalDuration(formatter) == '123k')
        formatter:SetBreakpoints({})
        assert(duration(123456):FormatTotalDuration(formatter) == '123456')
    "#,
    );
}

#[test]
fn invalid_breakpoints_are_atomic() {
    execute(
        r#"
        formatter:SetBreakpoints({row(1000, 'K', 100, 10)})
        local invalid = {
            row(0, 'K', 100, 10), row(-10, 'K', 100, 10),
            row(1000, 'K', 0, 10), row(1000, 'K', 100, 0),
            row(1000, 'K', 11, 10), row(0/0, 'K', 100, 10),
            row(math.huge, 'K', 100, 10), {}, false,
            {breakpoint=1000, abbreviation=4, significandDivisor=100, fractionDivisor=10},
            {breakpoint=1000, abbreviation='K', significandDivisor=100, fractionDivisor=10,
                abbreviationIsGlobal=7},
        }
        for _, bad in ipairs(invalid) do
            assert(not pcall(formatter.SetBreakpoints, formatter, {row(10000,'K',1000,1), bad}))
            assert(not pcall(formatter.AddBreakpoint, formatter, bad))
            assert(#formatter:GetBreakpoints() == 1)
            assert(duration(1234):FormatTotalDuration(formatter) == '1.2K')
        end
        assert(not pcall(formatter.AddBreakpoint, formatter, row(1000, 'other', 100, 10)))
        assert(not pcall(formatter.SetBreakpoints, formatter, {row(1000,'K',100,10), row(1000,'X',100,10)}))
        assert(duration(1234):FormatTotalDuration(formatter) == '1.2K')
        assert(not pcall(formatter.SetBreakpoints, formatter, {row(1000,'K',100,10), [3]=row(10000,'K',1000,1)}))
        assert(not pcall(formatter.SetBreakpoints, formatter, newproxy()))
    "#,
    );
}

#[test]
fn secret_duration_and_breakpoints_preserve_access_boundaries() {
    execute(
        r#"
        local d = duration(secretwrap(123456))
        local direct = formatter:FormatNumber(secretwrap(123456))
        assert(issecretvalue(direct) and secretunwrap(direct) == '123k')
        local text = d:FormatTotalDuration(formatter)
        assert(issecretvalue(text) and secretunwrap(text) == '123k')
        local config = row(1000, 'K', 100, 10)
        config.abbreviation = secretwrap('K')
        formatter:SetBreakpoints({config})
        local output = duration(1234):FormatTotalDuration(formatter)
        assert(issecretvalue(output) and secretunwrap(output) == '1.2K')
        local copied = formatter:Copy()
        assert(not issecretvalue(copied))
        assert(issecretvalue(copied:GetBreakpoints()[1].abbreviation))
        local plain = C_StringUtil.CreateAbbreviatedNumberFormatter()
        local ordinary = duration(123456)
        local wrappedRows = secretwrap({row(1000,'K',100,10)})
        local secretwrapNumber = secretwrap(123456)
        local function tainted()
            assert(not issecure())
            assert(not pcall(secretwrap, 1))
            assert(not pcall(d.FormatTotalDuration, d, plain))
            assert(not pcall(ordinary.FormatTotalDuration, ordinary, formatter))
            assert(not pcall(formatter.FormatNumber, formatter, 1234))
            assert(not pcall(plain.FormatNumber, plain, secretwrapNumber))
            assert(not pcall(plain.SetBreakpoints, plain, wrappedRows))
            assert(not pcall(plain.AddBreakpoint, plain, config))
            assert(ordinary:FormatTotalDuration(plain) == '123k')
            assert(not issecure(), 'formatting must not clear caller taint')
        end
        debug.setobjecttaint(tainted, 'AbbreviatedFormatterProbe')
        tainted()
        assert(ordinary:FormatTotalDuration(plain) == '123k')
        assert(not pcall(plain.SetBreakpoints, plain, {row(1000, secretwrap(5), 100, 10)}))
        assert(ordinary:FormatTotalDuration(plain) == '123k')
        formatter:ClearBreakpoints()
        assert(not issecretvalue(ordinary:FormatTotalDuration(formatter)))
        formatter:ResetBreakpoints()
        assert(not issecretvalue(ordinary:FormatTotalDuration(formatter)))
    "#,
    );
}

#[test]
fn default_abbreviation_breakpoints_match_new_formatter_rows() {
    execute(
        r#"
        local defaults = C_StringUtil.GetDefaultAbbreviationBreakpoints()
        assert(#defaults == 6)
        local first, last = defaults[1], defaults[6]
        assert(first.breakpoint == 1000 and first.abbreviation == 'k')
        assert(first.significandDivisor == 100 and first.fractionDivisor == 10)
        assert(first.abbreviationIsGlobal == false)
        assert(last.breakpoint == 10000000000 and last.abbreviation == 'b')
        assert(last.significandDivisor == 1000000000 and last.fractionDivisor == 1)
        local rows = formatter:GetBreakpoints()
        for index, row in ipairs(rows) do
            for key, value in pairs(row) do assert(defaults[index][key] == value) end
        end
        assert(#C_StringUtil.GetDefaultAbbreviationBreakpoints('enGB') == 6)
        -- Returned rows are copies; editing them leaves the defaults intact.
        defaults[1].abbreviation = 'x'
        assert(C_StringUtil.GetDefaultAbbreviationBreakpoints()[1].abbreviation == 'k')
        local custom = C_StringUtil.CreateAbbreviatedNumberFormatter()
        custom:SetBreakpoints(C_StringUtil.GetDefaultAbbreviationBreakpoints('enUS'))
        assert(custom:FormatNumber(1234) == '1.2k')
        assert(not pcall(C_StringUtil.GetDefaultAbbreviationBreakpoints, 'xxXX'))
        assert(not pcall(C_StringUtil.GetDefaultAbbreviationBreakpoints, 1))
    "#,
    );
}
