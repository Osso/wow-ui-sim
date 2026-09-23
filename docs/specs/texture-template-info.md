# Texture template metadata

`C_XMLUtil.GetTemplateInfo` exposes registered texture templates as well as frame templates, using existing XML registry state. Datamine's map canvas consumes its texture-template dimensions during `OnLoad`.

## What it must do

- [ ] Return type `Texture`, resolved dimensions and KeyValues for a registered texture template.
- [ ] Preserve inherited texture KeyValues alongside instance values through the existing resolver.
- [ ] Allow a 64×64 tile template to initialize a 64×64-tile canvas to 4096×4096.
- [ ] Return independent metadata snapshots; preserve existing frame results and unknown-template `nil`.

Texture support is inferred from unchanged Datamine `MapController.lua` and its declared 64×64 `DatamineMapTileTemplate`. Cached Forever XMLUtil documentation provides the metadata shape; this is not native verification. Existing resolver limits and zero defaults for absent dimensions are unchanged.

## How it works

- [XML template system](../xml-template-system.md)
- [Texture KeyValues](texture-keyvalue-inheritance.md)

## Implementation inventory

- `src/xml/template.rs`: reads registered texture templates and resolves metadata.
- `src/c_api/c_xml_util.rs`: serializes existing metadata structure into Lua results.

## Tests asserting this spec

- `tests/datamine_xml_lifecycle.rs::texture_template_info_sizes_datamine_map_canvas`: actual XML registration, map-canvas OnLoad, inherited fields, snapshot isolation, frame and unknown-name controls.

## Known gaps (current cycle)

- [ ] Re-evaluate unchanged Datamine startup/UI after the metadata producer is implemented.

## Out of scope

Native conformance, font-string metadata, new case-folding or recursive inheritance semantics, map data acquisition and 3D rendering.
