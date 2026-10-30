//! P3 numeric selectors: original meter consumers, not replacement providers.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use rilua::{LuaApi, LuaApiMut};
use wow_ui_sim::lua_api::WowLuaEnv;

fn read_blizzard_source(relative: &str) -> String {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    let path = ui.join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let original = read_blizzard_source("Blizzard_DamageMeter/DamageMeterEntry.lua");
    env.exec(&original)
        .expect("unchanged DamageMeterEntry source");
    let contribution = read_blizzard_source("Blizzard_Contribution/Blizzard_Contribution.lua");
    let setup = contribution
        .split_once("function ContributionRewardMixin:Setup")
        .unwrap()
        .1;
    let setup = setup.split_once("\nlocal function GetExtents").unwrap().0;
    env.exec(&format!(
        "ContributionRewardMixin = {{}}; function ContributionRewardMixin:Setup{setup}"
    ))
    .expect("unchanged original contribution reward setup");
    env.exec("RewardRow = CreateFrame('Frame'); RewardRow.Icon = RewardRow:CreateTexture(); RewardRow.RewardName = RewardRow:CreateFontString(); RewardRow.PadLock = CreateFrame('Frame', nil, RewardRow)").unwrap();
    install_secrets(&env);
    env
}

fn install_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, number) in [
        ("Selector", 19750.0),
        ("Missing", 900002.0),
        ("Fraction", 19750.5),
        ("Negative", -1.0),
        ("Infinite", f64::INFINITY),
        ("Nan", f64::NAN),
        ("Overflow", 4294967296.0),
    ] {
        let value = rilua::table_security::wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        lua.set_global_val(name, value).unwrap();
        lua.state_mut().pop();
    }
    let flag = rilua::table_security::wrap_host_secret_bool(lua.state_mut(), true);
    lua.state_mut().push(flag);
    lua.set_global_val("SecretBool", flag).unwrap();
    lua.state_mut().pop();
    let value = rilua::table_security::wrap_host_secret_string(lua.state_mut(), "19750");
    lua.state_mut().push(value);
    lua.set_global_val("SecretString", value).unwrap();
    lua.state_mut().pop();
}

fn invoke(env: &WowLuaEnv, code: &str, tainted: bool) {
    env.exec(&format!(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == {expected})
            {code}
            assert(issecretvalue(Selector))
            if {tainted} then
                assert(not canaccessvalue(Selector))
                assert(not pcall(secretunwrap, Selector))
                assert(not pcall(function() return Selector + 1 end))
            end
            assert(debug.getstacktaint() == before, 'caller taint preserved')
        end
        if {tainted} then debug.setobjecttaint(probe, 'PresentationProbe') end
        probe()
    "#,
        expected = if tainted {
            "'PresentationProbe'"
        } else {
            "nil"
        }
    ))
    .expect(code);
    for (name, expected) in [
        ("Selector", 19750.0),
        ("Fraction", 19750.5),
        ("Missing", 900002.0),
    ] {
        let value: rilua::Val = env.eval(&format!("return {name}")).unwrap();
        let lua = env.lua();
        assert_eq!(
            rilua::table_security::unwrap_secret(lua.state(), value).unwrap(),
            rilua::Val::Num(expected),
            "query preserves original {name} wrapper payload"
        );
    }
}

#[test]
fn original_meter_icon_numeric_secret_selector() {
    let env = fixture();
    let texture_arity = if cfg!(feature = "retail-12-1-0") {
        3
    } else {
        2
    };
    for tainted in [false, true] {
        for selector in ["19750", "Selector"] {
            invoke(
                &env,
                &format!(
                    r#"
                local row = {{spellID = {selector}}}
                local icon, original, override = DamageMeterSpellEntryMixin.GetIconTexture(row)
                assert(icon == 135907 and original == 135907 and override == nil, 'selected Flash of Light icon')
                assert(select('#', DamageMeterSpellEntryMixin.GetIconTexture(row)) == {texture_arity})
                assert(issecretvalue(row.spellID) == issecretvalue({selector}), 'row secrecy unchanged')
            "#
                ),
                tainted,
            );
        }
    }
}

#[test]
fn original_meter_name_numeric_secret_selector() {
    let env = fixture();
    for tainted in [false, true] {
        for selector in ["19750", "Selector"] {
            invoke(
                &env,
                &format!(
                    r#"
                local row = {{spellID = {selector}}}
                assert(DamageMeterSpellEntryMixin.GetNameText(row) == 'Flash of Light', 'selected name')
                assert(select('#', DamageMeterSpellEntryMixin.GetNameText(row)) == 1)
                assert(issecretvalue(row.spellID) == issecretvalue({selector}), 'row secrecy unchanged')
            "#
                ),
                tainted,
            );
        }
    }
}

#[test]
fn original_reward_info_numeric_secret_selector_preserves_identity_secrecy() {
    let env = fixture();
    for tainted in [false, true] {
        for selector in ["19750", "Selector"] {
            invoke(
                &env,
                &format!(
                    r#"
                ContributionRewardMixin.Setup(RewardRow, {selector}, true)
                assert(RewardRow.Icon:GetTexture() == 135907)
                assert(RewardRow.RewardName:GetText() == 'Flash of Light')
                assert(select('#', C_Spell.GetSpellInfo({selector})) == 1)
                local info = C_Spell.GetSpellInfo({selector})
                assert(not issecretvalue(info) and not issecretvalue(info.name))
                assert(not issecretvalue(info.iconID) and not issecretvalue(info.originalIconID))
                assert(info.name == 'Flash of Light' and info.iconID == 135907)
                assert(issecretvalue(info.spellID) == issecretvalue({selector}))
                if issecretvalue(info.spellID) then
                    if {tainted} then
                        assert(not canaccessvalue(info.spellID))
                        assert(not pcall(secretunwrap, info.spellID))
                    else assert(secretunwrap(info.spellID) == 19750) end
                else assert(info.spellID == 19750) end
            "#
                ),
                tainted,
            );
        }
    }
}

