#![cfg(feature = "client-mists")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_mists_nameplate_fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    // Explicit fixture inputs, not native CVar defaults. Enum values come from
    // the manifest-backed NamePlateConstantsDocumentation.lua.
    env.exec(
        r#"
        Enum.NamePlateSize = { Small = 1, Medium = 2, Large = 3, ExtraLarge = 4, Huge = 5 }
        Enum.NamePlateStyle = {
            Modern = 0, Thin = 1, Block = 2, HealthFocus = 3,
            CastFocus = 4, Legacy = 5, Classic = 6,
        }
        fixtureSize = Enum.NamePlateSize.Small
        fixtureStyle = Enum.NamePlateStyle.Modern
        GetCVarNumberOrDefault = function(name)
            assert(name == "nameplateSize", "unexpected scale CVar: " .. name)
            return fixtureSize
        end
        CVarCallbackRegistry = {
            SetCVarCachable = function() end,
            GetCVarNumberOrDefault = function(_, name)
                assert(name == "nameplateStyle", "unexpected registry CVar: " .. name)
                return fixtureStyle
            end,
        }
        "#,
    )
    .expect("Explicit Mists nameplate inputs should install");
    load_nameplate_lua(&env, "Blizzard_NamePlateConstants.lua");
    load_nameplate_lua(&env, "Blizzard_NamePlates.lua");
    env
}

fn load_nameplate_lua(env: &WowLuaEnv, file: &str) {
    let addons = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
        env!("CARGO_MANIFEST_DIR"),
    ));
    let path = addons.join("Blizzard_NamePlates").join(file);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("Could not read Mists fixture {}: {err}", path.display()));
    env.exec(&source)
        .unwrap_or_else(|err| panic!("Could not load Mists fixture {}: {err}", path.display()));
}

#[test]
fn nameplate_small_scale_selects_modern_and_classic_style() {
    let env = load_mists_nameplate_fixture();
    for (style, expected) in [
        ("Modern", (0.75, 0.8, 0.8, 0.75, 1.0)),
        ("Classic", (0.8, 0.8, 0.8, 0.8, 1.0)),
    ] {
        let scale: (f64, f64, f64, f64, f64) = env
            .eval(&format!(
                r#"
                local scale = NamePlateDriverMixin:GetNamePlateScale(Enum.NamePlateStyle.{style})
                return scale.horizontal, scale.vertical, scale.classification,
                    scale.aura, scale.aggroHighlight
                "#,
            ))
            .expect("Small scale should be selected from real Mists constants");
        assert_eq!(scale, expected, "Small {style} scale tuple");
    }
}

#[test]
fn nameplate_unsupported_size_99_uses_medium_scale_for_both_styles() {
    let env = load_mists_nameplate_fixture();
    env.exec("fixtureSize = 99")
        .expect("Unsupported size should be an explicit input");
    for style in ["Modern", "Classic"] {
        let scale: (f64, f64, f64, f64, f64) = env
            .eval(&format!(
                r#"
                local scale = NamePlateDriverMixin:GetNamePlateScale(Enum.NamePlateStyle.{style})
                return scale.horizontal, scale.vertical, scale.classification,
                    scale.aura, scale.aggroHighlight
                "#,
            ))
            .expect("Unsupported size should select the source's Medium fallback");
        assert_eq!(scale, (1.0, 1.0, 1.0, 1.0, 1.0), "Size 99 {style}");
    }
}

#[test]
fn nameplate_explicit_base_dimensions_override_small_scale_for_both_styles() {
    let env = load_mists_nameplate_fixture();
    for style in ["Modern", "Classic"] {
        let dimensions: (f64, f64) = env
            .eval(&format!(
                r#"
                local driver = setmetatable({{
                    baseNamePlateWidth = 128, baseNamePlateHeight = 32,
                }}, {{ __index = NamePlateDriverMixin }})
                local style = Enum.NamePlateStyle.{style}
                local scale = driver:GetNamePlateScale(style)
                return driver:GetNamePlateWidth(style, scale), driver:GetNamePlateHeight(style, scale)
                "#,
            ))
            .expect("Real dimension helpers should honor explicit base dimensions");
        assert_eq!(dimensions, (128.0, 32.0), "Small {style} override");
    }
}

