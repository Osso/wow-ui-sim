//! Tests for table-backed proxy objects (AbbreviateConfig, FunctionContainer,
//! LuaDurationObject, UnitHealPrediction, CurveUtil).
//!
//! Verifies that each type:
//! - Returns a table (not userdata) to Lua
//! - Supports method calls
//! - Supports per-instance field storage
//! - Protects read-only keys from assignment

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forbidden_partition_preserves_native_parent_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:SetSize(120, 40)
        local forbidden = GetForbiddenObjectTable(parent)
        forbidden.privateValue = "hidden"
        assert(parent.privateValue == nil, "partition fields remain private")

        local child = CreateFrame("Frame", nil, forbidden)
        assert(child:GetParent() == parent, "partition identifies the same native parent")
        assert(parent:GetNumChildren() == 1)
        assert(select(1, parent:GetChildren()) == child)

        local setWidth = parent.SetWidth
        setWidth(forbidden, 72)
        assert(parent:GetWidth() == 72, "native methods accept the partition identity")
        child:SetParent(UIParent)
        child:SetParent(forbidden)
        assert(child:GetParent() == parent)
        assert(GetForbiddenObjectTable(parent).privateValue == "hidden")
        "#,
    )
    .expect("forbidden partition remains the same native frame");
}

#[test]
fn forbidden_partition_does_not_turn_ordinary_tables_into_frame_parents() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local parent = CreateFrame("Frame", nil, UIParent)
        for _, ordinary in ipairs({ {}, { __wowPublicObject = parent }, { [0] = 123 } }) do
            local accepted, err = pcall(CreateFrame, "Frame", nil, ordinary)
            assert(not accepted and string.find(err, "parent must be a frame", 1, true))
            accepted, err = pcall(CreateFrame, "Frame", nil, GetForbiddenObjectTable(ordinary))
            assert(not accepted and string.find(err, "parent must be a frame", 1, true))
        end
        assert(parent:GetNumChildren() == 0)
        "#,
    )
    .expect("ordinary tables cannot impersonate native parents");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn forbidden_partition_transfer_preserves_ordinary_frame_fields() {
    let env = WowLuaEnv::new().unwrap();
    env.exec_maybe_secure(r#"
        local function inspect(frame)
            assert(frame.Child and frame.value == 17, 'ordinary frame fields lost crossing environments')
        end
        SwapToGlobalEnvironment()
        local frame = CreateFrame('Frame')
        frame.Child = CreateFrame('Frame', nil, frame)
        frame.value = 17
        inspect(frame)
    "#, true).expect("unpartitioned frames retain their public fields");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn forbidden_partition_aura_provider_creates_children_through_outbound_bridge() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(
                    r#"
                    local container = CreateFrame("AuraContainer", nil, UIParent, "CustomAuraContainerTemplate")
                    local forbidden = GetForbiddenObjectTable(container)
                    local provider = __secureenv.AuraContainerUtil.CreateCustomFrameProvider(forbidden, { batchSize = 1 })
                    local child = provider:AcquireFrame()
                    assert(child:GetParent() == container)
                    assert(provider:GetOwnedFrameCount() == 1)
                    assert(provider:GetOwnedFrame(1) == child)
                    assert(provider:IsFrameActive(child))
                    "#,
                )
                .expect("real aura provider accepts its forbidden container as native parent");
            },
        );
    });
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn forbidden_partition_aura_initializer_keeps_public_and_private_views_isolated() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(
                    r#"
                    local initializedButton, initializedCooldown
                    local publicDisplayCalls = 0
                    local function initialize(button)
                        initializedButton = button
                        assert(button.UpdateAuraDisplay == nil,
                            "public initializer must not receive the private method surface")
                        assert(GetForbiddenObjectTable(button) ~= button,
                            "public and private tables remain distinct")
                        initializedCooldown = CreateFrame("Cooldown", nil, button)
                        button:SetDurationCooldown(initializedCooldown)
                        button.UpdateAuraDisplay = function()
                            publicDisplayCalls = publicDisplayCalls + 1
                            error("secure provider used an addon-supplied display method")
                        end
                    end
                    local container = CreateFrame("AuraContainer", nil, UIParent, "CustomAuraContainerTemplate")
                    local provider = __secureenv.AuraContainerUtil.CreateCustomFrameProvider(
                        GetForbiddenObjectTable(container),
                        { batchSize = 1, initializeFrame = initialize }
                    )
                    local child = provider:AcquireFrame()
                    assert(publicDisplayCalls == 0,
                        "secure display update must not consult the public override")
                    assert(child == initializedButton,
                        "public caller receives the same public view as the initializer")
                    assert(initializedCooldown:GetParent() == child)
                    assert(child:GetParent() == container)
                    local private = GetForbiddenObjectTable(child)
                    assert(private ~= child)
                    assert(type(private.UpdateAuraDisplay) == "function")
                    assert(private.UpdateAuraDisplay ~= child.UpdateAuraDisplay,
                        "public assignment must not replace the private implementation")
                    assert(provider:GetOwnedFrame(1) == child)
                    "#,
                )
                .expect("real aura provider preserves partitions across outbound initialization");
            },
        );
    });
}

