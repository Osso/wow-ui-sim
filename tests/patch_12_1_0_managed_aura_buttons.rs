//! 12.1.0 managed AuraButton presentation (B12c): dispel textures and text,
//! application count and bar, duration displays, pandemic regions, click
//! cancellation, and post-configuration ownership rules.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{
    assert_no_lua_errors, aura, debuff, frame_id, load_env, set_player_auras, settle,
};
use wow_ui_sim::lua_api::{WowLuaEnv, state::AuraInfo};

/// - `AuditPresenter(configure)` creates a container with a `buffs` and a
///   `debuffs` group; every button gets an icon and is then passed to the
///   addon-owned `configure(frame)`.
/// - `AuditButtonFor(container, icon)` is the shown button displaying `icon`.
const PRESENTATION_HELPERS: &str = r#"
    function AuditPresenter(configure)
        local function initialize(frame)
            AuditInitIcon(frame)
            configure(frame)
        end
        debug.setobjecttaint(configure, AUDIT_ADDON)
        local options = {sortMethod = AuraContainerSortMethod.AuraInstanceIDOnly, initializeFrame = initialize}
        return AuditContainer({
            {'buffs', 'HELPFUL', options},
            {'debuffs', 'HARMFUL', CopyTable(options)},
        })
    end
    function AuditButtonFor(container, icon)
        for _, key in ipairs({'buffs', 'debuffs'}) do
            for _, entry in ipairs(AuditShown(container, key)) do
                if entry.icon == icon then return entry.frame end
            end
        end
    end
    function AuditRGB(region)
        local r, g, b = region:GetVertexColor()
        return string.format('%.1f/%.1f/%.1f', AuditPlain(r), AuditPlain(g), AuditPlain(b))
    end
"#;

fn stealable(mut aura: AuraInfo) -> AuraInfo {
    aura.is_stealable = true;
    aura
}

/// 301 helpful without dispel type, 302 stealable Magic, 303 Curse, 304 untyped debuff.
fn dispel_fixture() -> Vec<AuraInfo> {
    vec![
        aura(301),
        stealable(debuff(302, Some("Magic"))),
        debuff(303, Some("Curse")),
        debuff(304, None),
    ]
}

fn presentation_env(auras: Vec<AuraInfo>) -> WowLuaEnv {
    let env = load_env();
    env.exec(PRESENTATION_HELPERS).unwrap();
    set_player_auras(&env, auras);
    env
}

const STYLE: &str = "Enum.CustomAuraButtonDispelTypeTextureStyle";
const STEALABLE: &str = "Enum.CustomAuraButtonDispelTypeStealableFilter";

/// (case, AddDispelTypeTexture options, readout per aura 301..304: atlas or
/// vertex colour when shown, `-` when hidden)
fn dispel_texture_cases() -> Vec<(&'static str, String, &'static str)> {
    vec![
        (
            "default border with icon on typed debuffs",
            "nil".into(),
            "- ui-debuff-border-magic-icon ui-debuff-border-curse-icon -",
        ),
        (
            "border style",
            format!("{{style = {STYLE}.Border}}"),
            "- ui-debuff-border-magic-noicon ui-debuff-border-curse-noicon -",
        ),
        (
            "icon style",
            format!("{{style = {STYLE}.Icon}}"),
            "- RaidFrame-Icon-DebuffMagic RaidFrame-Icon-DebuffCurse -",
        ),
        (
            "stealable only",
            format!("{{stealableFilter = {STEALABLE}.Stealable}}"),
            "- ui-debuff-border-magic-icon - -",
        ),
        (
            "not stealable only",
            format!("{{stealableFilter = {STEALABLE}.NotStealable}}"),
            "- - ui-debuff-border-curse-icon -",
        ),
        (
            "show always",
            "{showAlways = true}".into(),
            "ui-debuff-border-default-noicon ui-debuff-border-magic-icon ui-debuff-border-curse-icon ui-debuff-border-default-noicon",
        ),
        (
            "helpful and untyped opt-ins",
            "{showWhenHelpful = true, showWithoutDispelType = true, showWhenHarmful = false}".into(),
            "ui-debuff-border-default-noicon - - -",
        ),
        (
            "colour map",
            format!(
                "{{style = {STYLE}.PreserveAsset, customDispelColorMap = {{Magic = CreateColor(0, 0, 1, 1), Curse = CreateColor(1, 0, 1, 1)}}}}"
            ),
            "- 0.0/0.0/1.0 1.0/0.0/1.0 -",
        ),
        (
            "colour curve",
            format!("{{style = {STYLE}.PreserveAsset, customDispelColorCurve = AuditDispelCurve}}"),
            "- 0.1/0.2/0.3 0.2/0.4/0.6 -",
        ),
    ]
}

