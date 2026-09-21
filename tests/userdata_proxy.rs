//! Tests for table-backed proxy objects (AbbreviateConfig, FunctionContainer,
//! LuaDurationObject, UnitHealPrediction, CurveUtil).
//!
//! Verifies that each type:
//! - Returns a table (not userdata) to Lua
//! - Supports method calls
//! - Supports per-instance field storage
//! - Protects read-only keys from assignment

use wow_ui_sim::lua_api::WowLuaEnv;

// Explicit simulator projection policy, not native identity or security conformance.
#[test]
fn forbidden_partition_interns_isolated_private_views() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = CreateFrame("Frame")
        local second = CreateFrame("Frame")
        local firstPrivate = GetForbiddenObjectTable(first)
        local secondPrivate = GetForbiddenObjectTable(second)
        assert(firstPrivate ~= first and secondPrivate ~= second,
            "public and private projections remain distinct")
        assert(firstPrivate ~= secondPrivate, "each frame owns a distinct private view")
        assert(GetForbiddenObjectTable(first) == firstPrivate,
            "repeated public projection returns the interned view")
        assert(GetForbiddenObjectTable(firstPrivate) == firstPrivate,
            "private projection is idempotent")

        firstPrivate.value = "first-private"
        secondPrivate.value = "second-private"
        firstPrivate.readValue = function(self) return self.value end
        secondPrivate.readValue = function(self) return self.value end
        assert(first.value == nil and second.value == nil)
        assert(first.readValue == nil and second.readValue == nil)

        first.value = "first-public"
        first.readValue = function(self) return "public:" .. self.value end
        assert(first:readValue() == "public:first-public")
        assert(firstPrivate:readValue() == "first-private",
            "public assignments cannot replace private fields or methods")
        assert(secondPrivate:readValue() == "second-private",
            "private fields remain independent between frames")
        firstPrivate.value = "first-updated"
        assert(GetForbiddenObjectTable(first):readValue() == "first-updated")
        assert(secondPrivate:readValue() == "second-private")
        assert(first:readValue() == "public:first-public")
        "#,
    )
    .expect("simulator private projections are interned and isolate per-frame fields");
}

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

#[cfg(feature = "forbidden-aspects")]
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

#[test]
fn forbidden_partition_secure_xml_delegate_preserves_argument_shape_and_isolation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(&format!(
        "projectDelegateArguments = {}",
        cfg!(feature = "forbidden-aspects")
    ))
    .unwrap();
    env.exec(
        r#"
        local owner = CreateFrame('Frame')
        local child = CreateFrame('Cooldown', nil, owner)
        local texture = owner:CreateTexture()
        local label = owner:CreateFontString()
        local privateOwner = GetForbiddenObjectTable(owner)
        local privateChild = GetForbiddenObjectTable(child)
        child.value = 'public'
        privateChild.value = 'private'
        local nested = { child = child }
        local spoof = { __wowPublicObject = child, [0] = rawget(child, 0) }
        local incoming
        local expectedPrivateValue = 'private'
        local mixin = { Accept = function(self, ...)
            assert(self == privateOwner)
            assert(select('#', ...) == 9, 'nil slots and trailing nil must survive')
            local c, empty, t, f, number, flag, tableArg, fake, trailing = ...
            assert(empty == nil and trailing == nil and number == 23 and flag == false)
            assert(tableArg == nested and tableArg.child == child, 'no recursive projection')
            assert(fake == spoof, 'ordinary identity-shaped tables must remain unchanged')
            if projectDelegateArguments then
                assert(c == privateChild and c.value == expectedPrivateValue)
                assert(t == GetForbiddenObjectTable(texture))
                assert(f == GetForbiddenObjectTable(label))
                c.value = 'changed-private'
                expectedPrivateValue = 'changed-private'
            else
                assert(c == child and c.value == 'public', 'legacy XML behavior must remain')
                assert(t == texture and f == label)
            end
            incoming = c
            return 'accepted', nil, 23
        end, Empty = function(self, ...)
            assert(self == privateOwner and select('#', ...) == 0)
            return 'empty'
        end }
        __wow_apply_xml_mixin(owner, mixin, 'public', 'forbidden', true)
        assert(owner:Empty() == 'empty')
        local result, empty, number = owner:Accept(child, nil, texture, label, 23, false, nested, spoof, nil)
        assert(result == 'accepted' and empty == nil and number == 23)
        assert(child.value == 'public', 'delegate must not write public instance fields')
        assert(privateChild.value == (projectDelegateArguments and 'changed-private' or 'private'))
        if projectDelegateArguments then
            owner:Accept(privateChild, nil, GetForbiddenObjectTable(texture), GetForbiddenObjectTable(label), 23, false, nested, spoof, nil)
            assert(incoming == privateChild, 'canonical private arguments stay interned')
        end
        "#,
    )
    .expect("secure XML delegates preserve direct argument shape and partition isolation");
}

#[test]
fn forbidden_partition_nonsecure_xml_delegate_keeps_public_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local owner = CreateFrame('Frame')
        local child = CreateFrame('Frame', nil, owner)
        child.value = 17
        local seen
        __wow_apply_xml_mixin(owner, { Accept = function(self, argument)
            assert(self == GetForbiddenObjectTable(owner))
            assert(argument == child and argument.value == 17)
            seen = true
        end }, 'public', 'forbidden', false)
        owner:Accept(child)
        assert(seen)
        "#,
    )
    .expect("receiver-only XML delegates retain public argument fields");
}

#[cfg(feature = "forbidden-aspects")]
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