// ============================================================================
// AbbreviateConfig
// ============================================================================

#[test]
fn abbreviate_config_is_table_with_methods() {
    let env = WowLuaEnv::new().unwrap();
    let (typ, has_get, has_set): (String, bool, bool) = env
        .eval(
            r#"
            local cfg = CreateAbbreviateConfig({})
            return type(cfg),
                   type(cfg.GetAbbreviateNumberData) == "function",
                   type(cfg.SetAbbreviateNumberData) == "function"
        "#,
        )
        .unwrap();
    assert_eq!(typ, "table");
    assert!(has_get);
    assert!(has_set);
}

#[test]
fn abbreviate_config_get_set_data_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    let (before, after): (bool, f64) = env
        .eval(
            r#"
            local cfg = CreateAbbreviateConfig({})
            local before = cfg:GetAbbreviateNumberData() == nil
            cfg:SetAbbreviateNumberData(42)
            return before, cfg:GetAbbreviateNumberData()
        "#,
        )
        .unwrap();
    assert!(before);
    assert!((after - 42.0).abs() < 0.001);
}

#[test]
fn abbreviate_config_per_instance_fields() {
    let env = WowLuaEnv::new().unwrap();
    let (val_a, val_b): (String, String) = env
        .eval(
            r#"
            local a = CreateAbbreviateConfig({})
            local b = CreateAbbreviateConfig({})
            a.custom = "alpha"
            b.custom = "beta"
            return a.custom, b.custom
        "#,
        )
        .unwrap();
    assert_eq!(val_a, "alpha");
    assert_eq!(val_b, "beta");
}

#[test]
fn abbreviate_config_readonly_keys() {
    let env = WowLuaEnv::new().unwrap();
    let blocked: bool = env
        .eval(
            r#"
            local cfg = CreateAbbreviateConfig({})
            local ok, err = pcall(function() cfg.GetAbbreviateNumberData = 1 end)
            return not ok and err:find("read%-only") ~= nil
        "#,
        )
        .unwrap();
    assert!(blocked);
}

#[test]
fn abbreviate_config_tostring() {
    let env = WowLuaEnv::new().unwrap();
    let has_prefix: bool = env
        .eval(
            r#"
            local cfg = CreateAbbreviateConfig({})
            return tostring(cfg):find("AbbreviateConfig:") ~= nil
        "#,
        )
        .unwrap();
    assert!(has_prefix);
}

// ============================================================================
// FunctionContainer
// ============================================================================

#[test]
fn function_container_is_userdata_with_methods() {
    let env = WowLuaEnv::new().unwrap();
    let (typ, has_cancel, has_invoke): (String, bool, bool) = env
        .eval(
            r#"
            local fc = C_FunctionContainers.CreateCallback(function() end)
            return type(fc),
                   type(fc.Cancel) == "function",
                   type(fc.Invoke) == "function"
        "#,
        )
        .unwrap();
    assert_eq!(typ, "userdata");
    assert!(has_cancel);
    assert!(has_invoke);
}

#[test]
fn function_container_cancel_and_invoke() {
    let env = WowLuaEnv::new().unwrap();
    let (not_cancelled, cancelled, invoked): (bool, bool, bool) = env
        .eval(
            r#"
            local called = false
            local fc = C_FunctionContainers.CreateCallback(function() called = true end)
            local before = not fc:IsCancelled()
            fc:Cancel()
            fc:Invoke()
            return before, fc:IsCancelled(), called
        "#,
        )
        .unwrap();
    assert!(not_cancelled);
    assert!(cancelled);
    assert!(!invoked); // Invoke does nothing after Cancel
}

