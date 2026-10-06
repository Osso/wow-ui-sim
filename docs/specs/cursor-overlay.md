# Cursor overlay

- Talent cursor payloads render the ability icon before the pointer in the topmost GUI overlay.
- Tooltip queries and cursor icons share `src/c_api/talent_spell.rs` lookup, preventing divergent interpretation.
- Talent IDs may identify nodes, entries or definitions. Nodes use their selected entry, or their first entry when unselected. Definitions prefer visible, override, then base spell IDs; unresolved IDs emit no ability icon.
- Existing action, spell and pet-action icons remain unchanged; item, macro, money and transmog-outfit icons are outside this port.
- Live IPC screenshots append the same overlay after UI quads. When canvas mouse position is absent, the simulator mouse position still renders the cursor for host-driven input.
- `cursor_overlay_uses_sim_mouse_position_when_app_position_is_empty` and `cursor_overlay_is_last_in_screenshot_batch` cover host-driven positioning and final screenshot geometry/order. Standalone screenshots without cursor state are unchanged.

Regression: `iced_app::render::tests::cursor_talent_icon_renders_in_overlay` observes icon and pointer texture requests for talent definition 137759. Before the port, only the pointer was emitted.
