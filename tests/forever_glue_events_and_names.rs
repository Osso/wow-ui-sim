#![cfg(feature = "client-wowforever")]

use crate::common;

#[test]
fn original_sd_hd_toggle_tracks_supplied_game_rule_state() {
    test_timeout! {
        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape_in_dir(
            &wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
            &["Blizzard_GlueXML", "Blizzard_CharacterCreate"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_CharacterCreate"));
                env.exec(r#"
                    local source = debug.getinfo(CharCreateSDToggleMixin.OnLoad, "S").source:gsub("\\", "/")
                    assert(source:find("Blizzard_CharacterCreate/Camelot/Blizzard_CharacterCreate.lua", 1, true))
                    assert(Enum.SdToggleUsageLocationUI.CharacterCreation == 1)
                    assert(type(rawget(C_GameRules, "IsSDHDToggleEnabled")) == "function")
                    assert(type(rawget(C_GameRules, "AccountHasSDEnabled")) == "function")
                    assert(type(rawget(C_GameRules, "SetSDHDToggleValue")) == "function")
                "#).expect("the selected original toggle must use the shared SD/HD surface");

                for enabled in [false, true, false] {
                    env.state().borrow_mut().game_rules.sd_hd_toggle_enabled = enabled;
                    let (available, shown, arity): (bool, bool, i32) = env.eval(r#"
                        CharCreateSDToggleMixin.OnLoad(SDToggleButton)
                        return C_GameRules.IsSDHDToggleEnabled(), SDToggleButton:IsShown(),
                               select('#', C_GameRules.IsSDHDToggleEnabled())
                    "#).expect("original toggle visibility must track the supplied capability");
                    assert_eq!((available, shown, arity), (enabled, enabled, 1));
                }

                for set_to_sd in [true, false] {
                    env.exec(&format!(
                        "assert(select('#', C_GameRules.SetSDHDToggleValue({set_to_sd}, Enum.SdToggleUsageLocationUI.CharacterCreation)) == 0); CharCreateSDToggleMixin.OnShow(SDToggleButton)"
                    )).expect("supplied SD/HD selection must update the original toggle");
                    let (account_has_sd, checked, arity): (bool, bool, i32) = env.eval(r#"
                        return C_GameRules.AccountHasSDEnabled(), SDToggleButton:GetChecked(),
                               select('#', C_GameRules.AccountHasSDEnabled())
                    "#).unwrap();
                    assert_eq!((account_has_sd, checked, arity), (set_to_sd, !set_to_sd, 1));
                    assert_eq!(env.state().borrow().game_rules.sd_toggle_usage_location, Some(1));
                }
            },
        );
    }
}

#[test]
fn selected_glue_consumers_register_forever_events() {
    test_timeout! {
        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape_in_dir(
            &wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
            &["Blizzard_GlueXML", "Blizzard_CharacterCreate"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_GlueXML"));
                for (frame, event) in [
                    ("SuperDistrictUI", "SUPER_DISTRICT_LIST_UPDATED"),
                    ("CharacterSelectUI", "GAME_RULES_CHANGED"),
                    ("CharacterSelectUI", "CHAR_SELECT_BACKGROUND_MODEL_READY"),
                    ("CharacterSelect", "CHARACTER_UNDELETE_STARTED"),
                    ("CharacterCreateFrame", "CHAR_CREATE_BACKGROUND_MODEL_READY"),
                ] {
                    assert!(wow_ui_sim::event::is_valid_event(event), "{event}");
                    env.exec(&format!("assert({frame}:IsEventRegistered('{event}'), '{frame}: {event}')"))
                        .unwrap_or_else(|error| panic!("{frame}: {event}: {error}"));
                }
                let error = env
                    .exec("CreateFrame('Frame'):RegisterEvent('FOREVER_UNDECLARED_GLUE_EVENT')")
                    .unwrap_err();
                assert!(error.to_string().contains("unknown event"), "{error}");
            },
        );
    }
}

#[test]
fn paid_name_change_surname_visibility_tracks_supplied_naming_policy() {
    test_timeout! {
        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape_in_dir(
            &wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
            &["Blizzard_GlueXML"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_GlueXML"));
                env.exec(r#"
                    for _, mixin in ipairs({NewSurnameLabelMixin, NewSurnameEditboxMixin}) do
                        local source = debug.getinfo(mixin.ShouldShow, "S").source:gsub("\\", "/")
                        assert(source:find("Blizzard_GlueXML/Shared/CharacterServicesPaidNameChange.lua", 1, true))
                    end
                "#).expect("surname predicates must come from the selected paid-name-change consumer");

                for enabled in [false, true, false] {
                    env.state().borrow_mut().player.regional_unique_names_enabled = enabled;
                    let (api_value, legacy_value, label_visible, editbox_visible, arity):
                        (bool, bool, bool, bool, i32) = env.eval(r#"
                            assert(type(rawget(C_CharacterCreation, "AreRegionalUniqueNamesEnabled")) == "function")
                            return C_CharacterCreation.AreRegionalUniqueNamesEnabled(),
                                   RegionalUniqueNamesEnabled(),
                                   NewSurnameLabelMixin:ShouldShow(),
                                   NewSurnameEditboxMixin:ShouldShow(),
                                   select('#', C_CharacterCreation.AreRegionalUniqueNamesEnabled())
                        "#).expect("original surname predicates must read the supplied policy");
                    assert_eq!(
                        (api_value, legacy_value, label_visible, editbox_visible, arity),
                        (enabled, enabled, enabled, enabled, 1),
                    );
                }
            },
        );
    }
}
