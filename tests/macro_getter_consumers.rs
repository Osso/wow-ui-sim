//! Native Retail macro getters, exercised through two independently loaded Blizzard consumers.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use crate::common::panel_fixtures::setup_env;
use wow_ui_sim::lua_api::{WowLuaEnv, state::MacroInfo};

fn macro_record(name: &str, icon: u32, body: &str) -> MacroInfo {
    MacroInfo {
        name: name.into(),
        icon: icon.to_string(),
        body: body.into(),
    }
}

fn seed(env: &WowLuaEnv, names: &[(&str, u32, &str)]) {
    env.state().borrow_mut().macros = names
        .iter()
        .map(|&(name, icon, body)| macro_record(name, icon, body))
        .collect();
}

#[test]
fn macro_ui_uses_supplied_macro_counts_and_details() {
    let env = setup_env();
    let first = ("NativeAlpha", 134400, "/say alpha");
    let second = ("NativeGamma", 134331, "/say gamma");
    seed(&env, &[first]);
    load_macro_ui(&env);
    assert_selected_macro(&env, 1, first);

    seed(&env, &[("NativeBeta", 132212, "/say beta"), second]);
    env.exec("MacroFrame:Update(); MacroFrame:SelectMacro(2)")
        .unwrap();
    assert_selected_macro(&env, 2, second);
    exercise_character_tab(&env);
}

fn load_macro_ui(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(not C_AddOns.IsAddOnLoaded("Blizzard_MacroUI"), "MacroUI starts load-on-demand")
        -- Pre-existing fixture prerequisite from pinned MacroConstantsDocumentation.lua:10–11.
        Constants.MacroConsts = {
            MAX_ACCOUNT_MACROS = 120, MAX_CHARACTER_MACROS = 30,
        }
        ShowMacroFrame()
        assert(C_AddOns.IsAddOnLoaded("Blizzard_MacroUI"), "original macro bootstrap must load UI")
        local source = debug.getinfo(MacroFrame.Update, "S").source
        assert(source:find("Blizzard_MacroUI.lua", 1, true), source)
        MacroFrame:SelectMacro(1)
        "#,
    )
    .expect("original MacroUI load-on-demand setup");
}

fn assert_selected_macro(env: &WowLuaEnv, count: u32, record: (&str, u32, &str)) {
    let observed: (f64, String, f64, String) = env
        .eval(
            r#"
        return MacroFrame.MacroSelector.numMacros, MacroFrameSelectedMacroName:GetText(),
            tonumber(MacroFrame.SelectedMacroButton.Icon:GetTexture()), MacroFrameText:GetText()
        "#,
        )
        .expect("original MacroUI count/title/icon/body");
    assert_eq!(
        observed,
        (
            f64::from(count),
            record.0.into(),
            f64::from(record.1),
            record.2.into()
        )
    );
}

fn exercise_character_tab(env: &WowLuaEnv) {
    let character = ("NativeCharacter", 132212, "/say character");
    {
        let mut sim = env.state().borrow_mut();
        sim.macros.resize_with(121, Default::default);
        sim.macros[120] = macro_record(character.0, character.1, character.2);
    }
    env.exec("MacroFrame:ChangeTab(2)").unwrap();
    assert_selected_macro(env, 1, character);
}

#[test]
fn click_binding_ui_uses_supplied_macro_name_and_numeric_icon() {
    let env = setup_env();
    seed(
        &env,
        &[
            ("ClickMacroAlpha", 134400, "/say alpha"),
            ("ClickMacroBeta", 132212, "/say beta"),
        ],
    );
    env.exec(
        r#"
        assert(not C_AddOns.IsAddOnLoaded("Blizzard_ClickBindingUI"), "click binding starts LoD")
        ClickBindingFrame_LoadUI()
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ClickBindingUI"))
        local function inspect(id, name, icon)
            local button = CreateFrame("Button", nil, UIParent, "ClickBindingLineTemplate")
            local source = debug.getinfo(button.Init, "S").source
            assert(source:find("Blizzard_ClickBindingUI.lua", 1, true), source)
            -- Original local ElementDataTypes.CustomsBinding = 4 (line 42).
            button:Init({
                elementType = 4,
                bindingInfo = { type = Enum.ClickBindingType.Macro, actionID = id },
            })
            assert(button.Name:GetText() == CLICK_BINDING_MACRO_TITLE:format(name), "macro title")
            assert(tonumber(button.Icon:GetTexture()) == icon, "macro numeric icon")
        end
        inspect(1, "ClickMacroAlpha", 134400)
        inspect(2, "ClickMacroBeta", 132212)
        "#,
    )
    .expect("original ClickBindingLineTemplate must distinguish two supplied macro rows");
}

