//! Cached ActionButton path, unchanged vendor Lua; tainted caller failure boundary.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/prefork_full_ui_preload.rs"]
mod full_ui;

fn assert_secret_setter_boundary(env: &WowLuaEnv) {
    env.exec(r#"
        assert(type(ActionButton_ApplyCooldown) == 'function')
        ProbeNormal = CreateFrame('Cooldown')
        ProbeCharge = CreateFrame('Cooldown')
        ProbeNormal:SetCooldown(10, 20)
        local source = C_Spell.GetSpellCooldown(642)
        ProbeDuration = C_DurationUtil.CreateDuration()
        ProbeDuration:SetTimeFromStart(source.startTime, source.duration, source.modRate)
        assert(ProbeDuration:HasSecretValues())
        local function addon()
            local info = C_Spell.GetSpellCooldown(642)
            assert(info.isActive and issecretvalue(info.startTime) and issecretvalue(info.duration))
            assert(debug.getstacktaint() == 'ExtractCooldownProbe')
            local ok, err = pcall(ActionButton_ApplyCooldown, ProbeNormal, info, ProbeCharge, nil, nil, nil)
            assert(not ok and type(err) == 'string', 'cached function must not clear taint through a secure delegate')
            for _, invoke in ipairs({
                function() ProbeNormal:SetCooldown(info.startTime, info.duration) end,
                function() ProbeNormal:SetCooldownFromExpirationTime(info.startTime, info.duration) end,
                function() ProbeNormal:SetCooldownDuration(info.duration) end,
                function() ProbeNormal:SetCooldownUNIX(info.startTime, info.duration) end,
            }) do
                local ok, err = pcall(invoke)
                assert(not ok and type(err) == 'string', 'tainted secret numeric setter must fail')
                assert(debug.getstacktaint() == 'ExtractCooldownProbe')
                local readable, start, span = pcall(function() return ProbeNormal:GetCooldownTimes() end)
                assert(readable, 'rejected setter must not mark public frame timing secret')
                assert(start == 10000 and span == 20000, 'rejected setter must preserve frame state')
            end
            local publicObject = C_Spell.GetSpellCooldownDuration(642)
            ProbeNormal:SetCooldownFromDurationObject(publicObject)
            assert(debug.getstacktaint() == 'ExtractCooldownProbe')
        end
        debug.setobjecttaint(addon, 'ExtractCooldownProbe')
        addon()
    "#).unwrap();
    // Separate secure host entry: taint propagates back to the calling Lua chunk.
    let duration: f64 = env.eval("return ProbeNormal:GetCooldownDisplayDuration()").unwrap();
    assert_eq!(duration, 300000.0);
}

// Retained RED: the duration core authenticates secret timing with the caller's
// VM guard; no opaque rendering accessor exists. Do not clear taint to bypass it.
#[test]
#[ignore = "pending opaque consumption of secret-bearing duration timing; see prose-2026-03-21-174"]
fn patch_12_0_1_tainted_secret_duration_object() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeFromStart(secretwrap(10, 20, 1))
        local frame = CreateFrame('Cooldown')
        local function addon()
            frame:SetCooldownFromDurationObject(duration)
            assert(debug.getstacktaint() == 'SecretDurationExtractProbe')
        end
        debug.setobjecttaint(addon, 'SecretDurationExtractProbe')
        addon()
    "#).unwrap();
}

#[test]
fn patch_12_0_1_cached_secure_delegate_removed() {
    crate::common::with_exclusive_workload(|| {
        let env = full_ui::preload_full_game_ui().expect("cached Game preload");
        env.exec("CastSpellByID(642)").unwrap();
        env.state().borrow_mut().cooldowns_restricted = true;
        assert_secret_setter_boundary(&env);
    });
}
