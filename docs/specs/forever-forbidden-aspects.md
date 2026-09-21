# Forever forbidden-aspect consumers

Forever shares the existing forbidden-aspect model and query/mutation methods with Retail 12.1. Native enum publication follows each client's authored mask: Retail 12.1 has eleven bits; PTR 12.1.5 and Forever additionally expose `QueryAnimationProgress` and `AddAnimations`.

## What it must do

- [ ] Publish Forever's thirteen native mask values through 4096 and matching metadata, plus propagation paths `Hierarchy=0` and `Layout=1`.
- [ ] Let unchanged SecureHandlers accept an unmarked frame and reject a frame with forbidden aspects, including its static FrameRef method invocation.
- [ ] Allow addon-tainted creation of the actual CustomAuraContainer shell and native aura-button group, retaining authored forbidden aspects and one-shot dirty dispatch.
- [ ] Preserve existing hierarchy inheritance and reject later parent/anchor changes that would implicitly acquire forbidden aspects.
- [ ] Project partitioned script objects into the destination environment across secure/global calls and returns. Secure callers receive the interned private table, global callers receive the original public table, and each partition retains its own fields and methods.
- [ ] Return the matching parent partition from `GetParent`; `GetObjectTable` exposes the public object table without replacing the private native mixin.

## How it works

- [Forbidden-aspect inheritance](forbidden-aspect-inheritance.md).
- [Ellesmere investigation](../wiki/investigations/ellesmereui-forever.md).

## Implementation inventory

- `Cargo.toml`: shared base capability and version-specific animation-mask extension.
- `src/lua_api/globals/enum_data/{widget,mod}.rs`: native mask/path values and metadata.
- `src/lua_api/env_init/enums.rs`: obsolete PTR-only duplicate extension removed.
- `src/lua_api/frame/methods/text_attribute_event/mod.rs`: forbidden methods and the demonstrated `GetObjectTable` dependency shared independently of unrelated Retail APIs.
- `src/lua_api/script_object_transfer.rs`: existing environment-transfer hook and parent projection enabled with the shared forbidden-aspect capability; no replacement aura display implementation.
- `src/lua_api/frame/methods/forbidden_aspects.rs` and `button_anchor_hierarchy/{anchors,hierarchy}.rs`: existing state queries and relationship enforcement gates.

## Tests asserting this spec

- `tests/forever_forbidden_consumers.rs` and `tests/fixtures/forever_forbidden_consumers.lua`: native SecureHandlers and tainted aura consumer boundaries, masks, and relationship rejection.
- `tests/userdata_proxy.rs::forbidden_partition_secure_global_roundtrip_preserves_identity_and_fields`: secure → global creation → secure return, global callback arguments, final public returns, parent identity, isolated fields, and native private mixin availability.

## Known gaps (current cycle)

- [ ] Compiled GREEN and full Ellesmere replay remain pending. After `GetObjectTable` exposure, the unchanged native aura provider still fails at `Blizzard_AuraContainerFrameProviders.lua:90` because the outbound-created button is not projected back to its private table. Precise consumer RED: `/tmp/ellesmere-forever/forbidden-followup-green.{stdout,stderr}` (2 passed, 1 failed).

## Out of scope

- Unrelated access-restriction methods, `ClearScripts`, or the whole Retail epoch.
- Optional-mask filtering in `HasAnyForbiddenAspects`: existing implementation answers whether any bit is set; demonstrated SecureHandlers consumer supplies no mask.
- New animation restriction enforcement: this slice publishes the native extension values; it does not claim previously unimplemented enforcement.
- Vendor/addon edits, synthetic native masks, and suppressed errors.
