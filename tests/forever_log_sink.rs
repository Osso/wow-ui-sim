//! Real logging sink publication, not permissive namespace lookup.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_message_sink(method: &str) {
    let env = WowLuaEnv::new().unwrap();
    let other = WowLuaEnv::new().unwrap();
    let before = env.state().borrow().console_output.clone();
    let other_before = other.state().borrow().console_output.clone();
    let count: i32 = env.eval(&format!(
        r#"local sink = rawget(C_Log, '{method}'); assert(type(sink) == 'function'); return select('#', sink('quest offer: "exact"\nline 2'))"#
    )).unwrap();
    assert_eq!(count, 0);
    let mut expected = before;
    expected.push("quest offer: \"exact\"\nline 2".into());
    assert_eq!(env.state().borrow().console_output, expected);
    assert_eq!(other.state().borrow().console_output, other_before);
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_error_sink_appends_exact_message_with_zero_returns() {
    assert_message_sink("LogErrorMessage");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn retail_ptr_publish_message_and_error_sinks() {
    assert_message_sink("LogMessage");
    // Retail 12.1.5 LogDocumentation.lua:11 declares LogErrorMessage.
    assert_message_sink("LogErrorMessage");
}
