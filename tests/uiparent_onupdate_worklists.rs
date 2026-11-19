use crate::common;

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_settled_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];

    common::load_profile_framexml(&env);
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("Failed to load Blizzard addon {name}: {err}"));
    }

    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

#[test]
fn uiparent_onupdate_skips_empty_worklists() {
    test_timeout! {
        let env = load_settled_game_ui();

        let (chat_pairs, pulse_pairs, shine_pairs): (i32, i32, i32) = env
            .eval(
                r#"
                local counts = { chat = 0, pulse = 0, shine = 0 }
                local emptyChatFrames = {}
                local emptyPulseButtons = {}
                local emptyShines = {}

                local originalPairs = pairs
                pairs = function(t)
                    if t == emptyChatFrames then
                        counts.chat = counts.chat + 1
                    elseif t == emptyPulseButtons then
                        counts.pulse = counts.pulse + 1
                    elseif t == emptyShines then
                        counts.shine = counts.shine + 1
                    end
                    return originalPairs(t)
                end

                local originalChatFrames = CHAT_FRAMES
                local originalPulseButtons = PULSEBUTTONS
                local originalShines = SHINES_TO_ANIMATE

                CHAT_FRAMES = emptyChatFrames
                PULSEBUTTONS = emptyPulseButtons
                SHINES_TO_ANIMATE = emptyShines

                local script = assert(UIParent:GetScript("OnUpdate"), "missing UIParent OnUpdate")
                script(UIParent, 0.016)

                CHAT_FRAMES = originalChatFrames
                PULSEBUTTONS = originalPulseButtons
                SHINES_TO_ANIMATE = originalShines
                pairs = originalPairs

                return counts.chat, counts.pulse, counts.shine
                "#,
            )
            .unwrap();

        assert_eq!(
            chat_pairs, 0,
            "UIParent OnUpdate should skip FCF_OnUpdate when CHAT_FRAMES is empty"
        );
        assert_eq!(
            pulse_pairs, 0,
            "UIParent OnUpdate should skip ButtonPulse_OnUpdate when PULSEBUTTONS is empty"
        );
        assert_eq!(
            shine_pairs, 0,
            "UIParent OnUpdate should skip AnimatedShine_OnUpdate when SHINES_TO_ANIMATE is empty"
        );
    }
}

#[test]
fn button_pulse_onupdate_still_updates_active_buttons() {
    test_timeout! {
        let env = load_settled_game_ui();

        env.exec(
            r#"
            local button = CreateFrame("Button", "ButtonPulseProbe", UIParent)
            button:Show()
            SetButtonPulse(button, 1.0, -0.2)
            "#,
        )
        .unwrap();

        env.fire_on_update(0.016).unwrap();

        let (highlight_locked, pulse_on, pulse_time_left): (bool, i32, f64) = env
            .eval(
                r#"
                return ButtonPulseProbe:IsHighlightLocked(), ButtonPulseProbe.pulseOn, ButtonPulseProbe.pulseTimeLeft
                "#,
            )
            .unwrap();

        assert!(highlight_locked, "active pulsing button should lock highlight");
        assert_eq!(pulse_on, 1, "active pulsing button should flip pulseOn");
        assert!(
            (0.0..1.0).contains(&pulse_time_left),
            "active pulsing button should consume elapsed time"
        );

        env.exec("ButtonPulse_StopPulse(ButtonPulseProbe)").unwrap();
    }
}

fn assert_legacy_animated_shine(env: &WowLuaEnv) {
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnimatedShineProbe", UIParent)
        owner:SetSize(100, 100)
        local shine = CreateFrame("Frame", "AnimatedShineProbeShine", owner, "AnimatedShineTemplate")
        shine:SetSize(100, 100)
        AnimatedShine_Start(owner)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.016).unwrap();
    let timer: f64 = env.eval("return AnimatedShineProbe.timer").unwrap();
    assert!(
        (timer - 0.016).abs() < f64::EPSILON,
        "active shine should consume elapsed time"
    );
    env.exec(
        r#"
        local point, relative, relativePoint, x, y = AnimatedShineProbeShine1:GetPoint()
        assert(point == "CENTER" and relative == AnimatedShineProbeShine)
        assert(relativePoint == "TOPLEFT" and x > 0 and y == 0)
        AnimatedShine_Stop(AnimatedShineProbe)
        assert(not AnimatedShineProbeShine1:IsShown())
        "#,
    )
    .unwrap();
}

#[test]
fn animated_shine_onupdate_still_updates_active_shines() {
    test_timeout! {
        let env = load_settled_game_ui();

        if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
            assert_legacy_animated_shine(&env);
            return;
        }

        env.exec(
            r#"
            local shine = CreateFrame("Frame", "AnimatedShineProbe", UIParent)
            shine:SetSize(100, 100)
            shine:Show()
            Mixin(shine, AnimatedShineMixin)

            for index = 1, 4 do
                local texture = shine:CreateTexture(nil, "ARTWORK")
                texture:SetSize(10, 10)
                shine["Shine" .. index] = texture
            end

            shine:Start()
            "#,
        )
        .unwrap();

        env.fire_on_update(0.016).unwrap();

        let (timer, point, relative_point, x, y): (f64, String, String, f64, f64) = env
            .eval(
                r#"
                local point, _, relativePoint, x, y = AnimatedShineProbe.Shine1:GetPoint()
                return AnimatedShineProbe.timer, point, relativePoint, x, y
                "#,
            )
            .unwrap();

        assert!((timer - 0.016).abs() < f64::EPSILON, "active shine should consume elapsed time");
        assert_eq!(point, "CENTER", "active shine should update its anchor point");
        assert_eq!(relative_point, "TOPLEFT", "active shine should follow the first animation path");
        assert!(x > 0.0, "active shine should move along its first animation path");
        assert_eq!(y, 0.0, "first animation path should not offset vertically");

        env.exec("AnimatedShineProbe:Stop()").unwrap();
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn selected_floating_chat_onupdate_preserves_original_consumer() {
    test_timeout! {
        let env = load_settled_game_ui();
        env.exec(r#"
            local callback = assert(EventRegistry.onUpdateCallbacksByOwner[FloatingChatFrameMixin])
            local savedFrames, savedMoving = CHAT_FRAMES, MOVING_CHATFRAME
            CHAT_FRAMES = {}
            callback(FloatingChatFrameMixin, 0.125)
            local frame, tab = ChatFrame1, ChatFrame1Tab
            local savedDocked = frame.isDocked
            frame:Show()
            CHAT_FRAMES, MOVING_CHATFRAME = { "ChatFrame1" }, frame
            for _, docked in ipairs({ false, true }) do
                frame.isDocked, frame.hasBeenFaded, frame.mouseOutTime = docked, false, 12
                tab:SetAlpha(0)
                callback(FloatingChatFrameMixin, 0.125)
                for tick = 1, 10 do UIFrameFade_OnUpdate(frame, 0.05) end
                assert(frame.mouseOutTime == 0, "moving chat must reset fade timer")
                assert(tab:GetAlpha() > 0, "original chat callback must fade in its tab")
            end
            frame.isDocked = savedDocked
            CHAT_FRAMES, MOVING_CHATFRAME = savedFrames, savedMoving
        "#).expect("selected original FloatingChatFrame OnUpdate consumer must run");
    }
}
