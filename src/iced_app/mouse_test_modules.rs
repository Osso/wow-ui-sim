use super::*;

#[path = "mouse_test_support.rs"]
mod test_support;

#[path = "mouse_tests.rs"]
mod tests;

#[cfg(any(feature = "client-retail", feature = "client-wrath"))]
#[path = "simple_html_tests.rs"]
mod simple_html_tests;

#[cfg(feature = "client-mists")]
#[path = "mouse_game_menu_tests.rs"]
mod game_menu_tests;

#[path = "mouse_drag_scaled_tests.rs"]
mod drag_scaled_tests;

#[path = "mouse_registration_tests.rs"]
mod registration_tests;

#[path = "mouse_party_tests.rs"]
mod party_tests;

#[path = "mouse_hover_tests.rs"]
mod hover_tests;

#[path = "mouse_hit_grid_tests.rs"]
mod hit_grid_tests;

#[path = "mouse_wheel_tests.rs"]
mod wheel_tests;