#[cfg(feature = "forbidden-aspects")]
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
fn heal_prediction_heal_absorb_clamp_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local hp = CreateUnitHealPredictionCalculator()
        local modes = Enum.UnitHealAbsorbClampMode
        assert(select('#', hp:SetHealAbsorbClampMode(modes.MaximumHealth)) == 0)
        assert(select('#', hp:GetHealAbsorbClampMode()) == 1)
        local maximum = hp:GetHealAbsorbClampMode()
        assert(type(maximum) == 'number' and maximum == modes.MaximumHealth)

        assert(select('#', hp:SetHealAbsorbClampMode(modes.CurrentHealth)) == 0)
        assert(select('#', hp:GetHealAbsorbClampMode()) == 1)
        local current = hp:GetHealAbsorbClampMode()
        assert(type(current) == 'number' and current == modes.CurrentHealth)
        "#,
    )
    .expect("configured heal absorb clamp modes roundtrip with exact return counts");
}

#[test]
fn heal_prediction_heal_absorb_clamp_instances_are_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = CreateUnitHealPredictionCalculator()
        local second = CreateUnitHealPredictionCalculator()
        local modes = Enum.UnitHealAbsorbClampMode
        first:SetHealAbsorbClampMode(modes.MaximumHealth)
        second:SetHealAbsorbClampMode(modes.CurrentHealth)
        assert(first:GetHealAbsorbClampMode() == modes.MaximumHealth)
        assert(second:GetHealAbsorbClampMode() == modes.CurrentHealth)

        first:SetHealAbsorbClampMode(modes.CurrentHealth)
        second:SetHealAbsorbClampMode(modes.MaximumHealth)
        assert(first:GetHealAbsorbClampMode() == modes.CurrentHealth)
        assert(second:GetHealAbsorbClampMode() == modes.MaximumHealth)
        "#,
    )
    .expect("each calculator retains its independently configured heal absorb clamp mode");
}

#[test]
fn heal_prediction_configuration_heal_absorb_mode_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local hp = CreateUnitHealPredictionCalculator()
        local modes = Enum.UnitHealAbsorbMode
        for _, mode in ipairs({modes.Total, modes.ReducedByIncomingHeals, modes.Total}) do
            assert(select('#', hp:SetHealAbsorbMode(mode)) == 0)
            assert(select('#', hp:GetHealAbsorbMode()) == 1)
            local actual = hp:GetHealAbsorbMode()
            assert(type(actual) == 'number' and actual == mode)
        end
        "#,
    )
    .expect("explicit heal absorb processing modes roundtrip with exact return counts");
}

#[test]
fn heal_prediction_configuration_heal_absorb_mode_instances_are_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = CreateUnitHealPredictionCalculator()
        local second = CreateUnitHealPredictionCalculator()
        local modes = Enum.UnitHealAbsorbMode
        first:SetHealAbsorbMode(modes.Total)
        second:SetHealAbsorbMode(modes.ReducedByIncomingHeals)
        assert(first:GetHealAbsorbMode() == modes.Total)
        assert(second:GetHealAbsorbMode() == modes.ReducedByIncomingHeals)

        first:SetHealAbsorbMode(modes.ReducedByIncomingHeals)
        second:SetHealAbsorbMode(modes.Total)
        assert(first:GetHealAbsorbMode() == modes.ReducedByIncomingHeals)
        assert(second:GetHealAbsorbMode() == modes.Total)
        "#,
    )
    .expect("heal absorb processing mode changes remain local to each calculator");
}

#[test]
fn heal_prediction_configuration_incoming_heal_clamp_mode_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local hp = CreateUnitHealPredictionCalculator()
        local modes = Enum.UnitIncomingHealClampMode
        for _, mode in ipairs({modes.MaximumHealth, modes.MissingHealth, modes.MaximumHealth}) do
            assert(select('#', hp:SetIncomingHealClampMode(mode)) == 0)
            assert(select('#', hp:GetIncomingHealClampMode()) == 1)
            local actual = hp:GetIncomingHealClampMode()
            assert(type(actual) == 'number' and actual == mode)
        end
        "#,
    )
    .expect("explicit incoming heal clamp modes roundtrip with exact return counts");
}

#[test]
fn heal_prediction_configuration_incoming_heal_clamp_mode_instances_are_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = CreateUnitHealPredictionCalculator()
        local second = CreateUnitHealPredictionCalculator()
        local modes = Enum.UnitIncomingHealClampMode
        first:SetIncomingHealClampMode(modes.MaximumHealth)
        second:SetIncomingHealClampMode(modes.MissingHealth)
        assert(first:GetIncomingHealClampMode() == modes.MaximumHealth)
        assert(second:GetIncomingHealClampMode() == modes.MissingHealth)

        first:SetIncomingHealClampMode(modes.MissingHealth)
        second:SetIncomingHealClampMode(modes.MaximumHealth)
        assert(first:GetIncomingHealClampMode() == modes.MissingHealth)
        assert(second:GetIncomingHealClampMode() == modes.MaximumHealth)
        "#,
    )
    .expect("incoming heal clamp mode changes remain local to each calculator");
}

#[test]
fn heal_prediction_configuration_incoming_heal_overflow_percent_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local hp = CreateUnitHealPredictionCalculator()
        -- Characterize stored numeric state, not native accepted ranges.
        for _, percent in ipairs({1.5, 0, 0.5, 1.5, 0}) do
            assert(select('#', hp:SetIncomingHealOverflowPercent(percent)) == 0)
            assert(select('#', hp:GetIncomingHealOverflowPercent()) == 1)
            local actual = hp:GetIncomingHealOverflowPercent()
            assert(type(actual) == 'number' and actual == percent)
        end
        "#,
    )
    .expect("explicit overflow percentages including zero roundtrip with exact return counts");
}