#[test]
fn function_container_per_instance_fields() {
    let env = WowLuaEnv::new().unwrap();
    let stored: String = env
        .eval(
            r#"
            local fc = C_FunctionContainers.CreateCallback(function() end)
            fc.myField = "hello"
            return fc.myField
        "#,
        )
        .unwrap();
    assert_eq!(stored, "hello");
}

#[test]
fn function_container_readonly_keys() {
    let env = WowLuaEnv::new().unwrap();
    let blocked: bool = env
        .eval(
            r#"
            local fc = C_FunctionContainers.CreateCallback(function() end)
            local ok, err = pcall(function() fc.Cancel = 1 end)
            return not ok and err:find("read%-only") ~= nil
        "#,
        )
        .unwrap();
    assert!(blocked);
}

// ============================================================================
// LuaDurationObject
// ============================================================================

#[test]
fn duration_object_is_table_with_methods() {
    let env = WowLuaEnv::new().unwrap();
    let (typ, has_get, has_zero): (String, bool, bool) = env
        .eval(
            r#"
            local d = C_DurationUtil.CreateDuration()
            return type(d),
                   type(d.GetStartTime) == "function",
                   type(d.IsZero) == "function"
        "#,
        )
        .unwrap();
    assert_eq!(typ, "table");
    assert!(has_get);
    assert!(has_zero);
}

#[test]
fn duration_object_is_zero_by_default() {
    let env = WowLuaEnv::new().unwrap();
    let is_zero: bool = env
        .eval(
            r#"
            local d = C_DurationUtil.CreateDuration()
            return d:IsZero()
        "#,
        )
        .unwrap();
    assert!(is_zero);
}

#[test]
fn duration_object_per_instance_fields() {
    let env = WowLuaEnv::new().unwrap();
    let stored: f64 = env
        .eval(
            r#"
            local d = C_DurationUtil.CreateDuration()
            d.tag = 99
            return d.tag
        "#,
        )
        .unwrap();
    assert!((stored - 99.0).abs() < 0.001);
}

#[test]
fn duration_object_tostring() {
    let env = WowLuaEnv::new().unwrap();
    let has_prefix: bool = env
        .eval(
            r#"
            local d = C_DurationUtil.CreateDuration()
            return tostring(d):find("LuaDurationObject:") ~= nil
        "#,
        )
        .unwrap();
    assert!(has_prefix);
}

// ============================================================================
// UnitHealPredictionCalculator
// ============================================================================

#[test]
fn heal_prediction_is_table_with_methods() {
    let env = WowLuaEnv::new().unwrap();
    let (typ, has_reset, has_get): (String, bool, bool) = env
        .eval(
            r#"
            local hp = CreateUnitHealPredictionCalculator()
            return type(hp),
                   type(hp.Reset) == "function",
                   type(hp.GetIncomingHeals) == "function"
        "#,
        )
        .unwrap();
    assert_eq!(typ, "table");
    assert!(has_reset);
    assert!(has_get);
}

#[test]
fn heal_prediction_set_get_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    let (mode_before, mode_after): (i64, i64) = env
        .eval(
            r#"
            local hp = CreateUnitHealPredictionCalculator()
            local before = hp:GetDamageAbsorbClampMode()
            hp:SetDamageAbsorbClampMode(3)
            return before, hp:GetDamageAbsorbClampMode()
        "#,
        )
        .unwrap();
    assert_eq!(mode_before, 0);
    assert_eq!(mode_after, 3);
}

#[test]
fn heal_prediction_maximum_health_methods_read_predicted_health_max() {
    let env = WowLuaEnv::new().unwrap();
    let max_health: i64 = env
        .eval(
            r#"
            A_Admin.SetPlayerHealth(75000, 200000)
            local hp = CreateUnitHealPredictionCalculator()
            hp:SetMaximumHealthMode(Enum.UnitMaximumHealthMode.Default)
            UnitGetDetailedHealPrediction("player", nil, hp)
            return hp:GetMaximumHealth()
        "#,
        )
        .unwrap();
    assert_eq!(max_health, 200000);
}

#[test]
fn heal_prediction_tostring() {
    let env = WowLuaEnv::new().unwrap();
    let has_prefix: bool = env
        .eval(
            r#"
            local hp = CreateUnitHealPredictionCalculator()
            return tostring(hp):find("UnitHealPredictionCalculator:") ~= nil
        "#,
        )
        .unwrap();
    assert!(has_prefix);
}

// ============================================================================
// CurveUtil (LuaCurveObject + LuaColorCurveObject)
// ============================================================================

