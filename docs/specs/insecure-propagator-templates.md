# Insecure propagator templates

Retained 12.0.5 source [`12.0.5-api-changes.txt`](../../data/patch-api/sources/12.0.5-api-changes.txt), line 173 (`prose-2026-03-12-173`), says “Add the following insecure templates for addon usage” and names the four templates below. Exact definitions were read from `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_FrameXML/SecureTemplatesBase.xml`; tests load that relative file from the cached active-profile path, unchanged, through the normal TOC/XML loader. See [XML template architecture](../xml-template-system.md).

## What it must do

- [ ] `CreateFrame` inheriting `InsecureMouseMotionPropagatorTemplate` reports motion propagation true and clicks false (`propagateMouseInput="Motion"`).
- [ ] `InsecureMouseClicksPropagatorTemplate` reports clicks true and motion false (`propagateMouseInput="Clicks"`).
- [ ] `InsecureKeyboardInputPropagatorTemplate` reports keyboard propagation true (`propagateKeyboardInput="true"`).
- [ ] `InsecureHyperlinkPropagatorTemplate` reports parent hyperlink propagation true (`propagateHyperlinksToParent="true"`).
- [ ] Ordinary no-template Frame controls retain false propagation flags before and after template instances are created.

## How it works

- [XML template system](../xml-template-system.md)
- [Widget system](../widget-system.md)

## Implementation inventory

- `tests/insecure_propagator_templates.rs`: three grouped cached-template inheritance tests, gated to `retail-12-0-5` plus `profile-retail`/`client-ptr`.
- `src/loader/xml_file.rs`: normal cached XML loading.
- `src/lua_api/frame/methods/misc/propagation.rs`: public mouse and hyperlink getters.
- `src/lua_api/frame/methods/text_attribute_event/events.rs`: keyboard propagation getter.

## Tests asserting this spec

`tests/insecure_propagator_templates.rs`: `insecure_propagator_templates_inherit_separate_mouse_masks`, `insecure_propagator_templates_inherit_keyboard_input_flag`, `insecure_propagator_templates_inherit_hyperlink_parent_flag`.

## Known gaps (current cycle)

- [ ] Execution proof pending parent-owned batch9; active batch8 excludes this file. Tests may already pass. No RED/GREEN or native parity claimed.
- [ ] Parent must wire this module into an existing grouped runner: `autotests = false`; this slice changes no manifests or global test registration.

## Out of scope

Physical mouse/keyboard input, hyperlink delivery, callbacks, protected writes and taint behavior need future native probes; names alone do not establish security privileges. Neighboring `SecureFrameTemplate` explicitly declares `protected="true"`; the four insecure definitions declare neither protection nor secure-template inheritance. Tests deliberately assert propagation settings only, not security semantics. Missing cached fixtures fail explicitly; no fabricated definitions or conditional skips.
