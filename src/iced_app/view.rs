//! App::view() and subscription methods plus UI building helpers.

mod hit_testing;
mod options_modal;

use iced::widget::shader::Shader;
use iced::widget::{
    Column, Container, button, checkbox, column, container, row, scrollable, space, stack, text,
    text_input,
};
use iced::{Border, Color, Element, Font, Length, Padding, Subscription};

use crate::LayoutRect;

use super::Message;
use super::app::App;
use super::layout::compute_frame_rect;
use super::styles::{event_button_style, input_style, palette, run_button_style};

/// Resolve a frame's display name, using the owner addon as fallback for anonymous frames.
fn anon_display_name(
    frame: &crate::widget::Frame,
    addons: &[crate::lua_api::state::AddonInfo],
) -> String {
    if let Some(ref name) = frame.name {
        return name.clone();
    }
    if let Some(idx) = frame.owner_addon {
        if let Some(addon) = addons.get(idx as usize) {
            return format!("({})", addon.folder_name);
        }
    }
    "(anon)".to_string()
}

fn console_text_from_log_messages(log_messages: &[String]) -> String {
    log_messages.join("\n")
}

fn should_dispatch_wow_key(status: iced::event::Status, wow_key: &str) -> bool {
    matches!(status, iced::event::Status::Ignored) || matches!(wow_key, "ESCAPE" | "CTRL-Q")
}

fn message_from_keyboard_event(
    event: &iced::Event,
    status: iced::event::Status,
) -> Option<Message> {
    use iced::keyboard;
    use std::time::Instant;

    let iced::Event::Keyboard(keyboard_event) = event else {
        return None;
    };
    if let keyboard::Event::ModifiersChanged(modifiers) = keyboard_event {
        return Some(Message::ModifiersChanged(*modifiers));
    }
    if let keyboard::Event::KeyReleased {
        key, physical_key, ..
    } = keyboard_event
    {
        let modifiers = keyboard::Modifiers::empty();
        let key = super::keybinds::iced_key_to_wow(key, modifiers)
            .or_else(|| super::keybinds::iced_physical_key_to_wow(physical_key, modifiers))?;
        return Some(Message::KeyRelease(key));
    }
    let keyboard::Event::KeyPressed {
        key,
        modifiers,
        physical_key,
        text,
        ..
    } = keyboard_event
    else {
        return None;
    };

    // Ctrl+R is a simulator-only shortcut.
    if modifiers.control() && *key == keyboard::Key::Character("r".into()) {
        return Some(Message::ReloadUI);
    }

    let wow_key = super::keybinds::iced_key_to_wow(key, *modifiers)
        .or_else(|| super::keybinds::iced_physical_key_to_wow(physical_key, *modifiers))?;
    if !should_dispatch_wow_key(status, &wow_key) {
        return None;
    }

    // Include raw text for character input into focused EditBox.
    // Skip text when Ctrl/Alt modifiers are held (shortcuts, not typing).
    let raw_text = if modifiers.control() || modifiers.alt() {
        None
    } else {
        text.as_ref().map(|t| t.to_string())
    };
    Some(Message::KeyPress(wow_key, raw_text, Instant::now()))
}

fn keyboard_subscription() -> Subscription<Message> {
    iced::event::listen_with(|event, status, _window| message_from_keyboard_event(&event, status))
}

fn timer_subscription(interval: std::time::Duration) -> Subscription<Message> {
    iced::time::every(interval).map(Message::ProcessTimers)
}

