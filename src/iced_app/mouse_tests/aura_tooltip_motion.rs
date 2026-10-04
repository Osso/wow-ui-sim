//! 12.1.0 "Aura tooltips: Automatically enabled but can be disabled via the
//! SetMouseMotionEnabled API." Real hover hit-testing over managed AuraButtons
//! created by the cached Blizzard_AuraContainer from addon code.
use super::*;
use crate::lua_api::state::AuraInfo;

fn load_aura_container_addon(app: &App) {
    let ui = crate::blizzard_ui_sync::default_cache_addons_path().expect("Blizzard UI cache");
    let env = app.env.borrow();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    let addons = crate::loader::discover_blizzard_addon_closure_for_screen_with_overrides(
        &ui,
        ScreenKind::Game,
        &["Blizzard_AuraContainer"],
        &[],
    );
    for (name, toc) in addons {
        let already_loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|addon| addon.folder_name == name);
        if !already_loaded {
            crate::loader::load_addon(&env.loader_env(), &toc)
                .unwrap_or_else(|error| panic!("{name} should load: {error}"));
        }
    }
}

fn player_aura(id: i32) -> AuraInfo {
    AuraInfo {
        name: format!("Hover Aura {id}"),
        spell_id: id,
        icon: id,
        duration: 30.0,
        expiration_time: 30.0,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: true,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: id,
    }
}

/// Addon code creates a player buff container at `x` whose buttons optionally
/// disable mouse motion in `initializeFrame`.
fn create_container(app: &App, global: &str, x: i32, disable_motion: bool) {
    app.env
        .borrow()
        .exec(&format!(
            r#"
            local function addon()
                local container = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                container:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', {x}, -100)
                container:SetUnit('player')
                local function init(frame)
                    frame:SetSize(20, 20)
                    local icon = frame:CreateTexture(nil, 'BACKGROUND')
                    icon:SetAllPoints(frame)
                    frame:SetIcon(icon)
                    if {disable_motion} then frame:SetMouseMotionEnabled(false) end
                end
                container:AddAuraGroup('buffs', 'HELPFUL', {{initializeFrame = init}})
                return container
            end
            debug.setobjecttaint(addon, 'HoverAuraAddon')
            {global} = addon()
            HoverTooltip = __secureenv.AuraContainerUtil.GetDefaultTooltip()
            function HoverTooltipShown()
                local shown = HoverTooltip:IsShown()
                if issecretvalue(shown) then shown = secretunwrap(shown) end
                return shown
            end
            function ShownAuraButton(container)
                for index = 1, container:GetAuraGroupFrameCount('buffs') do
                    local frame = container:GetAuraGroupFrame('buffs', index)
                    local shown = frame:IsShown()
                    if issecretvalue(shown) then shown = secretunwrap(shown) end
                    if shown then return frame end
                end
            end
            "#
        ))
        .expect("addon creates a managed aura container");
}

fn hover(app: &mut App, x: f32, y: f32) {
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(Point::new(
        x, y,
    ))));
}

#[test]
fn managed_aura_button_tooltip_follows_hover_unless_motion_is_disabled() {
    let mut app = build_test_app(ScreenKind::Game);
    load_aura_container_addon(&app);
    {
        let env = app.env.borrow();
        env.state().borrow_mut().player.buffs = vec![player_aura(101)];
        env.fire_unit_aura_full_update("player").unwrap();
    }
    create_container(&app, "HoverDefault", 100, false);
    create_container(&app, "HoverNoMotion", 300, true);
    app.env.borrow().fire_on_update(0.016).unwrap();
    rebuild_hittable_cache(&app);

    hover(&mut app, 110.0, 110.0);
    app.env
        .borrow()
        .exec(
            r#"
            local button = ShownAuraButton(HoverDefault)
            assert(button:IsMouseMotionEnabled())
            assert(HoverTooltipShown(), 'hovering a managed AuraButton shows its tooltip')
            assert(HoverTooltip:IsOwned(button))
            "#,
        )
        .expect("default aura tooltip");

    hover(&mut app, 500.0, 500.0);
    hover(&mut app, 310.0, 110.0);
    app.env
        .borrow()
        .exec(
            r#"
            local button = ShownAuraButton(HoverNoMotion)
            assert(not button:IsMouseMotionEnabled() and button:IsMouseClickEnabled())
            assert(not HoverTooltipShown(), 'motion-disabled AuraButtons get no tooltip')
            "#,
        )
        .expect("motion-disabled aura tooltip");
    assert!(app.env.borrow().state().borrow().lua_errors.is_empty());
}
