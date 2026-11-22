#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness as harness;

const THREAT_CONSUMER_CHECK: &str = r#"
    assert(type(CompactUnitFrame_UpdateAggroHighlight) == "function", "compact consumer loaded")
    assert(TargetFrame and TargetFrame.threatIndicator and TargetFrame.threatNumericIndicator, "target frame consumer loaded")
    assert(GetCVar("threatWarning") == "3", "registered threatWarning default")
    SetCVar("threatShowNumeric", "1")
    A_Admin.SetPartySize(1)
    local compact = CompactPartyFrame_Generate()
    compact:RefreshMembers()
    local frame = compact.memberUnitFrames[1]
    assert(frame.unit == "player" and frame.aggroHighlight, "party member frame uses player")
    A_Admin.SetUnitThreatSituation("player", "target", 0, 50, 50, 500, false, 50)
    A_Admin.SetUnitThreatSituation("pet", "target", 3, 100, 100, 1000, true, 100)
    CompactUnitFrame_UpdateAggroHighlight(frame)
    local first = frame.aggroHighlight:IsShown()
    SetCVar("threatWarning", "0")
    assert(not IsThreatWarningEnabled(), "threatWarning=0 disables threat indicators")
    UnitFrame_UpdateThreatIndicator(TargetFrame.threatIndicator, TargetFrame.threatNumericIndicator)
    assert(not TargetFrame.threatIndicator:IsShown(), "disabled CVar hides TargetFrame indicator")
    assert(not TargetFrame.threatNumericIndicator:IsShown(), "disabled CVar hides numeric indicator")
    SetCVar("threatWarning", "1")
    assert(not IsThreatWarningEnabled(), "threatWarning=1 requires an instance")
    A_Admin.SetInInstance(true)
    assert(IsThreatWarningEnabled(), "threatWarning=1 enables warnings in an instance")
    A_Admin.SetInInstance(false)
    SetCVar("threatWarning", "2")
    assert(IsThreatWarningEnabled(), "threatWarning=2 enables warnings in a group")
    SetCVar("threatWarning", "3")
    assert(IsThreatWarningEnabled(), "threatWarning=3 enables threat indicators")
    A_Admin.SetUnitEffectivelyTankAura("player", true)
    assert(UnitHasEffectivelyTankAura("player"), "supplied player aura state")
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
    assert(isTanking == false and status == 0 and scaled == 50 and raw == 50 and value == 500, "first detailed values")
    assert(firstSituation == 0 and firstLead == 50, "first situation and lead")
    assert(UnitDetailedThreatSituation("player", "target") == true, "second detailed tanking")
    assert(UnitThreatSituation("player") == 3, "second situation")
    assert(UnitThreatPercentageOfLead("player", "target") == 100, "second lead")
    assert(firstIndicator == false and firstNumeric and firstText == "50%", "first indicator text and visibility")
    -- UnitUtil.lua:110-115: the tank-aura path overrides a non-tanking status to 2.
    assert(firstR == 1 and firstG == 0.6 and firstB == 0, "first indicator color")
    assert(secondIndicator and secondNumeric and secondText == "100%", "second indicator text and visibility")
    assert(secondR == 1 and secondG == 0 and secondB == 0, "second indicator color")
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
