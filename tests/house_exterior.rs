use wow_ui_sim::lua_api::WowLuaEnv;

const HOUSE_EXTERIOR_TYPE_SCRIPT: &str = r#"
    local getCurrentHouseExteriorType = C_HouseExterior.GetCurrentHouseExteriorType
    if type(getCurrentHouseExteriorType) ~= "function" then
        return "not_callable", 0, 0, ""
    end

    local returnCount = select('#', getCurrentHouseExteriorType())
    local exteriorType, exteriorTypeName = getCurrentHouseExteriorType()
    return type(exteriorType), returnCount, exteriorType, exteriorTypeName
"#;

const HOUSE_EXTERIOR_SIZE_OPTIONS_SCRIPT: &str = r#"
    local getHouseExteriorSizeOptions = C_HouseExterior.GetHouseExteriorSizeOptions
    if type(getHouseExteriorSizeOptions) ~= "function" then
        return "not_callable"
    end

    local returnCount = select('#', getHouseExteriorSizeOptions())
    local result = getHouseExteriorSizeOptions()
    if returnCount ~= 1 or type(result) ~= "table" then
        return "wrong_return_shape"
    end

    if type(result.selectedSize) ~= "number" or result.selectedSize ~= 3 then
        return "wrong_selected_size"
    end

    if type(result.options) ~= "table" or #result.options ~= 2 then
        return "wrong_options_table"
    end

    local medium = result.options[1]
    if type(medium) ~= "table" or medium.size ~= 3 or medium.name ~= "Medium" then
        return "wrong_medium_option"
    end

    local large = result.options[2]
    if type(large) ~= "table" or large.size ~= 4 or large.name ~= "Large" then
        return "wrong_large_option"
    end

    return "ok"
"#;

const HOUSE_EXTERIOR_TYPE_OPTIONS_SCRIPT: &str = r#"
    local getHouseExteriorTypeOptions = C_HouseExterior.GetHouseExteriorTypeOptions
    if type(getHouseExteriorTypeOptions) ~= "function" then
        return "not_callable"
    end

    local returnCount = select('#', getHouseExteriorTypeOptions())
    local result = getHouseExteriorTypeOptions()
    if returnCount ~= 1 or type(result) ~= "table" then
        return "wrong_return_shape"
    end

    if type(result.selectedExteriorType) ~= "number" or result.selectedExteriorType ~= 1 then
        return "wrong_selected_type"
    end

    if type(result.options) ~= "table" or #result.options ~= 2 then
        return "wrong_options_table"
    end

    local cottage = result.options[1]
    if type(cottage) ~= "table" or cottage.houseExteriorTypeID ~= 1 or cottage.name ~= "Sunspire Cottage" then
        return "wrong_cottage_option"
    end

    local manor = result.options[2]
    if type(manor) ~= "table" or manor.houseExteriorTypeID ~= 2 or manor.name ~= "Sunspire Manor" then
        return "wrong_manor_option"
    end

    return "ok"
"#;

const HOUSE_EXTERIOR_SCRIPT: &str = r#"
    if C_HouseExterior.IsExteriorDecorHidden() ~= false then
        return "wrong_initial_hidden_state"
    end

    if C_HouseExterior.IsAnyDecorAttachedToHouseExterior() ~= true then
        return "wrong_house_exterior_attachment"
    end

    if C_HouseExterior.IsAnyDecorAttachedToDoor() ~= true then
        return "wrong_door_attachment"
    end

    if C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint() ~= true then
        return "wrong_selected_point_attachment"
    end

    if C_HouseExterior.IsAnyDecorAttachedToCoreFixture(Enum.HousingFixtureType.Base) ~= true then
        return "wrong_base_fixture_attachment"
    end

    if C_HouseExterior.IsAnyDecorAttachedToCoreFixture(Enum.HousingFixtureType.Roof) ~= false then
        return "wrong_roof_fixture_attachment"
    end

    if C_HouseExterior.IsAnyDecorAttachedToCoreFixture(99999) ~= false then
        return "wrong_unknown_fixture_attachment"
    end

    C_HouseExterior.SetExteriorDecorHidden(true)
    if C_HouseExterior.IsExteriorDecorHidden() ~= true then
        return "hide_toggle_not_applied"
    end

    C_HouseExterior.SetExteriorDecorHidden(false)
    if C_HouseExterior.IsExteriorDecorHidden() ~= false then
        return "hide_toggle_not_cleared"
    end

    return "ok"