#[test]
fn presentation_miss_and_invalid_numeric_secret_controls() {
    let env = fixture();
    for tainted in [false, true] {
        invoke(
            &env,
            r#"
            assert(select('#', C_Spell.GetSpellInfo(Missing)) == 1)
            assert(C_Spell.GetSpellInfo(Missing) == nil)
            assert(select('#', C_Spell.GetSpellTexture(Missing)) == 0)
            assert(select('#', C_Spell.GetSpellName(Missing)) == 0)
            assert(C_Spell.GetSpellName(Missing) == nil)
            assert(select('#', C_Spell.GetSpellTexture(900002)) == 0)
            assert(select('#', C_Spell.GetSpellName(900002)) == 0)
            assert(C_Spell.GetSpellTexture(19750.5) == 135907)
            assert(C_Spell.GetSpellTexture(Fraction) == 135907, 'retain texture numeric truncation')
            assert(C_Spell.GetSpellInfo(19750.5).spellID == 19750)
            local fractionalInfo = C_Spell.GetSpellInfo(Fraction)
            assert(fractionalInfo.name == 'Flash of Light' and issecretvalue(fractionalInfo.spellID))
            if canaccessvalue(fractionalInfo.spellID) then
                assert(secretunwrap(fractionalInfo.spellID) == 19750, 'resolved identity, not original fractional payload')
            end
            assert(not pcall(C_Spell.GetSpellName, Fraction), 'typed name rejects fractional selector')
            for _, value in ipairs({Negative, Infinite, Nan, Overflow, SecretString, SecretBool}) do
                assert(select('#', C_Spell.GetSpellInfo(value)) == 1)
                assert(C_Spell.GetSpellInfo(value) == nil)
                assert(select('#', C_Spell.GetSpellTexture(value)) == 0)
                assert(not pcall(C_Spell.GetSpellName, value), 'invalid typed name selector')
                assert(issecretvalue(value), 'invalid wrapper unchanged')
            end
            assert(select('#', C_Spell.GetSpellTexture({})) == 0)
            assert(not pcall(C_Spell.GetSpellName, {}))
        "#,
            tainted,
        );
    }
}

#[test]
fn original_contribution_tooltip_numeric_secret_description() {
    let env = tooltip_full_env_helpers::setup_full_env();
    let original = read_blizzard_source("Blizzard_Contribution/Blizzard_Contribution.lua");
    env.exec(&original)
        .expect("unchanged original Contribution source");
    install_secrets(&env);
    env.exec(
        r#"
        RewardRow = CreateFrame('Frame', nil, UIParent)
        RewardRow:SetSize(32,32); RewardRow:SetPoint('CENTER')
        RewardRow.Icon = RewardRow:CreateTexture(); RewardRow.Icon:SetSize(24,24)
        RewardRow.Icon:SetPoint('CENTER')
        RewardRow.RewardName = RewardRow:CreateFontString(nil, nil, 'GameFontNormal')
        RewardRow.PadLock = CreateFrame('Frame', nil, RewardRow)
        ContributionBuffTooltip = CreateFrame('Frame', nil, UIParent)
        ContributionBuffTooltip:SetSize(200,100)
        ContributionBuffTooltip.Icon = ContributionBuffTooltip:CreateTexture()
        ContributionBuffTooltip.Icon:SetSize(24,24)
        ContributionBuffTooltip.Icon:SetPoint('TOPLEFT')
        for _, key in ipairs({'Name','Description','Footer'}) do
            local text = ContributionBuffTooltip:CreateFontString(nil, nil, 'GameFontNormal')
            text:SetSize(180,20); text:SetPoint('TOPLEFT')
            ContributionBuffTooltip[key] = text
        end
        ExpectedDescription = C_Spell.GetSpellDescription(19750)
        assert(string.find(ExpectedDescription, 'Quickly heal a friendly target', 1, true) == 1)
    "#,
    )
    .unwrap();
    for tainted in [false, true] {
        for selector in ["19750", "Selector"] {
            invoke(
                &env,
                &format!(
                    r#"
                ContributionRewardMixin.Setup(RewardRow, {selector}, true)
                ContributionRewardMixin.OnEnter(RewardRow)
                assert(ContributionBuffTooltip:IsShown())
                assert(ContributionBuffTooltip.Name:GetText() == 'Flash of Light')
                local description = ContributionBuffTooltip.Description:GetText()
                assert(description == ExpectedDescription and not issecretvalue(description))
                assert(select('#', C_Spell.GetSpellDescription({selector})) == 1)
                assert(not issecretvalue(C_Spell.GetSpellDescription({selector})))
            "#
                ),
                tainted,
            );
        }
        invoke(
            &env,
            r#"
            assert(select('#', C_Spell.GetSpellDescription(Missing)) == 1)
            assert(C_Spell.GetSpellDescription(Missing) == '')
            for _, invalid in ipairs({Fraction,Negative,Overflow,Nan,Infinite,SecretString,SecretBool}) do
                assert(not pcall(C_Spell.GetSpellDescription, invalid))
            end
        "#,
            tainted,
        );
    }
}