#[test]
fn curve_object_is_userdata_with_methods() {
    let env = WowLuaEnv::new().unwrap();
    let (typ, has_add, has_eval): (String, bool, bool) = env
        .eval(
            r#"
            local c = C_CurveUtil.CreateCurve()
            return type(c),
                   type(c.AddPoint) == "function",
                   type(c.Evaluate) == "function"
        "#,
        )
        .unwrap();
    assert_eq!(typ, "userdata");
    assert!(has_add);
    assert!(has_eval);
}

#[test]
fn curve_object_add_and_evaluate() {
    let env = WowLuaEnv::new().unwrap();
    let result: f64 = env
        .eval(
            r#"
            local c = C_CurveUtil.CreateCurve()
            c:AddPoint(0, 10)
            c:AddPoint(1, 20)
            return c:Evaluate(0.5)
        "#,
        )
        .unwrap();
    assert!((result - 15.0).abs() < 0.001);
}

#[test]
fn curve_object_copy_returns_userdata() {
    let env = WowLuaEnv::new().unwrap();
    let (typ, count): (String, i64) = env
        .eval(
            r#"
            local c = C_CurveUtil.CreateCurve()
            c:AddPoint(0, 10)
            c:AddPoint(1, 20)
            local copy = c:Copy()
            return type(copy), copy:GetPointCount()
        "#,
        )
        .unwrap();
    assert_eq!(typ, "userdata");
    assert_eq!(count, 2);
}

#[test]
fn curve_object_per_instance_fields() {
    let env = WowLuaEnv::new().unwrap();
    let stored: String = env
        .eval(
            r#"
            local c = C_CurveUtil.CreateCurve()
            c.label = "test"
            return c.label
        "#,
        )
        .unwrap();
    assert_eq!(stored, "test");
}

#[test]
fn curve_object_tostring() {
    let env = WowLuaEnv::new().unwrap();
    let has_prefix: bool = env
        .eval(
            r#"
            local c = C_CurveUtil.CreateCurve()
            return tostring(c):find("LuaCurveObject:") ~= nil
        "#,
        )
        .unwrap();
    assert!(has_prefix);
}

#[test]
fn color_curve_object_is_userdata() {
    let env = WowLuaEnv::new().unwrap();
    let (typ, has_eval): (String, bool) = env
        .eval(
            r#"
            local cc = C_CurveUtil.CreateColorCurve()
            return type(cc),
                   type(cc.Evaluate) == "function"
        "#,
        )
        .unwrap();
    assert_eq!(typ, "userdata");
    assert!(has_eval);
}

#[test]
fn curve_objects_keep_native_identity_through_securecopy() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local scalar = C_CurveUtil.CreateCurve()
        scalar:AddPoint(0, 10)
        scalar:AddPoint(1, 20)
        local color = C_CurveUtil.CreateColorCurve()
        color:AddPoint(0, CreateColor(1, 0, 0, 0.2))
        color:AddPoint(1, CreateColor(0, 0, 1, 1))
        local options = {scalar=scalar, textColor={curve=color}}
        local copied = securecopy(options)
        assert(copied ~= options and copied.textColor ~= options.textColor)
        assert(copied.scalar == scalar and copied.textColor.curve == color)
        assert(copied.scalar:Evaluate(0.5) == 15)
        local r,g,b,a = copied.textColor.curve:Evaluate(0.5):GetRGBA()
        assert(r == 0.5 and g == 0 and b == 0.5 and math.abs(a - 0.6) < 0.00001)
        local independent = color:Copy()
        assert(independent ~= color and type(independent) == 'userdata')
        color:ClearPoints()
        assert(independent:GetPointCount() == 2 and color:GetPointCount() == 0)
        r,g,b,a = independent:Evaluate(0.5):GetRGBA()
        assert(r == 0.5 and b == 0.5 and math.abs(a - 0.6) < 0.00001)
        assert(not pcall(C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions,
            {textColor={curve={Evaluate=function() end, Copy=function() end}}}))
    "#,
    )
    .expect("securecopy retains curve handles and copied curves retain independent values");
}

#[test]
fn color_curve_copy_returns_userdata() {
    let env = WowLuaEnv::new().unwrap();
    let typ: String = env
        .eval(
            r#"
            local cc = C_CurveUtil.CreateColorCurve()
            cc:AddPoint(0, CreateColor(1, 0, 0, 1))
            local copy = cc:Copy()
            return type(copy)
        "#,
        )
        .unwrap();
    assert_eq!(typ, "userdata");
}
