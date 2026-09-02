#![cfg(feature = "gui")]

//! Integration test for the Blizzard chat frame.
//!
//! Loads the Blizzard UI, clicks on ChatFrame1EditBox, types a message,
//! presses Enter, and verifies the message was submitted via
//! C_ChatInfo.SendChatMessage.

use crate::common;
#[path = "chat_frame/layout_lock.rs"]
mod layout_lock;
#[path = "chat_frame/scrollbar.rs"]
mod scrollbar;

#[cfg(feature = "gui")]
use std::cell::RefCell;
use std::path::PathBuf;
#[cfg(feature = "gui")]
use std::rc::Rc;
#[cfg(feature = "gui")]
use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry, compute_frame_rect,
};
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(feature = "gui")]
use wow_ui_sim::render::{GlyphAtlas, QuadBatch, WowFontSystem};

const CHAT_LAYOUT_DEBUG_LUA: &str = r#"
    local frames = {
        {"ChatFrame1", ChatFrame1},
        {"ChatFrame1Background", ChatFrame1Background},
        {"ChatFrame1.ResizeButton", ChatFrame1.ResizeButton},
        {"ChatFrame1.ScrollToBottomButton", ChatFrame1.ScrollToBottomButton},
        {"ChatFrame1.ScrollBar", ChatFrame1.ScrollBar},
        {"ChatFrame1EditBox", ChatFrame1EditBox},
    }

    local out = {}
    for _, item in ipairs(frames) do
        local label, frame = item[1], item[2]
        if frame then
            local x, y, w, h = frame:GetRect()
            local points = {}
            for i = 1, frame:GetNumPoints() do
                local point, rel, relPoint, ox, oy = frame:GetPoint(i)
                local relName = rel and rel:GetName() or "$parent"
                table.insert(points, string.format("%s->%s:%s(%.0f,%.0f)", point, relName, relPoint, ox, oy))
            end
            table.sort(points)
            table.insert(
                out,
                string.format(
                    "%s rect=(%.0f,%.0f %.0fx%.0f) shown=%s points=%s",
                    label,
                    x or -1,
                    y or -1,
                    w or -1,
                    h or -1,
                    tostring(frame:IsShown()),
                    table.concat(points, " | ")
                )
            )
        else
            table.insert(out, label .. " <nil>")
        end
    end
    return table.concat(out, "\n")
"#;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Create a fully loaded environment with all Blizzard addons and startup events.
fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    let addons = discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }

    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

/// Fire startup events (same sequence as main.rs).
fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::call_global_if_present(env, "RequestTimePlayed");
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

#[derive(Debug)]
struct ChatVoiceButtonSurface {
    button_width: f64,
    button_height: f64,
    icon_width: f64,
    icon_height: f64,
    icon_points: f64,
    point: String,
    relative_name: String,
    relative_point: String,
    offset_x: f64,
    offset_y: f64,
    normal_atlas: String,
    icon_atlas_count: f64,
}

type ChatVoiceButtonSurfaceRow = (
    f64,
    f64,
    f64,
    f64,
    f64,
    String,
    String,
    String,
    f64,
    f64,
    String,
    f64,
);

const CHAT_VOICE_BUTTON_SURFACE_LUA: &str = r#"
    local button = ChatFrameChannelButton
    assert(button, "ChatFrameChannelButton should exist")
    local icon = button.Icon
    assert(icon, "ChatFrameChannelButton.Icon should exist")
    local point, relativeTo, relativePoint, offsetX, offsetY = icon:GetPoint(1)
    local iconAtlasCount = 0
    for _, child in ipairs({ button:GetRegions() }) do
        -- every state icon (voicechat / headset / ...): Blizzard keeps exactly
        -- one .Icon; a seeded second one under it showed the headset's green disc
        if child:GetObjectType() == "Texture" and child:GetAtlas() and string.find(child:GetAtlas(), "^chatframe%-button%-icon%-") then
            iconAtlasCount = iconAtlasCount + 1
        end
    end
    local normalTexture = button:GetNormalTexture()
    local normalAtlas = normalTexture and normalTexture:GetAtlas() or ""
    return button:GetWidth(),
           button:GetHeight(),
           icon:GetWidth(),
           icon:GetHeight(),
           icon:GetNumPoints(),
           point or "",
           relativeTo and relativeTo:GetName() or "",
           relativePoint or "",
           offsetX or 0,
           offsetY or 0,
           normalAtlas,
           iconAtlasCount
"#;