impl App {
    /// Build the title bar with FPS counter, frame time, and canvas size.
    fn build_title_bar(&self) -> Element<'_, Message> {
        let screen = self.screen_size.get();
        let screen_str = format!(" | screen:{}x{}", screen.width as i32, screen.height as i32);
        let title_text = format!(
            "WoW UI Simulator  [{}{}]",
            self.display_metrics.summary(),
            screen_str
        );
        text(title_text).size(20).color(palette::GOLD).into()
    }

    /// Build the canvas area with optional inspector panel overlay.
    fn build_canvas_area(&self) -> Container<'_, Message> {
        let shader: Shader<Message, &App> =
            Shader::new(self).width(Length::Fill).height(Length::Fill);

        let stacked: Element<'_, Message> = if self.inspector_visible {
            let inspector = self.build_inspector_panel();
            stack![shader, inspector].into()
        } else {
            shader.into()
        };

        container(stacked)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_DARK)),
                border: Border {
                    color: palette::BORDER_HIGHLIGHT,
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
    }

    /// Build the collapsible frames sidebar panel.
    fn build_sidebar_panel(&self) -> Container<'_, Message> {
        let toggle_label = if self.frames_panel_collapsed {
            ">> Frames"
        } else {
            "<< Frames"
        };
        let toggle_btn = button(text(toggle_label).size(12))
            .on_press(Message::ToggleFramesPanel)
            .padding([2, 6])
            .style(|_, _| button::Style {
                background: None,
                text_color: palette::TEXT_PRIMARY,
                ..Default::default()
            });

        let panel_style = |_: &_| container::Style {
            background: Some(iced::Background::Color(palette::BG_PANEL)),
            border: Border {
                color: palette::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        };

        if self.frames_panel_collapsed {
            container(toggle_btn).padding(6).style(panel_style)
        } else {
            self.build_expanded_sidebar(toggle_btn).style(panel_style)
        }
    }

    fn build_expanded_sidebar<'a>(
        &'a self,
        toggle_btn: button::Button<'a, Message>,
    ) -> Container<'a, Message> {
        let frames_list = self.build_frames_sidebar();
        container(
            column![
                toggle_btn,
                scrollable(frames_list).width(Length::Fill).height(600)
            ]
            .spacing(4),
        )
        .width(240)
        .padding(6)
    }

    /// Build the event trigger buttons row.
    fn build_event_buttons(&self) -> Element<'_, Message> {
        row![
            button(text("ADDON_LOADED").size(12))
                .on_press(Message::FireEvent("ADDON_LOADED".to_string()))
                .style(event_button_style),
            button(text("PLAYER_LOGIN").size(12))
                .on_press(Message::FireEvent("PLAYER_LOGIN".to_string()))
                .style(event_button_style),
            button(text("PLAYER_ENTERING_WORLD").size(12))
                .on_press(Message::FireEvent("PLAYER_ENTERING_WORLD".to_string()))
                .style(event_button_style),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the command input row.
    fn build_command_row(&self) -> Element<'_, Message> {
        row![
            text_input("/command", &self.command_input)
                .on_input(Message::CommandInputChanged)
                .on_submit(Message::ExecuteCommand)
                .width(Length::Fill)
                .style(input_style),
            button(text("Run").size(12))
                .on_press(Message::ExecuteCommand)
                .style(run_button_style),
        ]
        .spacing(6)
        .into()
    }

    /// Build the console output area showing full log history with scrollback.
    fn build_console(&self) -> Container<'_, Message> {
        let console_text = console_text_from_log_messages(&self.log_messages);

        container(
            scrollable(
                text(console_text)
                    .size(12)
                    .color(palette::CONSOLE_TEXT)
                    .font(Font::MONOSPACE),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(80)
        .padding(6)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(palette::BG_INPUT)),
            border: Border {
                color: palette::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = self.build_title_bar();
        let render_container = self.build_canvas_area();

        // Position sidebar at top-right corner, stack over canvas
        let sidebar_positioned = container(self.build_sidebar_panel())
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right);
        let content_row = stack![render_container, sidebar_positioned];

        let bottom_row = row![
            button(text("Options").size(12))
                .on_press(Message::ToggleOptionsModal)
                .style(event_button_style),
            self.build_command_row(),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        let main_column = column![title, content_row, bottom_row, self.build_console()]
            .spacing(5)
            .padding(7);

        let base: Element<'_, Message> = container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_DARK)),
                ..Default::default()
            })
            .into();

        if self.options_modal_visible {
            self.wrap_with_modal(base)
        } else {
            base
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard = keyboard_subscription();

        if let Some(interval) = self.compute_tick_interval() {
            if crate::logging::gui_trace_enabled() {
                crate::logging::eprintln_gui_trace(&format!(
                    "subscription tick interval={interval:?}"
                ));
            }
            let timer = timer_subscription(interval);
            Subscription::batch([timer, keyboard])
        } else {
            if crate::logging::gui_trace_enabled() {
                crate::logging::eprintln_gui_trace("subscription tick interval=none");
            }
            keyboard
        }
    }

    pub(crate) fn build_frames_sidebar(&self) -> Column<'_, Message> {
        let mut col = Column::new().spacing(2);

        let env = self.env.borrow();
        let state = env.state().borrow();

        let mut count = 0;
        for id in state.widgets.iter_ids() {
            let Some(frame) = state.widgets.get(id) else {
                continue;
            };
            if frame_should_be_hidden_in_sidebar(frame) {
                continue;
            }

            col = col.push(
                text(sidebar_frame_label(frame))
                    .size(14)
                    .color(palette::TEXT_MUTED),
            );

            count += 1;
            if count >= 15 {
                break;
            }
        }

        col
    }

    /// Build the inspector panel widget.
    pub(crate) fn build_inspector_panel(&self) -> Element<'_, Message> {
        let env = self.env.borrow();
        let state = env.state().borrow();

        let frame_id = self.inspected_frame.unwrap_or(0);
        let frame = state.widgets.get(frame_id);

        let (name, widget_type, computed_rect) = Self::inspector_frame_info(
            frame,
            frame_id,
            &state.widgets,
            &state.addons,
            self.screen_size.get().width,
            self.screen_size.get().height,
        );

        let content = self.inspector_panel_content(
            frame,
            frame_id,
            &state.widgets,
            &state.addons,
            &name,
            &widget_type,
            computed_rect,
        );

        Self::position_inspector_panel(content, self.inspector_position)
    }

    fn inspector_panel_content(
        &self,
        frame: Option<&crate::widget::Frame>,
        frame_id: u64,
        widgets: &crate::widget::WidgetRegistry,
        addons: &[crate::lua_api::state::AddonInfo],
        name: &str,
        widget_type: &str,
        computed_rect: LayoutRect,
    ) -> Column<'_, Message> {
        let title = Self::inspector_title_bar(name, widget_type);
        let id_row = text(format!(
            "ID: {}  Pos: ({:.0}, {:.0})",
            frame_id, computed_rect.x, computed_rect.y
        ))
        .size(11)
        .color(palette::TEXT_SECONDARY);
        let size_row = self.inspector_size_row();
        let alpha_level_row = self.inspector_alpha_level_row();
        let checkbox_row = self.inspector_checkbox_row();
        let parent_chain = Self::inspector_parent_chain(widgets, addons, frame_id);
        let anchors_display = Self::inspector_anchors_display(frame);
        let apply_btn = button(text("Apply").size(12))
            .on_press(Message::InspectorApply)
            .padding(Padding::from([4, 12]));

        column![
            title,
            id_row,
            size_row,
            alpha_level_row,
            checkbox_row,
            parent_chain,
            text("Anchors:").size(11).color(palette::TEXT_SECONDARY),
            anchors_display,
            apply_btn,
        ]
        .spacing(6)
        .padding(8)
    }

    /// Extract name, type, and computed rect for the inspected frame.
    fn inspector_frame_info(
        frame: Option<&crate::widget::Frame>,
        frame_id: u64,
        widgets: &crate::widget::WidgetRegistry,
        addons: &[crate::lua_api::state::AddonInfo],
        screen_width: f32,
        screen_height: f32,
    ) -> (String, String, LayoutRect) {
        match frame {
            Some(f) => {
                let rect = compute_frame_rect(widgets, frame_id, screen_width, screen_height);
                (
                    anon_display_name(f, addons),
                    f.widget_type.as_str().to_string(),
                    rect,
                )
            }
            None => ("(none)".to_string(), "".to_string(), LayoutRect::default()),
        }
    }

    /// Build the inspector title bar with close button.
    fn inspector_title_bar<'a>(name: &str, widget_type: &str) -> Element<'a, Message> {
        row![
            text(format!("{} [{}]", name, widget_type))
                .size(14)
                .color(palette::GOLD),
            space::horizontal(),
            button(text("x").size(14))
                .on_press(Message::InspectorClose)
                .padding(2)
                .style(|_, _| button::Style {
                    background: Some(iced::Background::Color(Color::TRANSPARENT)),
                    text_color: palette::TEXT_SECONDARY,
                    ..Default::default()
                }),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the width/height input row.
    fn inspector_size_row(&self) -> Element<'_, Message> {
        row![
            text("W:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.width)
                .on_input(Message::InspectorWidthChanged)
                .size(11)
                .width(50),
            text("H:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.height)
                .on_input(Message::InspectorHeightChanged)
                .size(11)
                .width(50),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the alpha/level input row.
    fn inspector_alpha_level_row(&self) -> Element<'_, Message> {
        row![
            text("Alpha:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.alpha)
                .on_input(Message::InspectorAlphaChanged)
                .size(11)
                .width(40),
            text("Level:").size(11).color(palette::TEXT_SECONDARY),
            text_input("", &self.inspector_state.frame_level)
                .on_input(Message::InspectorLevelChanged)
                .size(11)
                .width(40),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// Build the visible/mouse checkboxes row.
    fn inspector_checkbox_row(&self) -> Element<'_, Message> {
        row![
            checkbox(self.inspector_state.visible)
                .label("Visible")
                .on_toggle(Message::InspectorVisibleToggled)
                .size(14)
                .text_size(11),
            checkbox(self.inspector_state.mouse_enabled)
                .label("Mouse")
                .on_toggle(Message::InspectorMouseEnabledToggled)
                .size(14)
                .text_size(11),
        ]
        .spacing(10)
        .into()
    }

    /// Format anchor info for the inspected frame.
    fn inspector_anchors_display<'a>(frame: Option<&crate::widget::Frame>) -> Element<'a, Message> {
        let anchors_text = match frame {
            Some(f) if !f.anchors.is_empty() => {
                let anchor_strs: Vec<String> = f
                    .anchors
                    .iter()
                    .map(|a| {
                        let rel = a.relative_to.as_deref().unwrap_or("$parent");
                        format!(
                            "{:?}->{} {:?} ({:.0},{:.0})",
                            a.point, rel, a.relative_point, a.x_offset, a.y_offset
                        )
                    })
                    .collect();
                anchor_strs.join("\n")
            }
            _ => "No anchors".to_string(),
        };
        text(anchors_text)
            .size(10)
            .color(palette::TEXT_MUTED)
            .into()
    }

    /// Build parent chain display for the inspector.
    fn inspector_parent_chain<'a>(
        widgets: &crate::widget::WidgetRegistry,
        addons: &[crate::lua_api::state::AddonInfo],
        frame_id: u64,
    ) -> Element<'a, Message> {
        let mut ancestors = Vec::new();
        let mut current = widgets.get(frame_id).and_then(|f| f.parent_id);
        while let Some(pid) = current {
            let Some(parent) = widgets.get(pid) else {
                break;
            };
            ancestors.push(anon_display_name(parent, addons));
            if ancestors.len() >= 6 {
                ancestors.push("...".to_string());
                break;
            }
            current = parent.parent_id;
        }
        ancestors.reverse();
        let self_name = widgets
            .get(frame_id)
            .map(|f| anon_display_name(f, addons))
            .unwrap_or_else(|| "(anon)".to_string());
        ancestors.push(self_name);
        let chain_text = ancestors.join(" > ");
        text(chain_text).size(10).color(palette::TEXT_MUTED).into()
    }

    /// Wrap inspector content in a positioned panel container.
    fn position_inspector_panel<'a>(
        content: Column<'a, Message>,
        position: iced::Point,
    ) -> Element<'a, Message> {
        let panel: Container<'a, Message> = container(content)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_PANEL)),
                border: Border {
                    color: palette::BORDER_HIGHLIGHT,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .width(220);

        let x_pad = position.x.max(0.0);
        let y_pad = position.y.max(0.0);

        container(panel)
            .padding(Padding::new(0.0).top(y_pad).left(x_pad))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[cfg(test)]
mod key_dispatch_tests {
    use super::{message_from_keyboard_event, should_dispatch_wow_key};
    use crate::iced_app::Message;

    #[test]
    fn escape_dispatches_even_when_iced_captures_event() {
        assert!(should_dispatch_wow_key(
            iced::event::Status::Captured,
            "ESCAPE"
        ));
    }

    #[test]
    fn non_escape_keys_only_dispatch_when_ignored_by_iced() {
        assert!(should_dispatch_wow_key(iced::event::Status::Ignored, "M"));
        assert!(!should_dispatch_wow_key(iced::event::Status::Captured, "M"));
    }

    #[test]
    fn ctrl_q_dispatches_even_when_iced_captures_event() {
        assert!(should_dispatch_wow_key(
            iced::event::Status::Captured,
            "CTRL-Q"
        ));
    }

    #[test]
    fn ctrl_q_control_character_event_dispatches_quit_binding() {
        let event = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Character("\u{11}".into()),
            modified_key: iced::keyboard::Key::Character("\u{11}".into()),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::CTRL,
            text: None,
            repeat: false,
        });

        let message = message_from_keyboard_event(&event, iced::event::Status::Captured)
            .expect("Ctrl+Q should dispatch even when iced captures the event");

        assert!(
            matches!(message, Message::KeyPress(key, None, _) if key == "CTRL-Q"),
            "Ctrl+Q control-character event should map to CTRL-Q"
        );
    }

    #[test]
    fn ctrl_q_unidentified_logical_event_dispatches_quit_binding_from_physical_key() {
        let event = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Unidentified,
            modified_key: iced::keyboard::Key::Unidentified,
            physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::KeyQ),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::CTRL,
            text: None,
            repeat: false,
        });

        let message = message_from_keyboard_event(&event, iced::event::Status::Captured)
            .expect("Ctrl+Q should dispatch from physical KeyQ when logical key is unidentified");

        assert!(
            matches!(message, Message::KeyPress(key, None, _) if key == "CTRL-Q"),
            "Ctrl+Q physical-key fallback should map to CTRL-Q"
        );
    }
}

