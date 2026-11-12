# Texture template anchors

XML texture instances retain template anchors unless the instance declares its own anchors. The resolver lives in `crates/sim-xml/src/template.rs`; see [XML templates](../wiki/systems/xml-template-system.md).

## What it must do

- [ ] Preserve template anchors for XML-loaded and runtime-created frame children, including the half-width backgrounds and 51-pixel top inset used by PTR 70493 `Blizzard_PlayerSpells/Mainline/SpellBook/Blizzard_SpellBookTemplates.xml:41-51`.
- [ ] Use instance anchors in place of inherited anchors when explicitly supplied.

## How it works

- [XML template system](../wiki/systems/xml-template-system.md)

## Implementation inventory

- `crates/sim-xml/src/template.rs`: merges anchors with other texture properties.

## Tests asserting this spec

- `tests/spellbook_layout.rs::spellbook_texture_templates_preserve_half_page_anchors`: concrete half-page geometry and explicit instance replacement on both creation paths.
- `tests/spellbook_layout.rs::spellbook_layout_stays_locked`: unchanged original Blizzard spellbook geometry consumer.

## Known gaps (current cycle)

- [ ] Complete targeted profile proof.

## Out of scope

Native empty-anchor and conflicting `setAllPoints` precedence, recursive texture-template chains, rendering changes, and vendor edits.