fn read_chat_voice_button_surface(env: &WowLuaEnv) -> ChatVoiceButtonSurface {
    let row = env
        .eval(CHAT_VOICE_BUTTON_SURFACE_LUA)
        .expect("chat voice button geometry eval failed");
    chat_voice_button_surface_from_row(row)
}

fn chat_voice_button_surface_from_row(
    (
        button_width,
        button_height,
        icon_width,
        icon_height,
        icon_points,
        point,
        relative_name,
        relative_point,
        offset_x,
        offset_y,
        normal_atlas,
        icon_atlas_count,
    ): ChatVoiceButtonSurfaceRow,
) -> ChatVoiceButtonSurface {
    ChatVoiceButtonSurface {
        button_width,
        button_height,
        icon_width,
        icon_height,
        icon_points,
        point,
        relative_name,
        relative_point,
        offset_x,
        offset_y,
        normal_atlas,
        icon_atlas_count,
    }
}

fn chat_layout_debug(env: &WowLuaEnv) -> String {
    env.eval(CHAT_LAYOUT_DEBUG_LUA)
        .expect("chat layout debug eval failed")
}

#[cfg(feature = "gui")]
fn make_font_system() -> Rc<RefCell<WowFontSystem>> {
    Rc::new(RefCell::new(WowFontSystem::new()))
}

#[cfg(feature = "gui")]
fn build_screenshot_like_batch(
    env: &WowLuaEnv,
    width: u32,
    height: u32,
    filter: Option<&str>,
) -> QuadBatch {
    let font_system = make_font_system();
    env.set_font_system(Rc::clone(&font_system));
    env.set_screen_size(width as f32, height as f32);
    wow_ui_sim::startup::run_extra_update_ticks(env, 3);

    let mut glyph_atlas = GlyphAtlas::new();
    let mut font_system = font_system.borrow_mut();
    let buckets = {
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut font_system);
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    let tooltip_data = wow_ui_sim::iced_app::tooltip::collect_tooltip_data(&state);
    build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (width as f32, height as f32), &buckets)
            .root_name(filter)
            .text_ctx(Some((&mut font_system, &mut glyph_atlas)))
            .message_frames(Some(&state.message_frames))
            .tooltip_data(Some(&tooltip_data)),
    )
}

#[cfg(feature = "gui")]
fn quad_bounds(
    batch: &QuadBatch,
    vertex_start: usize,
    vertex_count: usize,
) -> (f32, f32, f32, f32) {
    let verts = &batch.vertices[vertex_start..vertex_start + vertex_count];
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for vert in verts {
        min_x = min_x.min(vert.position[0]);
        min_y = min_y.min(vert.position[1]);
        max_x = max_x.max(vert.position[0]);
        max_y = max_y.max(vert.position[1]);
    }
    (min_x, min_y, max_x, max_y)
}

/// Hook C_ChatInfo.SendChatMessage to capture submitted messages.
fn hook_send_chat_message(env: &WowLuaEnv) {
    env.exec(
        r#"
        _G.__test_sent_messages = {}
        local orig = C_ChatInfo.SendChatMessage
        C_ChatInfo.SendChatMessage = function(msg, chatType, language, target)
            table.insert(_G.__test_sent_messages, {
                message = msg,
                chatType = chatType,
                language = language,
                target = target,
            })
            if orig then orig(msg, chatType, language, target) end
        end
    "#,
    )
    .expect("Failed to hook SendChatMessage");
}

/// Type a string into the focused EditBox character by character.
fn type_text(env: &WowLuaEnv, text: &str) {
    for ch in text.chars() {
        let s = ch.to_string();
        let key = if ch == ' ' {
            "SPACE".to_string()
        } else {
            s.to_uppercase()
        };
        env.send_key_press(&key, Some(&s)).unwrap();
    }
}

/// Click on ChatFrame1EditBox and verify it gains focus.
fn click_chat_editbox(env: &WowLuaEnv) {
    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("ChatFrame1EditBox")
        .expect("ChatFrame1EditBox not found in widget registry");
    env.send_click(frame_id).expect("send_click failed");

    let has_focus: bool = env
        .eval("return ChatFrame1EditBox:HasFocus()")
        .expect("HasFocus failed");
    assert!(has_focus, "ChatFrame1EditBox should have focus after click");
}

