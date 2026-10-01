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
        "#,
    )
    .unwrap();
}
