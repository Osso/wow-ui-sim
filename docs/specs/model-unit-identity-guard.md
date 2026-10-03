# Model unit identity guard

Bounded non-3D observable behavior for retail 12.0.5 `prose-2026-04-10-194` and `prose-2026-04-10-195`, sourced from [API change text](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 194–195. These rows are not wholly within the permanent 3D rendering gap: identity denial and one-nil/no-error results are observable without rendering.

## What it must do

- [ ] ModelSceneActor:SetModelByUnit and Model:SetUnit deny an existing unit identity classified secret by explicit host state.
- [ ] Denied calls return exactly one public nil without throwing, preserving the previous stored unit binding.
- [ ] Classify the resolved identity GUID, not party-token spelling; target aliases of a secret identity remain denied.
- [ ] Both methods store a public existing unit token as a non-rendering binding and return one true for that assignment. **INFERRED**: this boolean measures simulated binding assignment, not successful native model loading.
- [ ] A missing unit identity returns one false without mutation. **INFERRED** simulator policy; native missing-unit behavior is not established here.
- [ ] Clearing host classification permits subsequent assignment; independent environments do not share classification or bindings.
- [ ] Ordinary public tokens whose identities are secret return nil for both secure and genuinely tainted addon callers, without changing caller taint.

## How it works

- [Widget state](../widget-system.md)
- [Lua state and method dispatch](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/unit_misc.rs`: shared state-backed predicate used by name/GUID getters and the model guard, delegating to `src/lua_api/globals/real/instanced_identity.rs`. Explicit GUID classification and host instance context follow the same [instanced identity policy](instanced-identity.md); token spelling does not classify secrecy.
- `src/lua_api/frame/methods/widgets/model/model_unit.rs`: common unit-token assignment/denial implementation.
- `src/lua_api/frame/methods/widgets/model.rs`: method selection for retail epochs; other profile methods remain unchanged at compile time.
- `src/lua_api/state/sim_state.rs`: existing `identity_secret_guids` and `instance_identity` host inputs, unchanged.
- Existing widget model state: `player_model_state.last_unit`, reused without new shadow state.

## Tests asserting this spec

- `tests/cast_events_identity.rs`: both real methods, prior binding, exact pcall arity, GUID alias, host recovery, missing unit, genuine addon taint, environment isolation, and instance-map restriction with ownership/map-exit recovery.

## Known gaps (current cycle)

- [ ] Model `SetUnit` registration is overwritten by later GameTooltip `SetUnit` registration on the shared frame metatable. Model calls currently execute tooltip behavior instead of the model guard; widget-aware dispatch is required.
- [ ] Post-fix compilation and test execution remain unrun; predicate unification alone does not resolve the `SetUnit` dispatch collision.

## Out of scope

- 3D geometry/model loading, rendering, auto-dressing, and visual-option side effects remain intentional permanent gaps. Do not credit simulated assignment as model-load success or whole-row native parity.
- Automatic map/world acquisition. Model binding consumes the existing host-declared [instanced identity policy](instanced-identity.md); it does not acquire instance context or establish native parity for that subsystem.
- Identity tokens not represented by the existing GUID resolver (including pet/vehicle/raid mappings) are not fabricated.
- Full AllowedWhenUntainted parity for every optional flag and secret argument is not credited. Unit token unwrapping uses the existing VM authentication; tests here cover public tokens and actual caller taint.