"#;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    modern_fixture::seed(&env);
    env
}

#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
mod modern_fixture {
    use super::WowLuaEnv;
    use wow_ui_sim::c_api::c_housing::catalog::HousingCatalogEntryVariantID;
    use wow_ui_sim::c_api::c_housing::exterior::{
        ExteriorDecorPlacement, ExteriorFixturePoint, ExteriorSizeOption, ExteriorTypeOption,
        HouseExteriorState,
    };

    fn size_option(size: i32, name: &str) -> ExteriorSizeOption {
        ExteriorSizeOption {
            size,
            name: name.into(),
            is_locked: false,
        }
    }

    fn type_option(id: u32, name: &str) -> ExteriorTypeOption {
        ExteriorTypeOption {
            id,
            name: name.into(),
            is_locked: false,
            is_invalid: false,
            reason: String::new(),
        }
    }

    fn attached_placement() -> ExteriorDecorPlacement {
        ExteriorDecorPlacement {
            variant_id: HousingCatalogEntryVariantID {
                record_id: 7101,
                entry_type: 1,
                variant_identifier: 0,
            },
            position: [1.0, 2.0, 3.0],
            fixture_point_owner_hash: Some(11),
        }
    }

    fn exterior_fixture() -> HouseExteriorState {
        HouseExteriorState {
            core_fixture: None,
            entry_door_hovered: false,
            selected_size: Some(3),
            selected_type_id: Some(1),
            size_options: vec![size_option(3, "Medium"), size_option(4, "Large")],
            type_options: vec![
                type_option(1, "Sunspire Cottage"),
                type_option(2, "Sunspire Manor"),
            ],
            selected_fixture_point: Some(ExteriorFixturePoint {
                owner_hash: 11,
                selected_fixture_id: Some(301),
                options: Vec::new(),
                can_remove: false,
            }),
            decor: [("point11".into(), attached_placement())].into(),
        }
    }

    pub(super) fn seed(env: &WowLuaEnv) {
        let mut state = env.state().borrow_mut();
        state.housing.inside_owned_plot = true;
        state.housing.active_house_editor_mode = 6;
        state.housing.exterior = exterior_fixture();
    }
}

#[test]
fn current_house_exterior_type_returns_seeded_type_and_name() {
    let env = env();
    let (value_type, return_count, exterior_type, exterior_type_name): (String, i64, i64, String) =
        env.eval(HOUSE_EXTERIOR_TYPE_SCRIPT)
            .expect("seeded C_HouseExterior type method should be queryable");

    assert_eq!(value_type, "number");
    assert_eq!(return_count, 2);
    assert_eq!(exterior_type, 1);
    assert_eq!(exterior_type_name, "Sunspire Cottage");
}

#[test]
fn house_exterior_size_options_return_seeded_options() {
    let env = env();
    let result: String = env
        .eval(HOUSE_EXTERIOR_SIZE_OPTIONS_SCRIPT)
        .expect("seeded C_HouseExterior size options should be queryable");
    assert_eq!(result, "ok");
}

#[test]
fn house_exterior_type_options_return_seeded_options() {
    let env = env();
    let result: String = env
        .eval(HOUSE_EXTERIOR_TYPE_OPTIONS_SCRIPT)
        .expect("seeded C_HouseExterior type options should be queryable");
    assert_eq!(result, "ok");
}

#[test]
fn house_exterior_attachment_queries_and_hidden_toggle_use_seeded_state() {
    let env = env();
    let result: String = env
        .eval(HOUSE_EXTERIOR_SCRIPT)
        .expect("seeded C_HouseExterior methods should be queryable");
    assert_eq!(result, "ok");
}
