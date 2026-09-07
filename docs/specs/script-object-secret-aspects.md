# Script-object secret aspects

Retail 12.1 aura initialization requires explicit secret-aspect declarations and an update-mode enum. The modeled surface lives in `src/lua_api/frame/methods/misc/secret.rs`; [script-object environments](script-object-environments.md) describes the separate public/secure boundary.

## What it must do

- [ ] `AddSecretAspect` accumulates an independent per-object bitmask, initially zero, without removing prior bits or changing sibling objects.
- [ ] `HasSecretAspect` and `HasAnySecretAspect` combine explicit bits with existing derived state. `HasSecretValues` includes explicit declarations without changing the existing protected/forbidden-only behavior.
- [ ] Preserve existing anchoring, protection, and prevention queries; turning prevention off does not erase explicit declarations.
- [ ] Publish `Enum.CustomAuraButtonUpdateMode` (`Assignment=0`, `Update=1`) and metadata (`MinValue=0`, `MaxValue=1`, `NumValues=2`) before secure-environment initialization.

## How it works

- [Script-object environments](script-object-environments.md)
- [Access restrictions](script-object-access-restrictions.md) — distinct conditional-access mask

## Implementation inventory

- `src/widget/frame.rs`, `frame_defaults.rs` — explicit mask storage and default.
- `src/lua_api/frame/methods/misc/secret.rs` — declaration and query union with derived state.
- `src/lua_api/globals/enum_data/{widget,mod}.rs` — early epoch-gated enum registration.

## Tests asserting this spec

- `tests/secret_value_security.rs` — accumulation, isolation, derived-state preservation, presentation regions, and public/secure enum values.
- `tests/userdata_proxy.rs` — existing real aura-provider initialization boundary; unchanged by this slice.

## Known gaps (current cycle)

- [ ] Focused GREEN verification awaits the shared Cargo build slot. Existing provider failures establish missing `AddSecretAspect` and update-mode enum: `/tmp/pi-aura-boundaries-current-green.stderr.log`.

## Out of scope

Per-aspect secret return tagging, context-dependent enforcement, and removal APIs are not implemented by mask storage. This is not a claim of complete secrecy enforcement. Vendor `UpdateAuraDisplay` remains unchanged.

## Native sources

Pinned retail `Blizzard_APIDocumentationGenerated/{SimpleFrameScriptObjectAPIDocumentation,SecretAspectConstantsDocumentation,AuraContainerSharedDocumentation}.lua` establishes the required method, bit values, and update-mode values/metadata.