#[test]
fn heal_prediction_configuration_incoming_heal_overflow_percent_instances_are_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = CreateUnitHealPredictionCalculator()
        local second = CreateUnitHealPredictionCalculator()
        first:SetIncomingHealOverflowPercent(1.5)
        second:SetIncomingHealOverflowPercent(0)
        assert(first:GetIncomingHealOverflowPercent() == 1.5)
        assert(second:GetIncomingHealOverflowPercent() == 0)

        first:SetIncomingHealOverflowPercent(0.5)
        second:SetIncomingHealOverflowPercent(1.5)
        assert(first:GetIncomingHealOverflowPercent() == 0.5)
        assert(second:GetIncomingHealOverflowPercent() == 1.5)
        "#,
    )
    .expect("stored overflow percentage changes remain local to each calculator");
}

#[test]
fn heal_prediction_set_predicted_values_stores_all_fields() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local calculator = CreateUnitHealPredictionCalculator()
        local values = {
            health = 137, healthMax = 991,
            totalDamageAbsorbs = 23, totalHealAbsorbs = 41,
            totalIncomingHeals = 79, totalIncomingHealsFromHealer = 53,
        }
        assert(select('#', calculator:SetPredictedValues(values)) == 0)
        local stored = calculator:GetPredictedValues()
        for field, expected in pairs(values) do
            assert(stored[field] == expected, field)
        end
        assert(calculator:GetCurrentHealth() == 137)
        assert(calculator:GetMaximumHealth() == 991)
        "#,
    )
    .expect("predicted values store all six fields with zero setter returns");
}

#[test]
fn heal_prediction_set_predicted_values_snapshots_input_and_output() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- Snapshot isolation is a simulator policy, not a native identity claim.
        local calculator = CreateUnitHealPredictionCalculator()
        local values = {
            health = 137, healthMax = 991,
            totalDamageAbsorbs = 23, totalHealAbsorbs = 41,
            totalIncomingHeals = 79, totalIncomingHealsFromHealer = 53,
        }
        calculator:SetPredictedValues(values)
        for field, value in pairs(values) do
            values[field] = value + 1000
        end
        local output = calculator:GetPredictedValues()
        assert(output.health == 137 and output.healthMax == 991)
        assert(output.totalDamageAbsorbs == 23 and output.totalHealAbsorbs == 41)
        assert(output.totalIncomingHeals == 79 and output.totalIncomingHealsFromHealer == 53)
        for field, value in pairs(output) do
            output[field] = value + 2000
        end
        local stored = calculator:GetPredictedValues()
        assert(stored.health == 137 and stored.healthMax == 991)
        assert(stored.totalDamageAbsorbs == 23 and stored.totalHealAbsorbs == 41)
        assert(stored.totalIncomingHeals == 79 and stored.totalIncomingHealsFromHealer == 53)
        assert(calculator:GetCurrentHealth() == 137)
        assert(calculator:GetMaximumHealth() == 991)
        "#,
    )
    .expect("input and output mutations leave stored predicted values unchanged");
}

#[test]
fn heal_prediction_set_predicted_values_replaces_without_changing_other_instance() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = CreateUnitHealPredictionCalculator()
        local second = CreateUnitHealPredictionCalculator()
        local original = {
            health = 137, healthMax = 991,
            totalDamageAbsorbs = 23, totalHealAbsorbs = 41,
            totalIncomingHeals = 79, totalIncomingHealsFromHealer = 53,
        }
        first:SetPredictedValues(original)
        second:SetPredictedValues(original)
        local replacement = {
            health = 251, healthMax = 1201,
            totalDamageAbsorbs = 37, totalHealAbsorbs = 61,
            totalIncomingHeals = 109, totalIncomingHealsFromHealer = 83,
        }
        assert(select('#', first:SetPredictedValues(replacement)) == 0)
        local firstValues = first:GetPredictedValues()
        local secondValues = second:GetPredictedValues()
        for field, expected in pairs(replacement) do
            assert(firstValues[field] == expected, field)
            assert(secondValues[field] == original[field], field)
        end
        assert(first:GetCurrentHealth() == 251 and first:GetMaximumHealth() == 1201)
        assert(second:GetCurrentHealth() == 137 and second:GetMaximumHealth() == 991)
        "#,
    )
    .expect("replacement changes every predicted field only on its calculator");
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