#[test]
fn dispel_type_textures_follow_options_for_each_aura() {
    let env = presentation_env(dispel_fixture());
    env.exec(
        r#"
        AuditDispelCurve = C_CurveUtil.CreateColorCurve()
        AuditDispelCurve:SetType(Enum.LuaCurveType.Step)
        for x = 0, 4 do AuditDispelCurve:AddPoint(x, CreateColor(x / 10, x / 5, 3 * x / 10, 1)) end
        function AuditDispelReadout(container)
            local parts = {}
            for _, icon in ipairs({301, 302, 303, 304}) do
                local texture = AuditButtonFor(container, icon):GetDispelTypeTexture(1)
                local value = '-'
                if AuditPlain(texture:IsShown()) then
                    value = AuditPlain(texture:GetAtlas()) or AuditRGB(texture)
                end
                parts[#parts + 1] = value
            end
            return table.concat(parts, ' ')
        end
    "#,
    )
    .unwrap();
    let cases = dispel_texture_cases();
    for (index, (_, options, _)) in cases.iter().enumerate() {
        env.exec(&format!(
            "AuditDispel{index} = AuditPresenter(function(frame)
                local texture = frame:CreateTexture(nil, 'OVERLAY')
                texture:SetAllPoints(frame)
                frame:AddDispelTypeTexture(texture, {options})
            end)"
        ))
        .unwrap();
    }
    settle(&env);
    let mismatches: Vec<String> = cases
        .iter()
        .enumerate()
        .filter_map(|(index, (case, _, expected))| {
            let actual = env
                .eval::<String>(&format!("return AuditDispelReadout(AuditDispel{index})"))
                .unwrap();
            (actual != *expected).then(|| format!("{case}: expected {expected:?}, got {actual:?}"))
        })
        .collect();
    assert!(mismatches.is_empty(), "{mismatches:#?}");
    assert_no_lua_errors(&env);
}

