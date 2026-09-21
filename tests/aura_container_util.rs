#![cfg(feature = "aura-containers")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn aura_update_and_texture_style_enums_match_native_contract() {
    let env = WowLuaEnv::new().unwrap();
    let contract = r#"
        local function check(name, expected)
            local actual = assert(Enum[name], name .. ' missing')
            local count = 0
            for key, value in pairs(actual) do
                assert(expected[key] == value, name .. '.' .. key)
                count = count + 1
            end
            local expectedCount = 0
            for key, value in pairs(expected) do
                assert(actual[key] == value)
                expectedCount = expectedCount + 1
            end
            assert(count == expectedCount)
            local meta = assert(Enum[name .. 'Meta'])
            assert(meta.MinValue == 0 and meta.MaxValue == count - 1)
            assert(meta.NumValues == count)
        end
        check('CustomAuraButtonUpdateMode', {Assignment = 0, Update = 1})
        check('CustomAuraButtonDispelTypeTextureStyle', {
            Border = 0, BorderWithIcon = 1, Icon = 2, PreserveAsset = 3, CustomAsset = 4,
        })
    "#;
    env.exec(contract).unwrap();
    env.exec_maybe_secure(contract, true).unwrap();
}

#[test]
fn aura_stealable_filter_enum_and_options_match_native_contract() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local function check()
            local filter = Enum.CustomAuraButtonDispelTypeStealableFilter
            assert(type(filter) == 'table', 'native stealable filter enum missing')
            assert(filter.Stealable == 0 and filter.NotStealable == 1)
            local count = 0
            for _ in pairs(filter) do count = count + 1 end
            assert(count == 2)
            local meta = Enum.CustomAuraButtonDispelTypeStealableFilterMeta
            assert(meta.MinValue == 0 and meta.MaxValue == 1 and meta.NumValues == 2)
            for _, value in ipairs({ filter.Stealable, filter.NotStealable }) do
                local input = { showWhenHelpful = true, stealableFilter = value }
                local output = C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextureOptions(input)
                assert(output ~= input and output.stealableFilter == value)
                assert(output.showWhenHelpful == true)
            end
        end
        check()
        setfenv(check, __secureenv)
        check()
    "#).unwrap();
}

#[test]
fn aura_option_defaults_are_available_in_public_and_secure_environments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local api = C_AuraContainerUtil
        local text = api.ProcessCustomAuraButtonDispelTypeTextOptions()
        assert(text.showWhenHarmful == true)
        assert(text.showWhenHelpful == false and text.showWithoutDispelType == false)
        local texture = api.ProcessCustomAuraButtonDispelTypeTextureOptions(nil)
        assert(texture.showAlways == false and texture.showWhenHarmful == true)
        assert(texture.showWhenHelpful == false and texture.showWithoutDispelType == false)
        assert(texture.style == Enum.CustomAuraButtonDispelTypeTextureStyle.BorderWithIcon)
        assert(texture.stealableFilter == nil and texture.customDispelAssetMap == nil)
        assert(next(api.ProcessCustomAuraButtonApplicationCountOptions()) == nil)
        assert(next(api.ProcessCustomAuraButtonDurationBarOptions()) == nil)
        assert(next(api.ProcessCustomAuraButtonDurationTextOptions()) == nil)
        local fn = function() return C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextOptions() end
        setfenv(fn, __secureenv)
        assert(fn().showWhenHarmful == true)
    "#).unwrap();
}

