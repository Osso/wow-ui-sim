# Retail 12.1.0 events — C06

Bounded simulator proof for [prose-2026-06-23-083; prose-2026-06-23-088](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L83: current EventRegistrations guards reject tainted callers and allow secure callers; eight event APIs, retained listener/callback delivery and unrestricted controls have simulator proof. This is not native parity for every guarded operation.
- [x] L88: the real AuraContainer intrinsic has EventRegistrations applied; Blizzard's registration survives, addon registration/query/clear is rejected, secure registration succeeds and plain-frame controls remain unrestricted. Named c75 PASS supersedes pending status, not the historical mask-zero RED.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/events.rs` — caller-sensitive event guards
- `src/lua_api/frame/methods/forbidden_aspects.rs` — intrinsic default and stored mask
- Cached `Blizzard_AuraContainer/Blizzard_AuraContainer.xml` — intrinsic constructor; default aspect is simulator-side, not a vendor XML patch

## Tests asserting this spec

- `tests/forbidden_aspect_creation.rs` — generic caller/guard and listener controls
- `tests/forbidden_aspect_enforcement.rs::aura_container_intrinsic_event_registrations_block_addons_not_blizzard` — actual intrinsic and secure/addon controls

[Saved c75 case/body reconciliation](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/c75-pending-spec-reconciliation-2026-10-11/README.md): intrinsic case PASS at raw line5837; related guard cases PASS at5792–5793. Current requested source/test bodies match c75. These executions are already in the suite totals, not new reruns. The enclosing suite remains FAIL on Garrison.

## Known gaps (current cycle)

- [ ] Native-client/default-aspect and per-operation parity remain unverified. Earlier mask-zero failure/excluded-test history is preserved; c75 establishes the bounded current intrinsic/control assertions only. Do not infer a ManagedAuraContainer obligation solely from its name.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