#[test]
fn nameplate_options_publish_small_style_scales_and_explicit_size() {
    let env = load_mists_nameplate_fixture();
    env.exec(
        r#"
        -- Option destinations and engine/frame collaborators are controlled
        -- inputs. Scale/update methods remain real; class-bar setup is isolated.
        NamePlateSetupOptions = {}
        NamePlateEnemyFrameOptions = {}
        NamePlateFriendlyFrameOptions = {}
        local mouseoverColor = {}
        YELLOW_FONT_COLOR = mouseoverColor
        GetCVarBool = function(name)
            assert(name == "nameplateShowClassColor" or name == "nameplateShowFriendlyClassColor",
                "unexpected boolean CVar: " .. name)
            return name == "nameplateShowClassColor"
        end
        local cases = {
            { style = Enum.NamePlateStyle.Modern, horizontal = 0.75,
              healthHeight = 16, borderWidth = 0, borderHeight = 0, classic = false },
            { style = Enum.NamePlateStyle.Classic, horizontal = 0.8,
              healthHeight = 8, borderWidth = 102.4, borderHeight = 12.8, classic = true },
        }
        for _, case in ipairs(cases) do
            fixtureStyle = case.style
            local sizeCalls, appliedFrames = 0, 0
            local capturedWidth, capturedHeight, scriptWidth, scriptHeight
            local nativePlate = {
                ApplyFrameOptions = function() appliedFrames = appliedFrames + 1 end,
            }
            local scriptPlate = {
                SetSize = function(_, width, height)
                    scriptWidth, scriptHeight = width, height
                end,
                ApplyFrameOptions = function() appliedFrames = appliedFrames + 1 end,
            }
            C_NamePlate = {
                SetNamePlateSize = function(width, height)
                    sizeCalls = sizeCalls + 1
                    capturedWidth, capturedHeight = width, height
                end,
                GetNamePlates = function() return { nativePlate } end,
            }
            local driver = setmetatable({
                baseNamePlateWidth = 128, baseNamePlateHeight = 32,
                scriptNamePlates = { preview = scriptPlate },
                -- Class resource bar setup is outside this isolated fixture.
                SetupClassNameplateBars = function() end,
            }, { __index = NamePlateDriverMixin })
            driver:UpdateNamePlateOptions()

            local options = NamePlateSetupOptions
            assert(options.horizontalScale == case.horizontal, "published horizontal scale")
            assert(options.verticalScale == 0.8, "published vertical scale")
            assert(options.classificationScale == 0.8, "published classification scale")
            assert(options.healthBarHeight == case.healthHeight, "published health bar height")
            assert(options.castBarHeight == 8, "published cast bar height")
            assert(options.useClassicHealthBar == case.classic, "health bar style")
            assert(options.useClassicCastBar == case.classic, "cast bar style")
            assert(options.healthBarBorderWidth == case.borderWidth, "health border width")
            assert(options.castBarBorderWidth == case.borderWidth, "cast border width")
            assert(options.healthBarBorderHeight == case.borderHeight, "health border height")
            assert(options.castBarBorderHeight == case.borderHeight, "cast border height")
            assert(NamePlateEnemyFrameOptions.useClassColors == true, "explicit enemy class color")
            assert(NamePlateFriendlyFrameOptions.useClassColors == false, "explicit friendly class color")
            for _, frameOptions in ipairs({ NamePlateEnemyFrameOptions, NamePlateFriendlyFrameOptions }) do
                assert(frameOptions.showLevel == case.classic, "level display style")
                assert(frameOptions.displaySelectionHighlight == case.classic, "selection style")
                if case.classic then
                    assert(frameOptions.nameMouseoverColor == mouseoverColor, "Classic mouseover color")
                else
                    assert(frameOptions.nameMouseoverColor == nil, "Modern has no mouseover override")
                end
            end
            assert(sizeCalls == 1, "one native size publication per update")
            assert(capturedWidth == 128 and capturedHeight == 32, "native base-size override")
            assert(scriptWidth == 128 and scriptHeight == 32, "script base-size override")
            assert(appliedFrames == 2, "options applied to native and script plates")
        end
        "#,
    )
    .expect("Mists option update with isolated class bars should publish Small scales and explicit sizes");
}
