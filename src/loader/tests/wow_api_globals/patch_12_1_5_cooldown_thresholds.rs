use crate::lua_api::WowLuaEnv;

#[test]
fn cooldown_threshold_abbrev_default_and_round_trip() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local cd = CreateFrame("Cooldown")
        assert(type(cd.GetCountdownAbbrevThreshold) == "function")
        assert(cd:GetCountdownAbbrevThreshold() == 0)
        cd:SetCountdownAbbrevThreshold(5.25)
        assert(cd:GetCountdownAbbrevThreshold() == 5.25)
        cd:SetCountdownAbbrevThreshold(0)
        assert(cd:GetCountdownAbbrevThreshold() == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn cooldown_threshold_milliseconds_default_and_round_trip() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local cd = CreateFrame("Cooldown")
        local other = CreateFrame("Cooldown")
        assert(type(cd.SetCountdownMillisecondsThreshold) == "function")
        assert(type(cd.GetCountdownMillisecondsThreshold) == "function")
        assert(cd:GetCountdownMillisecondsThreshold() == 0)
        cd:SetCountdownAbbrevThreshold(5.25)
        cd:SetCountdownMillisecondsThreshold(1.5)
        assert(cd:GetCountdownMillisecondsThreshold() == 1.5)
        assert(cd:GetCountdownAbbrevThreshold() == 5.25)
        assert(other:GetCountdownMillisecondsThreshold() == 0)
        cd:SetCountdownMillisecondsThreshold(0)
        assert(cd:GetCountdownMillisecondsThreshold() == 0)
    "#,
    )
    .unwrap();
}