fn frame_should_be_hidden_in_sidebar(frame: &crate::widget::Frame) -> bool {
    let Some(name) = frame.name.as_deref() else {
        return true;
    };
    if [
        "__",
        "DBM",
        "Details",
        "Avatar",
        "Plater",
        "WeakAuras",
        "UIWidget",
        "GameMenu",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
    {
        return true;
    }

    frame.width <= 0.0 || frame.height <= 0.0
}

fn sidebar_frame_label(frame: &crate::widget::Frame) -> String {
    let name = frame.name.as_deref().unwrap_or("(anon)");
    let visible = if frame.visible { "visible" } else { "hidden" };
    truncate_sidebar_label(format!(
        "{} [{}] {}x{} ({})",
        name,
        frame.widget_type.as_str(),
        frame.width as i32,
        frame.height as i32,
        visible
    ))
}

fn truncate_sidebar_label(display: String) -> String {
    if display.len() > 30 {
        format!("{}...", &display[..27])
    } else {
        display
    }
}

#[cfg(test)]
mod tests {
    use super::console_text_from_log_messages;

    #[test]
    fn console_text_includes_full_scrollback_without_truncation() {
        let lines = (0..8)
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>();
        let rendered = console_text_from_log_messages(&lines);
        assert_eq!(rendered.lines().count(), 8);
        assert!(rendered.contains("line-0"));
        assert!(rendered.contains("line-7"));
    }

    #[test]
    fn console_text_preserves_message_order() {
        let lines = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let rendered = console_text_from_log_messages(&lines);
        assert_eq!(rendered, "first\nsecond\nthird");
    }
}
