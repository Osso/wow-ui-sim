//! P6 uses original meter OnEnter and real TooltipDataHandler, independent of P3.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use rilua::{LuaApi, LuaApiMut};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::setup_full_env();
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    let original =
        std::fs::read_to_string(ui.join("Blizzard_DamageMeter/DamageMeterEntry.lua")).unwrap();
    let init = original
        .split("function DamageMeterSpellEntryMixin:Init(combatSpell)")
        .nth(1)
        .unwrap();
    let registration = init
        .split("\tself:GetIcon():SetScript(\"OnLeave\"")
        .next()
        .unwrap();
    let registration = registration
        .split_once("\tself:GetIcon():SetScript(\"OnEnter\"")
        .unwrap()
        .1;
    env.exec(&format!("function InstallOriginalMeterEnter(self)\n self:GetIcon():SetScript(\"OnEnter\"{registration}\nend")).unwrap();
    env.exec("MeterIcon = CreateFrame('Button', nil, UIParent); MeterIcon:SetSize(24, 24); MeterIcon:SetPoint('CENTER'); MeterRow = {GetIcon = function() return MeterIcon end}").unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, number) in [
        ("Selector", 19750.0),
        ("MountSelector", 458.0),
        ("Missing", 900002.0),
        ("Fraction", 19750.5),
        ("Negative", -1.0),
        ("Overflow", 4294967296.0),
        ("Nan", f64::NAN),
        ("Infinite", f64::INFINITY),
    ] {
        let value = rilua::table_security::wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        lua.set_global_val(name, value).unwrap();
        lua.state_mut().pop();
    }
    let flag = rilua::table_security::wrap_host_secret_bool(lua.state_mut(), true);
    lua.state_mut().push(flag);
    lua.set_global_val("SecretFlag", flag).unwrap();
    lua.state_mut().pop();
    let text = rilua::table_security::wrap_host_secret_string(lua.state_mut(), "19750");
    lua.state_mut().push(text);
    lua.set_global_val("SecretString", text).unwrap();
    lua.state_mut().pop();
    drop(lua);
    env.state().borrow_mut().lua_errors.clear();
    env
}

fn invoke(env: &WowLuaEnv, code: &str, tainted: bool) {
    env.exec(&format!(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == {expected})
            {code}
            assert(issecretvalue(Selector) and issecretvalue(MountSelector))
            if {tainted} then
                for _, value in ipairs({{Selector, MountSelector, Missing}}) do
                    assert(not canaccessvalue(value))
                    assert(not pcall(secretunwrap, value))
                    assert(not pcall(function() return value + 1 end))
                end
            end
            assert(debug.getstacktaint() == before)
        end
        if {tainted} then debug.setobjecttaint(probe, 'TooltipSelectorProbe') end
        probe()
    "#,
        expected = if tainted {
            "'TooltipSelectorProbe'"
        } else {
            "nil"
        }
    ))
    .unwrap_or_else(|error| {
        panic!(
            "{code}: {error}; collected errors: {:?}",
            env.state().borrow().lua_errors
        )
    });
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "original tooltip must not swallow errors"
    );
}

#[test]
fn original_meter_on_enter_numeric_secret_selector() {
    let env = fixture();
    for tainted in [false, true] {
        for selector in ["19750", "Selector"] {
            invoke(
                &env,
                &format!(
                    r#"
                GameTooltip:ClearLines(); GameTooltip:Hide()
                MeterRow.spellID = {selector}
                InstallOriginalMeterEnter(MeterRow)
                MeterIcon:GetScript('OnEnter')()
                assert(GameTooltip:IsShown())
                assert(GameTooltip:NumLines() > 1)
                assert(GameTooltip:GetLeftLine(1):GetText() == 'Flash of Light')
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(data.lines[1].leftText == 'Flash of Light')
                assert(issecretvalue(data.id) == issecretvalue({selector}))
            "#
                ),
                tainted,
            );
        }
    }
}

