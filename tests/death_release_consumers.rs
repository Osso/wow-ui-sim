#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! Original consumers: Blizzard_StaticPopup_Game (SKINNED_REPOP, RESURRECT*, AREA_SPIRIT_HEAL)
//! and Blizzard_Game EventImplementation (HandleResurrectRequest, area spirit healer handlers).

use crate::common::blizzard_addon_harness as harness;

const RELEASE_AND_RESURRECT: &str = r#"
    A_Admin.SetManualClock(50)
    local events = {}
    local watcher = CreateFrame("Frame")
    for _, event in ipairs({ "PLAYER_ALIVE", "PLAYER_UNGHOST", "CEMETERY_PREFERENCE_UPDATED" }) do
        watcher:RegisterEvent(event)
    end
    watcher:SetScript("OnEvent", function(_, event) events[#events + 1] = event end)
    A_Admin.SetPlayerHealth(0, 100)
    A_Admin.SetReleaseTimeRemaining(360)
    assert(UnitIsDead("player") and not UnitIsGhost("player"))
    StaticPopup_Show("SKINNED_REPOP")
    StaticPopup_FindVisible("SKINNED_REPOP").button1:Click()
    assert(UnitIsGhost("player") and not UnitIsDead("player") and UnitIsDeadOrGhost("player"))
    assert(GetReleaseTimeRemaining() == 0 and events[1] == "PLAYER_ALIVE", "release raised PLAYER_ALIVE")
    RepopMe()
    assert(#events == 1, "a ghost cannot release again")

    A_Admin.SetCorpseRecoveryDelay(30)
    local function offer(sickness, timer)
        A_Admin.OfferResurrect({ offerer = "Healer", hasSickness = sickness, hasTimer = timer })
    end
    offer(true, true)
    local dialog = StaticPopup_FindVisible("RESURRECT")
    assert(dialog and ResurrectHasSickness() and ResurrectHasTimer(), "sickness offer")
    assert(dialog.timeleft == 90, "RESURRECT OnShow reads GetCorpseRecoveryDelay")
    A_Admin.AdvanceTime(10)
    assert(GetCorpseRecoveryDelay() == 20)
    TimeoutResurrect()
    assert(ResurrectGetOfferer() == nil and not ResurrectHasSickness(), "timeout withdraws the offer")
    offer(false, true)
    assert(StaticPopup_FindVisible("RESURRECT_NO_SICKNESS") and not ResurrectHasSickness())
    offer(false, false)
    local noTimer = StaticPopup_FindVisible("RESURRECT_NO_TIMER")
    assert(noTimer and not ResurrectHasTimer() and noTimer.timeleft == 80)
    noTimer.button1:Click()
    assert(not UnitIsGhost("player") and UnitHealth("player") == 100)
    assert(not pcall(A_Admin.OfferResurrect, { offerer = "x", hasSickness = 1, hasTimer = true }))
    return true
"#;

const AREA_SPIRIT_HEALER: &str = r#"
    A_Admin.SetManualClock(10)
    A_Admin.SetPlayerHealth(0, 100)
    assert(GetAreaSpiritHealerTime() == 0)
    A_Admin.SetCemeterySelectionAvailable(false)
    A_Admin.SetAreaSpiritHealer(25)
    local dialog = StaticPopup_FindVisible("AREA_SPIRIT_HEAL")
    assert(dialog and dialog.timeleft == 25, "in-range handler queued and showed the wave time")
    assert(not dialog.button1:IsShown(), "no cemetery selection")
    A_Admin.AdvanceTime(5)
    assert(GetAreaSpiritHealerTime() == 20)
    dialog.button2:Click()
    assert(GetAreaSpiritHealerTime() == 0, "CancelAreaSpiritHeal leaves the queue")
    A_Admin.SetCemeterySelectionAvailable(true)
    A_Admin.SetAreaSpiritHealer(12)
    dialog = StaticPopup_FindVisible("AREA_SPIRIT_HEAL")
    assert(dialog.timeleft == 12 and dialog.button1:IsShown() and IsCemeterySelectionAvailable())
    A_Admin.SetAreaSpiritHealer(nil)
    assert(StaticPopup_FindVisible("AREA_SPIRIT_HEAL") == nil and GetAreaSpiritHealerTime() == 0)

    local updates = 0
    local watcher = CreateFrame("Frame")
    watcher:RegisterEvent("CEMETERY_PREFERENCE_UPDATED")
    watcher:SetScript("OnEvent", function() updates = updates + 1 end)
    assert(GetCemeteryPreference() == 0)
    SetCemeteryPreference(5)
    SetCemeteryPreference(5)
    SetCemeteryPreference(7)
    assert(GetCemeteryPreference() == 7 and updates == 2, "only changes raise the event")
    assert(GetResSicknessDuration() == nil)
    A_Admin.SetResSicknessDuration("10 Min")
    assert(GetResSicknessDuration() == "10 Min")
    A_Admin.SetResSicknessDuration(nil)
    assert(GetResSicknessDuration() == nil)
    return true
"#;

/// Runs `script` and returns the names of events queued for the next frame.
fn run(script: &str) -> Vec<String> {
    harness::with_blizzard_addon_startup_shape(
        &["Blizzard_Game", "Blizzard_StaticPopup_Game"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_StaticPopup_Game"));
            let result: bool = env.eval(script).expect("original death consumers run");
            assert!(result);
            let state = env.state().borrow();
            state.events.pending().iter().map(|event| event.name.clone()).collect()
        },
    )
}

#[test]
fn release_and_resurrect_offers_drive_original_dialogs() {
    let queued = run(RELEASE_AND_RESURRECT);
    assert!(
        queued.iter().any(|name| name == "PLAYER_UNGHOST"),
        "a resurrected ghost raises PLAYER_UNGHOST: {queued:?}"
    );
    assert!(!queued.iter().any(|name| name == "PLAYER_ALIVE"));
}

#[test]
fn area_spirit_healer_and_cemetery_preference_follow_inputs() {
    run(AREA_SPIRIT_HEALER);
}