// Ordinary-value selection only; fresh color copies are simulator policy, not native identity proof.
#[test]
#[cfg(feature = "retail-12-0-0")]
fn curve_boolean_colors_select_fresh_copies_without_mutating_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local yes = CreateColor(0.125, 0.25, 0.5, 0.75)
        local no = CreateColor(0.875, 0.625, 0.375, 0.25)
        local function rgba(color, r, g, b, a)
            local cr, cg, cb, ca = color:GetRGBA()
            assert(cr == r and cg == g and cb == b and ca == a)
        end
        local selectedYes = C_CurveUtil.EvaluateColorFromBoolean(true, yes, no)
        local selectedNo = C_CurveUtil.EvaluateColorFromBoolean(false, yes, no)
        rgba(selectedYes, 0.125, 0.25, 0.5, 0.75)
        rgba(selectedNo, 0.875, 0.625, 0.375, 0.25)
        assert(selectedYes ~= yes and selectedYes ~= no and selectedYes ~= selectedNo)
        assert(selectedNo ~= yes and selectedNo ~= no)
        local repeated = C_CurveUtil.EvaluateColorFromBoolean(true, yes, no)
        assert(repeated ~= selectedYes and repeated ~= yes and repeated ~= no)
        selectedYes.r = 1
        selectedNo.a = 0
        rgba(yes, 0.125, 0.25, 0.5, 0.75)
        rgba(no, 0.875, 0.625, 0.375, 0.25)
        yes.g = 1
        no.b = 0
        rgba(repeated, 0.125, 0.25, 0.5, 0.75)
        rgba(selectedYes, 1, 0.25, 0.5, 0.75)
        rgba(selectedNo, 0.875, 0.625, 0.375, 0)
        "#,
    )
    .expect("ordinary booleans select independent ColorMixin values");
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn curve_boolean_components_select_zero_and_fractional_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, values in ipairs({{0, 0.75}, {0.25, 0}, {0.5, 1}}) do
            local yes, no = values[1], values[2]
            local actualYes = C_CurveUtil.EvaluateColorValueFromBoolean(true, yes, no)
            local actualNo = C_CurveUtil.EvaluateColorValueFromBoolean(false, yes, no)
            assert(type(actualYes) == "number" and actualYes == yes)
            assert(type(actualNo) == "number" and actualNo == no)
        end
        "#,
    )
    .expect("ordinary booleans select numeric color components including zero");
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn curve_boolean_selectors_reject_non_boolean_conditions_atomically() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local color = C_CurveUtil.EvaluateColorFromBoolean
        local component = C_CurveUtil.EvaluateColorValueFromBoolean
        assert(type(color) == "function" and type(component) == "function")
        local yes = CreateColor(0.125, 0.25, 0.5, 0.75)
        local no = CreateColor(0.875, 0.625, 0.375, 0.25)
        local function reject(condition)
            assert(not pcall(color, condition, yes, no), "color condition must be boolean")
            assert(not pcall(component, condition, 0.25, 0.75), "component condition must be boolean")
            assert(yes.r == 0.125 and yes.g == 0.25 and yes.b == 0.5 and yes.a == 0.75)
            assert(no.r == 0.875 and no.g == 0.625 and no.b == 0.375 and no.a == 0.25)
        end
        reject(nil)
        for _, condition in ipairs({0, 1, "true", {}, function() end}) do reject(condition) end
        "#,
    )
    .expect("invalid ordinary conditions reject without mutating color inputs");
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn curve_boolean_colors_validate_both_inputs_atomically() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local selectColor = C_CurveUtil.EvaluateColorFromBoolean
        assert(type(selectColor) == "function")
        local valid = CreateColor(0.125, 0.25, 0.5, 0.75)
        local function reject(invalid)
            local before = {}
            if type(invalid) == "table" then
                for key, value in pairs(invalid) do before[key] = value end
            end
            for _, condition in ipairs({true, false}) do
                assert(not pcall(selectColor, condition, invalid, valid), "invalid true color")
                assert(not pcall(selectColor, condition, valid, invalid), "invalid false color")
                assert(valid.r == 0.125 and valid.g == 0.25 and valid.b == 0.5 and valid.a == 0.75)
                if type(invalid) == "table" then
                    for key, value in pairs(before) do assert(invalid[key] == value) end
                    for key, value in pairs(invalid) do assert(before[key] == value) end
                end
            end
        end
        reject(nil)
        for _, invalid in ipairs({false, 0.5, "color", function() end, {}}) do reject(invalid) end
        for _, channel in ipairs({"r", "g", "b", "a"}) do
            local missing = {r=0.125, g=0.25, b=0.5, a=0.75}
            missing[channel] = nil
            reject(missing)
            for _, wrongType in ipairs({"0.5", false, {}}) do
                local malformed = {r=0.125, g=0.25, b=0.5, a=0.75}
                malformed[channel] = wrongType
                reject(malformed)
            end
        end
        "#,
    )
    .expect("both color arguments require numeric RGBA channels even when unselected");
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn curve_boolean_components_validate_both_inputs_atomically() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local selectComponent = C_CurveUtil.EvaluateColorValueFromBoolean
        assert(type(selectComponent) == "function")
        local sentinel = {value="unchanged"}
        local function reject(invalid)
            for _, condition in ipairs({true, false}) do
                assert(not pcall(selectComponent, condition, invalid, 0.75), "invalid true component")
                assert(not pcall(selectComponent, condition, 0.25, invalid), "invalid false component")
                assert(sentinel.value == "unchanged")
                for key in pairs(sentinel) do assert(key == "value") end
            end
        end
        reject(nil)
        for _, invalid in ipairs({false, "0.5", sentinel, function() end}) do reject(invalid) end
        "#,
    )
    .expect("both components require numbers even when unselected; errors are not specified");
}

#[test]
#[cfg(not(feature = "retail-12-0-0"))]
fn curve_boolean_helpers_are_absent_before_retail_12_0_0() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(rawget(C_CurveUtil, "EvaluateColorFromBoolean") == nil)
        assert(rawget(C_CurveUtil, "EvaluateColorValueFromBoolean") == nil)
        assert(type(C_CurveUtil.CreateCurve) == "function")
        assert(type(C_CurveUtil.CreateColorCurve) == "function")
        assert(C_CurveUtil.CreateCurve():GetPointCount() == 0)
        assert(C_CurveUtil.CreateColorCurve():GetPointCount() == 0)
        "#,
    )
    .expect("earlier profiles retain existing factories without retail boolean helpers");
}

