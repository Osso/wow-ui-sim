# Script-object access restrictions

Retail-family 12.1 script objects store conditional access-restriction flags separately from their forbidden flag. The contract comes from cached `Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua`; `Blizzard_AuraContainerUtil.lua` applies the flags while creating aura buttons.

## What it must do

- [ ] `AddAccessRestrictions(mask)` accumulates the supplied mask with bitwise OR; zero and repeated additions do not remove existing flags.
- [ ] `GetAccessRestrictions()` returns the object's stored mask, initially zero.
- [ ] `HasAnyAccessRestrictions()` and an explicit nil argument test for any stored restriction; a supplied mask tests for any intersecting bit, with zero matching none.
- [ ] `HasAccessConstraints()` reports true for either the existing forbidden flag or a nonzero conditional-restriction mask.
- [ ] Restrictions remain per-object and do not change the forbidden flag or another object's mask.

## How it works

- [Frame data flow](../frame-data-flow.md)
- [Protected frame enforcement](../protected-frame-enforcement.md)

## Implementation inventory

- `src/widget/frame.rs`, `frame_defaults.rs` — zero-default `access_restrictions` mask.
- `src/lua_api/frame/methods/text_attribute_event/access_restrictions.rs` — mask mutation and queries.
- `src/lua_api/frame/methods/text_attribute_event/mod.rs` — registration under `retail-12-1-0`.
- `src/lua_api/frame/methods/text_attribute_event/attributes.rs` — combined `HasAccessConstraints` query.

## Tests asserting this spec

- `tests/forbidden_frames.rs` — mask accumulation/filtering, object isolation, and combination with forbidden state.

## Known gaps (current cycle)

- [ ] Targeted Rust tests await compilation; an existing-binary RED reproduced missing `AddAccessRestrictions`.
- Aura-secrecy activation and conditional API-access enforcement are not implemented by this mask/query slice. Existing taint and forbidden-state checks are unchanged. No claim of complete security enforcement is made.

## Out of scope

Inventing aura-secrecy policy, context-dependent access checks, new restrictions, or changing addon/vendor Lua. Only `DenyTaintedAccessWhenAurasAreSecret = 1` currently has a named enum value; generic mask algebra does not establish live-client validation rules for unnamed bits.
