//! Bounded Classic secret disablement; no native-client or whole-row claim.
#![cfg(any(
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]

use wow_ui_sim::c_api::charge_state::SpellChargeState;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn classic_secret_policy_identity_names_and_guid_are_ordinary() {
    let env = WowLuaEnv::new().expect("Classic identity environment");
    env.exec("A_Admin.SetPartySize(1)").unwrap();
    env.state().borrow_mut().party_members[0].name = "ClassicAlice".to_owned();
    env.exec(
        r#"
        local function checkIdentity()
            local name = UnitName('party1')
            local fullName, realm = UnitFullName('party1')
            local guid = UnitGUID('party1')
            assert(not issecretvalue(name), 'Classic UnitName must be ordinary')
            assert(not issecretvalue(fullName) and not issecretvalue(realm))
            assert(not issecretvalue(guid))
            assert(name == 'ClassicAlice' and fullName == 'ClassicAlice')
            assert(realm == 'SimRealm' and guid == 'Player-0000-00000002')
            assert(canaccessallvalues(name, fullName, realm, guid))
            assert(canaccesstable({name = name, guid = guid}))
            assert(name .. ':' .. guid == 'ClassicAlice:Player-0000-00000002')
            assert(select('#', UnitGUID('party1')) == 1)
        end
        checkIdentity()
        local function addonCaller()
            assert(debug.getstacktaint() == 'ClassicSecretPolicyAddon')
            checkIdentity()
            assert(debug.getstacktaint() == 'ClassicSecretPolicyAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicSecretPolicyAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("ordinary payload, arity, nested access and unchanged caller taint");
}

#[test]
fn classic_secret_policy_raid_roster_name_is_ordinary() {
    let env = WowLuaEnv::new().expect("Classic raid environment");
    env.exec("A_Admin.SetPartySize(1)").unwrap();
    env.state().borrow_mut().player.name = "ClassicRaidLeader".to_owned();
    env.exec(
        r#"
        local name = GetRaidRosterInfo(1)
        assert(not issecretvalue(name), 'Classic roster name must be ordinary')
        assert(name == 'ClassicRaidLeader')
        assert(canaccessvalue(name) and canaccesstable({name = name}))
        "#,
    )
    .expect("raid producer uses ordinary names in Classic");
}

#[test]
fn classic_secret_policy_restricted_stat_input_keeps_plain_numbers() {
    let env = WowLuaEnv::new().expect("Classic stat environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.stats.armor = 1234;
        state.unit_stats_restricted = true;
    }
    env.exec(
        r#"
        local function addonCaller()
            assert(debug.getstacktaint() == 'ClassicStatAddon')
            local base, effective, total, positive, negative = UnitArmor('player')
            assert(select('#', UnitArmor('player')) == 5)
            assert(base == 1234 and effective == 1234 and total == 1234)
            assert(positive == 0 and negative == 0 and base + 1 == 1235)
            for _, value in ipairs({base, effective, total, positive, negative}) do
                assert(type(value) == 'number' and not issecretvalue(value))
                assert(canaccessvalue(value))
            end
            assert(debug.getstacktaint() == 'ClassicStatAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicStatAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("restricted host flag cannot make Classic UnitArmor outputs secret");
    assert!(env.state().borrow().unit_stats_restricted);
}

#[test]
fn classic_secret_policy_restricted_cooldown_input_keeps_plain_numbers() {
    let env = WowLuaEnv::new().expect("Classic cooldown environment");
    {
        let mut state = env.state().borrow_mut();
        state.cooldowns_restricted = true;
        state.spell_charges.insert(
            19750,
            SpellChargeState {
                current_charges: 1,
                max_charges: 3,
                recharge_start: 12.0,
                recharge_duration: 40.0,
                charge_mod_rate: 2.0,
            },
        );
    }
    env.exec(
        r#"
        local function addonCaller()
            local info = C_Spell.GetSpellCharges(19750)
            assert(type(info) == 'table')
            assert(select('#', C_Spell.GetSpellCharges(19750)) == 1)
            assert(info.currentCharges == 1 and info.maxCharges == 3)
            assert(info.cooldownStartTime == 12 and info.cooldownDuration == 40)
            assert(info.chargeModRate == 2 and info.cooldownDuration / info.chargeModRate == 20)
            for _, value in pairs(info) do
                assert(type(value) == 'number' and not issecretvalue(value))
            end
            assert(canaccesstable(info))
            assert(debug.getstacktaint() == 'ClassicCooldownAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicCooldownAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("restricted host flag cannot make Classic charge outputs secret");
    assert!(env.state().borrow().cooldowns_restricted);
}

#[test]
fn classic_secret_policy_duration_and_curve_constructors_work() {
    let env = WowLuaEnv::new().expect("Classic duration environment");
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(104)
        local duration = C_DurationUtil.CreateDuration()
        duration:SetClock(clock)
        duration:SetTimeFromStart(100, 10)
        assert(duration:GetTotalDuration() == 10)
        assert(duration:GetElapsedDuration() == 4)
        assert(duration:GetRemainingDuration() == 6)
        assert(not duration:HasSecretValues())
        local curve = C_CurveUtil.CreateCurve()
        curve:AddPoint(0, 0)
        curve:AddPoint(10, 20)
        assert(curve:Evaluate(4) == 8)
        assert(duration:EvaluateElapsedDuration(curve) == 8)
        local colorCurve = C_CurveUtil.CreateColorCurve()
        colorCurve:AddPoint(0, CreateColor(0, 0, 0, 1))
        colorCurve:AddPoint(10, CreateColor(1, 0, 0, 1))
        local r, g, b, a = colorCurve:EvaluateUnpacked(5)
        assert(r == 0.5 and g == 0 and b == 0 and a == 1)
        clock:AdvanceTime(2)
        assert(duration:GetElapsedDuration() == 6)
        assert(duration:Copy():GetTotalDuration() == 10)
        "#,
    )
    .expect("existing Classic duration/curve registration has real public behavior");
}

#[test]
fn classic_secret_policy_loadstring_remains_tainted_not_secret() {
    let env = WowLuaEnv::new().expect("Classic closure environment");
    env.exec(
        r#"
        local function addonCaller()
            assert(debug.getstacktaint() == 'ClassicLoadstringAddon')
            local fn = assert(loadstring('ClassicTaintedSlot = 17; return debug.getstacktaint()'))
            assert(not issecretvalue(fn), 'Classic tainted closure must not be secret')
            assert(canaccessvalue(fn) and canaccessallvalues(fn, 17))
            assert(canaccesstable({fn = fn}))
            assert(fn() ~= nil, 'loadstring must retain taint')
            local secure, owner = issecurevariable('ClassicTaintedSlot')
            assert(not secure and owner ~= nil, 'slot taint must remain')
            assert(debug.getstacktaint() == 'ClassicLoadstringAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicLoadstringAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        assert(ClassicTaintedSlot == 17)
        "#,
    )
    .expect("secret classification changes without erasing pre-Midnight taint");
}

#[test]
fn classic_secret_policy_tainted_protected_action_still_denied() {
    let env = WowLuaEnv::new().expect("Classic protected action environment");
    env.exec(
        r#"
        local blocked = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_BLOCKED')
        listener:SetScript('OnEvent', function(_, _, _, action)
            blocked[#blocked + 1] = action
        end)
        local protected = CreateFrame('Frame', 'ClassicSecretProtectedFrame', UIParent)
        protected:SetSize(40, 20)
        A_Admin.SetFrameProtected('ClassicSecretProtectedFrame', true)
        A_Admin.SetInCombat(true)
        local function addonCaller()
            assert(debug.getstacktaint() == 'ClassicProtectedAddon', 'entry taint: ' .. tostring(debug.getstacktaint()))
            protected:SetWidth(90)
            assert(protected:GetWidth(true) == 40, 'blocked width: ' .. protected:GetWidth(true))
            assert(debug.getstacktaint() == 'ClassicProtectedAddon', 'post-write taint: ' .. tostring(debug.getstacktaint()))
        end
        debug.setobjecttaint(addonCaller, 'ClassicProtectedAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil, 'secure caller must regain untainted stack')
        protected:SetWidth(70)
        assert(protected:GetWidth(true) == 70, 'secure width: ' .. protected:GetWidth(true))
        -- Inspect addon-written event slots after the secure recovery probe:
        -- reading those tainted slots propagates addon taint to this caller.
        assert(#blocked == 1 and blocked[1] == 'ClassicSecretProtectedFrame:SetWidth()',
            'blocked events: ' .. table.concat(blocked, '|'))
        "#,
    )
    .expect("insecure combat write denied, exact blocked event, secure write allowed");
}
