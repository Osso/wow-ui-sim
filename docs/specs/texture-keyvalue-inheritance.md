# Texture KeyValue inheritance

XML textures retain inherited and instance `KeyValues` when resolving texture templates. The resolved values must be available to parent lifecycle handlers, including Datamine's custom-atlas controls. Source: `src/xml/template.rs`.

## What it must do

- [x] Keep template KeyValues not overridden by the texture instance.
- [x] Apply instance KeyValues after template values so duplicate keys use the instance value, including explicit `false`.
- [x] Expose the resulting fields before the parent's `OnLoad` consumes the texture; do not bypass or manually initialize the consumer.

These are simulator XML inheritance requirements supported by the unchanged Datamine control templates, not native-client probe evidence.

## How it works

- [XML template system](../xml-template-system.md)
- [Forever addon investigation](../wiki/investigations/forever-addon-comparison.md)

## Implementation inventory

- `src/xml/template.rs`: combines texture-template fields.
- `src/loader/xml_texture.rs`: assigns resolved KeyValues to texture objects.

## Tests asserting this spec

- `tests/datamine_xml_lifecycle.rs::inherited_texture_instance_key_values_reach_parent_onload`: real addon XML loading, inherited/default fields, explicit false override, and the parent's custom-atlas initialization.

## Known gaps (current cycle)

- [ ] Prove unchanged Datamine controls initialize and re-evaluate startup/UI failures.

## Out of scope

Atlas catalog expansion, native or pixel conformance, 3D rendering, and unrelated texture inheritance behavior. Loader error reporting is a separate contract; silent partial-creation recovery must not turn failed initialization into clean-startup evidence.
