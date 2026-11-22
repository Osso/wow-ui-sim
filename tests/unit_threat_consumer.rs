#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness as harness;

const THREAT_CONSUMER_CHECK: &str = r#"
    assert(type(CompactUnitFrame_UpdateAggroHighlight) == "function")
    assert(TargetFrame and TargetFrame.threatIndicator and TargetFrame.threatNumericIndicator)
    SetCVar("threatShowNumeric", "1")
    A_Admin.SetPartySize(1)
    local compact = CompactPartyFrame_Generate()
    compact:RefreshMembers()
    local frame = compact.memberUnitFrames[1]
    assert(frame.unit == "player" and frame.aggroHighlight)
    A_Admin.SetUnitThreatSituation("player", "target", 0, 50, 50, 500, false, 50)
    A_Admin.SetUnitThreatSituation("pet", "target", 3, 100, 100, 1000, true, 100)
    CompactUnitFrame_UpdateAggroHighlight(frame)
    local first = frame.aggroHighlight:IsShown()
    UnitFrame_UpdateThreatIndicator(TargetFrame.threatIndicator, TargetFrame.threatNumericIndicator)
    local firstIndicator = TargetFrame.threatIndicator:IsShown()
    local firstNumeric = TargetFrame.threatNumericIndicator:IsShown()
    local firstText = TargetFrame.threatNumericIndicator.text:GetText()
    local firstR, firstG, firstB = TargetFrame.threatNumericIndicator.bg:GetVertexColor()
    local isTanking, status, scaled, raw, value = UnitDetailedThreatSituation("player", "target")
    local firstSituation = UnitThreatSituation("player")
    local firstLead = UnitThreatPercentageOfLead("player", "target")
    A_Admin.SetUnitThreatSituation("player", "target", 3, 100, 100, 1200, true, 100)
    A_Admin.SetUnitThreatSituation("pet", "target", 0, 80, 80, 1000, false, 80)
    CompactUnitFrame_UpdateAggroHighlight(frame)
    local second = frame.aggroHighlight:IsShown()
    UnitFrame_UpdateThreatIndicator(TargetFrame.threatIndicator, TargetFrame.threatNumericIndicator)
    local secondIndicator = TargetFrame.threatIndicator:IsShown()
    local secondNumeric = TargetFrame.threatNumericIndicator:IsShown()
    local secondText = TargetFrame.threatNumericIndicator.text:GetText()
    local secondR, secondG, secondB = TargetFrame.threatNumericIndicator.bg:GetVertexColor()
    assert(first == false and second == true, "Blizzard aggro highlight must change")
    assert(isTanking == false and status == 0 and scaled == 50 and raw == 50 and value == 500)
    assert(firstSituation == 0 and firstLead == 50)
    assert(UnitDetailedThreatSituation("player", "target") == true)
    assert(UnitThreatSituation("player") == 3)
    assert(UnitThreatPercentageOfLead("player", "target") == 100)
    assert(firstIndicator == false and firstNumeric and firstText == "50%")
    assert(firstR == 0.69 and firstG == 0.69 and firstB == 0.69)
    assert(secondIndicator and secondNumeric and secondText == "100%")
    assert(secondR == 1 and secondG == 0 and secondB == 0)
    return true
"#;

#[test]
fn compact_aggro_highlight_tracks_two_supplied_states() {
    harness::with_blizzard_addon_startup_shape(
        &["Blizzard_UnitFrame", "Blizzard_CompactRaidFrames"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_UnitFrame"));
            assert!(
                loaded
                    .iter()
                    .any(|name| name == "Blizzard_CompactRaidFrames")
            );
            let result: bool = env
                .eval(THREAT_CONSUMER_CHECK)
                .expect("original Blizzard threat consumers run");
            assert!(result, "consumer output must follow supplied threat states");
        },
    );
}