#[test]
fn multiple_dispel_textures_and_set_aura_border_alias() {
    let env = presentation_env(dispel_fixture());
    env.exec(&format!(
        r#"
        AuditMulti = AuditPresenter(function(frame)
            local border = frame:CreateTexture(nil, 'OVERLAY')
            frame:AddDispelTypeTexture(border, {{style = {STYLE}.Border}})
            local badge = frame:CreateTexture(nil, 'OVERLAY')
            frame:AddDispelTypeTexture(badge, {{style = {STYLE}.Icon, stealableFilter = {STEALABLE}.Stealable}})
        end)
        AuditAlias = AuditPresenter(function(frame)
            frame:SetAuraBorder(frame:CreateTexture(nil, 'OVERLAY'), {{style = {STYLE}.Icon}})
            frame:SetAuraBorder(frame:CreateTexture(nil, 'OVERLAY'), {{style = {STYLE}.Border}})
        end)
    "#
    ))
    .unwrap();
    settle(&env);
    let readout = env
        .eval::<String>(
            r#"
            local parts = {}
            for _, icon in ipairs({302, 303}) do
                local button = AuditButtonFor(AuditMulti, icon)
                local states = {button:GetDispelTypeTextureCount()}
                for index = 1, button:GetDispelTypeTextureCount() do
                    local texture = button:GetDispelTypeTexture(index)
                    states[#states + 1] = AuditPlain(texture:IsShown()) and AuditPlain(texture:GetAtlas()) or '-'
                end
                parts[#parts + 1] = table.concat(states, ',')
            end
            local alias = AuditButtonFor(AuditAlias, 302)
            local border = alias:GetAuraBorder()
            parts[#parts + 1] = alias:GetDispelTypeTextureCount() .. ',' .. AuditPlain(border:GetAtlas())
            return table.concat(parts, ' ')
        "#,
        )
        .unwrap();
    assert_eq!(
        readout,
        "2,ui-debuff-border-magic-noicon,RaidFrame-Icon-DebuffMagic \
         2,ui-debuff-border-curse-noicon,- \
         1,ui-debuff-border-magic-noicon"
    );
    assert_no_lua_errors(&env);
}

#[test]
fn aura_symbol_text_validates_its_own_region_and_maps_dispel_types() {
    let env = presentation_env(dispel_fixture());
    env.exec(
        r#"
        AuditSymbol = AuditPresenter(function(frame)
            local text = frame:CreateFontString(nil, 'OVERLAY', 'GameFontNormal')
            frame:SetAuraSymbol(text, {customDispelTextMap = {Magic = 'M!', Curse = 'C!'}})
        end)
        AuditSymbolErrors = {}
        AuditPresenter(function(frame)
            local ok, err = pcall(frame.SetAuraSymbol, frame, frame:CreateTexture())
            AuditSymbolErrors[#AuditSymbolErrors + 1] = ok and 'accepted' or err
        end)
    "#,
    )
    .unwrap();
    settle(&env);
    let readout = env
        .eval::<String>(
            r#"
            local parts = {}
            for _, icon in ipairs({301, 302, 303, 304}) do
                local text = AuditButtonFor(AuditSymbol, icon):GetAuraSymbol()
                parts[#parts + 1] = AuditPlain(text:IsShown()) and AuditPlain(text:GetText()) or '-'
            end
            return table.concat(parts, ' ')
        "#,
        )
        .unwrap();
    assert_eq!(readout, "- M! C! -");
    let error = env
        .eval::<String>("return AuditSymbolErrors[1]")
        .unwrap();
    assert!(
        error.contains("expected object type 'FontString', got 'Texture'"),
        "{error}"
    );
    assert_no_lua_errors(&env);
}

#[test]
fn application_count_and_bar_track_applications() {
    let mut stacked = aura(311);
    stacked.applications = 3;
    let env = presentation_env(vec![stacked, aura(312)]);
    env.exec(
        r#"
        AuditStacks = AuditPresenter(function(frame)
            frame:SetApplicationCount(frame:CreateFontString(nil, 'OVERLAY', 'GameFontNormal'))
            frame:SetApplicationBar(CreateFrame('StatusBar', nil, frame), {maxApplications = 5})
        end)
    "#,
    )
    .unwrap();
    settle(&env);
    let readout = env
        .eval::<String>(
            r#"
            local parts = {}
            for _, icon in ipairs({311, 312}) do
                local button = AuditButtonFor(AuditStacks, icon)
                local count = button:GetApplicationCount()
                local bar, options = button:GetApplicationBar()
                local low, high = bar:GetMinMaxValues()
                parts[#parts + 1] = string.format('%q %d/%d..%d max=%d', tostring(AuditPlain(count:GetText())),
                    AuditPlain(bar:GetValue()), AuditPlain(low), AuditPlain(high), options.maxApplications)
            end
            return table.concat(parts, ' ')
        "#,
        )
        .unwrap();
    assert_eq!(readout, r#""3" 3/0..5 max=5 "" 1/0..5 max=5"#);
    assert_no_lua_errors(&env);
}

#[test]
fn duration_cooldown_and_text_follow_assigned_aura() {
    let env = presentation_env(vec![]);
    let now = env.eval::<f64>("return GetTime()").unwrap();
    let timed = |id: i32, duration: f64, remaining: f64| AuraInfo {
        duration,
        expiration_time: now + remaining,
        ..aura(id)
    };
    let permanent = AuraInfo {
        duration: 0.0,
        expiration_time: 0.0,
        ..aura(322)
    };
    set_player_auras(&env, vec![timed(321, 30.0, 25.0), permanent]);
    env.exec(
        r#"
        AuditDuration = AuditPresenter(function(frame)
            frame:SetDurationCooldown(CreateFrame('Cooldown', nil, frame, 'CooldownFrameTemplate'))
            frame:SetDurationText(frame:CreateFontString(nil, 'OVERLAY', 'GameFontNormal'))
        end)
        function AuditDurationReadout()
            local parts = {}
            for _, icon in ipairs({321, 322}) do
                local button = AuditButtonFor(AuditDuration, icon)
                local start, duration = button:GetDurationCooldown():GetCooldownTimes()
                local text = AuditPlain(button:GetDurationText():GetText())
                parts[#parts + 1] = string.format('%d:%d:%q', icon, AuditPlain(duration) / 1000, tostring(text))
            end
            return table.concat(parts, ' ')
        end
    "#,
    )
    .unwrap();
    settle(&env);
    settle(&env);
    assert_eq!(
        env.eval::<String>("return AuditDurationReadout()").unwrap(),
        r#"321:30:"24s" 322:0:"nil""#
    );
    assert_no_lua_errors(&env);
}

#[test]
fn pandemic_regions_show_only_inside_the_refresh_window() {
    let env = presentation_env(vec![]);
    let now = env.eval::<f64>("return GetTime()").unwrap();
    for spell in [331, 332] {
        env.state().borrow_mut().spell_aura_durations.insert(
            spell,
            wow_ui_sim::c_api::aura_duration::SpellAuraDuration {
                base_duration_seconds: 30.0,
                max_carryover_seconds: 9.0,
            },
        );
    }
    let timed = |id: i32, remaining: f64| AuraInfo {
        duration: 30.0,
        expiration_time: now + remaining,
        ..aura(id)
    };
    set_player_auras(&env, vec![timed(331, 5.0), timed(332, 25.0), aura(333)]);
    env.exec(
        r#"
        AuditPandemic = AuditPresenter(function(frame)
            assert(frame:AddPandemicRegion(frame:CreateTexture(nil, 'OVERLAY')) == 1)
        end)
    "#,
    )
    .unwrap();
    settle(&env);
    settle(&env);
    let readout = env
        .eval::<String>(
            r#"
            local parts = {}
            for _, icon in ipairs({331, 332, 333}) do
                local glow
                for _, region in ipairs({AuditButtonFor(AuditPandemic, icon):GetRegions()}) do
                    if region:GetDrawLayer() == 'OVERLAY' then glow = region end
                end
                parts[#parts + 1] = icon .. '=' .. tostring(AuditPlain(glow:IsShown()))
            end
            return table.concat(parts, ' ')
        "#,
        )
        .unwrap();
    assert_eq!(readout, "331=true 332=false 333=false");
    assert_no_lua_errors(&env);
}

#[test]
fn cancel_aura_buttons_cancel_only_on_configured_clicks() {
    let env = presentation_env(vec![aura(341), aura(342)]);
    env.exec(
        r#"
        AuditCancel = AuditPresenter(function(frame)
            frame:SetCancelAuraButtons('RightButtonUp')
        end)
    "#,
    )
    .unwrap();
    settle(&env);
    let button = frame_id(&env, "AuditButtonFor(AuditCancel, 341)");
    let click = |which: &str| {
        let token = env.lua_string(which);
        env.fire_script_handler(button, "OnClick", vec![token, rilua::Val::Bool(false)])
            .unwrap();
        settle(&env);
    };
    click("LeftButton");
    let remaining = || {
        env.state()
            .borrow()
            .player
            .buffs
            .iter()
            .map(|aura| aura.aura_instance_id)
            .collect::<Vec<_>>()
    };
    assert_eq!(remaining(), [341, 342], "left click is not a cancel click");
    click("RightButton");
    assert_eq!(remaining(), [342], "right click cancels the displayed aura");
    assert_eq!(
        crate::common::aura_container_harness::icons(&env, "AuditCancel", "buffs"),
        "342"
    );
    assert_no_lua_errors(&env);
}

#[test]
fn addons_configure_buttons_after_initialization_but_cannot_reparent_components() {
    let env = presentation_env(vec![aura(351)]);
    env.exec("AuditLate = AuditPresenter(function() end)")
        .unwrap();
    settle(&env);
    let readout = env
        .eval::<String>(
            r#"
            local button = AuditButtonFor(AuditLate, 351)
            local name = AuditAddon(function()
                local text = button:CreateFontString(nil, 'OVERLAY', 'GameFontNormal')
                button:SetSpellName(text)
                return text
            end)
            local results = {AuditPlain(name:GetText())}
            AuditAddon(function()
                local icon = button:GetIcon()
                for _, target in ipairs({UIParent, button}) do
                    local ok = pcall(icon.SetParent, icon, target)
                    results[#results + 1] = ok and 'reparented' or 'blocked'
                end
                results[#results + 1] = tostring(icon:GetParent() == button)
            end)
            return table.concat(results, ' ')
        "#,
        )
        .unwrap();
    assert_eq!(readout, "Audit Aura 351 blocked blocked true");
    assert_no_lua_errors(&env);
}
