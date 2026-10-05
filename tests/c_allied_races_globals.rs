//! Integration tests for the `C_AlliedRaces` surface registered in
//! `src/c_api/c_allied_races.rs`. Drives `Blizzard_AlliedRacesUI`'s
//! `LoadRaceData` path which expects every canonical allied race to
//! resolve and the returned `bannerColor` to expose `ColorMixin:GetRGB`.

use wow_ui_sim::lua_api::{AlliedRaceInfo, AlliedRaceRacialAbility, WowLuaEnv};

const CANONICAL_RACE_FILE_STRINGS: &[&str] = &[
    "lightforgeddraenei",
    "darkirondwarf",
    "voidelf",
    "mechagnome",
    "vulpera",
    "zandalaritroll",
    "highmountaintauren",
    "nightborne",
    "magharorc",
    "earthendwarf",
];

fn race_id_for(file_string: &str) -> i64 {
    let env = WowLuaEnv::new().expect("env");
    let state = env.state().borrow();
    state
        .allied_races
        .values()
        .find(|info| info.race_file_string == file_string)
        .unwrap_or_else(|| panic!("missing canonical race {file_string}"))
        .race_id
}

#[test]
fn race_info_survives_collection_in_color_callback() {
    let env = WowLuaEnv::new().expect("env");
    let id = race_id_for("lightforgeddraenei");
    env.exec(&format!(
        r#"
        local original = CreateColor
        CreateColor = function(...)
            collectgarbage('collect')
            return original(...)
        end
        local info = C_AlliedRaces.GetRaceInfoByID({id})
        assert(info.maleName == 'Lightforged Draenei')
        assert(info.femaleName == 'Lightforged Draenei')
        assert(info.raceFileString == 'lightforgeddraenei')
        assert(info.crestAtlas == 'alliedraces-icon-lightforgeddraenei')
        assert(type(info.description) == 'string' and #info.description > 0)
        assert(type(info.achievementIds) == 'table' and #info.achievementIds > 0)
        assert(type(info.bannerColor.GetRGB) == 'function')
        "#
    )).unwrap();
}

#[test]
fn unknown_race_id_returns_nil() {
    let env = WowLuaEnv::new().expect("env");
    let nil: bool = env
        .eval("return C_AlliedRaces.GetRaceInfoByID(99999) == nil")
        .unwrap();
    assert!(nil, "GetRaceInfoByID should return nil for unknown ids");
}

#[test]
fn missing_arg_returns_nothing() {
    let env = WowLuaEnv::new().expect("env");
    let count: f64 = env
        .eval("return select('#', C_AlliedRaces.GetRaceInfoByID())")
        .unwrap();
    assert_eq!(count, 0.0, "GetRaceInfoByID() with no args returns nothing");
}

#[test]
fn canonical_race_ids_are_seeded() {
    let env = WowLuaEnv::new().expect("env");
    let count = env.state().borrow().allied_races.len();
    assert_eq!(count, 10, "expected 10 canonical allied races to be seeded");
    for file_string in CANONICAL_RACE_FILE_STRINGS {
        let race_id = race_id_for(file_string);
        let resolved: bool = env
            .eval(&format!(
                "return C_AlliedRaces.GetRaceInfoByID({race_id}) ~= nil"
            ))
            .unwrap();
        assert!(
            resolved,
            "GetRaceInfoByID({race_id}) [{file_string}] should resolve"
        );
    }
}

#[test]
fn race_info_table_exposes_documented_fields() {
    let env = WowLuaEnv::new().expect("env");
    let lightforged_id = race_id_for("lightforgeddraenei");
    env.exec(&format!(
        "info = C_AlliedRaces.GetRaceInfoByID({lightforged_id})"
    ))
    .unwrap();

    let race_id: f64 = env.eval("return info.raceID").unwrap();
    let male_model_id: f64 = env.eval("return info.maleModelID").unwrap();
    let female_model_id: f64 = env.eval("return info.femaleModelID").unwrap();
    let male_name: String = env.eval("return info.maleName").unwrap();
    let female_name: String = env.eval("return info.femaleName").unwrap();
    let description: String = env.eval("return info.description").unwrap();
    let race_file_string: String = env.eval("return info.raceFileString").unwrap();
    let crest_atlas: String = env.eval("return info.crestAtlas").unwrap();
    let model_background_atlas: String = env.eval("return info.modelBackgroundAtlas").unwrap();
    let achievement_count: f64 = env.eval("return #info.achievementIds").unwrap();

    assert_eq!(race_id, lightforged_id as f64);
    assert_eq!(male_model_id, 82_729.0);
    assert_eq!(female_model_id, 82_730.0);
    assert_eq!(male_name, "Lightforged Draenei");
    assert_eq!(female_name, "Lightforged Draenei");
    assert_eq!(race_file_string, "lightforgeddraenei");
    assert_eq!(crest_atlas, "alliedraces-icon-lightforgeddraenei");
    assert_eq!(
        model_background_atlas,
        "alliedraces-background-lightforgeddraenei"
    );
    assert!(!description.is_empty());
    assert!(achievement_count >= 1.0);
}

#[test]
fn banner_color_supports_color_mixin_get_rgb() {
    let env = WowLuaEnv::new().expect("env");
    let nightborne_id = race_id_for("nightborne");
    env.exec(&format!(
        "info = C_AlliedRaces.GetRaceInfoByID({nightborne_id})"
    ))
    .unwrap();

    let r: f64 = env
        .eval("local r,_,_ = info.bannerColor:GetRGB() return r")
        .unwrap();
    let g: f64 = env
        .eval("local _,g,_ = info.bannerColor:GetRGB() return g")
        .unwrap();
    let b: f64 = env
        .eval("local _,_,b = info.bannerColor:GetRGB() return b")
        .unwrap();
    assert!((r - 0.62).abs() < 1e-3);
    assert!((g - 0.39).abs() < 1e-3);
    assert!((b - 0.85).abs() < 1e-3);
}

#[test]
fn race_info_reflects_state_mutations() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().allied_races.insert(
        9999,
        AlliedRaceInfo {
            race_id: 9999,
            male_model_id: 1,
            female_model_id: 2,
            achievement_ids: vec![100, 200, 300],
            male_name: "TestRaceMale".to_string(),
            female_name: "TestRaceFemale".to_string(),
            description: "A test race".to_string(),
            race_file_string: "testrace".to_string(),
            crest_atlas: "test-crest".to_string(),
            model_background_atlas: "test-background".to_string(),
            banner_color: (0.5, 0.25, 0.75),
            racial_abilities: vec![],
        },
    );

    let male_name: String = env
        .eval("return C_AlliedRaces.GetRaceInfoByID(9999).maleName")
        .unwrap();
    assert_eq!(male_name, "TestRaceMale");

    let third_achievement: f64 = env
        .eval("return C_AlliedRaces.GetRaceInfoByID(9999).achievementIds[3]")
        .unwrap();
    assert_eq!(third_achievement, 300.0);
}

#[test]
fn allied_races_load_data_path_runs_without_errors() {
    let env = WowLuaEnv::new().expect("env");
    let voidelf_id = race_id_for("voidelf");
    env.exec(&format!(
        r#"
        local raceInfo = C_AlliedRaces.GetRaceInfoByID({voidelf_id})
        assert(raceInfo, "raceInfo should not be nil")
        local r, g, b = raceInfo.bannerColor:GetRGB()
        assert(type(r) == "number" and type(g) == "number" and type(b) == "number",
            "bannerColor:GetRGB should return three numbers")
        assert(type(raceInfo.maleModelID) == "number")
        assert(type(raceInfo.femaleModelID) == "number")
        assert(type(raceInfo.achievementIds) == "table")
        assert(type(raceInfo.crestAtlas) == "string")
        assert(type(raceInfo.modelBackgroundAtlas) == "string")
        ALLIED_RACES_LOAD_DATA_OK = true
    "#
    ))
    .unwrap();

    let ok: bool = env
        .eval("return ALLIED_RACES_LOAD_DATA_OK == true")
        .unwrap();
    assert!(
        ok,
        "AlliedRacesFrameMixin:LoadRaceData-style probe should succeed"
    );
}

#[test]
fn racial_abilities_unknown_race_returns_nil() {
    let env = WowLuaEnv::new().expect("env");
    let nil: bool = env
        .eval("return C_AlliedRaces.GetAllRacialAbilitiesFromID(99999) == nil")
        .unwrap();
    assert!(
        nil,
        "GetAllRacialAbilitiesFromID should return nil for unknown ids"
    );
}

#[test]
fn racial_abilities_missing_arg_returns_nothing() {
    let env = WowLuaEnv::new().expect("env");
    let count: f64 = env
        .eval("return select('#', C_AlliedRaces.GetAllRacialAbilitiesFromID())")
        .unwrap();
    assert_eq!(
        count, 0.0,
        "GetAllRacialAbilitiesFromID() with no args returns nothing"
    );
}

#[test]
fn racial_abilities_for_canonical_race_expose_documented_fields() {
    let env = WowLuaEnv::new().expect("env");
    let nightborne_id = race_id_for("nightborne");
    env.exec(&format!(
        "abilities = C_AlliedRaces.GetAllRacialAbilitiesFromID({nightborne_id})"
    ))
    .unwrap();

    let kind: String = env.eval("return type(abilities)").unwrap();
    assert_eq!(kind, "table");

    let count: f64 = env.eval("return #abilities").unwrap();
    assert!(
        count >= 1.0,
        "Nightborne should expose at least one racial ability"
    );

    let first_name: String = env.eval("return abilities[1].name").unwrap();
    let first_description: String = env.eval("return abilities[1].description").unwrap();
    let first_icon: f64 = env.eval("return abilities[1].icon").unwrap();
    assert!(!first_name.is_empty());
    assert!(!first_description.is_empty());
    assert!(first_icon > 0.0, "icon should be a positive fileID");
}

#[test]
fn racial_abilities_reflect_state_mutations() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().allied_races.insert(
        9999,
        AlliedRaceInfo {
            race_id: 9999,
            male_model_id: 1,
            female_model_id: 2,
            achievement_ids: vec![],
            male_name: "TestRaceMale".to_string(),
            female_name: "TestRaceFemale".to_string(),
            description: "A test race".to_string(),
            race_file_string: "testrace".to_string(),
            crest_atlas: "test-crest".to_string(),
            model_background_atlas: "test-background".to_string(),
            banner_color: (0.5, 0.25, 0.75),
            racial_abilities: vec![
                AlliedRaceRacialAbility {
                    name: "Test Ability".to_string(),
                    description: "Does test things.".to_string(),
                    icon: 4_242,
                },
                AlliedRaceRacialAbility {
                    name: "Second".to_string(),
                    description: "Does other test things.".to_string(),
                    icon: 5_353,
                },
            ],
        },
    );

    let count: f64 = env
        .eval("return #C_AlliedRaces.GetAllRacialAbilitiesFromID(9999)")
        .unwrap();
    assert_eq!(count, 2.0);

    let second_name: String = env
        .eval("return C_AlliedRaces.GetAllRacialAbilitiesFromID(9999)[2].name")
        .unwrap();
    assert_eq!(second_name, "Second");

    let second_icon: f64 = env
        .eval("return C_AlliedRaces.GetAllRacialAbilitiesFromID(9999)[2].icon")
        .unwrap();
    assert_eq!(second_icon, 5_353.0);
}

#[test]
fn racial_abilities_data_path_runs_without_errors() {
    let env = WowLuaEnv::new().expect("env");
    let voidelf_id = race_id_for("voidelf");
    env.exec(&format!(
        r#"
        local racialAbilities = C_AlliedRaces.GetAllRacialAbilitiesFromID({voidelf_id})
        assert(racialAbilities, "racialAbilities should not be nil")
        for i, ability in ipairs(racialAbilities) do
            assert(type(ability.name) == "string", "ability.name must be string")
            assert(type(ability.description) == "string", "ability.description must be string")
            assert(type(ability.icon) == "number", "ability.icon must be a fileID number")
        end
        RACIAL_ABILITIES_DATA_OK = #racialAbilities >= 1
    "#
    ))
    .unwrap();

    let ok: bool = env.eval("return RACIAL_ABILITIES_DATA_OK == true").unwrap();
    assert!(
        ok,
        "AlliedRacesFrameMixin:RacialAbilitiesData-style probe should succeed"
    );
}
