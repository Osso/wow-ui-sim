#![cfg(feature = "client-wowforever")]

use crate::common;

#[test]
fn original_superdistrict_keeps_offline_realm_catalogs_empty() {
    test_timeout! {
        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape_in_dir(
            &wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
            &["Blizzard_GlueXML"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_GlueXML"));
                env.exec(r#"
                    local source = debug.getinfo(SuperDistrictMixin.OnShow, "S").source:gsub("\\", "/")
                    assert(source:find("Blizzard_GlueXML/Mainline/SuperDistrict.lua", 1, true))
                    assert(type(C_Login.GetSubRegions()) == "table" and #C_Login.GetSubRegions() == 0)
                    assert(select('#', C_Login.GetSubRegions()) == 1)
                    assert(type(C_DistrictUtils.GetAvailableSuperDistricts()) == "table")
                    assert(#C_DistrictUtils.GetAvailableSuperDistricts() == 0)
                    assert(select('#', C_DistrictUtils.GetAvailableSuperDistricts()) == 1)

                    SuperDistrictUI:RefreshSuperDistrictButtons()
                    SuperDistrictUI:UpdateSelection(nil)
                    assert(#SuperDistrictUI.districtButtons == 0, "offline catalog must create no district buttons")
                    assert(not SuperDistrictUI.SelectButton:IsEnabled(), "an empty catalog cannot have a selected district")
                    SuperDistrictUI.SubRegionDropdown:Show()
                    SuperDistrictMixin.OnShow(SuperDistrictUI)
                    assert(not SuperDistrictUI.SubRegionDropdown:IsShown(), "an empty catalog must hide the subregion selector")
                "#).expect("the selected original district consumer must handle empty offline catalogs");
            },
        );
    }
}

#[test]
fn original_game_logo_hides_backdrop_without_offline_display_record() {
    test_timeout! {
        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape_in_dir(
            &wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
            &["Blizzard_GlueXML"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_GlueXML"));
                env.exec(r#"
                    local source = debug.getinfo(GameLogoDarkBackdropMixin.Update, "S").source:gsub("\\", "/")
                    assert(source:find("Blizzard_GlueXML/Shared/GameLogoDarkBackdrop.lua", 1, true))
                    assert(type(rawget(C_GameRules, "GetCurrentGameModeDisplayInfo")) == "function")
                    assert(C_GameRules.GetCurrentGameModeDisplayInfo() == nil)
                    assert(select('#', C_GameRules.GetCurrentGameModeDisplayInfo()) == 1)

                    local frame = CreateFrame("Frame")
                    frame.BackdropTexture = frame:CreateTexture()
                    frame.BackdropTexture:Show()
                    assert(frame.BackdropTexture:IsShown())
                    GameLogoDarkBackdropMixin.Update(frame)
                    assert(not frame.BackdropTexture:IsShown(), "offline display must use the original no-backdrop branch")
                "#).expect("the selected original logo consumer must accept the offline nil record");
            },
        );
    }
}

#[test]
fn original_navbar_does_not_add_offline_cheat_realms_button() {
    test_timeout! {
        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape_in_dir(
            &wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
            &["Blizzard_GlueXML"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_CharacterSelectNavBar"));
                env.exec(r#"
                    local source = debug.getinfo(CharacterSelectNavBarMixin.TrySetUpRealmsButton, "S").source:gsub("\\", "/")
                    assert(source:find("Blizzard_CharacterSelectNavBar/CharacterSelectNavBar.lua", 1, true))
                    assert(type(rawget(C_RealmList, "ShouldShowRealmsAsCheatButton")) == "function")
                    assert(C_RealmList.ShouldShowRealmsAsCheatButton() == false)
                    assert(select('#', C_RealmList.ShouldShowRealmsAsCheatButton()) == 1)

                    local frame = CreateFrame("Frame")
                    frame.realmsButtonAvailable = false
                    local childCount = frame:GetNumChildren()
                    CharacterSelectNavBarMixin.TrySetUpRealmsButton(frame)
                    assert(frame.RealmsButton == nil, "offline policy must not add a cheat realms button")
                    assert(frame:GetNumChildren() == childCount)
                "#).expect("the selected original navbar must keep the offline cheat realms button absent");
            },
        );
    }
}
