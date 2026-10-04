//! 12.1.0 managed AuraButton tooltips (B12d): automatic tooltips, anchors,
//! combat hiding, 200ms refresh throttle, global tooltip styling, and
//! visibility changes while hovered.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{
    assert_no_lua_errors, aura, frame_id, load_env, set_player_auras, settle,
};
use rilua::Val;
use wow_ui_sim::lua_api::WowLuaEnv;

/// `AuditTooltipText()` is `shown|anchor|line1` for the shared aura tooltip.
const TOOLTIP_HELPERS: &str = r#"
    AuditAuraTooltip = __secureenv.AuraContainerUtil.GetDefaultTooltip()
    function AuditTooltipText()
        local tooltip = AuditAuraTooltip
        local line = ''
        for _, region in ipairs({tooltip:GetRegions()}) do
            if region:GetObjectType() == 'FontString' and region:GetName() == 'AuraButtonTooltipTextLeft1' then
                line = tostring(AuditPlain(region:GetText()))
            end
        end
        if not AuditPlain(tooltip:IsShown()) then return 'hidden' end
        return string.format('shown|%s|%s', tostring(tooltip:GetAnchorType()), line)
    end
"#;

fn tooltip_env() -> WowLuaEnv {
    let env = load_env();
    env.exec(TOOLTIP_HELPERS).unwrap();
    set_player_auras(&env, vec![aura(401)]);
    env
}

/// Engine hover dispatch after hit-testing resolves the button.
fn enter(env: &WowLuaEnv, button: u64) {
    env.fire_script_handler(button, "OnEnter", vec![Val::Bool(true)])
        .unwrap();
}

fn leave(env: &WowLuaEnv, button: u64) {
    env.fire_script_handler(button, "OnLeave", vec![Val::Bool(true)])
        .unwrap();
}

fn tooltip(env: &WowLuaEnv) -> String {
    env.eval::<String>("return AuditTooltipText()").unwrap()
}

fn hovered_button(env: &WowLuaEnv, initializer: &str) -> u64 {
    env.exec(&format!(
        "AuditTipHost = AuditContainer({{{{'buffs', 'HELPFUL', {{initializeFrame = function(frame)
            AuditInitIcon(frame)
            {initializer}
        end}}}}}})"
    ))
    .unwrap();
    settle(&env);
    frame_id(env, "AuditShown(AuditTipHost, 'buffs')[1].frame")
}

#[test]
fn hovering_a_managed_button_shows_its_aura_tooltip_until_leave() {
    let env = tooltip_env();
    let button = hovered_button(&env, "");
    assert_eq!(tooltip(&env), "hidden");
    enter(&env, button);
    assert_eq!(tooltip(&env), "shown|ANCHOR_BOTTOMLEFT|Audit Aura 401");
    assert!(
        env.eval::<bool>("return AuditAuraTooltip:IsOwned(AuditShown(AuditTipHost, 'buffs')[1].frame)")
            .unwrap()
    );
    leave(&env, button);
    assert_eq!(tooltip(&env), "hidden");
    assert_no_lua_errors(&env);
}

#[test]
fn tooltip_anchor_point_is_configurable_per_button() {
    let env = tooltip_env();
    let button = hovered_button(&env, "frame:SetTooltipAnchorPoint('ANCHOR_TOPRIGHT', 4, -6)");
    enter(&env, button);
    assert_eq!(tooltip(&env), "shown|ANCHOR_TOPRIGHT|Audit Aura 401");
    env.exec("AuditHovered = AuditShown(AuditTipHost, 'buffs')[1].frame")
        .unwrap();
    let anchor = env
        .eval::<String>(
            "return AuditAddon(function()
                return string.format('%s,%d,%d', AuditHovered:GetTooltipAnchorPoint())
            end)",
        )
        .unwrap();
    assert_eq!(anchor, "ANCHOR_TOPRIGHT,4,-6");
    let invalid = env
        .eval::<bool>(
            "return AuditAddon(function()
                return pcall(AuditHovered.SetTooltipAnchorPoint, AuditHovered, 'ANCHOR_SIDEWAYS')
            end)",
        )
        .unwrap();
    assert!(!invalid, "unknown anchor names are rejected");
    assert_no_lua_errors(&env);
}

