//! Event-registration surface for `Blizzard_AdventureMap`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_AdventureMap";
const FRAME_EVENT: &str = "ADVENTURE_MAP_UPDATE_INSETS";
const DATA_PROVIDER_EVENTS: &[&str] = &[
    "ADVENTURE_MAP_UPDATE_POIS",
    "ADVENTURE_MAP_QUEST_UPDATE",
    "QUEST_ACCEPTED",
];
const QUEST_DIALOG_SHOW_EVENTS: &[&str] = &["ADVENTURE_MAP_QUEST_UPDATE", "QUEST_LOG_UPDATE"];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_frame_and_quest_data_providers_register_events() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let frame_event_registered: bool = env
            .eval(&format!(
                "return AdventureMapFrame:IsEventRegistered({FRAME_EVENT:?})"
            ))
            .expect("AdventureMapFrame:IsEventRegistered probe must run cleanly");

        assert!(
            frame_event_registered,
            "`AdventureMapMixin:OnLoad` must register `{FRAME_EVENT}` on AdventureMapFrame"
        );

        for provider_name in QUEST_DATA_PROVIDER_MIXINS {
            assert_data_provider_events(env, provider_name);
        }
    });
}
}

// Wrath has no AdventureMap vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn quest_choice_dialog_registers_events_only_while_shown() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for event in QUEST_DIALOG_SHOW_EVENTS {
            let lifecycle: QuestDialogEventLifecycle = env
                .eval(&format!(
                    r#"
                    AdventureMapQuestChoiceDialog:Hide()
                    local beforeShow = AdventureMapQuestChoiceDialog:IsEventRegistered({event:?})
                    AdventureMapQuestChoiceDialog:Show()
                    local afterShow = AdventureMapQuestChoiceDialog:IsEventRegistered({event:?})
                    AdventureMapQuestChoiceDialog:Hide()
                    local afterHide = AdventureMapQuestChoiceDialog:IsEventRegistered({event:?})
                    return beforeShow, afterShow, afterHide
                    "#
                ))
                .expect("AdventureMapQuestChoiceDialog show/hide event probe must run cleanly");

            assert_quest_dialog_event_lifecycle(event, lifecycle);
        }
    });
}
}

type QuestDialogEventLifecycle = (bool, bool, bool);

const QUEST_DATA_PROVIDER_MIXINS: &[&str] = &[
    "AdventureMap_QuestChoiceDataProviderMixin",
    "AdventureMap_QuestOfferDataProviderMixin",
];

fn assert_data_provider_events(env: &wow_ui_sim::lua_api::WowLuaEnv, provider_name: &str) {
    for event in DATA_PROVIDER_EVENTS {
        let registered: bool = env
            .eval(&format!(
                r#"
                local expectedOnAdded = _G[{provider_name:?}].OnAdded
                for provider in pairs(AdventureMapFrame.dataProviders) do
                    if provider.OnAdded == expectedOnAdded then
                        return provider.registeredEvents and provider.registeredEvents[{event:?}] == true
                    end
                end
                return false
                "#
            ))
            .expect("AdventureMap data-provider event probe must run cleanly");

        assert!(
            registered,
            "`{provider_name}:OnAdded` must register `{event}` when added to AdventureMapFrame"
        );
    }
}

fn assert_quest_dialog_event_lifecycle(event: &str, lifecycle: QuestDialogEventLifecycle) {
    let (before_show, after_show, after_hide) = lifecycle;

    assert!(
        !before_show,
        "`AdventureMapQuestChoiceDialog` must not register `{event}` while hidden"
    );
    assert!(
        after_show,
        "`AdventureMapQuestChoiceDialog:OnShow` must register `{event}`"
    );
    assert!(
        !after_hide,
        "`AdventureMapQuestChoiceDialog:OnHide` must unregister `{event}`"
    );
}