#[test]
fn macro_getter_ordinary_argument_and_return_contract() {
    let env = WowLuaEnv::new().expect("Lua environment");
    let mut records = vec![MacroInfo::default(); 241];
    records[0] = macro_record("Found", 134400, "");
    records[1] = macro_record("", 0, "/say empty name");
    records[120] = macro_record("Character", 132212, "/say character");
    records[239] = macro_record("Last", 0, "");
    records[240] = macro_record("Outside", 134400, "");
    env.state().borrow_mut().macros = records;

    env.exec(
        r##"
        local function tuple(...)
            assert(select("#", ...) == 3, "record still has icon slot when nil")
            return ...
        end
        local name, icon, body = tuple(GetMacroInfo(" 1.9 ", false))
        assert(name == "Found" and icon == 134400 and body == "")
        name, icon, body = tuple(GetMacroInfo("Found"))
        assert(name == "Found" and icon == 134400 and body == "")
        name, icon, body = tuple(GetMacroInfo(2))
        assert(name == "" and icon == nil and body == "/say empty name")
        assert(GetMacroInfo(240) == "Last")
        for _, arg in ipairs({0, -1, 241, true, {}, "unmatched"}) do
            assert(select("#", GetMacroInfo(arg)) == 0, tostring(arg))
        end
        assert(select("#", GetMacroInfo(nil, true)) == 0)
        assert(select("#", GetMacroInfo()) == 0)
        "##,
    )
    .expect("R88l.19 ordinary selector/result shape");
    assert_counter_snapshot(&env, (2, 1));
    seed(
        &env,
        &[
            ("First", 134400, ""),
            ("Second", 132212, ""),
            ("Third", 134331, ""),
        ],
    );
    assert_counter_snapshot(&env, (3, 0));
}

fn assert_counter_snapshot(env: &WowLuaEnv, expected: (u32, u32)) {
    let observed: (f64, f64) = env.eval(r##"
        local account, character = GetNumMacros(false, {})
        assert(select("#", GetNumMacros()) == 2 and type(account) == "number" and type(character) == "number")
        return account, character
        "##).expect("VERIFIED two numbers and ignored arguments");
    assert_eq!(
        observed,
        (f64::from(expected.0), f64::from(expected.1)),
        "INFERRED simulator account/character partition"
    );
}

#[test]
fn macro_getter_inferred_host_policies_and_unknown_seams() {
    let env = WowLuaEnv::new().expect("Lua environment");
    seed(
        &env,
        &[("CaseSensitive", 134400, ""), ("", 0, "/say partial")],
    );
    {
        let mut sim = env.state().borrow_mut();
        sim.macros[0].icon = r"Interface\Icons\INV_Misc_QuestionMark".into();
        sim.macros.push(MacroInfo::default());
    }
    env.exec(
        r##"
        assert(GetMacroInfo("0x1") == "CaseSensitive") -- host numeric parser
        assert(GetMacroInfo(4294967297) == "CaseSensitive") -- chosen u32 wrap
        assert(select("#", GetMacroInfo(9223372036854775808)) == 0) -- outside host i64 domain
        local name, icon = GetMacroInfo("CaseSensitive") -- exact host bytes
        assert(name == "CaseSensitive" and icon == 134400) -- host listfile
        assert(select("#", GetMacroInfo("casesensitive")) == 0)
        assert(GetMacroInfo(2) == "") -- partial record is occupied
        assert(select("#", GetMacroInfo(3)) == 0) -- default host hole
        assert(C_Macro.GetMacroName(1) == nil and C_Macro.GetSelectedMacroIcon(121) == nil)
    "##,
    )
    .expect("INFERRED host selection/listfile policies and UNKNOWN nil seams");
}