#[test]
fn aura_tooltip_options_copy_documented_nested_fields_and_colors() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local api = C_AuraContainerUtil
        local input = {
            backdropInfo = { bgFile = 134400, edgeFile = 'Interface\\Buttons\\WHITE8X8',
                edgeSize = 12, tile = false, tileEdge = true, tileSize = 8, insets = { left = 3 } },
            borderColor = CreateColor(0.1, 0.2, 0.3, 0.4),
            centerColor = CreateColor(0.5, 0.6, 0.7, 0.8),
            anchorOffsets = { top = 5 }, ignored = 'not in the structure',
        }
        local output = api.ProcessAuraTooltipBackdropOptions(input)
        assert(output ~= input and output.backdropInfo ~= input.backdropInfo)
        assert(output.backdropInfo.insets ~= input.backdropInfo.insets)
        assert(output.backdropInfo.insets.left == 3 and output.backdropInfo.insets.right == 0)
        assert(output.backdropInfo.insets.top == 0 and output.backdropInfo.insets.bottom == 0)
        assert(output.backdropInfo.bgFile == 134400 and output.backdropInfo.tile == false)
        assert(output.anchorOffsets.top == 5 and output.anchorOffsets.left == 0)
        assert(output.ignored == nil and input.backdropInfo.insets.right == nil)
        assert(output.borderColor ~= input.borderColor)
        local r,g,b,a = output.borderColor:GetRGBA()
        assert(r == 0.1 and g == 0.2 and b == 0.3 and a == 0.4)
        output.backdropInfo.insets.left = 99
        assert(input.backdropInfo.insets.left == 3)
        local nine = api.ProcessAuraTooltipNineSliceOptions({ layoutName = 'TooltipDefaultLayout', anchorOffsets = {} })
        assert(nine.layoutName == 'TooltipDefaultLayout' and nine.anchorOffsets.bottom == 0)
        local slice = api.ProcessAuraTooltipTextureSliceOptions({ asset = 'SomeAtlas',
            sliceMargins = { left = 1, top = 2, right = 3, bottom = 4 },
            sliceMode = Enum.UITextureSliceMode.Tiled, drawLayer = 'OVERLAY' })
        assert(slice.asset == 'SomeAtlas' and slice.drawLayerSublevel == 0)
        assert(slice.sliceMargins.bottom == 4 and slice.drawLayer == 'OVERLAY')
    "#).unwrap();
}

#[test]
fn aura_dispel_maps_are_copied_and_texture_coordinate_defaults_are_applied() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local input = { showWhenHarmful = false, showWhenHelpful = true,
            style = Enum.CustomAuraButtonDispelTypeTextureStyle.CustomAsset,
            customDispelAssetMap = { Magic = { asset = 134400, texCoords = { right = 0.75 } } },
            customDispelColorMap = { Magic = { r = 0.2, g = 0.4, b = 0.6 } },
            customDispelColorCurve = C_CurveUtil.CreateColorCurve() }
        local result = C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextureOptions(input)
        local asset = result.customDispelAssetMap.Magic
        assert(result.showWhenHarmful == false and result.showWhenHelpful == true)
        assert(result.customDispelAssetMap ~= input.customDispelAssetMap)
        assert(asset ~= input.customDispelAssetMap.Magic and asset.useAtlasSize == false)
        assert(asset.texCoords.left == 0 and asset.texCoords.right == 0.75)
        assert(asset.texCoords.top == 0 and asset.texCoords.bottom == 1)
        assert(result.customDispelColorMap.Magic ~= input.customDispelColorMap.Magic)
        assert(result.customDispelColorMap.Magic.g == 0.4)
        assert(result.customDispelColorCurve == input.customDispelColorCurve)
        local textInput = { customDispelTextMap = { [''] = 'Unclassified', Magic = 'Magic!' } }
        local text = C_AuraContainerUtil.ProcessCustomAuraButtonDispelTypeTextOptions(textInput)
        assert(text.customDispelTextMap ~= textInput.customDispelTextMap)
        assert(text.customDispelTextMap[''] == 'Unclassified' and text.customDispelTextMap.Magic == 'Magic!')
    "#).unwrap();
}

