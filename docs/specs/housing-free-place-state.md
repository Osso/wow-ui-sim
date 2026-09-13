# Housing free-place state

Pinned retail 12.0.0 signatures declare `C_HousingBasicMode.IsFreePlaceEnabled() -> boolean` and `SetFreePlaceEnabled(boolean)` with no returns. Unmodified housing controllers and the standard binding contain the matching query/set call path. Loaded-consumer proof is separately scoped below.

## Contract

- Explicit true/false writes roundtrip through the getter, including repeated toggles.
- Getter returns one boolean; setter returns no values.
- State belongs to each simulator environment's housing state; changing it does not mutate another environment or unrelated housing-service state.
- Rust publication starts at retail 12.0.0. The former hardcoded-true getter and no-op setter are removed from the shared temporary bootstrap.
- Simulator policy retains initially enabled state through `HousingState.free_place_disabled = false`. This inverse flag avoids changing the existing housing default initializer. Native default and reset behavior are unverified.
- Simulator validation requires a boolean; native coercion/errors are unverified.

## Evidence

`tests/housing.rs`, committed through `220897051`: RED 2/4; arity and unrelated housing-service preservation passed, while explicit false and environment isolation failed at the unchanged true getter. Runtime `3839f707f` adds per-environment state in `src/c_api/c_housing/basic_mode.rs`. Independent proof at `3839f707f` passes 4/4 each on retail 12.0.0/12.0.5/12.0.7, plus fmt/check/build/startup (`[]`)/readability. Historical warnings remain 6/6/1. Ledger: `/tmp/verify-housing-freeplace-ledger.json`.

Separately, current-default loaded unmodified `HousingFramesUtil.SetFreePlaceEnabled` forwarded false/true/false and returned zero values, with its debug source identified. This is not native WoW, historical-profile consumer or full-LoD proof. Correction-only metadata verification at `804a6b073` awards exactly two bounded credits: 14,917 fresh hashes, zero stale, 139 renewals and 14 additions; totals **2315 / 1093 / 2**. The 1,119-row blocker snapshot retains 282 plans, 281 unchanged from before this credit.

## Gaps

No placement, collision, rendering, persistence, reset/lifecycle or event-order behavior is established. The separate current-default forwarder check does not establish placement-controller or keybinding execution, native behavior, or 12.0.0/12.0.5/12.0.7 loaded UI consumers. Native validation, earlier-profile availability and secret/taint enforcement remain unproven.
