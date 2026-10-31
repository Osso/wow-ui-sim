//! Window icon for the iced desktop app. Defers pixel rendering to the
//! always-compiled `app_icon_render` module so the same artwork backs both the
//! runtime window icon and the build-time `installer/wow-sim.ico` generator.

use iced::{Size, window};

#[cfg(target_os = "linux")]
use crate::app_icon_render::FREEDESKTOP_APP_ID;
use crate::app_icon_render::{SIZE, render_icon};

pub(super) fn settings() -> window::Settings {
    window::Settings {
        size: initial_window_size(),
        icon: window::icon::from_rgba(render_icon(), SIZE, SIZE).ok(),
        platform_specific: platform_settings(),
        ..window::Settings::default()
    }
}

#[cfg(target_os = "linux")]
fn platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        application_id: FREEDESKTOP_APP_ID.to_string(),
        ..Default::default()
    }
}

#[cfg(not(target_os = "linux"))]
fn platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific::default()
}

pub(super) fn initial_window_size() -> Size {
    Size::new(1024.0, 768.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_include_runtime_icon() {
        assert!(settings().icon.is_some());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_settings_use_freedesktop_app_id() {
        assert_eq!(
            settings().platform_specific.application_id,
            FREEDESKTOP_APP_ID
        );
    }
}