/// Assert exactly one message was sent with expected text and chat type.
fn assert_message_sent(env: &WowLuaEnv, expected_text: &str, expected_type: &str) {
    let count: i32 = env
        .eval("return #_G.__test_sent_messages")
        .expect("eval failed");
    assert_eq!(count, 1, "Exactly one message should have been sent");

    let message: String = env
        .eval("return _G.__test_sent_messages[1].message")
        .expect("eval failed");
    assert_eq!(
        message, expected_text,
        "Sent message should match typed text"
    );

    let chat_type: String = env
        .eval("return _G.__test_sent_messages[1].chatType")
        .expect("eval failed");
    assert_eq!(chat_type, expected_type, "Chat type should match expected");

    let text_after: String = env
        .eval("return ChatFrame1EditBox:GetText() or ''")
        .expect("GetText failed");
    assert_eq!(text_after, "", "EditBox should be cleared after submit");
}

#[test]
fn test_chat_editbox_click_type_and_submit() {
    test_timeout! {
        let env = setup_env();

        let exists: bool = env
            .eval("return ChatFrame1EditBox ~= nil")
            .expect("eval failed");
        assert!(exists, "ChatFrame1EditBox should exist after loading Blizzard UI");

        hook_send_chat_message(&env);

        let has_focus: bool = env
            .eval("return ChatFrame1EditBox:HasFocus()")
            .expect("HasFocus failed");
        assert!(!has_focus, "ChatFrame1EditBox should not have focus initially");

        click_chat_editbox(&env);
        type_text(&env, "hello world");

        let text: String = env
            .eval("return ChatFrame1EditBox:GetText()")
            .expect("GetText failed");
        assert_eq!(text, "hello world", "EditBox should contain typed text");

        env.send_key_press("ENTER", None)
            .expect("ENTER key press failed");

        assert_message_sent(&env, "hello world", "SAY");

        let message: String = env
            .eval("return _G.__test_sent_messages[1].message")
            .expect("eval failed");
        assert_eq!(message, "hello world", "Sent message should match typed text");

        let chat_type: String = env
            .eval("return _G.__test_sent_messages[1].chatType")
            .expect("eval failed");
        assert_eq!(chat_type, "SAY", "Default chat type should be SAY");

        let text_after: String = env
            .eval("return ChatFrame1EditBox:GetText() or ''")
            .expect("GetText failed");
        assert_eq!(text_after, "", "EditBox should be cleared after submit");
    }
}

