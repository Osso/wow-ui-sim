//! O source-rewrite retirement through runtime-selected original consumers.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn shared_owner() -> WowLuaEnv {
    let ui = crate::common::blizzard_ui_dir();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_SharedXML"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_SharedXML"));
    // SharedXML's Both TOC directly includes EventUtil.lua and Shared/Slider/MinimalSlider.xml.
    let toc =
        std::fs::read_to_string(ui.join("Blizzard_SharedXML/Blizzard_SharedXML.toc")).unwrap();
    assert!(toc.contains("EventUtil.lua"));
    env
}

#[test]
fn stub_o_eventutil_original_loaded_addon_invokes_callable_tables() {
    let env = shared_owner();
    env.exec(r#"
        local _, loaded = C_AddOns.IsAddOnLoaded("Blizzard_SharedXML")
        assert(loaded, "exercise original immediate loaded-addon branch")
        local calls = 0
        local callback = setmetatable({}, { __call = function(self)
            calls = calls + 1
        end })
        EventUtil.ContinueOnAddOnLoaded("Blizzard_SharedXML", callback)
        assert(calls == 1, "original callback() accepts Lua-callable tables; no type=function rewrite")
        local ok = pcall(EventUtil.ContinueOnAddOnLoaded, "Blizzard_SharedXML", 42)
        assert(not ok, "original callback() rejects non-callable values rather than silently skipping")
    "#).expect("original EventUtil callback contract");
}

#[test]
fn stub_o_menu_original_template_owns_text_and_rejects_missing_child() {
    let env = shared_owner();
    env.exec(r#"
        local dropdown = CreateFrame("DropdownButton", "OOwnedDropdown", UIParent, "WowStyle1DropdownTemplate")
        assert(dropdown.Text, "original selected template/style owns Text")
        local originalText = dropdown.Text
        dropdown:SetText("O original dropdown")
        assert(dropdown.Text == originalText and originalText:GetText() == "O original dropdown")
        -- Invalid receiver assertion independently rejects source rewrite's invented child/early return.
        -- This is Lua error-contract evidence, not a claim about native XML creation failure.
        local malformed = CreateFrame("DropdownButton", nil, UIParent)
        Mixin(malformed, DropdownTextMixin)
        local ok = pcall(malformed.SetText, malformed, "missing child")
        assert(not ok, "original UpdateText does not invent a child for malformed receivers")
    "#).expect("original dropdown owner and error contract");
}

#[test]
fn stub_o_slider_original_template_owns_thumb_and_rejects_missing_child() {
    let env = shared_owner();
    env.exec(r#"
        local slider = CreateFrame("Frame", "OOwnedSlider", UIParent, "MinimalSliderWithSteppersTemplate")
        assert(slider.Slider and slider.Slider.Thumb, "original template creates thumb")
        slider:SetEnabled(false)
        local disabledAlpha = slider.Slider.Thumb:GetAlpha()
        slider:SetEnabled(true)
        assert(slider.Slider.Thumb:GetAlpha() > disabledAlpha, "original ConfigureSlider updates the actual thumb")
        local malformed = CreateFrame("Frame", "OMissingThumbSlider", UIParent, "MinimalSliderWithSteppersTemplate")
        -- Deliberately malformed child binding; all other template-owned behavior remains intact.
        malformed.Slider.Thumb = nil
        local ok = pcall(malformed.SetEnabled, malformed, false)
        assert(not ok, "original ConfigureSlider rejects missing thumb instead of suppressing it")
    "#).expect("original slider owner and error contract");
}
