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
            end
            local object = C_Spell.GetSpellCooldownDuration(642)
            ProbeNormal:SetCooldownFromDurationObject(object)
            assert(debug.getstacktaint() == 'ExtractCooldownProbe')
        end
        debug.setobjecttaint(addon, 'ExtractCooldownProbe')
        addon()
        assert(ProbeNormal:GetCooldownDisplayDuration() == 300000)
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
