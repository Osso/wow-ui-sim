//! Exact enum publication only; no FragmentID consumer semantics.

use crate::lua_api::WowLuaEnv;

fn assert_fragment_publication(env: &WowLuaEnv, expected: &str, count: usize) {
    env.exec(&format!(
        r#"
        local expected = {{ {expected} }}
        assert(type(Enum.FragmentID) == "table")
        for name, value in pairs(expected) do
            assert(Enum.FragmentID[name] == value,
                name .. ": expected " .. value .. ", got " .. tostring(Enum.FragmentID[name]))
        end
        local count = 0
        for name in pairs(Enum.FragmentID) do
            assert(expected[name] ~= nil, "unexpected member: " .. name)
            count = count + 1
        end
        assert(count == {count}, "member count mismatch")
        "#,
    ))
    .expect("exact FragmentID publication");
}

fn assert_fragment_metadata(env: &WowLuaEnv, count: i32) {
    let actual: (i32, i32, i32, i32) = env
        .eval(
            r#"
            local count = 0
            for _ in pairs(Enum.FragmentIDMeta) do count = count + 1 end
            return Enum.FragmentIDMeta.MinValue, Enum.FragmentIDMeta.MaxValue,
                Enum.FragmentIDMeta.NumValues, count
            "#,
        )
        .expect("FragmentID metadata");
    assert_eq!(actual, (0, 255, count, 3));
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_fragment_id_enums_ptr() {
    // Frozen semantic fixture: Gethe a89e9d0c -> 49b69918, not runtime enum data.
    let register: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../data/patch-api/sources/12.1.5-register.json"
    ))
    .unwrap();
    let target = &register["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["symbol"] == "Enum.FragmentID")
        .unwrap()["after"];
    let fields = target["Fields"].as_array().unwrap();
    assert_eq!(fields.len(), 78);
    let expected = fields
        .iter()
        .map(|field| {
            format!(
                "{} = {},",
                field["Name"].as_str().unwrap(),
                field["EnumValue"].as_i64().unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let env = WowLuaEnv::new().unwrap();
    assert_fragment_publication(&env, &expected, 78);
    assert_fragment_metadata(&env, 78);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_fragment_publication(&env, &expected, 78);
    assert_fragment_metadata(&env, 78);
}

#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_fragment_id_enums_preserves_retail() {
    let env = WowLuaEnv::new().unwrap();
    // Existing retail publication, including its two bootstrap-added members.
    let expected = r#"
        MirrorState=0, EntityPosition=1, CgObject=2, JamDispatcher=3,
        HeartbeatData=4, FTransportLink=5, ClientObservablesData=6,
        TimerQueues=7, TransferSuspensionData=8, DeferredMessages=9,
        FPersistableJamServers=10, FPlayerJamServers=11, FLootObjectList=12,
        FPlayerOwnershipLink=13, FUnitAreaTriggerLink=14, Actor=15,
        FPhaseShiftData=16, FVendor=17, MirroredObjectC=18, FMeshObjectData=19,
        FHousingDecor=20, FHousingRoom=21, FHousingRoomComponentMesh=22,
        FHousingPlayerHouse=23, FHousingHouseDecorSet=24, FHousingHouseRoomSet=25,
        FHousingDecorProxy=26, FJamHousingCornerstone=27, FHousingDecorActor=28,
        FHousingPlotAreaTrigger=29, FNeighborhoodMirrorData=30,
        FMirroredPositionData=31, FPlayerHouseInfo=32, FHousingStorageMirrorData=33,
        FHousingFixture=34, FHousingHouseFixtureSet=35, Timer=36,
        FPlayerInitiativeInfo=37, FNeighborhoodStateData=38, FUnitAIGroupLink=39,
        TagItem=200, TagContainer=201, TagAzeriteEmpoweredItem=202,
        TagAzeriteItem=203, TagUnit=204, TagPlayer=205, TagGameObject=206,
        TagDynamicObject=207, TagCorpse=208, TagAreaTrigger=209,
        TagSceneObject=210, TagConversation=211, TagAIGroup=212, TagScenario=213,
        TagLootObject=214, TagActivePlayer=215, TagActiveClientS=216,
        TagActiveObjectC=217, TagVisibleObjectC=218, TagUnitVehicle=219,
        TagHousingRoom=220, TagMeshObject=221, TagHousingSubdivisionObject=222,
        TagHousingPoolObject=223, TagHouseExteriorPiece=224, TagHouseExteriorRoot=225,
        TestFragment_1=250, TestFragment_2=251, TestFragment_3=252,
        TestFragment_4=253, TestFragment_5=254, ReservedArchetypeSeparator=255,
        FMapObject=256, FWorldStateListenerData=257,
    "#;
    assert_fragment_publication(&env, expected, 74);
    assert_fragment_metadata(&env, 72);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_fragment_publication(&env, expected, 74);
    assert_fragment_metadata(&env, 72);
}
