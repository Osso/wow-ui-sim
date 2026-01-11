//! Iced-based UI for rendering WoW frames.

use super::LayoutRect;
use crate::lua_api::WowLuaEnv;
use crate::widget::WidgetType;
use iced::widget::canvas::{self, Cache, Canvas, Geometry, Path, Stroke};
use iced::widget::{column, container, row, text, Column};
use iced::{Color, Element, Length, Point, Rectangle, Size, Theme};
use std::cell::RefCell;
use std::rc::Rc;

/// Run the iced UI with the given Lua environment.
pub fn run_ui(env: WowLuaEnv) -> iced::Result {
    iced::application("WoW UI Simulator", App::update, App::view)
        .theme(|_| Theme::Dark)
        .window_size((1024.0, 768.0))
        .run_with(move || {
            (
                App {
                    env: Rc::new(RefCell::new(env)),
                    lua_input: String::new(),
                    log_messages: Vec::new(),
                    frame_cache: Cache::new(),
                },
                iced::Task::none(),
            )
        })
}

struct App {
    env: Rc<RefCell<WowLuaEnv>>,
    lua_input: String,
    log_messages: Vec<String>,
    frame_cache: Cache,
}

/// Owned frame info for rendering.
#[derive(Debug, Clone)]
struct FrameInfo {
    #[allow(dead_code)]
    id: u64,
    name: Option<String>,
    #[allow(dead_code)]
    widget_type: WidgetType,
    rect: LayoutRect,
    visible: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    LuaInputChanged(String),
    ExecuteLua,
    FireEvent(String),
}

impl App {
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::LuaInputChanged(input) => {
                self.lua_input = input;
            }
            Message::ExecuteLua => {
                let code = self.lua_input.clone();
                if !code.is_empty() {
                    let env = self.env.borrow();
                    match env.exec(&code) {
                        Ok(_) => {
                            self.log_messages.push(format!("> {}", code));
                        }
                        Err(e) => {
                            self.log_messages.push(format!("Error: {}", e));
                        }
                    }
                    self.lua_input.clear();
                    self.frame_cache.clear();
                }
            }
            Message::FireEvent(event) => {
                let env = self.env.borrow();
                if let Err(e) = env.fire_event(&event) {
                    self.log_messages.push(format!("Event error: {}", e));
                } else {
                    self.log_messages.push(format!("Fired: {}", event));
                }
                self.frame_cache.clear();
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Collect frame info while holding the borrow
        let (frame_infos, frame_list_items): (Vec<FrameInfo>, Vec<String>) = {
            let env = self.env.borrow();
            let state = env.state().borrow();

            let mut infos = Vec::new();
            let mut list_items = Vec::new();

            for id in state.widgets.all_ids() {
                if let Some(frame) = state.widgets.get(id) {
                    let rect = compute_frame_rect_owned(&state.widgets, id, 800.0, 600.0);
                    infos.push(FrameInfo {
                        id,
                        name: frame.name.clone(),
                        widget_type: frame.widget_type,
                        rect,
                        visible: frame.visible,
                    });

                    let name = frame.name.as_deref().unwrap_or("(anonymous)");
                    let visible = if frame.visible { "visible" } else { "hidden" };
                    list_items.push(format!(
                        "{} [{}] {}x{} ({})",
                        name,
                        frame.widget_type.as_str(),
                        frame.width,
                        frame.height,
                        visible
                    ));
                }
            }

            (infos, list_items)
        };

        // Create canvas for rendering frames
        let canvas = Canvas::new(FrameRenderer {
            frames: frame_infos,
            cache: &self.frame_cache,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        // Frame list sidebar
        let mut frame_list = Column::new().spacing(2).padding(5);
        frame_list = frame_list.push(text("Frames:").size(14));

        for item in frame_list_items {
            frame_list = frame_list.push(text(item).size(11));
        }

        // Log area
        let mut log_col = Column::new().spacing(2).padding(5);
        log_col = log_col.push(text("Log:").size(14));
        for msg in self.log_messages.iter().rev().take(10) {
            log_col = log_col.push(text(msg).size(11));
        }

        // Event buttons
        let event_buttons = row![
            iced::widget::button("ADDON_LOADED")
                .on_press(Message::FireEvent("ADDON_LOADED".to_string())),
            iced::widget::button("PLAYER_LOGIN")
                .on_press(Message::FireEvent("PLAYER_LOGIN".to_string())),
            iced::widget::button("PLAYER_ENTERING_WORLD")
                .on_press(Message::FireEvent("PLAYER_ENTERING_WORLD".to_string())),
        ]
        .spacing(5);

        // Main layout
        let content = column![
            text("WoW UI Simulator").size(20),
            row![
                container(canvas)
                    .width(Length::FillPortion(3))
                    .height(Length::Fill)
                    .style(container::bordered_box),
                container(frame_list.width(Length::Fixed(250.0)))
                    .height(Length::Fill)
                    .style(container::bordered_box),
            ]
            .height(Length::FillPortion(3)),
            event_buttons,
            container(log_col)
                .height(Length::Fixed(150.0))
                .width(Length::Fill)
                .style(container::bordered_box),
        ]
        .spacing(10)
        .padding(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Canvas renderer for WoW frames.
struct FrameRenderer<'a> {
    frames: Vec<FrameInfo>,
    cache: &'a Cache,
}

impl canvas::Program<Message> for FrameRenderer<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Draw background
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgb(0.1, 0.1, 0.15),
            );

            // Recompute layout for actual canvas size
            let scale_x = bounds.width / 800.0;
            let scale_y = bounds.height / 600.0;

            // Color palette for frames
            let colors = [
                Color::from_rgba(0.2, 0.4, 0.8, 0.6),
                Color::from_rgba(0.8, 0.3, 0.2, 0.6),
                Color::from_rgba(0.2, 0.7, 0.3, 0.6),
                Color::from_rgba(0.7, 0.5, 0.2, 0.6),
                Color::from_rgba(0.5, 0.2, 0.7, 0.6),
                Color::from_rgba(0.2, 0.6, 0.6, 0.6),
            ];

            // Draw each visible frame
            for (i, info) in self.frames.iter().enumerate() {
                if !info.visible {
                    continue;
                }

                // Skip UIParent
                if info.name.as_deref() == Some("UIParent") {
                    continue;
                }

                // Skip frames with no size
                if info.rect.width <= 0.0 || info.rect.height <= 0.0 {
                    continue;
                }

                let rect = LayoutRect {
                    x: info.rect.x * scale_x,
                    y: info.rect.y * scale_y,
                    width: info.rect.width * scale_x,
                    height: info.rect.height * scale_y,
                };

                let color = colors[i % colors.len()];

                // Draw filled rectangle
                frame.fill_rectangle(
                    Point::new(rect.x, rect.y),
                    Size::new(rect.width, rect.height),
                    color,
                );

                // Draw border
                let border_path = Path::rectangle(
                    Point::new(rect.x, rect.y),
                    Size::new(rect.width, rect.height),
                );
                frame.stroke(
                    &border_path,
                    Stroke::default().with_color(Color::WHITE).with_width(1.0),
                );

                // Draw frame name
                if let Some(name) = &info.name {
                    frame.fill_text(canvas::Text {
                        content: name.clone(),
                        position: Point::new(rect.x + 2.0, rect.y + 2.0),
                        color: Color::WHITE,
                        size: iced::Pixels(10.0),
                        ..Default::default()
                    });
                }
            }

            // Draw coordinate guides (crosshair at center)
            let center_x = bounds.width / 2.0;
            let center_y = bounds.height / 2.0;

            let h_line = Path::line(
                Point::new(center_x - 20.0, center_y),
                Point::new(center_x + 20.0, center_y),
            );
            let v_line = Path::line(
                Point::new(center_x, center_y - 20.0),
                Point::new(center_x, center_y + 20.0),
            );
            let guide_stroke = Stroke::default()
                .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.3))
                .with_width(1.0);
            frame.stroke(&h_line, guide_stroke.clone());
            frame.stroke(&v_line, guide_stroke);
        });

        vec![geometry]
    }
}

