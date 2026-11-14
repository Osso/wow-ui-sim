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
    seed(&env, &[("NativeAlpha", 134400, "/say alpha")]);
    env.exec(
        r#"
        assert(not C_AddOns.IsAddOnLoaded("Blizzard_MacroUI"), "MacroUI starts load-on-demand")
        -- Match the existing LoD fixture when the simulator lacks these unrelated
        -- constants. The values are pinned MacroConstantsDocumentation.lua:10–11.
        Constants.MacroConsts = {
            MAX_ACCOUNT_MACROS = 120, MAX_CHARACTER_MACROS = 30,
        }
        ShowMacroFrame()
        assert(C_AddOns.IsAddOnLoaded("Blizzard_MacroUI"), "original macro bootstrap must load UI")
        local source = debug.getinfo(MacroFrame.Update, "S").source
        assert(source:find("Blizzard_MacroUI.lua", 1, true), source)
        MacroFrame:SelectMacro(1)
        assert(MacroFrame.MacroSelector.numMacros == 1, "account HUD must use supplied count")
        assert(MacroFrameSelectedMacroName:GetText() == "NativeAlpha", "selected title A")
        assert(MacroFrameText:GetText() == "/say alpha", "selected body A")
        assert(tonumber(MacroFrame.SelectedMacroButton.Icon:GetTexture()) == 134400, "selected icon A")
        "#,
    )
    .expect("original MacroUI first host snapshot");

    seed(
        &env,
        &[
            ("NativeBeta", 132212, "/say beta"),
            ("NativeGamma", 134331, "/say gamma"),
        ],
    );
    env.exec(
        r#"
        MacroFrame:Update()
        MacroFrame:SelectMacro(2)
        assert(MacroFrame.MacroSelector.numMacros == 2, "account HUD must update count")
        assert(MacroFrameSelectedMacroName:GetText() == "NativeGamma", "selected title B")
        assert(MacroFrameText:GetText() == "/say gamma", "selected body B")
        assert(tonumber(MacroFrame.SelectedMacroButton.Icon:GetTexture()) == 134331, "selected icon B")
        "#,
    )
    .expect("original MacroUI second host snapshot");
    {
        let mut sim = env.state().borrow_mut();
        sim.macros.resize_with(121, Default::default);
        sim.macros[120] = macro_record("NativeCharacter", 132212, "/say character");
    }
    env.exec(
        r#"
        MacroFrame:ChangeTab(2)
        assert(MacroFrame.MacroSelector.numMacros == 1, "character tab observes its own count")
        assert(MacroFrameSelectedMacroName:GetText() == "NativeCharacter", "character tab consumes offset index")
        assert(MacroFrameText:GetText() == "/say character", "character tab consumes body")
        assert(tonumber(MacroFrame.SelectedMacroButton.Icon:GetTexture()) == 132212, "character tab consumes icon")
        "#,
    )
    .expect("original MacroUI character tab reads its independent count and record");
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
    records[0].icon = r"Interface\Icons\INV_Misc_QuestionMark".into();
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
        assert(GetMacroInfo("0x1") == "Found")
        name, icon, body = tuple(GetMacroInfo("Found"))
        assert(name == "Found" and icon == 134400 and body == "")
        name, icon, body = tuple(GetMacroInfo(2))
        assert(name == "" and icon == nil and body == "/say empty name")
        assert(GetMacroInfo(240) == "Last")
        for _, arg in ipairs({0, -1, 3, 241, true, {}, "unknown"}) do
            assert(select("#", GetMacroInfo(arg)) == 0, tostring(arg))
        end
        assert(select("#", GetMacroInfo(nil, true)) == 0)
        assert(select("#", GetMacroInfo()) == 0)

        local account, character = GetNumMacros(false, {})
        assert(select("#", GetNumMacros()) == 2 and type(account) == "number" and type(character) == "number")
        -- The simulator's partition interpretation is inferred from the MacroUI tabs.
        assert(account == 2 and character == 1)
        assert(C_Macro.GetMacroName(1) == nil and C_Macro.GetSelectedMacroIcon(121) == nil)
        "##,
    )
    .expect("R88l.18–19 ordinary argument/return shape; unknown C_Macro getters stay neutral");
}