#[test]
fn curve_get_type_scalar_tracks_mode_and_copy_independence() {
    assert_curve_get_type_tracks_mode_and_copy("CreateCurve");
}

#[test]
fn curve_get_type_color_tracks_mode_and_copy_independence() {
    assert_curve_get_type_tracks_mode_and_copy("CreateColorCurve");
}

fn assert_curve_get_type_tracks_mode_and_copy(factory: &str) {
    let env = WowLuaEnv::new().unwrap();
    env.exec(&format!(
        r#"
        local function assert_type(expected, ...)
            assert(select('#', ...) == 1, 'GetType must return one value')
            local actual = ...
            assert(type(actual) == 'number', 'GetType must return a number')
            assert(actual == expected, 'unexpected curve type')
        end
        assert(Enum.LuaCurveType.Linear == 0 and Enum.LuaCurveType.Step == 1)
        local source = C_CurveUtil.{factory}()
        assert_type(0, source:GetType())
        source:SetType(Enum.LuaCurveType.Step)
        assert_type(1, source:GetType())
        local copy = source:Copy()
        assert_type(1, copy:GetType())
        source:SetType(Enum.LuaCurveType.Linear)
        assert_type(0, source:GetType())
        assert_type(1, copy:GetType())
        copy:SetType(Enum.LuaCurveType.Linear)
        assert_type(0, copy:GetType())
        source:SetType(Enum.LuaCurveType.Step)
        assert_type(1, source:GetType())
        assert_type(0, copy:GetType())
        copy:SetType(Enum.LuaCurveType.Step)
        assert_type(1, copy:GetType())
        source:SetType(Enum.LuaCurveType.Linear)
        assert_type(0, source:GetType())
        assert_type(1, copy:GetType())
        "#,
    ))
    .expect("GetType follows independent source and copy modes");
}

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
    env.exec(
        r#"
        local source = C_CurveUtil.CreateCurve()
        source:AddPoint(0, 10)
        source:AddPoint(1, 20)
        local function one_result(...)
            assert(select('#', ...) == 1, 'Copy returns one value')
            return ...
        end
        local copy = one_result(source:Copy())
        -- Distinct simulator handles, not native identity or lifecycle proof.
        assert(type(copy) == 'userdata' and not rawequal(copy, source))
        assert(copy:GetPointCount() == 2)
        assert(source:Evaluate(0.5) == 15 and copy:Evaluate(0.5) == 15)

        source:ClearPoints()
        assert(source:GetPointCount() == 0 and copy:GetPointCount() == 2)
        assert(copy:Evaluate(0.5) == 15, 'source clearing preserves copied points')
        source:AddPoint(0, 30)
        source:AddPoint(1, 50)
        assert(source:Evaluate(0.5) == 40 and copy:Evaluate(0.5) == 15)

        copy:ClearPoints()
        assert(copy:GetPointCount() == 0 and source:GetPointCount() == 2)
        copy:AddPoint(0, 70)
        copy:AddPoint(1, 90)
        assert(copy:GetPointCount() == 2)
        assert(copy:Evaluate(0.5) == 80 and source:Evaluate(0.5) == 40)
        "#,
    )
    .expect("scalar curve copies retain independently mutable point values");
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
fn color_curve_get_point_returns_ordered_points_and_missing_nil() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function one_result(...)
            assert(select('#', ...) == 1, 'GetPoint returns one value')
            return ...
        end
        local curve = C_CurveUtil.CreateColorCurve()
        -- One-based indexing and missing-index nil are simulator policies.
        assert(one_result(curve:GetPoint(1)) == nil)
        curve:AddPoint(0.25, CreateColor(0.125, 0.25, 0.5, 0.75))
        curve:AddPoint(0.75, CreateColor(0.875, 0.5, 0.25, 1))
        local first = one_result(curve:GetPoint(1))
        local second = one_result(curve:GetPoint(2))
        assert(first.x == 0.25 and second.x == 0.75)
        assert(first.y.r == 0.125 and first.y.g == 0.25)
        assert(first.y.b == 0.5 and first.y.a == 0.75)
        assert(second.y.r == 0.875 and second.y.g == 0.5)
        assert(second.y.b == 0.25 and second.y.a == 1)
        assert(one_result(curve:GetPoint(3)) == nil)
        "#,
    )
    .expect("color point lookup preserves configured order and channels");
}

#[test]
fn color_curve_get_point_returns_independent_point_and_color_snapshots() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0.25, CreateColor(0.125, 0.25, 0.5, 0.75))
        curve:AddPoint(0.75, CreateColor(0.875, 0.5, 0.25, 1))
        local first = curve:GetPoint(1)
        local other = curve:GetPoint(1)
        -- Fresh output is a simulator policy, not native identity proof.
        assert(not rawequal(first, other) and not rawequal(first.y, other.y))
        first.x = 9
        first.y.r, first.y.g, first.y.b, first.y.a = 1, 1, 1, 1
        assert(other.x == 0.25)
        assert(other.y.r == 0.125 and other.y.g == 0.25)
        assert(other.y.b == 0.5 and other.y.a == 0.75)
        local later = curve:GetPoint(1)
        assert(later.x == 0.25)
        assert(later.y.r == 0.125 and later.y.g == 0.25)
        assert(later.y.b == 0.5 and later.y.a == 0.75)
        local r, g, b, a = curve:Evaluate(0.5):GetRGBA()
        assert(r == 0.5 and g == 0.375 and b == 0.375 and a == 0.875)
        "#,
    )
    .expect("mutating returned point and color leaves curve evaluation unchanged");
}

