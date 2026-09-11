# Animation factory

`SimpleAnimGroup:CreateAnimation(animationType?, name?, templateName?)` returns an owned animation. Both pinned revisions have the same optional arguments; PTR adds an `AddAnimations` forbidden-aspect check. [Pinned register](../../data/patch-api/sources/12.1.5-register.json). Virtual XML animation definitions are separate from frame/group templates; see [XML templates](../xml-template-system.md).

## What it must do

- [x] Omitted/nil type and name create distinct generic animations with order one and duration zero; explicit type/name and group ownership are retained.
- [x] Named instances advance on the owner's ordered timeline and deliver their own completion callback once.
- [ ] Register top-level virtual animation definitions, including typed animation tags, without creating runtime objects or publishing template names as globals.
- [ ] Resolve inherited animation templates parent-first, left-to-right, and apply their properties through the same path as inline XML. Explicit instance properties override inherited values.
- [ ] Preserve requested runtime type, name, and owner. Template `name`, `virtual`, and `parentKey` do not replace instance identity.
- [ ] Apply timing, alpha, flipbook, supported target configuration, KeyValues, and Scripts before a single `OnLoad` invocation. Inline instance parent keys are bound before that callback; inherited script chaining follows existing loader rules.
- [ ] Reject unknown template names and inheritance cycles before factory instance allocation; do not silently ignore them. This is simulator error policy, not verified native error behavior.
- [ ] Clear the animation registry with other template registries when creating a fresh environment.
- [ ] Preserve inline XML animations and inherited animation-group templates on PTR and earlier retail.

## How it works

- [Animation query lifecycle](animation-query-lifecycle.md)
- [Addon loading](../addon-loading-pipeline.md)
- [XML templates](../xml-template-system.md)

## Implementation inventory

- `src/xml/animation_templates.rs`: dedicated virtual-animation registry and checked inheritance resolution.
- `src/xml/types_animation.rs`, `src/xml/types.rs`: animation attributes/children and top-level typed tags.
- `src/loader/xml_file.rs`: registration of virtual animation declarations.
- `src/loader/helpers_anim.rs`: shared property, key-value, script, and inline-instance application.
- `src/lua_api/frame/methods/button_anchor_hierarchy/animations/templates.rs`: compile checked template application before allocation, then apply to the registered instance.
- `src/lua_api/frame/methods/button_anchor_hierarchy/animations/creation.rs`: optional-template factory integration and named-instance publication.

## Tests asserting this spec

- `tests/animation_factory.rs`: retained real TOC/XML regression, optional arguments, named lifecycle.
- `tests/animation_factory_templates.rs`: inheritance, alpha/flipbook configuration, child target, KeyValues/scripts, OnLoad ordering, instance isolation, invalid/cyclic templates, fresh-environment clearing, and inline/group inheritance.
- `tests/xml_animation_group_onload.rs`, `tests/animation_group.rs`: existing regressions.

## Known gaps (current cycle)

- [ ] Run focused new/existing animation and loader tests for both profiles after implementation. RED at `8819650b5` reproduced duration zero instead of `1.75`; expanded RED suite also fails missing inheritance/scripts/typed-template behavior (`/tmp/animation-templates-red-ptr.log`).
- [ ] Existing engine methods for transforms/origin and named/key targets remain no-ops; XML values are dispatched through the shared path, not claimed as functional motion/target support. `childKey` targeting is tested.
- [ ] Path control points/curve and texture-coordinate offsets have no shared XML application support. No implementation claim for those fields, vertex-color animation data, or native coercion/errors.
- [ ] Template script runtime errors can occur after instance registration. Unknown names/cycles and generated-chunk syntax errors are checked before factory allocation; arbitrary script failures are not transactional.
- [ ] Native lifecycle, script environment/security, template-type compatibility, and forbidden-aspect behavior remain unverified.

## Out of scope

Vendor edits, a broad FrameTemplate rewrite, new animation engine capabilities, native security enforcement, broad suites, final check/readability gates, audit artifact credit, delegation, publishing, and deployment are excluded from this bounded implementation task.
