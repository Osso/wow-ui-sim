# Cursor overlay

- Talent cursor payloads render the ability icon before the pointer in the topmost GUI overlay.
- Talent IDs may identify nodes, entries or definitions. Nodes use their selected entry, or their first entry when unselected. Definitions prefer visible, override, then base spell IDs; unresolved IDs emit no ability icon.
- Existing action, spell and pet-action icons remain unchanged; item, macro, money and transmog-outfit icons are outside this port.

Regression: `iced_app::render::tests::cursor_talent_icon_renders_in_overlay` observes icon and pointer texture requests for talent definition 137759. Before the port, only the pointer was emitted.
