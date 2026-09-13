# Housing free-place state

Pinned retail 12.0.0 signatures declare `C_HousingBasicMode.IsFreePlaceEnabled() -> boolean` and `SetFreePlaceEnabled(boolean)` with no returns. Unmodified housing controllers and the standard binding toggle query/set this state.

## Contract

- Explicit true/false writes roundtrip through the getter, including repeated toggles.
- Getter returns one boolean; setter returns no values.
- State belongs to each simulator environment's housing state; changing it does not mutate another environment or unrelated housing-service state.
- Rust publication starts at retail 12.0.0. The former hardcoded-true getter and no-op setter are removed from the shared temporary bootstrap.
- Simulator policy retains initially enabled state through `HousingState.free_place_disabled = false`. This inverse flag avoids changing the existing housing default initializer. Native default and reset behavior are unverified.
- Simulator validation requires a boolean; native coercion/errors are unverified.

## Evidence

`tests/housing.rs`, committed through `220897051`: RED 2/4; arity and existing housing-service preservation passed, while explicit false and environment isolation failed at the unchanged true getter. Runtime implementation is in `src/c_api/c_housing/basic_mode.rs`. Post-change proof pending.

## Gaps

No placement, collision, rendering, persistence, reset/lifecycle or event-order behavior is established. Unmodified consumer source supports the toggle contract; API-level tests do not establish execution of the loaded UI consumer. Native validation, earlier-profile availability and secret/taint enforcement remain unproven.