#[test]
fn hide_in_combat_suppresses_tooltips_only_while_in_combat() {
    let env = tooltip_env();
    let button = hovered_button(&env, "frame:SetHideTooltipInCombat(true)");
    env.state().borrow_mut().player.in_combat = true;
    enter(&env, button);
    assert_eq!(tooltip(&env), "hidden");
    leave(&env, button);
    env.state().borrow_mut().player.in_combat = false;
    enter(&env, button);
    assert_eq!(tooltip(&env), "shown|ANCHOR_BOTTOMLEFT|Audit Aura 401");
    // Entering combat while hovered hides the tooltip at its next refresh.
    env.state().borrow_mut().player.in_combat = true;
    env.fire_on_update(0.25).unwrap();
    assert_eq!(tooltip(&env), "hidden");
    assert_no_lua_errors(&env);
}

#[test]
fn hovered_tooltip_refreshes_at_most_every_200ms() {
    let env = tooltip_env();
    let button = hovered_button(&env, "");
    enter(&env, button);
    env.fire_on_update(0.25).unwrap();
    assert_eq!(tooltip(&env), "shown|ANCHOR_BOTTOMLEFT|Audit Aura 401");
    env.state().borrow_mut().player.buffs[0].name = "Renamed Aura".into();
    env.fire_on_update(0.1).unwrap();
    assert_eq!(
        tooltip(&env),
        "shown|ANCHOR_BOTTOMLEFT|Audit Aura 401",
        "no refresh within 200ms"
    );
    env.fire_on_update(0.11).unwrap();
    assert_eq!(tooltip(&env), "shown|ANCHOR_BOTTOMLEFT|Renamed Aura");
    assert_no_lua_errors(&env);
}

#[test]
fn toggling_container_visibility_while_hovered_raises_no_error() {
    let env = tooltip_env();
    let button = hovered_button(&env, "");
    enter(&env, button);
    for _ in 0..2 {
        env.exec("AuditAddon(function() AuditTipHost:Hide() end)")
            .unwrap();
        env.fire_on_update(0.25).unwrap();
        env.exec("AuditAddon(function() AuditTipHost:Show() end)")
            .unwrap();
        settle(&env);
        env.fire_on_update(0.25).unwrap();
    }
    leave(&env, button);
    assert_eq!(tooltip(&env), "hidden");
    assert_no_lua_errors(&env);
}

#[test]
fn global_tooltip_style_apis_restyle_the_shared_aura_tooltip() {
    let env = tooltip_env();
    let button = hovered_button(&env, "");
    let style = |call: &str| {
        env.exec(&format!("AuditAddon(function() {call} end)"))
            .unwrap();
        enter(&env, button);
        env.eval::<String>(
            r#"
            local tooltip = AuditAuraTooltip
            local function shown(region) return region ~= nil and AuditPlain(region:IsShown()) == true end
            local slice = tooltip.TextureSliceBackground
            return string.format('nine=%s slice=%s:%s backdrop=%s',
                tostring(shown(tooltip.NineSlice)),
                tostring(shown(slice)), tostring(slice and AuditPlain(slice:GetTexture())),
                tostring(shown(tooltip.BackdropContainer)))
        "#,
        )
        .unwrap()
    };
    assert_eq!(
        style(
            "AuraContainerInbound.SetTooltipTextureSlice({asset = 'Interface\\\\Tooltips\\\\UI-Tooltip-Background'})"
        ),
        "nine=false slice=true:137056 backdrop=false"
    );
    assert_eq!(
        style(
            "AuraContainerInbound.SetTooltipBackdrop({backdropInfo = {bgFile = 'Interface\\\\Tooltips\\\\UI-Tooltip-Background'}})"
        ),
        "nine=false slice=false:nil backdrop=true"
    );
    assert_eq!(
        style("AuraContainerInbound.ResetTooltipStyle()"),
        "nine=true slice=false:nil backdrop=false"
    );
    assert_no_lua_errors(&env);
}
