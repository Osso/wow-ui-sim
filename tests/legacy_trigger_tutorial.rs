use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn trigger_tutorial_publication_matches_pinned_legacy_consumers() {
    let env = WowLuaEnv::new().expect("environment");
    let published: bool = env
        .eval("return type(rawget(_G, 'TriggerTutorial')) == 'function'")
        .expect("raw publication query");
    let has_consumer = matches!(
        ACTIVE,
        ClientProfile::Mists
            | ClientProfile::Era
            | ClientProfile::Anniversary
            | ClientProfile::Wrath
    );
    // Retail 12.1.5 publishes `_G.TriggerTutorial` (VERIFIED R88y row, e93399f5e3).
    let retail_published = cfg!(all(feature = "client-retail", feature = "retail-12-1-5"));
    assert_eq!(published, has_consumer || retail_published);
    assert!(env.state().borrow().tutorial_requests.is_empty());
}

#[cfg(any(
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary",
    feature = "client-wrath"
))]
#[test]
fn trigger_tutorial_records_ordered_requests_without_tutorial_policy() {
    let env = WowLuaEnv::new().expect("environment");
    let other = WowLuaEnv::new().expect("independent environment");
    env.exec(
        r#"
        local events = 0
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("TUTORIAL_TRIGGER")
        listener:SetScript("OnEvent", function() events = events + 1 end)
        TriggerTutorial(59)
        TriggerTutorial(58)
        TriggerTutorial(59)
        assert(events == 0, "the recorder must not invent event dispatch")
        "#,
    )
    .expect("legacy tutorial requests");
    assert_eq!(env.state().borrow().tutorial_requests, [59, 58, 59]);
    assert!(other.state().borrow().tutorial_requests.is_empty());
    other.exec("TriggerTutorial(2)").expect("separate request");
    assert_eq!(other.state().borrow().tutorial_requests, [2]);
    assert_eq!(env.state().borrow().tutorial_requests, [59, 58, 59]);
}
