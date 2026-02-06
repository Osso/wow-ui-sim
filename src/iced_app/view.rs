//! App::view() and subscription methods plus UI building helpers.

use iced::widget::shader::Shader;
use iced::widget::{
    button, checkbox, column, container, row, scrollable, space, stack, text, text_input, Column,
    Container,
};
use iced::{Border, Color, Element, Font, Length, Padding, Subscription};

use crate::render::texture::UI_SCALE;
use crate::LayoutRect;

use crate::widget::FrameStrata;

use super::app::App;
use super::layout::compute_frame_rect;
use super::styles::{event_button_style, input_style, palette, run_button_style};
use super::Message;

/// Frame names excluded from hit testing (full-screen or non-interactive overlays).
const HIT_TEST_EXCLUDED: &[&str] = &[
    "UIParent", "Minimap", "WorldFrame",
    "DEFAULT_CHAT_FRAME", "ChatFrame1",
    "EventToastManagerFrame", "EditModeManagerFrame",
];

/// Collect visible, mouse-enabled frames with strata/level info for hit testing.
fn collect_hittable_frames(
    widgets: &crate::widget::WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
) -> Vec<(u64, FrameStrata, i32, LayoutRect)> {
    widgets
        .all_ids()
        .into_iter()
        .filter_map(|id| {
            let frame = widgets.get(id)?;
            if !frame.visible || !frame.mouse_enabled {
                return None;
            }
            if frame.name.as_deref().is_some_and(|n| HIT_TEST_EXCLUDED.contains(&n)) {
                return None;
            }
            let rect = compute_frame_rect(widgets, id, screen_width, screen_height);
            Some((id, frame.frame_strata, frame.frame_level, rect))
        })
        .collect()
}

