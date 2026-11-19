use wow_ui_sim::lua_api::WowLuaEnv;

fn pool_test_env() -> WowLuaEnv {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        WowLuaEnv::new().expect("Lua environment should initialize")
    } else {
        crate::common::env_with_shared_xml()
    }
}

const CACHED_POOL_CONSTRUCTORS_PROBE: &str = r#"
    for _, name in ipairs({"CreateFramePool", "CreateTexturePool", "CreateFontStringPool", "CreateFramePoolCollection", "CreateFrameFactory"}) do
        assert(type(_G[name]) == "function", name .. " must come from the cached provider")
    end

    local resetCalls = 0
    local pool = CreateFramePool("Frame", UIParent, nil, function(_, _, isNew)
        resetCalls = resetCalls + (isNew and 1 or 10)
    end)
    local first, isNew = pool:Acquire()
    assert(first and isNew and pool:IsActive(first) and pool:DoesObjectBelongToPool(first))
    pool:Release(first)
    assert(not pool:IsActive(first) and resetCalls == 11)
    local reused, reusedIsNew = pool:Acquire()
    assert(reused == first and not reusedIsNew and pool:IsActive(reused))

    local texture = CreateTexturePool(UIParent, "ARTWORK"):Acquire()
    local fontString = CreateFontStringPool(UIParent, "ARTWORK"):Acquire()
    assert(texture and fontString)

    local collection = CreateFramePoolCollection()
    local collectedPool = collection:CreatePool("Frame", UIParent, "")
    local collected = collectedPool:Acquire()
    assert(collected and collectedPool:IsActive(collected) and collection:GetNumActive() == 1)
    collection:ReleaseAll()
    assert(not collectedPool:IsActive(collected) and collection:GetNumActive() == 0)

    CreateTemplateInfoCache = function()
        return { GetTemplateInfo = function() return nil end }
    end
    local factory = CreateFrameFactory()
    local factoryFrame = factory:Create(UIParent, "Frame")
    assert(factoryFrame and factory:GetNumActive() == 1)
    factory:ReleaseAll()
    assert(factory:GetNumActive() == 0)
    return "ok"
"#;

// VERIFIED — Blizzard_SharedXMLBase/Pools.lua:65,102,136 resets new and released objects.
fn assert_pool_release_all_behavior(env: &WowLuaEnv) {
    env.exec(
        r#"
        local newResets, releaseResets = 0, 0
        local pool = CreateFramePool("Frame", UIParent, nil, function(_, frame, isNew)
            if isNew then
                newResets = newResets + 1
            else
                releaseResets = releaseResets + 1
            end
            frame.wasReset = true
        end)

        local a = pool:Acquire()
        local b = pool:Acquire()
        assert(a ~= nil and b ~= nil)
        assert(pool:IsActive(a))
        assert(pool:IsActive(b))

        pool:ReleaseAll()

        TEST_POOL_NEW_RESETS = newResets
        TEST_POOL_RELEASE_RESETS = releaseResets
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
        local newResets, releaseResets = 0, 0
        local pool = collection:CreatePool("Frame", UIParent, nil, function(_, frame, isNew)
            if isNew then
                newResets = newResets + 1
            else
                releaseResets = releaseResets + 1
            end
            frame.collectionReset = true
        end)

        local frame = collection:Acquire(nil)
        assert(frame ~= nil)
        assert(pool:IsActive(frame))
        assert(collection:IsActive(frame))

        collection:Release(frame)

        TEST_COLLECTION_NEW_RESETS = newResets
        TEST_COLLECTION_RELEASE_RESETS = releaseResets
        TEST_COLLECTION_ACTIVE = collection:IsActive(frame)
        TEST_COLLECTION_RESET = frame.collectionReset
        TEST_COLLECTION_ENUM_EMPTY = collection:EnumerateActive()() == nil
        "#,
    )
    .unwrap();
}

#[test]
fn create_frame_pool_supports_release_all_and_reset_callback() {
    let env = pool_test_env();
    assert_pool_release_all_behavior(&env);

    let resets: (i32, i32) = env
        .eval("return TEST_POOL_NEW_RESETS, TEST_POOL_RELEASE_RESETS")
        .unwrap();
    let a_active: bool = env.eval("return TEST_POOL_A_ACTIVE").unwrap();
    let b_active: bool = env.eval("return TEST_POOL_B_ACTIVE").unwrap();
    let a_reset: bool = env.eval("return TEST_POOL_A_RESET").unwrap();
    let b_reset: bool = env.eval("return TEST_POOL_B_RESET").unwrap();
    let enum_empty: bool = env.eval("return TEST_POOL_ENUM_EMPTY").unwrap();

    assert_eq!(resets, (2, 2));
    assert!(!a_active);
    assert!(!b_active);
    assert!(a_reset);
    assert!(b_reset);
    assert!(enum_empty);
}

#[test]
fn create_frame_pool_collection_can_create_and_release_by_template() {
    let env = pool_test_env();
    assert_pool_collection_behavior(&env);

    let resets: (i32, i32) = env
        .eval("return TEST_COLLECTION_NEW_RESETS, TEST_COLLECTION_RELEASE_RESETS")
        .unwrap();
    let active: bool = env.eval("return TEST_COLLECTION_ACTIVE").unwrap();
    let reset: bool = env.eval("return TEST_COLLECTION_RESET").unwrap();
    let enum_empty: bool = env.eval("return TEST_COLLECTION_ENUM_EMPTY").unwrap();

    assert_eq!(resets, (1, 1));
    assert!(!active);
    assert!(reset);
    assert!(enum_empty);
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn cached_pool_constructors_preserve_original_behavior() {
    let env = crate::common::env_with_shared_xml();
    let result: String = env
        .eval(CACHED_POOL_CONSTRUCTORS_PROBE)
        .expect("cached pool provider behavior probe should run");
    assert_eq!(result, "ok");
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
