# Lua math extensions

Retail 12.1.5 adds native functions to the Lua `math` table. The simulator exposes the documented numeric surface used by PTR `Blizzard_SharedXMLBase/MathUtil.lua`.

## What it must do

- [x] Under `retail-12-1-5` or `client-wowforever`, publish `math.clamp`, `isfinite`, `isinf`, `isnan`, `lerp`, `normalize`, `remap`, `round`, `saturate`, `sign`, and `wrap`.
- [x] `round` rounds halfway values away from zero at positive or negative decimal places.
- [x] `wrap` uses `[minimum, maximum)` and returns `minimum` when endpoints are equal.
- [ ] Do not publish these extensions to earlier API epochs.

## How it works

- [Lua API](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/math_extensions.rs` — native `math` table functions.
- `src/lua_api/globals/register.rs` — PTR 12.1.5 and Forever registration; Forever does not enable a retail epoch.

## Tests asserting this spec

- `patch-tests/patch_12_1/math_extensions.rs` — PTR numeric behavior.
- `tests/wowforever_math.rs` — real Forever `MathUtil.lua` aliases and consumers, all eleven documented primitives, and earlier-profile exclusion. Source: Forever 1.60.1.69913 `LuaMathExtensionsDocumentation.lua`. This proves simulator behavior, not native conformance.

## Known gaps (current cycle)

- [ ] Exact live-client behavior for invalid or degenerate ranges beyond equal-endpoint `wrap` is not captured.

## Out of scope

- Secret-value propagation semantics from the API documentation.