impl App {
    /// Build the title bar with FPS counter, frame time, canvas size, and mouse coords.
    fn build_title_bar(&self) -> Element<'_, Message> {
        let mouse_str = match self.mouse_position {
            Some(pos) => format!(" | mouse:({:.0},{:.0})", pos.x, pos.y),
            None => String::new(),
        };
        let screen = self.screen_size.get();
        let screen_str = format!(" | screen:{}x{}", screen.width as i32, screen.height as i32);
        let title_text = format!(
            "WoW UI Simulator  [{:.1} FPS | {:.2}ms{}{}]",
            self.fps, self.frame_time_display, screen_str, mouse_str
        );
        text(title_text).size(20).color(palette::GOLD).into()
    }

    /// Build the canvas area with optional inspector panel overlay.
    fn build_canvas_area(&self) -> Container<'_, Message> {
        let shader: Shader<Message, &App> = Shader::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

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
        let toggle_label = if self.frames_panel_collapsed { ">> Frames" } else { "<< Frames" };
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
            border: Border { color: palette::BORDER, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        };

        if self.frames_panel_collapsed {
            container(toggle_btn).padding(6).style(panel_style)
        } else {
            let frames_list = self.build_frames_sidebar();
            container(
                column![toggle_btn, scrollable(frames_list).width(Length::Fill).height(600)]
                    .spacing(4),
            )
            .width(240)
            .padding(6)
            .style(panel_style)
        }
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

    /// Build the console output area showing recent log messages.
    fn build_console(&self) -> Container<'_, Message> {
        let console_text: String = self
            .log_messages
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        container(
            scrollable(
                text(console_text).size(12).color(palette::CONSOLE_TEXT).font(Font::MONOSPACE),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(80)
        .padding(6)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(palette::BG_INPUT)),
            border: Border { color: palette::BORDER, width: 1.0, radius: 4.0.into() },
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

        let main_column = column![
            title,
            content_row,
            self.build_event_buttons(),
            self.build_command_row(),
            self.build_console(),
        ]
        .spacing(5)
        .padding(7);

        container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(palette::BG_DARK)),
                ..Default::default()
            })
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Timer for processing WoW timers and debug commands (~60fps)
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::ProcessTimers)
    }

    pub(crate) fn build_frames_sidebar(&self) -> Column<'_, Message> {
        let mut col = Column::new().spacing(2);

        let env = self.env.borrow();
        let state = env.state().borrow();

        let mut count = 0;
        for id in state.widgets.all_ids() {
            if let Some(frame) = state.widgets.get(id) {
                let name = match &frame.name {
                    Some(n) => n.as_str(),
                    None => continue,
                };
                if name.starts_with("__")
                    || name.starts_with("DBM")
                    || name.starts_with("Details")
                    || name.starts_with("Avatar")
                    || name.starts_with("Plater")
                    || name.starts_with("WeakAuras")
                    || name.starts_with("UIWidget")
                    || name.starts_with("GameMenu")
                {
                    continue;
                }
                if frame.width <= 0.0 || frame.height <= 0.0 {
                    continue;
                }

                let visible = if frame.visible { "visible" } else { "hidden" };
                let display = format!(
                    "{} [{}] {}x{} ({})",
                    name,
                    frame.widget_type.as_str(),
                    frame.width as i32,
                    frame.height as i32,
                    visible
                );

                let display = if display.len() > 30 {
                    format!("{}...", &display[..27])
                } else {
                    display
                };

                col = col.push(text(display).size(14).color(palette::TEXT_MUTED));

                count += 1;
                if count >= 15 {
                    break;
                }
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
            frame, frame_id, &state.widgets,
            self.screen_size.get().width, self.screen_size.get().height,
        );

        let title = Self::inspector_title_bar(&name, &widget_type);
        let id_row = text(format!("ID: {}  Pos: ({:.0}, {:.0})", frame_id, computed_rect.x, computed_rect.y))
            .size(11)
            .color(palette::TEXT_SECONDARY);
        let size_row = self.inspector_size_row();
        let alpha_level_row = self.inspector_alpha_level_row();
        let checkbox_row = self.inspector_checkbox_row();
        let anchors_display = Self::inspector_anchors_display(frame);

        let apply_btn = button(text("Apply").size(12))
            .on_press(Message::InspectorApply)
            .padding(Padding::from([4, 12]));

        let content = column![
            title,
            id_row,
            size_row,
            alpha_level_row,
            checkbox_row,
            text("Anchors:").size(11).color(palette::TEXT_SECONDARY),
            anchors_display,
            apply_btn,
        ]
        .spacing(6)
        .padding(8);

        Self::position_inspector_panel(content, self.inspector_position)
    }

    /// Extract name, type, and computed rect for the inspected frame.
    fn inspector_frame_info(
        frame: Option<&crate::widget::Frame>,
        frame_id: u64,
        widgets: &crate::widget::WidgetRegistry,
        screen_width: f32,
        screen_height: f32,
    ) -> (String, String, LayoutRect) {
        match frame {
            Some(f) => {
                let rect = compute_frame_rect(widgets, frame_id, screen_width, screen_height);
                (
                    f.name.clone().unwrap_or_else(|| "(anon)".to_string()),
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
        text(anchors_text).size(10).color(palette::TEXT_MUTED).into()
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

    /// Hit test to find frame under cursor.
    pub(crate) fn hit_test(&self, pos: iced::Point) -> Option<u64> {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let screen_width = self.screen_size.get().width;
        let screen_height = self.screen_size.get().height;

        let mut frames = collect_hittable_frames(&state.widgets, screen_width, screen_height);
        frames.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| a.0.cmp(&b.0))
        });

        // Iterate top-to-bottom (highest strata/level first)
        frames.iter().rev().find_map(|(id, _, _, rect)| {
            let scaled = LayoutRect {
                x: rect.x * UI_SCALE,
                y: rect.y * UI_SCALE,
                width: rect.width * UI_SCALE,
                height: rect.height * UI_SCALE,
            };
            if pos.x >= scaled.x
                && pos.x <= scaled.x + scaled.width
                && pos.y >= scaled.y
                && pos.y <= scaled.y + scaled.height
            {
                Some(*id)
            } else {
                None
            }
        })
    }

    /// Dump WoW frames for debug server.
    pub(crate) fn dump_wow_frames(&self) -> String {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let screen_width = self.screen_size.get().width;
        let screen_height = self.screen_size.get().height;

        let mut lines = Vec::new();
        lines.push("WoW UI Simulator - Frame Dump".to_string());
        lines.push(format!("Screen: {}x{}", screen_width as i32, screen_height as i32));
        lines.push(String::new());

        // Find root frames (no parent or parent is UIParent)
        let mut root_ids: Vec<u64> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter(|&id| {
                state
                    .widgets
                    .get(id)
                    .map(|f| f.parent_id.is_none() || f.parent_id == Some(1))
                    .unwrap_or(false)
            })
            .collect();
        root_ids.sort();

        for id in root_ids {
            self.dump_frame_recursive(&state.widgets, id, 0, screen_width, screen_height, &mut lines);
        }

        lines.join("\n")
    }

    fn dump_frame_recursive(
        &self,
        registry: &crate::widget::WidgetRegistry,
        id: u64,
        depth: usize,
        screen_width: f32,
        screen_height: f32,
        lines: &mut Vec<String>,
    ) {
        let Some(frame) = registry.get(id) else {
            return;
        };

        let rect = compute_frame_rect(registry, id, screen_width, screen_height);
        let indent = "  ".repeat(depth);

        let name = frame.name.as_deref().unwrap_or("(anon)");
        let type_str = frame.widget_type.as_str();

        // Build warning flags
        let mut warnings = Vec::new();
        if rect.width <= 0.0 {
            warnings.push("ZERO_WIDTH");
        }
        if rect.height <= 0.0 {
            warnings.push("ZERO_HEIGHT");
        }
        if rect.x + rect.width < 0.0 || rect.x > screen_width {
            warnings.push("OFFSCREEN_X");
        }
        if rect.y + rect.height < 0.0 || rect.y > screen_height {
            warnings.push("OFFSCREEN_Y");
        }
        if !frame.visible {
            warnings.push("HIDDEN");
        }

        let warning_str = if warnings.is_empty() {
            String::new()
        } else {
            format!(" ! {}", warnings.join(", "))
        };

        lines.push(format!(
            "{}{} [{}] ({:.0},{:.0} {}x{}){}",
            indent, name, type_str,
            rect.x, rect.y, rect.width as i32, rect.height as i32,
            warning_str
        ));

        // Recurse into children
        for &child_id in &frame.children {
            self.dump_frame_recursive(registry, child_id, depth + 1, screen_width, screen_height, lines);
        }
    }

    /// Build a frame tree dump with absolute screen coordinates (WoW units).
    pub(crate) fn build_frame_tree_dump(&self, filter: Option<&str>, visible_only: bool) -> String {
        let env = self.env.borrow();
        let state = env.state().borrow();
        // Use WoW logical screen size for layout calculation
        let screen_width = self.screen_size.get().width;
        let screen_height = self.screen_size.get().height;

        let mut lines = Vec::new();

        // Find root frames (no parent) - UIParent children are shown under UIParent
        let mut root_ids: Vec<u64> = state
            .widgets
            .all_ids()
            .into_iter()
            .filter(|&id| {
                state
                    .widgets
                    .get(id)
                    .map(|f| f.parent_id.is_none())
                    .unwrap_or(false)
            })
            .collect();
        root_ids.sort();

        for id in root_ids {
            self.build_tree_recursive(
                &state.widgets,
                id,
                "",
                true,
                screen_width,
                screen_height,
                filter,
                visible_only,
                &mut lines,
            );
        }

        if lines.is_empty() {
            "No frames found".to_string()
        } else {
            lines.join("\n")
        }
    }

    fn build_tree_recursive(
        &self,
        registry: &crate::widget::WidgetRegistry,
        id: u64,
        prefix: &str,
        is_last: bool,
        screen_width: f32,
        screen_height: f32,
        filter: Option<&str>,
        visible_only: bool,
        lines: &mut Vec<String>,
    ) {
        let Some(frame) = registry.get(id) else {
            return;
        };

        if visible_only && !frame.visible {
            return;
        }

        let name = Self::tree_display_name(frame);
        let matches_filter = filter.map(|f| name.to_lowercase().contains(&f.to_lowercase())).unwrap_or(true);
        let rect = compute_frame_rect(registry, id, screen_width, screen_height);

        let mut children: Vec<u64> = frame.children.iter().copied().collect();
        if filter.is_some() || visible_only {
            children.retain(|&child_id| {
                self.subtree_matches(registry, child_id, screen_width, screen_height, filter, visible_only)
            });
        }

        if !matches_filter && children.is_empty() {
            return;
        }

        let connector = if is_last { "+- " } else { "+- " };
        let size_info = Self::tree_size_mismatch_info(frame, &rect);
        let vis_str = if frame.visible { "" } else { " [hidden]" };
        lines.push(format!(
            "{}{}{} ({}) @ ({:.0},{:.0}) {:.0}x{:.0}{}{}",
            prefix, connector, name, frame.widget_type.as_str(),
            rect.x, rect.y, rect.width, rect.height, size_info, vis_str
        ));

        let child_prefix = format!("{}{}", prefix, if is_last { "   " } else { "|  " });
        Self::emit_anchor_lines(registry, frame, &child_prefix, screen_width, screen_height, lines);

        if let Some(tex_path) = &frame.texture {
            lines.push(format!("{}   [texture] {}", child_prefix, tex_path));
        }

        for (i, &child_id) in children.iter().enumerate() {
            self.build_tree_recursive(
                registry, child_id, &child_prefix, i == children.len() - 1,
                screen_width, screen_height, filter, visible_only, lines,
            );
        }
    }

    /// Derive a display name for a frame in the tree dump.
    fn tree_display_name(frame: &crate::widget::Frame) -> &str {
        let raw_name = frame.name.as_deref();
        let is_anon = raw_name
            .map(|n| n.starts_with("__anon_") || n.starts_with("__fs_") || n.starts_with("__tex_"))
            .unwrap_or(true);
        if is_anon && frame.text.is_some() {
            let text = frame.text.as_ref().unwrap();
            if text.len() > 20 {
                Box::leak(format!("\"{}...\"", &text[..17]).into_boxed_str())
            } else {
                Box::leak(format!("\"{}\"", text).into_boxed_str())
            }
        } else {
            raw_name.unwrap_or("(anon)")
        }
    }

    /// Format size mismatch info if stored size differs from computed rect.
    fn tree_size_mismatch_info(frame: &crate::widget::Frame, rect: &LayoutRect) -> String {
        if (frame.width - rect.width).abs() > 0.1 || (frame.height - rect.height).abs() > 0.1 {
            format!(" [stored={:.0}x{:.0}]", frame.width, frame.height)
        } else {
            String::new()
        }
    }

    /// Emit anchor detail lines for a frame in the tree dump.
    fn emit_anchor_lines(
        registry: &crate::widget::WidgetRegistry,
        frame: &crate::widget::Frame,
        child_prefix: &str,
        screen_width: f32,
        screen_height: f32,
        lines: &mut Vec<String>,
    ) {
        use super::layout::anchor_position;

        let parent_rect = if let Some(parent_id) = frame.parent_id {
            compute_frame_rect(registry, parent_id, screen_width, screen_height)
        } else {
            LayoutRect { x: 0.0, y: 0.0, width: screen_width, height: screen_height }
        };

        for anchor in &frame.anchors {
            let (rel_name, relative_rect) = if let Some(rel_id) = anchor.relative_to_id {
                let rel_rect = compute_frame_rect(registry, rel_id as u64, screen_width, screen_height);
                let name = registry.get(rel_id as u64)
                    .and_then(|f| f.name.as_deref())
                    .unwrap_or("(anon)");
                (name, rel_rect)
            } else {
                (anchor.relative_to.as_deref().unwrap_or("$parent"), parent_rect)
            };

            let (anchor_x, anchor_y) = anchor_position(
                anchor.relative_point,
                relative_rect.x, relative_rect.y,
                relative_rect.width, relative_rect.height,
            );
            lines.push(format!(
                "{}   [anchor] {} -> {}:{} offset({:.0},{:.0}) -> ({:.0},{:.0})",
                child_prefix, anchor.point.as_str(),
                rel_name, anchor.relative_point.as_str(),
                anchor.x_offset, anchor.y_offset,
                anchor_x + anchor.x_offset, anchor_y - anchor.y_offset
            ));
        }
    }

    /// Check if a frame or any descendant matches the filter criteria.
    fn subtree_matches(
        &self,
        registry: &crate::widget::WidgetRegistry,
        id: u64,
        screen_width: f32,
        screen_height: f32,
        filter: Option<&str>,
        visible_only: bool,
    ) -> bool {
        let Some(frame) = registry.get(id) else {
            return false;
        };

        if visible_only && !frame.visible {
            return false;
        }

        let name = frame.name.as_deref().unwrap_or("(anon)");
        let matches = filter.map(|f| name.to_lowercase().contains(&f.to_lowercase())).unwrap_or(true);

        if matches {
            return true;
        }

        // Check children
        for &child_id in &frame.children {
            if self.subtree_matches(registry, child_id, screen_width, screen_height, filter, visible_only) {
                return true;
            }
        }

        false
    }
}
