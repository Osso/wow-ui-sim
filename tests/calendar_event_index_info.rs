//! Numeric IDs and nil-on-miss/filter outcomes are simulator fixture policy,
//! not claims about native CalendarEventID representation or native parity.

use wow_ui_sim::lua_api::WowLuaEnv;

const QUERY_ASSERTIONS: &str = r#"
        assert(type(C_Calendar) == 'table', 'public Calendar namespace')
        local get = C_Calendar.GetEventIndexInfo
        assert(type(get) == 'function', 'public event index getter')

        local function index(offset, day, position, ...)
            assert(select('#', ...) == 1, 'exactly one DTO return')
            local result = ...
            assert(type(result) == 'table', 'configured fixture must return a DTO')
            assert(result.offsetMonths == offset, 'offsetMonths')
            assert(result.monthDay == day, 'monthDay')
            assert(result.eventIndex == position, 'eventIndex')
            local count = 0
            for key in pairs(result) do
                assert(key == 'offsetMonths' or key == 'monthDay' or key == 'eventIndex',
                    'unexpected DTO field: ' .. tostring(key))
                count = count + 1
            end
            assert(count == 3, 'exact DTO fields')
        end

        local function missing(...)
            assert(select('#', ...) == 1, 'miss returns exactly one value')
            assert((...) == nil, 'miss returns nil')
        end

        local function rejected(...)
            local ok, message = pcall(get, ...)
            assert(not ok, 'invalid input must raise an error')
            assert(type(message) == 'string' and #message > 0, 'error has context')
        end
    "#;

fn query_probe(body: &str) {
    let env = WowLuaEnv::new().expect("bare calendar environment");
    env.exec(&format!("{QUERY_ASSERTIONS}\n{body}"))
        .expect("public calendar event index behavior");
}

#[test]
fn calendar_event_index_default_session_returns_three_exact_dtos() {
    query_probe(
        r#"
        index(0, 15, 1, get(910001))
        index(0, 15, 2, get(910002))
        index(1, 4, 1, get(910003))
        "#,
    );
}

#[test]
fn calendar_event_index_omitted_nil_and_matching_filters_return_dtos() {
    query_probe(
        r#"
        for _, event in ipairs({
            {910001, 0, 15, 1}, {910002, 0, 15, 2}, {910003, 1, 4, 1}
        }) do
            local id, offset, day, position = unpack(event)
            index(offset, day, position, get(id))
            index(offset, day, position, get(id, nil))
            index(offset, day, position, get(id, nil, nil))
            index(offset, day, position, get(id, offset))
            index(offset, day, position, get(id, offset, nil))
            index(offset, day, position, get(id, nil, day))
            index(offset, day, position, get(id, offset, day))
        end
        "#,
    );
}

#[test]
fn calendar_event_index_each_mismatching_filter_returns_one_nil() {
    query_probe(
        r#"
        for _, event in ipairs({{910001, 0, 15}, {910002, 0, 15}, {910003, 1, 4}}) do
            local id, offset, day = unpack(event)
            missing(get(id, offset + 1))
            missing(get(id, offset + 1, nil))
            missing(get(id, nil, day + 1))
            missing(get(id, offset + 1, day))
            missing(get(id, offset, day + 1))
            missing(get(id, offset + 1, day + 1))
        end
        index(0, 15, 1, get(910001, 0, 15))
        "#,
    );
}

#[test]
fn calendar_event_index_unknown_identity_returns_one_nil() {
    query_probe(
        r#"
        for _, id in ipairs({910000, 910004, 999999}) do
            missing(get(id))
            missing(get(id, nil, nil))
            missing(get(id, 0))
            missing(get(id, nil, 15))
            missing(get(id, 0, 15))
        end
        index(0, 15, 2, get(910002))
        "#,
    );
}

#[test]
fn calendar_event_index_result_tables_are_independent_snapshots() {
    query_probe(
        r#"
        local first, second = get(910001), get(910001, nil, 15)
        index(0, 15, 1, first)
        index(0, 15, 1, second)
        assert(not rawequal(first, second), 'each query returns a fresh table')
        first.offsetMonths, first.monthDay, first.eventIndex = 8, nil, 99
        first.extra = 'caller mutation'
        index(0, 15, 1, second)
        local third = get(910001, 0, 15)
        assert(not rawequal(second, third), 'subsequent snapshot is also fresh')
        index(0, 15, 1, third)
        index(0, 15, 2, get(910002))
        index(1, 4, 1, get(910003))
        missing(get(910001, 8))
        "#,
    );
}

#[test]
fn calendar_event_index_observes_owned_state_updates_and_removal() {
    let env = WowLuaEnv::new().expect("calendar environment");
    env.exec(&format!(
        "{QUERY_ASSERTIONS}\nCalendarBefore = get(910001)\nindex(0, 15, 1, CalendarBefore)"
    ))
    .expect("initial owned record");
    {
        let mut state = env.state().borrow_mut();
        let record = state
            .calendar_event_indices
            .get_mut(&910001)
            .expect("seeded record");
        record.offset_months = 2;
        record.month_day = 20;
        record.event_index = 4;
    }
    env.exec(&format!(
        r#"{QUERY_ASSERTIONS}
        index(2, 20, 4, get(910001, 2, 20))
        missing(get(910001, 0, 15))
        index(0, 15, 1, CalendarBefore)
        index(0, 15, 2, get(910002))
        "#
    ))
    .expect("updated position and preserved snapshots");
    env.state()
        .borrow_mut()
        .calendar_event_indices
        .remove(&910001);
    env.exec(&format!(
        r#"{QUERY_ASSERTIONS}
        missing(get(910001))
        missing(get(910001, 2, 20))
        index(0, 15, 1, CalendarBefore)
        index(1, 4, 1, get(910003))
        "#
    ))
    .expect("removed record without affecting other identities");
}

#[test]
fn calendar_event_index_restoration_preserves_namespace_and_unrelated_functions() {
    let env = WowLuaEnv::new().expect("calendar environment");
    env.exec(
        r#"
        CalendarNamespaceBefore = C_Calendar
        CalendarInviteSentinel = function() return 7 end
        C_Calendar.GetNumInvites = CalendarInviteSentinel
        C_Calendar.GetEventIndexInfo = nil
    "#,
    )
    .expect("calendar cleanup fixture");
    env.restore_post_cleanup_globals();
    // ad5b8fb59 (citing a07283768): post-cleanup restore does not re-register public
    // namespace members, so the cleared getter stays nil.
    env.exec(
        r#"
        assert(C_Calendar.GetEventIndexInfo == nil, 'cleared public getter stays nil')
        assert(rawequal(C_Calendar, CalendarNamespaceBefore), 'namespace preserved')
        assert(rawequal(C_Calendar.GetNumInvites, CalendarInviteSentinel), 'unrelated function preserved')
        assert(C_Calendar.GetNumInvites() == 7)
        "#,
    )
    .expect("cleared getter stays nil and Calendar namespace is preserved");
}

#[test]
fn calendar_event_index_required_and_optional_numbers_reject_wrong_types() {
    query_probe(
        r#"
        rejected()
        rejected(nil)
        rejected(nil, 0, 15)
        for _, value in ipairs({'910001', '0', false, true, {}, function() end}) do
            rejected(value)
            rejected(910001, value)
            rejected(910001, nil, value)
            rejected(910001, 0, value)
            rejected(999999, value, 15)
            rejected(999999, nil, value)
        end
        index(0, 15, 1, get(910001, nil, 15))
        "#,
    );
}

#[test]
fn calendar_event_index_invalid_numbers_cannot_alias_fixture_positions() {
    query_probe(
        r#"
        -- Fractions would alias live fixtures if silently truncated.
        for _, id in ipairs({910001.5, 910002.5, 910003.5}) do
            missing(get(id))
        end
        missing(get(910001, 0.5, 15))
        missing(get(910001, nil, 15.5))
        missing(get(910003, 1.5, 4))
        missing(get(910003, nil, 4.5))
        -- Numeric values outside the configured identity/filter values do not alias them.
        for _, value in ipairs({0/0, math.huge, -math.huge, 1e308, -1e308}) do
            missing(get(value))
            missing(get(910001, value, 15))
            missing(get(910001, nil, value))
            missing(get(999999, value))
            missing(get(999999, nil, value))
        end
        index(0, 15, 1, get(910001, 0, 15))
        index(1, 4, 1, get(910003, 1, 4))
        "#,
    );
}

#[test]
#[cfg(feature = "retail-12-1-0")]
fn calendar_event_index_untainted_secret_inputs_preserve_wrappers() {
    query_probe(
        r#"
        debug.settaintmode(true)
        assert(debug.getstacktaint() == nil, 'untainted entry')
        local id, offset, day = secretwrap(910001), secretwrap(0), secretwrap(15)
        index(0, 15, 1, get(id))
        index(0, 15, 1, get(910001, offset))
        index(0, 15, 1, get(910001, nil, day))
        index(0, 15, 1, get(id, offset, day))
        missing(get(secretwrap(999999), offset, day))
        missing(get(id, secretwrap(1), day))
        for _, value in ipairs({id, offset, day}) do
            assert(issecretvalue(value), 'query preserves secret wrapper')
        end
        assert(secretunwrap(id) == 910001 and secretunwrap(offset) == 0)
        assert(secretunwrap(day) == 15)
        assert(debug.getstacktaint() == nil, 'query preserves caller context')
        "#,
    );
}

#[test]
#[cfg(feature = "retail-12-1-0")]
fn calendar_event_index_tainted_secret_inputs_are_denied_before_lookup() {
    query_probe(
        r#"
        debug.settaintmode(true)
        local id, offset, day = secretwrap(910001), secretwrap(0), secretwrap(15)
        local function addon()
            assert(debug.getstacktaint() == 'CalendarIndexProbe', 'tainted entry')
            index(0, 15, 1, get(910001, 0, 15))
            missing(get(999999, 0, 15))
            for _, args in ipairs({
                {id, nil, nil}, {910001, offset, nil}, {910001, nil, day},
                {id, offset, day}, {999999, offset, nil}, {999999, nil, day},
                {false, offset, nil}, {nil, nil, day}
            }) do
                local ok, message = pcall(get, unpack(args, 1, 3))
                assert(not ok and type(message) == 'string')
                assert(string.find(message, 'untainted caller', 1, true),
                    'authentication precedes validation and lookup: ' .. tostring(message))
                assert(debug.getstacktaint() == 'CalendarIndexProbe', 'taint preserved')
            end
            for _, value in ipairs({id, offset, day}) do
                assert(issecretvalue(value), 'denial preserves wrapper')
                assert(not pcall(secretunwrap, value), 'no secret declassification')
            end
            index(0, 15, 1, get(910001))
        end
        debug.setobjecttaint(addon, 'CalendarIndexProbe')
        addon()
        assert(debug.getstacktaint() == nil, 'untainted caller restored')
        index(0, 15, 1, get(id, offset, day))
        "#,
    );
}