#[test]
fn aura_duration_options_preserve_object_handles_and_copy_format_components() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local formatter = C_StringUtil.CreateNumericRuleFormatter()
        local binding = assert(C_DurationUtil.CreateDurationTextBinding(),
            'modeled duration binding factory must return a handle')
        local curve = assert(C_CurveUtil.CreateColorCurve(),
            'modeled color curve factory must return a handle')
        local options = { binding = binding, textFormatter = formatter,
            textFormat = { formatString = '%s remaining', components = {
                { property = Enum.DurationTextBindingProperty.RemainingDuration, formatter = formatter } } },
            textColor = { curve = curve, property = Enum.DurationTextBindingProperty.RemainingPercent } }
        local result = C_AuraContainerUtil.ProcessCustomAuraButtonDurationTextOptions(options)
        assert(result ~= options and result.binding == binding and result.textFormatter == formatter)
        assert(result.textFormat ~= options.textFormat and result.textFormat.components ~= options.textFormat.components)
        assert(result.textFormat.components[1] ~= options.textFormat.components[1])
        assert(result.textFormat.components[1].formatter == formatter)
        assert(result.textColor ~= options.textColor and result.textColor.curve == curve)
        local count = C_AuraContainerUtil.ProcessCustomAuraButtonApplicationCountOptions({formatter = formatter})
        assert(count.formatter == formatter)
        local bar = C_AuraContainerUtil.ProcessCustomAuraButtonApplicationBarOptions({maxApplications = 5, interpolation = 1})
        assert(bar.maxApplications == 5 and bar.interpolation == 1)
        local duration = C_AuraContainerUtil.ProcessCustomAuraButtonDurationBarOptions({interpolation = 0, direction = 1})
        assert(duration.interpolation == 0 and duration.direction == 1)
    "#).unwrap();
}

#[test]
fn aura_options_reject_missing_required_values_and_wrong_nested_types() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local api = C_AuraContainerUtil
        local cases = {
            {'ProcessAuraTooltipBackdropOptions', nil},
            {'ProcessAuraTooltipBackdropOptions', {}},
            {'ProcessAuraTooltipNineSliceOptions', {layoutName = false}},
            {'ProcessAuraTooltipTextureSliceOptions', {asset = {}}},
            {'ProcessCustomAuraButtonApplicationBarOptions', {}},
            {'ProcessCustomAuraButtonApplicationBarOptions', {maxApplications = 'five'}},
            {'ProcessCustomAuraButtonApplicationCountOptions', {formatter = false}},
            {'ProcessCustomAuraButtonDispelTypeTextOptions', {showWhenHelpful = 1}},
            {'ProcessCustomAuraButtonDispelTypeTextOptions', {customDispelTextMap = {[1] = 'bad key'}}},
            {'ProcessCustomAuraButtonDispelTypeTextureOptions', {style = 5}},
            {'ProcessCustomAuraButtonDispelTypeTextureOptions', {customDispelAssetMap = {Magic = {}}}},
            {'ProcessCustomAuraButtonDispelTypeTextureOptions', {customDispelColorMap = {Magic = {r=1,g=1}}}},
            {'ProcessCustomAuraButtonDurationBarOptions', {direction = 0.5}},
            {'ProcessCustomAuraButtonDurationTextOptions', {textFormat = {formatString = '%s'}}},
            {'ProcessCustomAuraButtonDurationTextOptions', {textFormat = {formatString = '%s', components = {{property = 0}}}}},
            {'ProcessCustomAuraButtonDurationTextOptions', {textColor = {property = 0}}},
            {'ProcessAuraTooltipTextureSliceOptions', {asset = 'atlas', drawLayer = 'INVALID'}},
            {'ProcessAuraTooltipTextureSliceOptions', {asset = 'atlas', sliceMargins = {left=1}}},
        }
        for _, case in ipairs(cases) do
            local ok, err = pcall(api[case[1]], case[2])
            assert(not ok and type(err) == 'string', case[1] .. ' must reject invalid options')
        end
    "#).unwrap();
}

#[test]
fn aura_options_support_the_real_custom_button_public_initializer() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(r#"
                    local container = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                    local provider = __secureenv.AuraContainerUtil.CreateCustomFrameProvider(
                        GetForbiddenObjectTable(container), { batchSize = 1,
                        templateNames = {'CustomAuraButtonTemplate'}, initializeFrame = function(button)
                            local count = button:CreateFontString(nil, 'OVERLAY')
                            button:SetApplicationCount(count)
                            local exported, options = button:GetApplicationCount()
                            assert(exported == count and next(options) == nil)
                            auraOptionsInitializerRan = true
                        end })
                    assert(provider:AcquireFrame() ~= nil)
                    assert(auraOptionsInitializerRan == true)
                "#).expect("native option processing completes the real public initializer");
            },
        );
    });
}
