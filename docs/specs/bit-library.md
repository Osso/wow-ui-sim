# `bit` library

The client exposes a global `bit` table (`band`, `bor`, `bxor`, `bnot`, `lshift`, `rshift`, `arshift`, `mod`) as native code. rilua has no bit library, so the simulator supplies it on every profile. Addons call it in hot loops: Questie's Mists login coroutine calls `bit.band(npcFlags, mask)` for every NPC and townsfolk type.

## What it must do

- [x] Return the same value as the former Lua compatibility fallback for every input (VERIFIED: 40,056-call differential corpus, identical output; see below).
- [x] Convert each argument as `tonumber(v) or 0`: numbers, numeric strings (hex, exponent, surrounding spaces), and 0 for nil, booleans, tables, functions and non-numeric strings. Then floor it and wrap it into `[0, 2^32)` with Lua's `%`; infinities and NaN become NaN.
- [x] `band` and `bor` fold every argument up to the `#{...}` border of the argument list, so `bit.band(12, nil, 10)` is 0 and `bit.band(12, 10, nil)` is 8. With no arguments they return 4294967295 and 0.
- [x] In `band`, `bor` and `bxor`, a NaN operand behaves as in the old bit loop: it clears every bit in `band`, contributes no bits in `bor`, and in `bxor` sets every bit below the other operand's highest set bit.
- [x] `lshift` wraps its result; a shift that overflows to infinity returns NaN. `rshift` floors and does not wrap. `arshift` is the same function value as `rshift`.
- [x] `bit.mod(a, b)` returns `1 / b` when `tonumber(b)` truncates to zero (`-1 < b < 1`); otherwise it returns `a % b` on the wrapped operands (3.3.5a lbitlib; Retail 12.1.5 observation in Meorawr/elune#12).
- [x] Each call costs a fixed number of native operations; it runs no Lua instructions.

Native argument errors for non-number input are UNVERIFIED and not modelled; the conversions above are the existing simulator contract. The former fallback looked up `tonumber` and `math.floor` in `_G` on every call, so an addon that replaced either global changed its results; the native library does not read them (INFERRED to match the native client, whose `bit` is C code).

## How it works

- `src/lua_api/globals/real/bit_library.rs` — Rust functions that repeat the old Lua arithmetic in the same order with rilua's f64 operators (`%` is `fma(floor(a / b), -b, a)`, `^` is `powf`). Registered from `register_compat_and_admin` only when `bit` is still nil.

## Tests asserting this spec

- `tests/bit_library.rs` — 66 expressions whose expected values were printed by the former Lua fallback.
- `tests/bit_library_perf.rs` (`optional-tests`) — 200,000 Questie-shaped `bit.band` calls under a 2 s budget.
- `tests/utility_api.rs` `test_bit_*` and `compat_overrides::tests::installs_legacy_bit_library` — existing value checks.
