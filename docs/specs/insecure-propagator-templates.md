# Insecure propagator templates

Retained 12.0.5 source [`12.0.5-api-changes.txt`](../../data/patch-api/sources/12.0.5-api-changes.txt), line 173 (`prose-2026-03-12-173`), says “Add the following insecure templates for addon usage” and names the four templates below. Exact definitions were read from `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_FrameXML/SecureTemplatesBase.xml`; tests load that relative file from the cached active-profile path, unchanged, through the normal TOC/XML loader.

## What it must do

- [ ] `InsecureMouseMotionPropagatorTemplate` reports motion true and clicks false (`propagateMouseInput="Motion"`); `InsecureMouseClicksPropagatorTemplate` reports clicks true and motion false (`propagateMouseInput="Clicks"`).
- [ ] `InsecureKeyboardInputPropagatorTemplate` reports keyboard propagation true (`propagateKeyboardInput="true"`); `InsecureHyperlinkPropagatorTemplate` reports parent hyperlink propagation true (`propagateHyperlinksToParent="true"`).
- [ ] Ordinary XML instances and runtime `CreateFrame` inheritance apply both boolean attributes independently. Omitted fields preserve inherited values; explicit false overrides true. Chains use existing base-to-derived ordering and multiple templates use existing registry ordering/deduplication; instance declarations win last.
- [ ] No-template Frame controls retain false before and after template instances are created.

Checkboxes remain pending fresh parent-owned integration proof, not implementation inventory.

## How it works

- [XML template architecture](../xml-template-system.md)
- [Widget system](../widget-system.md)

## Implementation inventory

`src/xml/types.rs` models both attributes as `Option<bool>` using standard serde XML attribute annotations. `src/lua_api/globals/template/direct.rs::apply_xml_propagation_flags` resolves both fields and writes the existing Frame booleans without changing public getters. Shared application occurs beside existing propagation handling in ordinary `src/loader/xml_frame/setup.rs::apply_xml_properties_direct` and runtime `src/lua_api/globals/create_frame/template_chain/runtime.rs::apply_runtime_child_direct_properties_with_inherits` (used by runtime template roots and children). No vendor edits or profile-specific application gates.

## Tests asserting this spec

Five cases in the same grouped `tests/insecure_propagator_templates.rs` module, gated to `retail-12-0-5` plus `profile-retail`/`client-ptr`:

- `insecure_propagator_templates_inherit_separate_mouse_masks`: unchanged cached mouse declarations.
- `insecure_propagator_templates_inherit_keyboard_input_flag`: cached keyboard declaration and default controls; labeled Lua assertions.
- `insecure_propagator_templates_inherit_hyperlink_parent_flag`: cached hyperlink declaration and default controls; labeled Lua assertions.
- `insecure_propagator_templates_ordinary_xml_preserves_explicit_false`: inline instance true/false, chained false, independent flags/omissions, both multiple-template orders, defaults.
- `insecure_propagator_templates_runtime_chains_preserve_explicit_false`: public getters after `CreateFrame` with true base, derived false, omitted leaf, independent flags/omissions, both multiple-template orders, defaults.

The inline fixtures use the normal TOC/XML loader, not model/source-shape assertions. No new Cargo target or manifest changes.

## Evidence and pending proof

Actual batch9 at `4f9e1607c` compiled successfully. Saved `/tmp/patch-12.0.5-batch9-integration-5.log` records **1 PASS / 2 FAIL**: mouse passes; keyboard and hyperlink fail at Lua behavioral assertions. Root cause: `FrameXml` omitted both attributes, and ordinary/runtime declarative application had no corresponding flag application; getters already read real Frame booleans defaulting false.

Implementation is committed for a fresh parent build; **GREEN pending**. Existing cached RED covers this root cause; new inline/chain fixtures have not been separately executed RED or GREEN. Filter `insecure_propagator_templates::` now contains five cases. No Cargo, tests, checks, readability or broad gates ran in this bounded implementation slice.

## Out of scope

Gamepad and other input attributes, physical mouse/keyboard input, hyperlink delivery, callbacks, protected writes and taint behavior. Native input/security parity remains unclaimed. Names alone do not establish security privileges: neighboring `SecureFrameTemplate` declares `protected="true"`; the four insecure definitions declare neither protection nor secure-template inheritance. Missing cached fixtures fail explicitly; no fabricated cache definitions or conditional skips.