#[test]
fn test_chat_message_contains_timestamp() {
    test_timeout! {
        let env = setup_env();

        // Enable timestamps (default CVar is "none")
        env.exec(r#"SetCVar("showTimestamps", "%H:%M ")"#).unwrap();

        // Send a chat message — C_ChatInfo.SendChatMessage adds it to ChatFrame1
        env.exec(r#"C_ChatInfo.SendChatMessage("Test timestamp", "SAY")"#)
            .unwrap();

        // Get the last message text from ChatFrame1
        let msg: String = env
            .eval(
                r#"
        local n = ChatFrame1:GetNumMessages()
        local text = ChatFrame1:GetMessageInfo(n)
        return text
    "#,
            )
            .unwrap();

        // Message should start with a time like "14:32 " (HH:MM followed by space)
        let has_time = msg.len() >= 6
            && msg.as_bytes()[2] == b':'
            && msg.as_bytes()[0].is_ascii_digit()
            && msg.as_bytes()[1].is_ascii_digit()
            && msg.as_bytes()[3].is_ascii_digit()
            && msg.as_bytes()[4].is_ascii_digit()
            && msg.as_bytes()[5] == b' ';
        assert!(
            has_time,
            "Chat message should start with HH:MM timestamp, got: {msg:.40}"
        );
    }
}

#[test]
fn test_chat_editbox_text_color_after_activation() {
    test_timeout! {
        let env = setup_env();

        click_chat_editbox(&env);

        // After activation, ActivateChat should have called UpdateHeader
        // which sets text color to white (ChatTypeInfo default = 1.0, 1.0, 1.0)
        let (r, g, b): (f64, f64, f64) = env
            .eval("return ChatFrame1EditBox:GetTextColor()")
            .expect("GetTextColor failed");
        assert!(
            (r - 1.0).abs() < 0.01 && (g - 1.0).abs() < 0.01 && (b - 1.0).abs() < 0.01,
            "EditBox text color should be white after activation, got ({r}, {g}, {b})"
        );

        // Alpha should be 1.0 after activation
        let alpha: f64 = env
            .eval("return ChatFrame1EditBox:GetAlpha()")
            .expect("GetAlpha failed");
        assert!(
            (alpha - 1.0).abs() < 0.01,
            "EditBox alpha should be 1.0 after activation, got {alpha}"
        );
    }
}

#[test]
fn test_chat_background_uses_default_black_tint_and_alpha() {
    test_timeout! {
        let env = setup_env();
        let _ = env.fire_event("UPDATE_CHAT_WINDOWS");

        let (r, g, b, a): (f64, f64, f64, f64) = env
            .eval("return ChatFrame1Background:GetVertexColor()")
            .expect("ChatFrame1Background:GetVertexColor failed");
        assert!(
            r.abs() < 0.01 && g.abs() < 0.01 && b.abs() < 0.01,
            "ChatFrame1Background should be tinted black after startup, got ({r}, {g}, {b}, {a})"
        );
        assert!(
            (a - 1.0).abs() < 0.01,
            "ChatFrame1Background vertex alpha should stay 1.0, got {a}"
        );

        let alpha: f64 = env
            .eval("return ChatFrame1Background:GetAlpha()")
            .expect("ChatFrame1Background:GetAlpha failed");
        assert!(
            (alpha - 0.25).abs() < 0.01,
            "ChatFrame1Background alpha should be 0.25 after startup, got {alpha}"
        );
    }
}

#[cfg(feature = "gui")]
#[test]
fn test_chat_background_batch_uses_chat_frame_bounds_and_alpha_tint() {
    test_timeout! {
        let env = setup_env();
        let _ = env.fire_event("UPDATE_CHAT_WINDOWS");

        let (background_rect, chat_rect, background_parent_name) = {
            let state = env.state().borrow();
            let chat_id = state
                .widgets
                .get_id_by_name("ChatFrame1")
                .expect("ChatFrame1 should exist");
            let background_id = state
                .widgets
                .get_id_by_name("ChatFrame1Background")
                .expect("ChatFrame1Background should exist");
            let background_parent_name = state
                .widgets
                .get(background_id)
                .and_then(|frame| frame.parent_id)
                .and_then(|parent_id| state.widgets.get(parent_id))
                .and_then(|frame| frame.name.clone())
                .unwrap_or_else(|| "<none>".to_string());
            let chat_rect = compute_frame_rect(&state.widgets, chat_id, 1024.0, 768.0);
            let rect = compute_frame_rect(&state.widgets, background_id, 1024.0, 768.0);
            (
                (rect.x, rect.y, rect.width, rect.height),
                (chat_rect.x, chat_rect.y, chat_rect.width, chat_rect.height),
                background_parent_name,
            )
        };

        let full_batch = build_screenshot_like_batch(&env, 1024, 768, None);
        let background_request = full_batch
            .texture_requests
            .iter()
            .find(|request| {
                request.path.contains("ChatFrameBackground") && {
                    let bounds = quad_bounds(
                        &full_batch,
                        request.vertex_start as usize,
                        request.vertex_count as usize,
                    );
                    let rect_right = background_rect.0 + background_rect.2;
                    let rect_bottom = background_rect.1 + background_rect.3;
                    bounds.0 < rect_right
                        && bounds.2 > background_rect.0
                        && bounds.1 < rect_bottom
                        && bounds.3 > background_rect.1
                }
            })
            .expect("full batch should include the ChatFrame1Background texture request");
        let background_bounds = quad_bounds(
            &full_batch,
            background_request.vertex_start as usize,
            background_request.vertex_count as usize,
        );
        let background_vertex = full_batch.vertices[background_request.vertex_start as usize];
        assert_eq!(
            background_parent_name, "ChatFrame1",
            "ChatFrame1Background should stay parented to ChatFrame1"
        );
        assert!(
            background_rect.2 > chat_rect.2,
            "Chat background should extend past the chat frame to cover the scrollbar gutter. chat_rect={chat_rect:?} background_rect={background_rect:?}"
        );
        assert!(
            background_bounds.2 - background_bounds.0 > chat_rect.2,
            "Chat background quad should be wider than the chat frame. bounds={background_bounds:?} chat_rect={chat_rect:?}"
        );
        assert!(
            background_vertex.color[0].abs() < 0.01
                && background_vertex.color[1].abs() < 0.01
                && background_vertex.color[2].abs() < 0.01,
            "Chat background quad should keep a black tint, got {:?}",
            background_vertex.color
        );
        assert!(
            (background_vertex.color[3] - 0.25).abs() < 0.01,
            "Chat background quad should render at alpha 0.25, got {:?}",
            background_vertex.color
        );
    }
}

#[test]
fn test_chat_frame2_starts_disabled() {
    test_timeout! {
        let env = setup_env();

        let (shown, frame_shown, tab_shown): (bool, bool, bool) = env
            .eval(
                r#"
                local shown = select(7, GetChatWindowInfo(2))
                local frameShown = ChatFrame2 and ChatFrame2:IsShown() or false
                local tabShown = ChatFrame2Tab and ChatFrame2Tab:IsShown() or false
                return shown, frameShown, tabShown
            "#,
            )
            .expect("ChatFrame2 startup state eval failed");

        assert!(!shown, "GetChatWindowInfo(2) should report chat window 2 hidden");
        assert!(!frame_shown, "ChatFrame2 should start hidden");
        assert!(!tab_shown, "ChatFrame2Tab should start hidden");
    }
}

#[test]
fn test_chat_frame2_can_be_enabled_explicitly() {
    test_timeout! {
        let env = setup_env();

        env.exec(
            r#"
            SetChatWindowShown(2, true)
            if FloatingChatFrame_Update then
                FloatingChatFrame_Update(2)
            end
            if FCF_DockUpdate then
                FCF_DockUpdate()
            end
        "#,
        )
        .expect("ChatFrame2 enable setup failed");

        let (shown, frame_shown, tab_shown): (bool, bool, bool) = env
            .eval(
                r#"
                local shown = select(7, GetChatWindowInfo(2))
                local frameShown = ChatFrame2 and ChatFrame2:IsShown() or false
                local tabShown = ChatFrame2Tab and ChatFrame2Tab:IsShown() or false
                return shown, frameShown, tabShown
            "#,
            )
            .expect("ChatFrame2 enabled state eval failed");

        assert!(shown, "GetChatWindowInfo(2) should report chat window 2 visible after enabling it");
        assert!(frame_shown, "ChatFrame2 should become visible after enabling it");
        assert!(tab_shown, "ChatFrame2Tab should become visible after enabling it");
    }
}

#[test]
fn test_chat_window_name_round_trips_through_info() {
    test_timeout! {
        let env = setup_env();

        let name: String = env
            .eval(
                r#"
                SetChatWindowName(4, "Pet Battle")
                return GetChatWindowInfo(4)
            "#,
            )
            .expect("chat window name eval failed");

        assert_eq!(name, "Pet Battle");
    }
}

#[test]
fn test_chat_window_docked_state_round_trips_through_info() {
    test_timeout! {
        let env = setup_env();

        let (docked, undocked): (bool, bool) = env
            .eval(
                r#"
                SetChatWindowDocked(5, 1)
                local docked = select(9, GetChatWindowInfo(5))
                SetChatWindowDocked(5, nil)
                local undocked = select(9, GetChatWindowInfo(5))
                return docked, undocked
            "#,
            )
            .expect("chat window docked state eval failed");

        assert!(docked, "GetChatWindowInfo(5) should report docked after docking");
        assert!(!undocked, "GetChatWindowInfo(5) should report undocked after clearing dock");
    }
}

#[test]
fn test_chat_voice_button_uses_template_sized_centered_icon() {
    test_timeout! {
        let env = setup_env();
        let surface = read_chat_voice_button_surface(&env);

        assert!(
            surface.normal_atlas == "chatframe-button-up",
            "ChatFrameChannelButton should keep its atlas-backed normal texture, got {:?}",
            surface.normal_atlas
        );
        assert!(
            (surface.icon_atlas_count - 1.0).abs() < 0.01,
            "ChatFrameChannelButton should expose exactly one voice icon texture child, got {}",
            surface.icon_atlas_count
        );
        assert!(
            (surface.button_width - 27.0).abs() < 0.01
                && (surface.button_height - 26.0).abs() < 0.01,
            "voice button should keep the VoiceToggleButtonTemplate button size, got {}x{}",
            surface.button_width,
            surface.button_height
        );
        assert!(
            (surface.icon_width - 15.0).abs() < 0.01
                && (surface.icon_height - 15.0).abs() < 0.01,
            "voice button icon should keep the template's fixed 15x15 size instead of stretching to the full button, got {}x{}",
            surface.icon_width,
            surface.icon_height
        );
        assert_eq!(
            surface.icon_points, 1.0,
            "voice button icon should use a single centered point, not SetAllPoints semantics"
        );
        assert_eq!(surface.point, "CENTER");
        assert_eq!(surface.relative_name, "ChatFrameChannelButton");
        assert_eq!(surface.relative_point, "CENTER");
        assert!(
            surface.offset_x.abs() < 0.01 && surface.offset_y.abs() < 0.01,
            "voice button icon should stay centered with zero offset, got ({}, {})",
            surface.offset_x,
            surface.offset_y
        );
    }
}
