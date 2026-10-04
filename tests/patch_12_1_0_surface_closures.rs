//! Lua-visible behavior for 12.1.0 surface added or retired after the publication sweep.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create Lua environment")
}

#[test]
fn removed_random_training_ground_join_is_not_fabricated() {
    let (raw, lookup): (String, String) = env()
        .eval(
            r#"return type(rawget(C_PvP, "JoinRandomTrainingGround")),
                type(C_PvP.JoinRandomTrainingGround)"#,
        )
        .unwrap();
    assert_eq!((raw.as_str(), lookup.as_str()), ("nil", "nil"));
}

#[test]
fn removed_auction_cvars_have_no_value_or_default() {
    let removed: bool = env()
        .eval(
            r#"for _, name in ipairs({"auctionDisplayOnCharacter", "auctionSortByBuyoutPrice",
                                      "auctionSortByUnitPrice"}) do
                if C_CVar.GetCVar(name) ~= nil or C_CVar.GetCVarDefault(name) ~= nil then
                    return false
                end
            end
            return true"#,
        )
        .unwrap();
    assert!(removed);
}

#[test]
fn added_display_cvars_default_off_and_round_trip() {
    let observed: String = env()
        .eval(
            r#"local out = {}
            for _, name in ipairs({"tooltipShowAuraSpellIDs", "worldMapShowCursorCoords",
                                   "worldMapShowPlayerCoords"}) do
                local before = C_CVar.GetCVar(name) .. "/" .. C_CVar.GetCVarDefault(name)
                C_CVar.SetCVar(name, "1")
                out[#out + 1] = before .. "->" .. tostring(GetCVarBool(name))
            end
            return table.concat(out, ",")"#,
        )
        .unwrap();
    assert_eq!(observed, "0/0->true,0/0->true,0/0->true");
}

#[test]
fn patch_12_1_event_registerability() {
    let observed: String = env()
        .eval(
            r#"local frame = CreateFrame("Frame")
            local out = {}
            for _, event in ipairs({"FULLSCREEN_BROWSER_SPINNER_SHOW",
                                    "FULLSCREEN_BROWSER_SPINNER_HIDE",
                                    "HOUSING_LAYOUT_OCCUPIED_FLOOR_RANGE_CHANGED",
                                    "HOUSING_LAYOUT_NUM_FLOORS_CHANGED"}) do
                local ok = pcall(frame.RegisterEvent, frame, event)
                out[#out + 1] = tostring(ok and frame:IsEventRegistered(event) == true)
            end
            return table.concat(out, ",")"#,
        )
        .unwrap();
    assert_eq!(observed, "true,true,true,false");
}

#[test]
fn radial_progress_animation_stores_sweep_percents() {
    let observed: (String, f64, f64, f64, f64, bool) = env()
        .eval(
            r#"local group = CreateFrame("Frame"):CreateAnimationGroup()
            local radial = group:CreateAnimation("RadialProgress")
            local from, to = radial:GetFromPercent(), radial:GetToPercent()
            radial:SetFromPercent(0.25)
            radial:SetToPercent(0.75)
            local rejects = not pcall(radial.SetFromPercent, radial, "half")
            return radial:GetObjectType(), from, to,
                radial:GetFromPercent(), radial:GetToPercent(), rejects"#,
        )
        .unwrap();
    assert_eq!(
        observed,
        ("RadialProgress".to_string(), 0.0, 1.0, 0.25, 0.75, true)
    );
}

#[test]
fn seconds_formatter_reports_configured_rounding() {
    let rounding: (f64, f64) = env()
        .eval(
            r#"local formatter = C_StringUtil.CreateSecondsFormatter()
            local default = formatter:GetRounding()
            formatter:SetRounding(Enum.SecondsFormatterRounding.RoundUp)
            return default, formatter:GetRounding()"#,
        )
        .unwrap();
    assert_eq!(rounding, (1.0, 0.0));
}

#[test]
fn font_string_records_embedded_texture_desaturation() {
    let env = env();
    env.exec(
        r#"local text = CreateFrame("Frame"):CreateFontString("DesaturateProbe")
        text:SetDesaturateEmbeddedTextures(true)
        CreateFrame("Frame"):CreateFontString("PlainProbe")"#,
    )
    .unwrap();
    let state = env.state().borrow();
    let flag = |name| {
        state
            .widgets
            .get_by_name(name)
            .map(|frame| frame.desaturate_embedded_textures)
    };
    assert_eq!(
        (flag("DesaturateProbe"), flag("PlainProbe")),
        (Some(true), Some(false))
    );
}

#[test]
fn roleset_membership_is_not_filtered_without_active_filters() {
    let filtered: (bool, bool) = env()
        .eval(
            r#"local frame = CreateFrame("Frame")
            local before = frame:IsRolesetFiltered()
            frame:SetRolesets("Healer", "Tank")
            return before, frame:IsRolesetFiltered()"#,
        )
        .unwrap();
    assert_eq!(filtered, (false, false));
}

#[test]
fn context_access_denies_tainted_callers_on_restricted_objects() {
    let observed: (bool, bool, bool, bool) = env()
        .eval(
            r#"local open = CreateFrame("Frame")
            local restricted = CreateFrame("Frame")
            restricted:AddAccessRestrictions(1)
            local secureRestricted = restricted:CanBeAccessedInContext()
            forceinsecure()
            local taintedOpen = open:CanBeAccessedInContext()
            local taintedRestricted = restricted:CanBeAccessedInContext()
            debug.setstacktaint(nil)
            return secureRestricted, taintedOpen, taintedRestricted,
                restricted:CanBeAccessedInContext()"#,
        )
        .unwrap();
    assert_eq!(observed, (true, true, false, true));
}
