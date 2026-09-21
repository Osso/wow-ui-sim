# Forever forbidden-aspect consumers

Forever shares the existing forbidden-aspect model and query/mutation methods with Retail 12.1. Native enum publication follows each client's authored mask: Retail 12.1 has eleven bits; PTR 12.1.5 and Forever additionally expose `QueryAnimationProgress` and `AddAnimations`.

## What it must do

- [ ] Publish Forever's thirteen native mask values through 4096 and matching metadata, plus propagation paths `Hierarchy=0` and `Layout=1`.
- [ ] Let unchanged SecureHandlers accept an unmarked frame and reject a frame with forbidden aspects, including its static FrameRef method invocation.
- [ ] Allow addon-tainted creation of the actual CustomAuraContainer shell and native aura-button group, retaining authored forbidden aspects and one-shot dirty dispatch.
- [ ] Preserve existing hierarchy inheritance and reject later parent/anchor changes that would implicitly acquire forbidden aspects.

## How it works

- [Forbidden-aspect inheritance](forbidden-aspect-inheritance.md).
- [Ellesmere investigation](../wiki/investigations/ellesmereui-forever.md).

## Implementation inventory

- `Cargo.toml`: shared base capability and version-specific animation-mask extension.
- `src/lua_api/globals/enum_data/{widget,mod}.rs`: native mask/path values and metadata.
- `src/lua_api/env_init/enums.rs`: obsolete PTR-only duplicate extension removed.
- `src/lua_api/frame/methods/text_attribute_event/mod.rs`: four forbidden methods split from unrelated Retail APIs.
- `src/lua_api/frame/methods/forbidden_aspects.rs` and `button_anchor_hierarchy/{anchors,hierarchy}.rs`: existing state queries and relationship enforcement gates.

## Tests asserting this spec

- `tests/forever_forbidden_consumers.rs` and `tests/fixtures/forever_forbidden_consumers.lua`: native SecureHandlers and tainted aura consumer boundaries, masks, and relationship rejection.

## Known gaps (current cycle)

- [ ] Compiled GREEN and full Ellesmere replay remain pending. Identical Lua fixtures reproduced the missing method and unknown XML aspect against the existing binary.

## Out of scope

- Unrelated access-restriction methods, `GetObjectTable`, `ClearScripts`, or the whole Retail epoch.
- Optional-mask filtering in `HasAnyForbiddenAspects`: existing implementation answers whether any bit is set; demonstrated SecureHandlers consumer supplies no mask.
- New animation restriction enforcement: this slice publishes the native extension values; it does not claim previously unimplemented enforcement.
- Vendor/addon edits, synthetic native masks, and suppressed errors.