#[test]
fn original_mount_dynamic_handler_numeric_secret_selector() {
    let env = fixture();
    for tainted in [false, true] {
        for selector in ["458", "MountSelector"] {
            invoke(
                &env,
                &format!(
                    r#"
                GameTooltip:ClearLines()
                GameTooltip:SetOwner(MeterIcon, 'ANCHOR_NONE')
                GameTooltip:SetMountBySpellID({selector}, false)
                assert(GameTooltip:NumLines() > 0)
                assert(GameTooltip:GetLeftLine(1):GetText() == 'Brown Horse')
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(issecretvalue(data.id) == issecretvalue({selector}))
            "#
                ),
                tainted,
            );
        }
    }
}

#[test]
fn tooltip_numeric_secret_misses_and_never_secret_controls() {
    let env = fixture();
    for tainted in [false, true] {
        invoke(
            &env,
            r#"
            for _, query in ipairs({C_TooltipInfo.GetSpellByID, C_TooltipInfo.GetMountBySpellID}) do
                assert(select('#', query(Missing)) == 1)
                local data = query(Missing)
                assert(type(data) == 'table' and #data.lines == 0)
                assert(issecretvalue(data.id), 'unknown identity remains secret')
                if canaccessvalue(data.id) then assert(secretunwrap(data.id) == 900002) end
                for _, invalid in ipairs({Fraction, Negative, Overflow, Nan, Infinite, SecretString, SecretFlag, {}}) do
                    assert(not pcall(query, invalid))
                end
            end
            for position = 2, 6 do
                local args = {Selector, nil, nil, nil, nil, nil}
                for _, value in ipairs({SecretFlag, Selector, SecretString}) do
                    args[position] = value
                    local ok, err = pcall(C_TooltipInfo.GetSpellByID, unpack(args, 1, 6))
                    assert(not ok and string.find(err, 'NeverSecret', 1, true))
                end
            end
            local ok, err = pcall(C_TooltipInfo.GetMountBySpellID, MountSelector, SecretFlag)
            assert(not ok and string.find(err, 'NeverSecret', 1, true))
        "#,
            tainted,
        );
    }
}

#[test]
fn tooltip_numeric_secret_direct_arity_identity_and_model_preservation() {
    let env = fixture();
    let original_aliases = env.state().borrow().spell_id_aliases.clone();
    let original_mounts = format!("{:?}", env.state().borrow().world.mounts);
    for tainted in [false, true] {
        for (query, public, secret, title) in [
            ("GetSpellByID", 19750, "Selector", "Flash of Light"),
            (
                "GetMountBySpellID",
                458,
                "MountSelector",
                "Brown Horse",
            ),
        ] {
            for selector in [public.to_string(), secret.into()] {
                invoke(
                    &env,
                    &format!(
                        r#"
                    assert(select('#', C_TooltipInfo.{query}({selector})) == 1)
                    DirectTooltipData = C_TooltipInfo.{query}({selector})
                    assert(not issecretvalue(DirectTooltipData))
                    assert(DirectTooltipData.lines[1].leftText == '{title}')
                    assert(not issecretvalue(DirectTooltipData.lines[1].leftText))
                    assert(issecretvalue(DirectTooltipData.id) == issecretvalue({selector}))
                "#
                    ),
                    tainted,
                );
                let id: rilua::Val = env.eval("return DirectTooltipData.id").unwrap();
                let lua = env.lua();
                assert_eq!(
                    rilua::table_security::unwrap_secret(lua.state(), id).unwrap(),
                    rilua::Val::Num(f64::from(public))
                );
            }
        }
        for query in ["GetSpellByID", "GetMountBySpellID"] {
            for selector in ["900002", "Missing"] {
                invoke(
                    &env,
                    &format!(
                        r#"
                    assert(select('#', C_TooltipInfo.{query}({selector})) == 1)
                    DirectTooltipData = C_TooltipInfo.{query}({selector})
                    assert(next(DirectTooltipData.lines) == nil)
                    assert(issecretvalue(DirectTooltipData.id) == issecretvalue({selector}))
                "#
                    ),
                    tainted,
                );
                let id: rilua::Val = env.eval("return DirectTooltipData.id").unwrap();
                let lua = env.lua();
                assert_eq!(
                    rilua::table_security::unwrap_secret(lua.state(), id).unwrap(),
                    rilua::Val::Num(900002.0)
                );
            }
        }
    }
    assert_eq!(env.state().borrow().spell_id_aliases, original_aliases);
    assert_eq!(
        format!("{:?}", env.state().borrow().world.mounts),
        original_mounts
    );
}
