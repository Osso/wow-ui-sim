//! Event-registration surface for `Blizzard_AnimaDiversionUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::loader::BlizzardAddonOverride;

const ROOT: &str = "Blizzard_AnimaDiversionUI";
const IMPLICIT_DEPS: &[&str] = &["Blizzard_MapCanvas", "Blizzard_SharedMapDataProviders"];
const CLOSURE_OVERRIDES: &[BlizzardAddonOverride<'_>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: IMPLICIT_DEPS,
}];
const FRAME_SHOW_EVENTS: &[&str] = &["ANIMA_DIVERSION_CLOSE", "CURRENCY_DISPLAY_UPDATE"];
const DATA_PROVIDER_SHOW_EVENTS: &[&str] = &[
    "ANIMA_DIVERSION_TALENT_UPDATED",
    "CURRENCY_DISPLAY_UPDATE",
    "GARRISON_TALENT_COMPLETE",
    "GARRISON_TALENT_EVENT_UPDATE",
    "GARRISON_TALENT_UNLOCKS_RESULT",
];
const EVENT_LIFECYCLE_PROBE: &str = r#"
AnimaDiversionFrame:Hide()
local beforeShow = {
    AnimaDiversionFrame:IsEventRegistered("ANIMA_DIVERSION_CLOSE"),
    AnimaDiversionFrame:IsEventRegistered("CURRENCY_DISPLAY_UPDATE"),
}

AnimaDiversionFrame.mapID = C_Map.GetCurrentMapID()
AnimaDiversionFrame.bolsterProgress = 0
AnimaDiversionFrame.covenantData = { animaChannelActiveSoundKit = 0 }
AnimaDiversionFrame:Show()

local afterShow = {
    AnimaDiversionFrame:IsEventRegistered("ANIMA_DIVERSION_CLOSE"),
    AnimaDiversionFrame:IsEventRegistered("CURRENCY_DISPLAY_UPDATE"),
}

AnimaDiversionFrame:Hide()
local afterHide = {
    AnimaDiversionFrame:IsEventRegistered("ANIMA_DIVERSION_CLOSE"),
    AnimaDiversionFrame:IsEventRegistered("CURRENCY_DISPLAY_UPDATE"),
}

return beforeShow, afterShow, afterHide
"#;
const DATA_PROVIDER_EVENT_LIFECYCLE_PROBE: &str = r#"
local function findProvider()
    for provider in pairs(AnimaDiversionFrame.dataProviders) do
        if provider.OnShow == AnimaDiversionDataProviderMixin.OnShow then
            return provider
        end
    end
end

local function collect(provider)
    local events = provider and provider.registeredEvents
    return {
        events ~= nil and events["ANIMA_DIVERSION_TALENT_UPDATED"] == true,
        events ~= nil and events["CURRENCY_DISPLAY_UPDATE"] == true,
        events ~= nil and events["GARRISON_TALENT_COMPLETE"] == true,
        events ~= nil and events["GARRISON_TALENT_EVENT_UPDATE"] == true,
        events ~= nil and events["GARRISON_TALENT_UNLOCKS_RESULT"] == true,
    }
end

AnimaDiversionFrame:Hide()
local provider = findProvider()
local beforeShow = collect(provider)

AnimaDiversionFrame.mapID = C_Map.GetCurrentMapID()
AnimaDiversionFrame.bolsterProgress = 0
AnimaDiversionFrame.covenantData = { animaChannelActiveSoundKit = 0 }
AnimaDiversionFrame:Show()
local afterShow = collect(provider)

AnimaDiversionFrame:Hide()
local afterHide = collect(provider)

return beforeShow, afterShow, afterHide
"#;

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_frame_registers_show_events_only_while_shown() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let lifecycle: FrameEventLifecycle = env
            .eval(EVENT_LIFECYCLE_PROBE)
            .expect("AnimaDiversionFrame show/hide event probe must run cleanly");

        assert_frame_event_lifecycle(lifecycle);
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_data_provider_registers_events_only_while_map_is_shown() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let lifecycle: DataProviderEventLifecycle = env
            .eval(DATA_PROVIDER_EVENT_LIFECYCLE_PROBE)
            .expect("AnimaDiversionDataProviderMixin show/hide event probe must run cleanly");

        assert_data_provider_event_lifecycle(lifecycle);
    });
}

type FrameEventLifecycle = (Vec<bool>, Vec<bool>, Vec<bool>);
type DataProviderEventLifecycle = (Vec<bool>, Vec<bool>, Vec<bool>);

fn assert_frame_event_lifecycle(lifecycle: FrameEventLifecycle) {
    let (before_show, after_show, after_hide) = lifecycle;

    for (index, event) in FRAME_SHOW_EVENTS.iter().enumerate() {
        assert_show_event_lifecycle(
            event,
            before_show[index],
            after_show[index],
            after_hide[index],
        );
    }
}

fn assert_show_event_lifecycle(event: &str, before_show: bool, after_show: bool, after_hide: bool) {
    assert!(
        !before_show,
        "`AnimaDiversionFrame` must not register `{event}` while hidden"
    );
    assert!(
        after_show,
        "`AnimaDiversionFrameMixin:OnShow` must register `{event}`"
    );
    assert!(
        !after_hide,
        "`AnimaDiversionFrameMixin:OnHide` must unregister `{event}`"
    );
}

fn assert_data_provider_event_lifecycle(lifecycle: DataProviderEventLifecycle) {
    let (before_show, after_show, after_hide) = lifecycle;

    for (index, event) in DATA_PROVIDER_SHOW_EVENTS.iter().enumerate() {
        assert_data_provider_show_event(
            event,
            before_show[index],
            after_show[index],
            after_hide[index],
        );
    }
}

fn assert_data_provider_show_event(
    event: &str,
    before_show: bool,
    after_show: bool,
    after_hide: bool,
) {
    assert!(
        !before_show,
        "`AnimaDiversionDataProviderMixin` must not register `{event}` while hidden"
    );
    assert!(
        after_show,
        "`AnimaDiversionDataProviderMixin:OnShow` must register `{event}`"
    );
    assert!(
        !after_hide,
        "`AnimaDiversionDataProviderMixin:OnHide` must unregister `{event}`"
    );
}
