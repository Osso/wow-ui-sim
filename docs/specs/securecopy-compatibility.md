# Securecopy compatibility

## Contract

- Preserve the existing retail 12.1/PTR compatibility helper and expose the same helper to Forever before addon loading and secure-environment creation.
- Copy table keys and values recursively, preserving cycles and shared references within the resulting graph.
- Preserve non-table values, including curve userdata identity.
- Do not copy metatables or replace an already-defined `securecopy`.
- Do not enable the rest of the PTR bootstrap on Forever.

## Evidence and limits

Forever `Blizzard_AuraContainer/Blizzard_AuraContainerShared.lua` exports option/default tables through `securecopy`. `tests/wowforever_securecopy.rs` covers that actual source consumer, independent nested copies, cycles, table keys, absent metatables, and curve identity.

This preserves existing simulator compatibility semantics, not native security or taint conformance. The helper remains under `src/lua_api/workarounds/temporary/` until native security/taint semantics are modeled. Other profiles retain their existing exposure.
