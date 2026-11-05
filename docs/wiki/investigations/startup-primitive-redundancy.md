# Startup Primitive Redundancy

Two simulator-side startup optimizations remove duplicate bookkeeping without changing Blizzard Lua. SetText optimization remains deferred; its new tests preserve current behavior only.

## Changes

`LayoutCache.resolved` already receives a zero-layout sentinel before parent/anchor recursion. Recursive lookups return that sentinel before reaching the separate private `visiting` set. Removing that set and its begin/finish probes preserves the sentinel, final overwrite, independent `recomputed` set and invalidation. Cyclic anchor results remain entry-order-dependent; this is intentionally preserved, not corrected.

UI-string replacement on Retail discarded an unconditional raw lookup. Restoration performed that lookup twice with no intervening write. Registration now makes one policy decision: Preserve reads once and retains every non-nil value; Replace reads only for Forever's function-preservation rule. Integer/float/string constants, raw lookup helpers, interning and security behavior are unchanged.

## Bounded preservation

Tests-first `85e8f8eda` precedes layout `f74ecfc64` and registration `ee9b563d8`. Retail/default dev opt1: **42/42 selected tests PASS both before and after**, including seven new cases. Coverage includes cyclic geometry/effective scale/cache removal/missing frames; raw registration values, function/table identity and absence of `__index` callbacks; text nil/empty/Unicode/NUL, repeated public SetText after style changes, and existing layout/cursor/callback tests.

Six additional after-only existing unit cases also pass (five core layout tests and one post-event-layout case), giving 48 after cases versus 42 before. Default Retail `cargo check`, whole-workspace formatting and changed-line readability checks pass. Documentation-only follow-ups do not invalidate the sealed code/test scope.

This does not establish native parity, all-profile acceptance, rendered-cycle equivalence or a performance gain. Forever's branch is source-inspected, not executed in this Retail-only gate.

## Measurement

Three captures per sealed binary completed with unchanged `--no-addons --no-saved-vars lua-errors`, `WOW_SIM_CASC=0`, `WOW_SIM_NO_SOUND=1`, cold/recompiling bytecode-cache access, 4 GiB scopes and private copies of current CVar bytes. All six startups return exit 0 and `[]`; original CVar stat/hash guards pass. Sampling uses `cpu-clock:u`, 199 Hz, DWARF 65528, 512K mmap and the prior explicit-CFI recovery algorithm.

**No gain demonstrated.** Wall-time variance is large and main reported concurrent Syncthing disk/CPU contention during an incomplete Retail 70391→70448 install sync. No install extraction, refresh or re-pin was performed. These observations cannot establish a final speedup or causal regression. Three captures also lose samples; recovered complete-root coverage is about 94–96%, so inclusive percentages are qualified observations.

Neither seal exposes a standalone `table_get_with_key` ELF symbol. Its observed zero standalone-frame count is not zero cost: the report separately measures `table_get`/`table_get_static` carrier frames and the broader UI-string registration envelope. SetText remains an unchanged control, not an optimized path.

Private per-run wall times, inclusive means/standard deviations/variances, seals, argv, recovery limits, admission correction and proof ledger: private handoff report `perf-build/report.md` (not committed). No gain claimed from source deletion or historical profile percentages.

## Sources

- [Layout implementation](../../../src/layout.rs) and [cycle preservation](../../../tests/layout_cycle.rs).
- [String registration and tests](../../../src/lua_api/globals/strings/mod.rs).
- [Text preservation tests](../../../src/lua_api/frame/methods/text_attribute_event/text/tests.rs).
- [Profiling protocol](../../../.agents/skills/flamegraph/SKILL.md) — acquisition guidance; private report records the matched cold-start protocol and recovery limitations.

## See Also

- [[layout-profile]] — historical release layout profile; not the current A/B comparison.
- [[intern-string-ranking]] — related interning investigation; no interner redesign here.
