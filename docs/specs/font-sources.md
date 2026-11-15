# Font sources

`src/render/font.rs` loads fonts for simulator text drawing and measurement. Addon files and the optional CI fallback pin extend the existing CASC font source. See [font sources](../wiki/systems/rendering-pipeline.md#font-sources).

## What it must do

- [ ] Resolve addon font paths without case sensitivity and use the loaded face's family, weight, style and stretch for drawing and measurement.
- [ ] Reject parent traversal during addon path parsing and reject resolved files outside the canonical AddOns root, including directory and file symlinks. Links within the root remain usable.
- [ ] When CASC supplies no font aliases, use `WOW_SIM_FALLBACK_FONT` if set to a readable font; when unset, preserve the existing system-font selection. An invalid pin warns and uses system fonts, as specified by PR #60.
- [ ] Report unavailable font-test fixtures as failures, not successful skipped coverage.
- [ ] Keep contributor-only native error assertions in `optional-tests` pending independent profile evidence.

## How it works

- [Rendering pipeline](../wiki/systems/rendering-pipeline.md#font-sources).

## Implementation inventory

- `src/render/font.rs`: addon path lookup, root containment, lazy face loading and fallback selection.
- `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs`: missing addon font error before font mutation.

## Tests asserting this spec

- `src/render/font.rs`: path, symlink, face and pinned fallback unit tests.
- `tests/set_font_addon_file.rs`: optional contributor-observed missing-file error and retained font.

## Known gaps (current cycle)

- [ ] Build and run the added tests and affected suites on the combined tree; no runtime proof is claimed yet.
- [ ] Obtain independent native evidence for exact missing-file error text, retained state and profile applicability. The production error is currently cross-profile; a Forever observation does not establish that contract elsewhere.

## Out of scope

Native invalid-font-file behavior and cross-machine pixel identity are not established. The invalid-pin system-font path is compatibility requested by PR #60, not a guarantee of deterministic CI output. Native tests must not treat simulator fallback policy as native evidence.
