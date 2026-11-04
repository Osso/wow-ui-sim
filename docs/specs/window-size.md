# GUI window size

`wow-sim --window-size` selects the initial GUI client area and fixed game display in physical pixels. The CLI reference is in [CLI commands](../wiki/reference/cli-commands.md).

## What it must do

- [x] Accept `WIDTHxHEIGHT` or `WIDTHXHEIGHT`, with positive unsigned 32-bit integer dimensions. Invalid dimensions produce `expected WIDTHxHEIGHT with positive integer dimensions`.
- [ ] Reject the option with a subcommand: it is available only when opening the GUI. The option is available only in GUI builds.
- [x] Seed the physical display before addons load and derive the initial UI scale from its height; honor imported manual UI scale when applicable without persisting that override.
- [x] Keep the game display dimensions fixed when the GUI canvas is resized, scaling the display to the canvas instead of changing its layout.
- [x] Map canvas pointer coordinates to the game layout; a release outside the canvas ends a game drag without a click.
- [x] Rasterize glyphs at native texels without changing layout dimensions.

## How it works

- [CLI commands](../wiki/reference/cli-commands.md)
- [Rendering pipeline](../rendering-pipeline.md)

## Implementation inventory

- `src/bin/wow_sim/main.rs` — argument parsing and GUI-only validation.
- `src/bin/wow_sim/startup.rs` and `saved_var_config.rs` — initial display and UI scale.
- `src/iced_app/` — window sizing, canvas mapping and pointer dispatch.
- `src/font/` — glyph rasterization.

## Tests asserting this spec

- `src/bin/wow_sim/startup.rs` and `saved_var_config.rs` — dimensions, validation and startup scale.
- `src/iced_app/app_tests.rs`, `render_tests.rs` and `mouse_tests.rs` — resize and pointer behavior.
- `src/iced_app/` glyph raster scale test — native texels and layout.

## Known gaps (current cycle)

- [ ] Live GUI and visual screenshot comparison, physical DPI behavior, overlay usability and actual window-manager resize behavior are not proved.
- [ ] Other profiles, screenshots and end-to-end SavedVariables/session persistence were not tested in the PR verification. The available test results cover Retail and direct test binaries.

## Out of scope

- Headless subcommands: this flag configures the GUI window, not screenshot or dump dimensions.
