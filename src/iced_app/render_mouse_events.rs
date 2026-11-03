use iced::widget::shader;
use iced::{Point, Rectangle, Size, mouse};

use super::super::Message;
use super::super::state::CanvasMessage;

/// Map a mouse event to a canvas message action.
pub(super) fn handle_mouse_event(
    mouse_event: &mouse::Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
    scene_size: Option<Size>,
) -> Option<shader::Action<Message>> {
    let to_scene = |position: Point| {
        scene_size.map_or(position, |size| {
            Point::new(
                position.x / bounds.width * size.width,
                position.y / bounds.height * size.height,
            )
        })
    };
    let cursor_position = || cursor.position_in(bounds).map(to_scene);

    match mouse_event {
        mouse::Event::CursorMoved { position } => {
            let covered_by_controls = scene_size.is_some() && cursor.is_levitating();
            let message = if bounds.contains(*position) && !covered_by_controls {
                let local = Point::new(position.x - bounds.x, position.y - bounds.y);
                CanvasMessage::MouseMove(to_scene(local))
            } else {
                CanvasMessage::MouseLeave
            };
            publish_canvas_event(message)
        }
        mouse::Event::CursorLeft => publish_canvas_event(CanvasMessage::MouseLeave),
        mouse::Event::ButtonPressed(button) => handle_button_pressed(*button, cursor_position()),
        mouse::Event::ButtonReleased(button) => handle_button_released(*button, cursor_position()),
        mouse::Event::WheelScrolled { delta } if scene_size.is_none() || cursor.is_over(bounds) => {
            publish_scroll_event(delta)
        }
        _ => None,
    }
}

fn handle_button_pressed(
    button: mouse::Button,
    position: Option<Point>,
) -> Option<shader::Action<Message>> {
    let to_message = match button {
        mouse::Button::Left => CanvasMessage::MouseDown,
        mouse::Button::Right => CanvasMessage::RightMouseDown,
        mouse::Button::Middle => CanvasMessage::MiddleClick,
        _ => return None,
    };

    publish_cursor_event(position, to_message)
}

fn handle_button_released(
    button: mouse::Button,
    position: Option<Point>,
) -> Option<shader::Action<Message>> {
    let message = match (button, position) {
        (mouse::Button::Left, Some(position)) => CanvasMessage::MouseUp(position),
        (mouse::Button::Right, Some(position)) => CanvasMessage::RightMouseUp(position),
        (button, None) => CanvasMessage::mouse_up_outside(button)?,
        _ => return None,
    };
    publish_canvas_event(message)
}

fn publish_cursor_event(
    position: Option<Point>,
    to_message: impl Fn(Point) -> CanvasMessage,
) -> Option<shader::Action<Message>> {
    position.and_then(|point| publish_canvas_event(to_message(point)))
}

fn publish_scroll_event(delta: &mouse::ScrollDelta) -> Option<shader::Action<Message>> {
    let dy = match delta {
        mouse::ScrollDelta::Lines { y, .. } => *y,
        mouse::ScrollDelta::Pixels { y, .. } => *y / 30.0,
    };
    Some(shader::Action::publish(Message::Scroll(0.0, dy)))
}

fn publish_canvas_event(message: CanvasMessage) -> Option<shader::Action<Message>> {
    Some(shader::Action::publish(Message::CanvasEvent(message)))
}
