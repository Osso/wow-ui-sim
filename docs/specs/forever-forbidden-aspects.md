# Forever forbidden-aspect consumers

Forever shares the existing forbidden-aspect model and query/mutation methods with Retail 12.1. Native enum publication follows each client's authored mask: Retail 12.1 has eleven bits; PTR 12.1.5 and Forever additionally expose `QueryAnimationProgress` and `AddAnimations`.

## What it must do

- [ ] Publish Forever's thirteen native mask values through 4096 and matching metadata, plus propagation paths `Hierarchy=0` and `Layout=1`.
- [ ] Let unchanged SecureHandlers accept an unmarked frame and reject a frame with forbidden aspects, including its static FrameRef method invocation.
- [ ] Allow addon-tainted creation of the actual CustomAuraContainer shell and native aura-button group, retaining authored forbidden aspects and one-shot dirty dispatch.
- [ ] Preserve existing hierarchy inheritance and reject later parent/anchor changes that would implicitly acquire forbidden aspects.
- [ ] Project partitioned script objects into the destination environment across secure/global calls and returns. Secure callers receive the interned private table, global callers receive the original public table, and each partition retains its own fields and methods.
- [ ] Return the matching parent partition from `GetParent`; `GetObjectTable` exposes the public object table without replacing the private native mixin.
- [ ] Publish the existing `AddSecretAspect` mask mutation under `forbidden-aspects` (Retail 12.1 and Forever), preserving older-profile absence. Native CustomAuraButton duration setters must retain `Cooldown` and `Shown` on cooldowns and `BarValue` on status bars, observable through both public and private views. This does not expand masks or enforce secret values.

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
- `src/lua_api/frame/methods/misc/secret.rs`: existing `AddSecretAspect` registration and mask mutation use the shared capability. Native Forever declares the method in `SimpleFrameScriptObjectAPIDocumentation.lua`; `Blizzard_CustomAuraButton.lua` calls it in duration cooldown/bar setters.

## Tests asserting this spec

- `tests/forever_forbidden_consumers.rs` and `tests/fixtures/forever_forbidden_consumers.lua`: native SecureHandlers and tainted aura consumer boundaries, masks, and relationship rejection.
- `tests/userdata_proxy.rs::forbidden_partition_` cases: existing ordinary-frame transfer, real provider acquisition and initializer regressions shared with Forever; public/private identity, native parent identity, field isolation and private mixin dispatch remain asserted.

## Known gaps (current cycle)

- [x] At `9476efcf5`, `secure-chain-tests-ledger.json` records native forbidden consumers 3/3 and the native AuraContainer initializer/partition group 8/8 after `AddSecretAspect` publication and scoped secure delegation. This proves the modeled aspect masks, transfer, and initializer boundary.
- [ ] Full Ellesmere GUI aura acceptance remains open. Later runtime tracing finds `C_UnitAuras.GetUnitAuraInstanceIDs` and `C_UnitAurasPrivate.GetAllPrivateAuraInstanceIDs` return nil in both public and secure namespaces before `ParseAllAuras`; this is a separate enumeration-registration gate, not an aspect/identity failure.

## Out of scope

- Unrelated access-restriction methods, `ClearScripts`, or the whole Retail epoch.
- Optional-mask filtering in `HasAnyForbiddenAspects`: existing implementation answers whether any bit is set; demonstrated SecureHandlers consumer supplies no mask.
- New animation restriction enforcement: this slice publishes the native extension values; it does not claim previously unimplemented enforcement.
- Vendor/addon edits, synthetic native masks, and suppressed errors.
