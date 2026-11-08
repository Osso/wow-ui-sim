# Region alpha numeric range

Ordinary numeric `Region:SetAlpha` inputs follow the [10.0.0 change-note summary](../../data/patch-api/sources/10.0.0-api-changes.txt): “Region:SetAlpha() errors if the argument is not between 0.0 and 1.0.” This bounded contract targets Retail/PTR Frame, Texture and FontString values. [Widget architecture](../wiki/systems/widget-system.md) describes stored and effective alpha.

## What it must do

- [x] Accept numeric zero, one and interior values, returning no values.
- [x] Reject ordinary numeric inputs outside the inclusive range, including infinities and NaN, before changing stored or inherited alpha.
- [x] Validate the original numeric precision before narrowing to stored `f32`; values just beyond either boundary must not become valid through rounding.
- [x] Preserve valid-value dirty/no-op behavior. Earlier profiles retain the existing clamping behavior as simulator compatibility, not independently verified native history.

## How it works

- [Widget architecture](../wiki/systems/widget-system.md)
- [Rendering architecture](../rendering-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/core_state/alpha.rs`: public alpha setters and effective-alpha propagation; ordinary Retail/PTR numeric range validation precedes narrowing/mutation, while earlier profiles retain clamping.
- `tests/region_alpha_validation.rs` and `region_alpha_validation/ordinary_range.lua`: concrete ordinary-numeric boundaries and rejection/state-preservation contracts.
- `tests/noop_hot_setters.rs`: existing valid-value dirty/no-op behavior and historical clamping expectation.
- `src/loader/tests/layout_alpha.rs`: Lua-file loading boundary asserts Retail/PTR rejection leaves the seeded alpha unchanged, while earlier profiles retain clamping. The old unconditional clamp assertions failed in the default Retail suite; setter policy is unchanged.

## Tests asserting this spec

[Retained evidence packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/region-alpha-numeric-range-2026-10-13/README.md): 16 whole-text payloads/18 files; initial and closing public-copy/privacy gates passed. Prior 14 payload proofs were reused through Git/stat metadata at closing, not reopened.

Producer `71127a8e1` has bounded Retail/PTR runtime acceptance: five unique Rust cases/profile, **10 passing outcomes**, comprising two numeric contracts and three no-op/dirty-state preservation cases. Frame, Texture and FontString reach all 9 valid and 21 invalid combinations/profile, including near-boundary numbers, infinities and NaN. Rejection preserves stored and effective alpha; getters are not replaced. Nested child summaries are not additional tests.

Two separate normal startups complete both fixture modes with CLEAN diagnostics and `[]`; not additional Rust cases. CASC, sound, third-party addons and SavedVariables are disabled. Normal budgets are 2.5 GiB hard/2 GiB soft; rounded `2G` peaks do not establish below-soft-limit operation. Separate OLD480 Mists observation covers nine clamp/valid combinations, not current-Mists, native history or all-Classic acceptance.

Retained full format and default Retail library/six-bin check pass: 733 JSON records, zero compiler-message records, six manifest lint-key deprecations. Format peak 566022144 bytes exceeds soft high 536870912 by 29151232 bytes, below the 1 GiB hard maximum.

Pre-producer valid modes passed; invalid modes failed at the first Frame `SetAlpha(-0.25)` rejection assertion, leaving later assertions unreached. Original Retail admission denial and PTR setup failure remain zero-launch. Prior PTR4258 authentication/delivery and global historical hash-read total remain UNKNOWN; fresh4259 copy-integrity evidence does not recover them. Public retention is verified at this bounded scope.

Current default Retail repair at compiled`1b94ffff3` additionally passes4 loader Lua-file cases, including both above/below rejection/state-preservation bodies and2 unaffected controls. The [bounded repair matrix](../wiki/investigations/fullsuite-983f02-2026-10-14.md#bounded-repair-follow-up) records independent49-case/default-check/full-format proof and retained diagnostics; earlier-profile/native/full-parent limits are not upgraded.

## Known gaps (current cycle)

- [x] Publish and independently verify privacy-safe evidence retention; this does not close native/render or parent acceptance.
- [ ] Current-Mists/all-Classic and native/render/full-parent acceptance remain unproved; dirty-state assertions are not rendered-pixel proof.

## Out of scope

`SetAlphaFromBoolean`, secret/coercion redesign, other numeric methods and native/render parity are outside this bounded ordinary-numeric contract. Cached current declarations use `SingleColorValue` and allow tainted arguments; the historical range note does not establish their complete secret-value semantics or native error wording.