#[test]
fn color_curve_get_point_copy_survives_original_clear() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0.25, CreateColor(0.125, 0.25, 0.5, 0.75))
        curve:AddPoint(0.75, CreateColor(0.875, 0.5, 0.25, 1))
        local copy = curve:Copy()
        curve:ClearPoints()
        assert(curve:GetPoint(1) == nil and curve:GetPoint(2) == nil)
        local first, second = copy:GetPoint(1), copy:GetPoint(2)
        assert(first.x == 0.25 and second.x == 0.75)
        assert(first.y.r == 0.125 and first.y.g == 0.25)
        assert(first.y.b == 0.5 and first.y.a == 0.75)
        assert(second.y.r == 0.875 and second.y.g == 0.5)
        assert(second.y.b == 0.25 and second.y.a == 1)
        curve:AddPoint(0.5, CreateColor(1, 0, 0, 1))
        assert(copy:GetPoint(1).x == 0.25 and copy:GetPoint(2).x == 0.75)
        assert(curve:GetPoint(1).x == 0.5 and curve:GetPoint(2) == nil)
        "#,
    )
    .expect("copied point retrieval remains independent after original clear and reuse");
}

#[test]
fn color_curve_get_points_returns_one_ordered_collection() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function one_table(...)
            assert(select('#', ...) == 1, 'GetPoints returns one value')
            local result = ...
            assert(type(result) == 'table')
            return result
        end
        local curve = C_CurveUtil.CreateColorCurve()
        assert(next(one_table(curve:GetPoints())) == nil)
        -- Insertion order is a simulator policy, not native ordering proof.
        curve:AddPoint(0.75, CreateColor(0.875, 0.5, 0.25, 1))
        curve:AddPoint(0.25, CreateColor(0.125, 0.25, 0.5, 0.75))
        local points = one_table(curve:GetPoints())
        assert(#points == 2 and points[3] == nil)
        assert(points[1].x == 0.75 and points[2].x == 0.25)
        assert(points[1].y.r == 0.875 and points[1].y.g == 0.5)
        assert(points[1].y.b == 0.25 and points[1].y.a == 1)
        assert(points[2].y.r == 0.125 and points[2].y.g == 0.25)
        assert(points[2].y.b == 0.5 and points[2].y.a == 0.75)
        "#,
    )
    .expect("color collection query returns one table in configured order");
}

#[test]
fn color_curve_get_points_isolates_collection_point_and_color_mutation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0.25, CreateColor(0.125, 0.25, 0.5, 0.75))
        curve:AddPoint(0.75, CreateColor(0.875, 0.5, 0.25, 1))
        local points, other = curve:GetPoints(), curve:GetPoints()
        -- Fresh output snapshots are a simulator policy.
        points[1].x = 9
        points[1].y.r, points[1].y.g = 1, 1
        points[1].y.b, points[1].y.a = 1, 0
        points[2] = nil
        points[3] = {x=10, y=CreateColor(1, 1, 1, 1)}
        local later = curve:GetPoints()
        for _, snapshot in ipairs({other, later}) do
            assert(#snapshot == 2 and snapshot[3] == nil)
            assert(snapshot[1].x == 0.25 and snapshot[2].x == 0.75)
            assert(snapshot[1].y.r == 0.125 and snapshot[1].y.g == 0.25)
            assert(snapshot[1].y.b == 0.5 and snapshot[1].y.a == 0.75)
        end
        local r, g, b, a = curve:Evaluate(0.5):GetRGBA()
        assert(r == 0.5 and g == 0.375 and b == 0.375 and a == 0.875)
        "#,
    )
    .expect("returned collection mutations leave snapshots and evaluation unchanged");
}

#[test]
fn color_curve_get_points_snapshots_survive_copy_clear_and_reuse() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0.25, CreateColor(0.125, 0.25, 0.5, 0.75))
        local copy = curve:Copy()
        local snapshot = curve:GetPoints()
        curve:ClearPoints()
        assert(next(curve:GetPoints()) == nil)
        curve:AddPoint(0.75, CreateColor(1, 0, 0, 1))
        local copied = copy:GetPoints()
        assert(#copied == 1 and copied[1].x == 0.25)
        copy:ClearPoints()
        assert(next(copy:GetPoints()) == nil)
        copy:AddPoint(0.5, CreateColor(0, 1, 0, 1))
        for _, points in ipairs({snapshot, copied}) do
            assert(#points == 1 and points[1].x == 0.25)
            assert(points[1].y.r == 0.125 and points[1].y.g == 0.25)
            assert(points[1].y.b == 0.5 and points[1].y.a == 0.75)
        end
        assert(curve:GetPoints()[1].x == 0.75)
        assert(copy:GetPoints()[1].x == 0.5)
        assert(curve:GetPoints()[1].y.r == 1)
        assert(copy:GetPoints()[1].y.g == 1)
        "#,
    )
    .expect("collection snapshots survive independent curve clear and reuse");
}

#[test]
fn color_curve_remove_point_compacts_first_middle_and_last() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- Valid one-based indices are a simulator policy, not native proof.
        for removed = 1, 3 do
            local curve = C_CurveUtil.CreateColorCurve()
            for index = 1, 3 do
                curve:AddPoint(index / 4, CreateColor(index / 4, 0.5, 0.75, 1))
            end
            assert(select('#', curve:RemovePoint(removed)) == 0)
            assert(curve:GetPointCount() == 2)
            local points = curve:GetPoints()
            assert(#points == 2 and points[3] == nil)
            local remaining = 1
            for original = 1, 3 do
                if original ~= removed then
                    assert(points[remaining].x == original / 4)
                    local r, g, b, a = points[remaining].y:GetRGBA()
                    assert(r == original / 4 and g == 0.5 and b == 0.75 and a == 1)
                    remaining = remaining + 1
                end
            end
        end
        "#,
    )
    .expect("valid color point removal compacts remaining points without returns");
}

