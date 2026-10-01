//! Populated Area POI publication through the registered runtime namespace.

use wow_ui_sim::lua_api::WowLuaEnv;

fn query_populated_poi_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local api = C_AreaPoiInfo
        local mapped = api.GetAreaPOIInfo(84, 7000)
        local unbound = api.GetAreaPOIInfo(nil, 7000)
        assert(type(mapped) == "table", "matching map must return a populated POI")
        assert(type(unbound) == "table", "nil map must return a populated POI")
        for _, poi in ipairs({mapped, unbound}) do
            assert(poi.areaPoiID == 7000)
            assert(poi.uiMapID == 84)
            assert(poi.name == "Stormwind Portal Room")
            assert(poi.description == "Portals to every capital city.")
            assert(poi.atlasName == "Mage-Portal")
            local x, y = poi.position:GetXY()
            assert(x == 0.52 and y == 0.38)
            assert(poi.isCurrentEvent == false and poi.shouldGlow == false)
        end
        assert(select('#', api.GetAreaPOIInfo(13, 7000)) == 0)
        assert(select('#', api.GetAreaPOIInfo(84, 91237)) == 0)
        "#,
    )
    .unwrap();
    {
        let mut state = env.state().borrow_mut();
        for (id, name, position, is_suppressible, is_locked) in [
            (91237, "Suppressible test POI", (0.17, 0.29), true, false),
            (91409, "Locked test POI", (0.63, 0.81), false, true),
        ] {
            let mut poi = state.area_pois.get(&7000).unwrap().clone();
            poi.area_poi_id = id;
            poi.name = name.into();
            poi.ui_map_id = Some(88007);
            poi.position = position;
            poi.is_suppressible = is_suppressible;
            poi.is_locked = is_locked;
            state.area_pois.insert(id, poi);
        }
    }
    env.exec(
        r#"
        local api = C_AreaPoiInfo
        for _, expected in ipairs({
            {91237, "Suppressible test POI", 0.17, 0.29},
            {91409, "Locked test POI", 0.63, 0.81},
        }) do
            local mapped = api.GetAreaPOIInfo(88007, expected[1])
            local unbound = api.GetAreaPOIInfo(nil, expected[1])
            assert(select('#', api.GetAreaPOIInfo(88007, expected[1])) == 1)
            assert(select('#', api.GetAreaPOIInfo(nil, expected[1])) == 1)
            for _, poi in ipairs({mapped, unbound}) do
                assert(poi.areaPoiID == expected[1] and poi.name == expected[2])
                assert(poi.uiMapID == 88007)
                local x, y = poi.position:GetXY()
                assert(x == expected[3] and y == expected[4])
            end
            assert(select('#', api.GetAreaPOIInfo(84, expected[1])) == 0)
        end
        local ids = api.GetAreaPOIForMap(88007)
        assert(#ids == 2 and ids[1] == 91237 and ids[2] == 91409)
        "#,
    )
    .unwrap();
    env
}

#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
mod patch_publication {
    use super::query_populated_poi_env;

    #[test]
    fn populated_poi_publishes_default_suppressible_boolean() {
        let env = query_populated_poi_env();
        env.exec(
            r#"
            local poi = C_AreaPoiInfo.GetAreaPOIInfo(84, 7000)
            assert(type(poi.isSuppressible) == "boolean", "isSuppressible must be a boolean")
            assert(poi.isSuppressible == false, "existing POI uses inferred false suppression default")
            for _, id in ipairs({91237, 91409}) do
                local mapped = C_AreaPoiInfo.GetAreaPOIInfo(88007, id)
                local unbound = C_AreaPoiInfo.GetAreaPOIInfo(nil, id)
                for _, row in ipairs({mapped, unbound}) do
                    assert(type(row.isSuppressible) == "boolean")
                    assert(row.isSuppressible == (id == 91237))
                end
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn populated_poi_publishes_default_locked_boolean() {
        let env = query_populated_poi_env();
        env.exec(
            r#"
            local poi = C_AreaPoiInfo.GetAreaPOIInfo(84, 7000)
            assert(type(poi.isLocked) == "boolean", "isLocked must be a boolean")
            assert(poi.isLocked == false, "existing POI uses documented false locking default")
            for _, id in ipairs({91237, 91409}) do
                local mapped = C_AreaPoiInfo.GetAreaPOIInfo(88007, id)
                local unbound = C_AreaPoiInfo.GetAreaPOIInfo(nil, id)
                for _, row in ipairs({mapped, unbound}) do
                    assert(type(row.isLocked) == "boolean")
                    assert(row.isLocked == (id == 91409))
                end
            end
            "#,
        )
        .unwrap();
    }
}

#[cfg(not(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
)))]
#[test]
fn legacy_populated_poi_does_not_publish_patch_fields() {
    let env = query_populated_poi_env();
    env.exec(
        r#"
        local poi = C_AreaPoiInfo.GetAreaPOIInfo(84, 7000)
        assert(poi.isSuppressible == nil, "preserve legacy suppression field absence")
        assert(poi.isLocked == nil, "preserve legacy locking field absence")
        for _, id in ipairs({91237, 91409}) do
            local mapped = C_AreaPoiInfo.GetAreaPOIInfo(88007, id)
            local unbound = C_AreaPoiInfo.GetAreaPOIInfo(nil, id)
            for _, row in ipairs({mapped, unbound}) do
                assert(row.isSuppressible == nil and row.isLocked == nil)
            end
        end
        "#,
    )
    .unwrap();
}
