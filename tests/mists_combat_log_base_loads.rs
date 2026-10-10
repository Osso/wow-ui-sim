//! Real Mists TOC loading; no test-produced school masks or colors.
#![cfg(feature = "client-mists")]

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::{discover_blizzard_startup_addons_for_screen, load_startup_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn log_loading_scoped_script_env(env: &WowLuaEnv, stage: &str) {
    let scoped_env = env.state().borrow().loading_scoped_script_env;
    eprintln!(
        "{stage} loading_scoped_script_env: present={}, reference={scoped_env:?}",
        scoped_env.is_some()
    );
}

#[test]
fn mists_combat_log_base_school_colors_load_from_toc() {
    crate::common::with_timeout(90, || {
        assert_eq!(ACTIVE, ClientProfile::Mists);
        let ui =
            wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("resolve Mists cached UI");
        assert!(
            ui.ends_with("mists/AddOns"),
            "wrong cache: {}",
            ui.display()
        );
        let env = WowLuaEnv::new().expect("initialize Mists globals");
        let initialized: String = env
            .eval(
                r#"
                local masks = Enum.Damageclass
                local observations = {}
                for _, school in ipairs({"None", "Physical", "Holy", "Fire", "Nature", "Frost", "Shadow", "Arcane"}) do
                    assert(type(masks["Mask" .. school]) == "number", school .. " enum mask missing")
                    table.insert(observations, school .. ":" .. tostring(rawget(_G, "SCHOOL_MASK_" .. school:upper())) .. "/" .. tostring(masks["Mask" .. school]))
                end
                return table.concat(observations, ",")
                "#,
            )
            .expect("inspect initialized school globals and authoritative enum masks");
        eprintln!("Mists initialized school masks: {initialized}");
        env.set_screen_mode(ScreenKind::Game);
        env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
        wow_ui_sim::xml::register_intrinsic_templates();
        let addons = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
        let consumer = addons
            .iter()
            .position(|addon| addon.name == "Blizzard_CombatLogBase")
            .expect("CombatLogBase participates in normal Mists startup");
        for name in ["Blizzard_FrameXMLBase", "Blizzard_FrameXML"] {
            eprintln!(
                "{name} startup position: {:?}; CombatLogBase: {consumer}",
                addons.iter().position(|addon| addon.name == name)
            );
        }
        for addon in &addons[..=consumer] {
            if addon.name == "Blizzard_CombatLogBase" {
                log_loading_scoped_script_env(&env, "Pre-CombatLogBase");
                let observations: String = env
                    .eval(
                        r#"
                        local observations = {}
                        local function observe(name, direct, raw)
                            table.insert(observations, name .. ": direct=" .. tostring(direct)
                                .. " (" .. type(direct) .. "), raw=" .. tostring(raw)
                                .. " (" .. type(raw) .. "), equal=" .. tostring(direct == raw))
                        end
                        local globals = {
                            {"COMBATLOG_FILTER_MINE", COMBATLOG_FILTER_MINE},
                            {"COMBATLOG_FILTER_MY_PET", COMBATLOG_FILTER_MY_PET},
                            {"COMBATLOG_FILTER_FRIENDLY_UNITS", COMBATLOG_FILTER_FRIENDLY_UNITS},
                            {"COMBATLOG_FILTER_HOSTILE_UNITS", COMBATLOG_FILTER_HOSTILE_UNITS},
                            {"COMBATLOG_FILTER_HOSTILE_PLAYERS", COMBATLOG_FILTER_HOSTILE_PLAYERS},
                            {"COMBATLOG_FILTER_NEUTRAL_UNITS", COMBATLOG_FILTER_NEUTRAL_UNITS},
                            {"COMBATLOG_FILTER_UNKNOWN_UNITS", COMBATLOG_FILTER_UNKNOWN_UNITS},
                            {"SCHOOL_MASK_NONE", SCHOOL_MASK_NONE},
                            {"SCHOOL_MASK_PHYSICAL", SCHOOL_MASK_PHYSICAL},
                            {"SCHOOL_MASK_HOLY", SCHOOL_MASK_HOLY},
                            {"SCHOOL_MASK_FIRE", SCHOOL_MASK_FIRE},
                            {"SCHOOL_MASK_NATURE", SCHOOL_MASK_NATURE},
                            {"SCHOOL_MASK_FROST", SCHOOL_MASK_FROST},
                            {"SCHOOL_MASK_SHADOW", SCHOOL_MASK_SHADOW},
                            {"SCHOOL_MASK_ARCANE", SCHOOL_MASK_ARCANE},
                        }
                        for _, entry in ipairs(globals) do
                            observe(entry[1], entry[2], rawget(_G, entry[1]))
                        end
                        local directNone
                        if Enum and Enum.CombatLogObject then
                            directNone = Enum.CombatLogObject.None
                        end
                        local rawEnum = rawget(_G, "Enum")
                        local rawObject = type(rawEnum) == "table" and rawget(rawEnum, "CombatLogObject") or nil
                        local rawNone
                        if type(rawObject) == "table" then
                            rawNone = rawget(rawObject, "None")
                        end
                        observe("Enum.CombatLogObject.None", directNone, rawNone)
                        return table.concat(observations, "\n")
                        "#,
                    )
                    .expect("inspect all color keys immediately before consumer TOC");
                eprintln!("Pre-CombatLogBase color keys:\n{observations}");
            }
            let load_result =
                load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None);
            if addon.name == "Blizzard_CombatLogBase" {
                log_loading_scoped_script_env(&env, "Post-CombatLogBase");
            }
            load_result.unwrap_or_else(|error| panic!("load {}: {error}", addon.name));
        }
        let errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|error| error.contains("Blizzard_CombatLogBase"))
            .cloned()
            .collect();
        assert!(
            errors.is_empty(),
            "Mists CombatLogBase TOC errors: {errors:#?}"
        );
        let colors: bool = env
            .eval(
                r#"
                local masks = Enum.Damageclass
                for _, school in ipairs({"None", "Physical", "Holy", "Fire", "Nature", "Frost", "Shadow", "Arcane"}) do
                    assert(rawget(_G, "SCHOOL_MASK_" .. school:upper()) == masks["Mask" .. school], school .. " legacy mask differs")
                end
                assert(COMBAT_LOG_SCHOOL_MASK_NONE == masks.MaskNone)
                local colors = COMBATLOG_DEFAULT_COLORS.schoolColoring
                local none, fire = colors[masks.MaskNone], colors[masks.MaskFire]
                return type(none) == "table" and none.a == 1 and none.r == 1 and none.g == 1 and none.b == 1
                    and type(fire) == "table" and fire.a == 1 and fire.r == 1 and fire.g == 0.5 and fire.b == 0
                "#,
            )
            .expect("query actual Wrath schoolColoring selected by Mists TOC");
        assert!(
            colors,
            "Mists None/Fire school colors differ from vendor colors"
        );
    });
}