#[test]
fn color_curve_remove_point_last_leaves_empty_curve() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0.25, CreateColor(0.5, 0.25, 0.75, 1))
        assert(select('#', curve:RemovePoint(1)) == 0)
        assert(curve:GetPointCount() == 0)
        assert(next(curve:GetPoints()) == nil)
        assert(curve:GetPoint(1) == nil)
        "#,
    )
    .expect("removing the sole color point leaves an empty curve");
}

#[test]
fn color_curve_remove_point_updates_evaluation_independently_of_copy() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0, CreateColor(0, 0, 0, 0))
        curve:AddPoint(0.5, CreateColor(1, 0, 1, 0))
        curve:AddPoint(1, CreateColor(1, 1, 1, 1))
        local copy = curve:Copy()
        assert(select('#', curve:RemovePoint(2)) == 0)
        local r, g, b, a = curve:Evaluate(0.5):GetRGBA()
        assert(r == 0.5 and g == 0.5 and b == 0.5 and a == 0.5)
        assert(copy:GetPointCount() == 3 and copy:GetPoint(2).x == 0.5)
        r, g, b, a = copy:Evaluate(0.5):GetRGBA()
        assert(r == 1 and g == 0 and b == 1 and a == 0)
        assert(select('#', copy:RemovePoint(1)) == 0)
        assert(curve:GetPointCount() == 2 and curve:GetPoint(1).x == 0)
        assert(copy:GetPointCount() == 2 and copy:GetPoint(1).x == 0.5)
        "#,
    )
    .expect("middle removal changes evaluation without mutating a copied curve");
}

#[test]
fn color_curve_set_points_replaces_and_empty_clears() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(9, CreateColor(1, 1, 1, 1))
        -- Stored input order is a simulator valid-input policy.
        assert(select('#', curve:SetPoints({
            {x=0.75, y=CreateColor(0.875, 0.5, 0.25, 1)},
            {x=0.25, y=CreateColor(0.125, 0.25, 0.5, 0.75)},
        })) == 0)
        local points = curve:GetPoints()
        assert(curve:GetPointCount() == 2 and #points == 2)
        assert(points[1].x == 0.75 and points[2].x == 0.25)
        local r, g, b, a = points[1].y:GetRGBA()
        assert(r == 0.875 and g == 0.5 and b == 0.25 and a == 1)
        r, g, b, a = points[2].y:GetRGBA()
        assert(r == 0.125 and g == 0.25 and b == 0.5 and a == 0.75)
        assert(select('#', curve:SetPoints({})) == 0)
        assert(curve:GetPointCount() == 0 and next(curve:GetPoints()) == nil)
        assert(curve:GetPoint(1) == nil)
        "#,
    )
    .expect("color point replacement preserves configured order and empty clears");
}

#[test]
fn color_curve_set_points_copies_input_array_records_and_colors() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        local first = {x=0.25, y=CreateColor(0.125, 0.25, 0.5, 0.75)}
        local color = first.y
        local input = {first, {x=0.75, y=CreateColor(0.875, 0.5, 0.25, 1)}}
        assert(select('#', curve:SetPoints(input)) == 0)
        -- Input copying is a simulator valid-input policy, not native proof.
        first.x = 9
        color.r, color.g, color.b, color.a = 1, 1, 1, 0
        first.y = CreateColor(0, 0, 0, 0)
        input[1], input[2] = nil, nil
        input[3] = {x=10, y=CreateColor(1, 1, 1, 1)}
        local points = curve:GetPoints()
        assert(#points == 2 and points[3] == nil)
        assert(points[1].x == 0.25 and points[2].x == 0.75)
        local r, g, b, a = points[1].y:GetRGBA()
        assert(r == 0.125 and g == 0.25 and b == 0.5 and a == 0.75)
        r, g, b, a = curve:Evaluate(0.5):GetRGBA()
        assert(r == 0.5 and g == 0.375 and b == 0.375 and a == 0.875)
        "#,
    )
    .expect("input mutations leave stored color points and evaluation unchanged");
}

#[test]
fn color_curve_set_points_keeps_copy_replacements_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        assert(select('#', curve:SetPoints({{x=0.25, y=CreateColor(1, 0, 0, 1)}})) == 0)
        local copy = curve:Copy()
        assert(select('#', curve:SetPoints({{x=0.5, y=CreateColor(0, 1, 0, 1)}})) == 0)
        assert(copy:GetPointCount() == 1 and copy:GetPoint(1).x == 0.25)
        assert(copy:Evaluate(0.25).r == 1 and copy:Evaluate(0.25).g == 0)
        assert(select('#', copy:SetPoints({{x=0.75, y=CreateColor(0, 0, 1, 0.5)}})) == 0)
        assert(curve:GetPointCount() == 1 and curve:GetPoint(1).x == 0.5)
        assert(curve:Evaluate(0.5).g == 1 and curve:Evaluate(0.5).b == 0)
        assert(copy:GetPoint(1).x == 0.75 and copy:Evaluate(0.75).b == 1)
        assert(copy:Evaluate(0.75).a == 0.5)
        "#,
    )
    .expect("replacing either copied curve leaves the other unchanged");
}

