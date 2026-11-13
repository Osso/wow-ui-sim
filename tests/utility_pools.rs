use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_pool_release_all_behavior(env: &WowLuaEnv) {
    env.exec(
        r#"
        local resets = 0
        local pool = CreateFramePool("Frame", UIParent, nil, function(_, frame)
            resets = resets + 1
            frame.wasReset = true
        end)

        local a = pool:Acquire()
        local b = pool:Acquire()
        assert(a ~= nil and b ~= nil)
        assert(pool:IsActive(a))
        assert(pool:IsActive(b))

        pool:ReleaseAll()

        TEST_POOL_RESETS = resets
        TEST_POOL_A_ACTIVE = pool:IsActive(a)
        TEST_POOL_B_ACTIVE = pool:IsActive(b)
        TEST_POOL_A_RESET = a.wasReset
        TEST_POOL_B_RESET = b.wasReset
        TEST_POOL_ENUM_EMPTY = pool:EnumerateActive()() == nil
        "#,
    )
    .unwrap();
}

fn assert_pool_collection_behavior(env: &WowLuaEnv) {
    env.exec(
        r#"
        local collection = CreateFramePoolCollection()
        local resets = 0
        local pool = collection:CreatePool("Frame", UIParent, nil, function(_, frame)
            resets = resets + 1
            frame.collectionReset = true
        end)

        local frame = collection:Acquire(nil)
        assert(frame ~= nil)
        assert(pool:IsActive(frame))
        assert(collection:IsActive(frame))

        collection:Release(frame)

        TEST_COLLECTION_RESETS = resets
        TEST_COLLECTION_ACTIVE = collection:IsActive(frame)
        TEST_COLLECTION_RESET = frame.collectionReset
        TEST_COLLECTION_ENUM_EMPTY = collection:EnumerateActive()() == nil
        "#,
    )
    .unwrap();
}

#[test]
fn create_frame_pool_supports_release_all_and_reset_callback() {
    let env = WowLuaEnv::new().unwrap();
    assert_pool_release_all_behavior(&env);

    let resets: i32 = env.eval("return TEST_POOL_RESETS").unwrap();
    let a_active: bool = env.eval("return TEST_POOL_A_ACTIVE").unwrap();
    let b_active: bool = env.eval("return TEST_POOL_B_ACTIVE").unwrap();
    let a_reset: bool = env.eval("return TEST_POOL_A_RESET").unwrap();
    let b_reset: bool = env.eval("return TEST_POOL_B_RESET").unwrap();
    let enum_empty: bool = env.eval("return TEST_POOL_ENUM_EMPTY").unwrap();

    assert_eq!(resets, 2);
    assert!(!a_active);
    assert!(!b_active);
    assert!(a_reset);
    assert!(b_reset);
    assert!(enum_empty);
}

#[test]
fn create_frame_pool_collection_can_create_and_release_by_template() {
    let env = WowLuaEnv::new().unwrap();
    assert_pool_collection_behavior(&env);

    let resets: i32 = env.eval("return TEST_COLLECTION_RESETS").unwrap();
    let active: bool = env.eval("return TEST_COLLECTION_ACTIVE").unwrap();
    let reset: bool = env.eval("return TEST_COLLECTION_RESET").unwrap();
    let enum_empty: bool = env.eval("return TEST_COLLECTION_ENUM_EMPTY").unwrap();

    assert_eq!(resets, 1);
    assert!(!active);
    assert!(reset);
    assert!(enum_empty);
}

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
#[test]
fn retail_frame_factory_resets_new_objects_and_rejects_duplicate_release() {
    use crate::common::panel_fixtures::blizzard_ui_dir;

    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let frame_factory_source =
        std::fs::read_to_string(blizzard_ui_dir().join("Blizzard_SharedXMLBase/FrameFactory.lua"))
            .expect("Retail Blizzard SharedXMLBase FrameFactory.lua should be present");
    env.exec(&frame_factory_source)
        .expect("original Blizzard FrameFactory consumer should load");
    env.exec(
        r#"
        CreateTemplateInfoCache = function()
            return { GetTemplateInfo = function() return nil end }
        end
        "#,
    )
    .expect("FrameFactory template-info dependency should be seeded");

    let failures: String = env
        .eval(
            r#"
            local failures = {}
            local resetCount = 0
            local factory = CreateFrameFactory()
            local function reset(pool, frame, isNew)
                resetCount = resetCount + 1
                if isNew and pool:IsActive(frame) then
                    table.insert(failures, "new-object reset ran after activation")
                elseif not isNew and not pool:IsActive(frame) then
                    table.insert(failures, "release reset ran after deactivation")
                end
            end

            local first, isNew = factory:Create(UIParent, "Frame", reset)
            if not isNew or resetCount ~= 1 then
                table.insert(failures, "new object was not reset on acquire")
            end

            factory:Release(first)
            if resetCount ~= 2 then
                table.insert(failures, "release did not reset the active object")
            end

            local duplicateReleaseSucceeded = pcall(function()
                factory:Release(first)
            end)
            if duplicateReleaseSucceeded then
                table.insert(failures, "inactive object was released twice")
            end

            local reused, reusedIsNew = factory:Create(UIParent, "Frame", reset)
            local second, secondIsNew = factory:Create(UIParent, "Frame", reset)
            if reused ~= first or reusedIsNew then
                table.insert(failures, "released object was not reused")
            end
            if second == reused or not secondIsNew then
                table.insert(failures, "duplicate release made one object available twice")
            end
            if resetCount ~= 3 then
                table.insert(failures, "unexpected reset callback count")
            end

            return table.concat(failures, "\n")
            "#,
        )
        .expect("original Blizzard FrameFactory behavior probe should run");

    assert!(
        failures.is_empty(),
        "pool lifecycle mismatches:\n{failures}"
    );
}
