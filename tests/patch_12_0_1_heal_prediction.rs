#![cfg(feature = "retail-12-0-5")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_0_1_heal_prediction_uses_values_modes_and_curves() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local c = CreateUnitHealPredictionCalculator()
        c:SetPredictedValues({ health = 400, healthMax = 1000,
            totalDamageAbsorbs = 200, totalHealAbsorbs = 100,
            totalIncomingHeals = 300, totalIncomingHealsFromHealer = 120 })
        assert(c:GetCurrentHealthPercent() == 0.4)
        assert(c:GetMissingHealth() == 600)
        assert(c:GetMissingHealthPercent() == 0.6)
        assert(c:GetTotalDamageAbsorbs() == 200)
        assert(c:GetTotalHealAbsorbs() == 100)
        assert(c:GetTotalIncomingHeals() == 300)
        assert(c:GetTotalIncomingHealsFromHealer() == 120)
        assert(c:GetMaximumHealAbsorbs() == 400)
        assert(c:GetMaximumIncomingHeals() == 600)
        -- Incoming heals reduce heal absorbs by 100, leaving 200 effective heals.
        assert(c:GetMaximumDamageAbsorbs() == 400)
        c:SetDamageAbsorbClampMode(1)
        assert(c:GetMaximumDamageAbsorbs() == 600)
        c:SetDamageAbsorbClampMode(2)
        assert(c:GetMaximumDamageAbsorbs() == 1000)
        c:SetHealAbsorbClampMode(1)
        c:SetIncomingHealClampMode(1)
        assert(c:GetMaximumHealAbsorbs() == 1000)
        assert(c:GetMaximumIncomingHeals() == 1000)
        c:SetMaximumHealthMode(1)
        assert(c:GetMaximumHealth() == 1200)
        assert(c:GetMissingHealth() == 800)
        local curve = C_CurveUtil.CreateCurve()
        curve:AddPoint(0, 0)
        curve:AddPoint(1, 100)
        c:SetMaximumHealthMode(0)
        assert(c:EvaluateCurrentHealthPercent(curve) == 40)
        assert(c:EvaluateMissingHealthPercent(curve) == 60)
        c:ResetPredictedValues()
        assert(c:GetCurrentHealthPercent() == 0)
        assert(c:GetMissingHealthPercent() == 0)
        assert(c:GetMaximumIncomingHeals() == 0)
    "#).unwrap();
}
