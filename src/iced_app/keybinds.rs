//! Keyboard key mapping from iced keys to WoW key names.

/// Convert an iced keyboard key to a WoW key name string.
pub(super) fn iced_key_to_wow(key: &iced::keyboard::Key) -> Option<String> {
    use iced::keyboard::Key;
    match key {
        Key::Named(named) => iced_named_key_to_wow(named),
        Key::Character(c) => Some(c.to_uppercase()),
        _ => None,
    }
}

/// Convert an iced named key to a WoW key name.
fn iced_named_key_to_wow(named: &iced::keyboard::key::Named) -> Option<String> {
    use iced::keyboard::key::Named;
    let s = match named {
        Named::Escape => "ESCAPE",
        Named::Enter => "ENTER",
        Named::Tab => "TAB",
        Named::Space => "SPACE",
        Named::Backspace => "BACKSPACE",
        Named::Delete => "DELETE",
        Named::ArrowUp => "UP",
        Named::ArrowDown => "DOWN",
        Named::ArrowLeft => "LEFT",
        Named::ArrowRight => "RIGHT",
        Named::Home => "HOME",
        Named::End => "END",
        Named::PageUp => "PAGEUP",
        Named::PageDown => "PAGEDOWN",
        Named::Insert => "INSERT",
        Named::F1 => "F1",
        Named::F2 => "F2",
        Named::F3 => "F3",
        Named::F4 => "F4",
        Named::F5 => "F5",
        Named::F6 => "F6",
        Named::F7 => "F7",
        Named::F8 => "F8",
        Named::F9 => "F9",
        Named::F10 => "F10",
        Named::F11 => "F11",
        Named::F12 => "F12",
        _ => return None,
    };
    Some(s.to_string())
}