// Numeric expectations below are simulator policies, not native numeric proof.
#[test]
fn color_curve_evaluate_unpacked_empty() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        assert(select('#', curve:EvaluateUnpacked(0.375)) == 4)
        local r, g, b, a = curve:EvaluateUnpacked(0.375)
        assert(type(r) == 'number' and type(g) == 'number')
        assert(type(b) == 'number' and type(a) == 'number')
        assert(r == 0 and g == 0 and b == 0 and a == 0)
        "#,
    )
    .expect("empty color curve returns exactly four numeric zero channels");
}

#[test]
fn color_curve_evaluate_unpacked_single_point() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0.375, CreateColor(0.125, 0.25, 0.5, 0.875))
        assert(select('#', curve:EvaluateUnpacked(0.375)) == 4)
        local r, g, b, a = curve:EvaluateUnpacked(0.375)
        assert(type(r) == 'number' and type(g) == 'number')
        assert(type(b) == 'number' and type(a) == 'number')
        assert(r == 0.125 and g == 0.25 and b == 0.5 and a == 0.875)
        "#,
    )
    .expect("single color point returns exactly four numeric RGBA channels");
}

#[test]
fn color_curve_evaluate_unpacked_linear_interior() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:SetType(Enum.LuaCurveType.Linear)
        curve:AddPoint(0.25, CreateColor(0.125, 0.25, 0.5, 0.875))
        curve:AddPoint(0.75, CreateColor(0.625, 0.75, 0.25, 0.5))
        assert(select('#', curve:EvaluateUnpacked(0.375)) == 4)
        local r, g, b, a = curve:EvaluateUnpacked(0.375)
        assert(type(r) == 'number' and type(g) == 'number')
        assert(type(b) == 'number' and type(a) == 'number')
        assert(r == 0.25 and g == 0.375 and b == 0.4375 and a == 0.78125)
        "#,
    )
    .expect("linear interior returns exactly four interpolated numeric RGBA channels");
}

#[test]
fn color_curve_evaluate_unpacked_step_interior() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateColorCurve()
        curve:SetType(Enum.LuaCurveType.Step)
        curve:AddPoint(0.25, CreateColor(0.125, 0.25, 0.5, 0.875))
        curve:AddPoint(0.75, CreateColor(0.625, 0.75, 0.25, 0.5))
        assert(select('#', curve:EvaluateUnpacked(0.375)) == 4)
        local r, g, b, a = curve:EvaluateUnpacked(0.375)
        assert(type(r) == 'number' and type(g) == 'number')
        assert(type(b) == 'number' and type(a) == 'number')
        assert(r == 0.125 and g == 0.25 and b == 0.5 and a == 0.875)
        "#,
    )
    .expect("Step interior returns exactly four numeric left-point RGBA channels");
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
#[cfg(feature = "retail-12-1-0")]
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
    env.exec(
        r#"
        local function assert_color(curve, r, g, b, a)
            local actualR, actualG, actualB, actualA = curve:Evaluate(0.5):GetRGBA()
            assert(actualR == r and actualG == g and actualB == b and actualA == a,
                'unexpected copied color channels')
        end
        local function one_result(...)
            assert(select('#', ...) == 1, 'Copy returns one value')
            return ...
        end
        local source = C_CurveUtil.CreateColorCurve()
        assert(source:GetPointCount() == 0)
        source:AddPoint(0, CreateColor(1, 0, 0, 0.25))
        assert(source:GetPointCount() == 1)
        source:AddPoint(1, CreateColor(0, 0, 1, 0.75))
        assert(source:GetPointCount() == 2)
        source:SetType(Enum.LuaCurveType.Step)
        local copy = one_result(source:Copy())
        -- Modeled point/configuration independence, not native or security proof.
        assert(type(copy) == 'userdata' and not rawequal(copy, source))
        assert(copy:GetPointCount() == 2)
        assert_color(source, 1, 0, 0, 0.25)
        assert_color(copy, 1, 0, 0, 0.25)

        source:SetType(Enum.LuaCurveType.Linear)
        assert_color(source, 0.5, 0, 0.5, 0.5)
        assert_color(copy, 1, 0, 0, 0.25)
        source:SetType(Enum.LuaCurveType.Step)
        copy:SetType(Enum.LuaCurveType.Linear)
        assert_color(source, 1, 0, 0, 0.25)
        assert_color(copy, 0.5, 0, 0.5, 0.5)

        source:ClearPoints()
        assert(source:GetPointCount() == 0 and copy:GetPointCount() == 2)
        assert_color(copy, 0.5, 0, 0.5, 0.5)
        source:AddPoint(0, CreateColor(0, 1, 0, 0.5))
        assert(source:GetPointCount() == 1)
        source:AddPoint(1, CreateColor(1, 1, 0, 1))
        assert(source:GetPointCount() == 2)
        source:SetType(Enum.LuaCurveType.Linear)
        assert_color(source, 0.5, 1, 0, 0.75)
        assert_color(copy, 0.5, 0, 0.5, 0.5)

        copy:ClearPoints()
        assert(copy:GetPointCount() == 0 and source:GetPointCount() == 2)
        assert_color(source, 0.5, 1, 0, 0.75)
        copy:AddPoint(0, CreateColor(1, 0, 1, 0))
        assert(copy:GetPointCount() == 1)
        copy:AddPoint(1, CreateColor(0, 1, 1, 0.5))
        assert(copy:GetPointCount() == 2)
        assert_color(copy, 0.5, 0.5, 1, 0.25)
        assert_color(source, 0.5, 1, 0, 0.75)
        "#,
    )
    .expect("color curve copies retain independently mutable points and interpolation modes");
}
