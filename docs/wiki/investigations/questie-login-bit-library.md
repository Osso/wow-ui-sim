# Questie login coroutine: Lua `bit` fallback (lt-face)

After the [metadata index](addon-metadata-lookup-performance.md), Questie 12.9.0 + QuestieDB 1.1.1 on Mists still ran its login coroutine (ThreadLib timer 12) for 86 callbacks and about 15–22 s. Almost all of it was the simulator's Lua implementation of the `bit` library. A native `bit` library ([contract](../../specs/bit-library.md)) cut timer 12 to 0.67–1.02 s.

## Symptoms

- VERIFIED (Mists release, `998b136cd`): timer 12 ran 86 callbacks, 12.96–21.92 s in total, longest 257–378 ms. `dump-tree` took 25.5–39.2 s to settle.
- VERIFIED (flat and call-graph `perf`): `execute_inner`, rilua taint bookkeeping (`propagate_slot_read_taint`, `TaintKey` SipHash, `get_closure_taint`) and table-security checks. These symbols do not show which Lua code runs.

## Root cause

- VERIFIED (Lua count-hook sampling of Questie's coroutine threads, 25,636 samples every 20,000 instructions): 97.7% of the login thread's samples were in the fallback's `fold`, `band2` (`step`), `normalize` and `band` lines. `Townsfolk._PopulateTownsfolkTypes` (`Townsfolk.lua:55`) calls `bitband(flags, folkType.mask)` for every NPC and townsfolk type: at least 624,656 calls per login.
- rilua ships no `bit` library. The simulator installed a Lua fallback in `compat_overrides.rs`. Each `bit.band` call built a `{...}` table, called `math.floor` and `tonumber` through `_G`, and looped once for each bit.
- VERIFIED (frame-pointer `perf`, samples under `timer_dispatch::fire` → `co_resume`): within timer 12, `propagate_slot_read_taint` is 22.6% inclusive and `get_closure_taint` is 11.2%. The fallback is an untainted Lua function, so each of its `_G` reads (`math`, `tonumber`) looked up per-slot taint, and each call to a helper looked up a closure taint stamp. INFERRED: this taint cost follows from the Lua fallback and is not a separate root cause.

## Fix

- `src/lua_api/globals/real/bit_library.rs` implements the same functions in Rust and repeats the old f64 steps in the same order. The Lua fallback is removed.
- VERIFIED: a 40,056-call deterministic differential corpus (nil holes, numeric strings, NaN, infinities, negatives, fractions, values past 2^53, all eight functions) gives byte-identical output on the old and new binaries.

## Measurements (Mists release with frame pointers, back-to-back A/B, 2026-11-17)

| | Old (`998b136cd`) | New (`8c569d711`) |
|---|---:|---:|
| Timer 12 total, 2 runs | 21,923 / 12,964 ms | 1,023 / 671 ms |
| Timer 12 longest callback | 378 / 286 ms | 101 / 49 ms |
| `dump-tree` settle | 39.2 / 25.5 s | 18.2 / 7.6 s |
| `lua-errors` with Questie | 10.7 s, `[]` | 6.8 s, `[]` |
| Timer-12 `perf` samples (999 Hz) | 15,436 | 771 |

## Remaining

- VERIFIED: after the fix, about 77% of the `dump-tree` samples are addon loading, mostly TOC parsing work (SipHash, `HashMap<String, String>` insert and clone, `StrSearcher`). That area belongs to the separate TOC-parse-once work.
- rilua overheads that stay, proposed in the handoff report and not changed here: an interned-string registry lookup and `String` allocation for every call (`get_closure_taint`), and `Vec<u8>` key copies with SipHash for every slot-taint read in frames without their own taint stamp.