/// Compute frame rect - owned version that doesn't borrow.
fn compute_frame_rect_owned(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {

    let frame = match registry.get(id) {
        Some(f) => f,
        None => return LayoutRect::default(),
    };

    let width = frame.width;
    let height = frame.height;

    // If no anchors, default to center of parent
    if frame.anchors.is_empty() {
        let parent_rect = if let Some(parent_id) = frame.parent_id {
            compute_frame_rect_owned(registry, parent_id, screen_width, screen_height)
        } else {
            LayoutRect {
                x: 0.0,
                y: 0.0,
                width: screen_width,
                height: screen_height,
            }
        };

        return LayoutRect {
            x: parent_rect.x + (parent_rect.width - width) / 2.0,
            y: parent_rect.y + (parent_rect.height - height) / 2.0,
            width,
            height,
        };
    }

    let anchor = &frame.anchors[0];

    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect_owned(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        }
    };

    let (parent_anchor_x, parent_anchor_y) = anchor_position(
        anchor.relative_point,
        parent_rect.x,
        parent_rect.y,
        parent_rect.width,
        parent_rect.height,
    );

    let target_x = parent_anchor_x + anchor.x_offset;
    let target_y = parent_anchor_y + anchor.y_offset;

    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect {
        x: frame_x,
        y: frame_y,
        width,
        height,
    }
}

fn anchor_position(
    point: crate::widget::AnchorPoint,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
    match point {
        AnchorPoint::TopLeft => (x, y),
        AnchorPoint::Top => (x + w / 2.0, y),
        AnchorPoint::TopRight => (x + w, y),
        AnchorPoint::Left => (x, y + h / 2.0),
        AnchorPoint::Center => (x + w / 2.0, y + h / 2.0),
        AnchorPoint::Right => (x + w, y + h / 2.0),
        AnchorPoint::BottomLeft => (x, y + h),
        AnchorPoint::Bottom => (x + w / 2.0, y + h),
        AnchorPoint::BottomRight => (x + w, y + h),
    }
}

fn frame_position_from_anchor(
    point: crate::widget::AnchorPoint,
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
    match point {
        AnchorPoint::TopLeft => (anchor_x, anchor_y),
        AnchorPoint::Top => (anchor_x - w / 2.0, anchor_y),
        AnchorPoint::TopRight => (anchor_x - w, anchor_y),
        AnchorPoint::Left => (anchor_x, anchor_y - h / 2.0),
        AnchorPoint::Center => (anchor_x - w / 2.0, anchor_y - h / 2.0),
        AnchorPoint::Right => (anchor_x - w, anchor_y - h / 2.0),
        AnchorPoint::BottomLeft => (anchor_x, anchor_y - h),
        AnchorPoint::Bottom => (anchor_x - w / 2.0, anchor_y - h),
        AnchorPoint::BottomRight => (anchor_x - w, anchor_y - h),
    }
}
