# Rilua GC Scan Reduction

Plan to cut GC mark cost in rilua. Three sequenced tracks plus one investigation.

## Measured motivation

From `/tmp/claude/perf.data` (release `lua-errors`, --no-addons --no-saved-vars, 2026-04-17):

- `Gc::traverse_table` self-time: 2.55% (50M cycles)
- `Gc::mark_value`: 1.19%
- Total GC category: ~6% of startup cycles

Top traversed tables (measured via `dump-tree --exec-lua`):

| Table | Entries | Track |
|---|---|---|
| `_G` | 66,331 | 2 (freeze) |
| `_G.__secureenv` | 28,327 | 2 (freeze) |
| `REGISTRY.__rilua_frame_refs` | 47,333 | 1 (pin frame tables) |
| `REGISTRY.__rilua_frame_fields` | 47,333 | 3 (investigate) |
| `REGISTRY.__scripts` | 20,713 | 3 (investigate) |

Combined Tracks 0+1+2 remove **~142k entries** from every GC mark cycle. Track 3 addresses the remaining ~68k.

## Track -1: Defer GC during bootstrap and per tick

Independent of every other track. Ship first.

Rilua exposes the Rust API directly — `state.gc_stop()`, `state.gc_restart()`, `state.gc_step(budget: i64)`, `state.full_gc()`, `state.gc_count()` (defined at `~/Repos/rilua/src/vm/state.rs:1612`). No need to call `collectgarbage("stop")` from Lua and pay the stringly-typed dispatch.

### Why it works

- **Bootstrap allocates monotonically.** Frames never deleted, globals stay around, bytecode constants retained. Almost nothing becomes garbage during startup. Stopping GC = stopping the *scan*, not stopping *cleanup*. Memory cost is near zero.
- **Tick allocations are small.** Per-tick churn is event objects + temporary closures. Easily handled by one bounded `gc_step` at end-of-tick.
- **Kills `barrier_back` overhead.** PLAN.md notes the barrier short-circuits when `phase != Propagate`, but stopping GC makes that 100% guaranteed during bootstrap.
- **Cache locality.** One contiguous GC cycle is friendlier than many small interleaved slices.

### Ordering with Track 2

The full collect MUST run before `freeze_table`. Otherwise bootstrap transients (parser scratch, addon load temporaries, etc.) get pinned permanently by the freeze walk:

```
gc_stop → bootstrap → full_gc → freeze_table(_G) → freeze_table(__secureenv) → gc_restart
```

### Phases

- [x] Phase -1a — Wow-sim: at start of `register_globals`, call `state.gc_stop()`. After bootstrap + addon loads, call `state.full_gc()` then `state.gc_restart()`. (landed 651334c1)
- [x] Phase -1b — Wow-sim: in the OnUpdate dispatch loop, wrap each tick — `gc_stop` at entry, `gc_step(budget)` at exit. Starting budget 1024 (one `GCSTEPSIZE`). (landed 0376da7b)
- [x] Phase -1c — Measure wall time before/after. See "Measured results" below.

### Measured results (2026-04-17)

`lua-errors` wall time, release build, 5 runs `--no-addons --no-saved-vars`, 3 runs with Blizzard + third-party addons. Medians reported. Baseline = commit `8ef4b347` (pre-Track -1); after = tip with both brackets installed.

| Path | Before | After | Delta |
|---|---|---|---|
| `--no-addons --no-saved-vars lua-errors` | 2.753 s | 1.682 s | **-39%** |
| `--no-saved-vars lua-errors` (with addons) | 2.965 s | 1.652 s | **-44%** |

Bigger than projected: deferring the collector across the monotonic startup region avoids both the mark-phase walks and the auto-step checks that piled up as `_G` and the frame registry grew. The with-addons path shows the larger absolute win because addon loading contributes the bulk of the allocation.

## Track 0: Quick win (independent)

`traverse_table` re-borrows the arena once per element:

```rust
// ~/Repos/rilua/src/vm/gc/collector.rs:438-460
for i in 0..array_len {
    let val = self.tables.get(r).and_then(|t| t.array_get(i));
    if let Some(val) = val { self.mark_value(val); }
}
```

For a 47k-entry table, that's 47k arena lookups per traversal. Hoist the borrow:

```rust
let mut values: SmallVec<[Val; 64]> = SmallVec::new();
if let Some(t) = self.tables.get(r) {
    for i in 0..array_len {
        if let Some(v) = t.array_get(i) { values.push(v); }
    }
    for i in 0..hash_count {
        if let Some((k, v)) = t.hash_node_kv(i) {
            if v.is_nil() { continue; }
            if !weak_keys { values.push(k); }
            if !weak_values { values.push(v); }
        }
    }
}
for v in values { self.mark_value(v); }
```

Single arena borrow per table. Expected: ~50M fewer arena lookups (2.5% of startup cycles). No semantic change. Independent of every other track.

- [ ] Phase 0a — Rilua: hoist the borrow in `traverse_table` (`~/Repos/rilua/src/vm/gc/collector.rs:438-460`).

## Track 1: Pin frame backing tables (smallest change, immediate win)

### Insight

Frames in wow-ui-sim are **never deleted**. Confirmed: `WidgetRegistry::register` exists at `src/widget/registry/mod.rs:86`, but no remove/delete method anywhere. Frame re-creation (AGENTS.md: `Blizzard_GameTooltip` orphan case) creates a NEW id; the old entry lingers. `__rilua_frame_refs` and `__rilua_frame_fields` grow monotonically.

Frame backing tables (the values stored in `__rilua_frame_refs`) are effectively **leaf objects** in GC terms. They hold:

- A `(lo, hi)` u32 backing pair — not a GcRef.
- The shared `__rilua_frame_mt` metatable — bootstrap-allocated, lives forever.

Crucially: addon-mutated state does NOT live on the frame backing table. Per AGENTS.md, custom fields go to `__rilua_frame_fields[id]` (the env table at `debug.getfenv(frame)[1]`). The backing table itself never gains new outgoing references after creation.

### What pinning gets us

If we pin the frame backing table at creation:

- It stays alive without `__rilua_frame_refs` needing to mark it.
- Its metatable is shared with all frames; either pin it once, or let Track 2 freeze it as part of `_G` (it's stored under `__rilua_frame_mt` in the registry).
- We can mark `__rilua_frame_refs` itself as skip-traverse: its values don't need scanning to stay alive.

Result: **47,333 entries off every GC cycle.**

### What pinning does NOT get us

`__rilua_frame_fields` is a separate problem. Field tables hold:

- Custom properties (`frame.MyData = someTable`)
- Script handlers as Lua closures with upvalues
- EditMode overrides (`SetPointOverride`, `ClearAllPointsOverride`)
- Mixin overrides

These reference fresh closures, tables, and other Lua-heap objects that change at runtime. We can't transitively pin them. Field tables stay on the normal GC scan path.

Same story for `__scripts` (closures with upvalues).

### Design

New rilua API:

```rust
impl Gc {
    /// Marks an object as permanently reachable. The collector will:
    ///   - never sweep it (treated as Black for sweep)
    ///   - never include it in mark traversal (no descendant scan)
    /// Caller must ensure the object holds no references to non-pinned,
    /// non-frozen objects, or those references will dangle after sweep.
    pub fn pin_object<T>(&mut self, r: GcRef<T>);

    /// Convenience: mark a table as skip-traverse during mark, but still
    /// participate in sweep normally. Used for "the values inside are
    /// kept alive by other roots, so don't bother scanning."
    pub fn mark_table_no_traverse(&mut self, r: GcRef<Table>);
}
```

`pin_object` is for things we know are leaves and immortal (frame backing tables, the shared `__rilua_frame_mt`).

`mark_table_no_traverse` is for tables whose contents are kept alive by other means (`__rilua_frame_refs` once its frame values are pinned).

### Phases

- [x] Phase 1a — Rilua: added `Flag::Pinned` and `Flag::SkipTraverse` bits (FLAG_PINNED = 1<<1, FLAG_SKIP_TRAVERSE = 1<<2) alongside existing `FLAG_FROZEN`. (rilua commit 16b06b3)
- [x] Phase 1b — Rilua: `Gc::pin_object(Val)` sets Pinned; `Gc::mark_table_no_traverse(GcRef<Table>)` sets SkipTraverse. `traverse_table` short-circuits to Black when SkipTraverse is set; `Arena::sweep` / `sweep_partial` keep Pinned entries alive. (rilua commit be718b6)
- [x] Phase 1c — Wow-sim: `frame_ref` pins each backing table after `attach_frame_metatable`; `init_frame_metatable` pins the shared metatable + its `__index` clone. (wow-sim commit 8bb02284)
- [x] Phase 1d — Wow-sim: `register_globals` ends with `mark_frame_ref_cache_no_traverse(state)` on `__rilua_frame_refs`. (wow-sim commit f8f7db3d)
- [x] Phase 1e — Re-flamegraph. `traverse_table` median cycles dropped from 145.9M to 126.5M (−13.3%) over 3 perf runs after Phase 1. `__rilua_frame_refs` traversal samples are zero by construction (rilua unit test `skip_traverse_and_pinned_together_survive_full_gc_without_marking_children`). The 25% target assumed a pre-Track-1 baseline; Track -1 had already taken most of the eligible walks offline, leaving Phase 1 to capture the residual. Combined Track -1 + Phase 1 drop vs the original 242M baseline is ~48%.

### Risks

1. **What if a frame backing table ever does gain a non-trivial outgoing reference?** Audit `attach_frame_metatable` and any `raw_set` on a frame Val::Table. Today the only writes are during the initial setup (metatable + backing pair). If wow-sim ever caches per-frame state directly on the backing table (not in fields), pinning becomes unsafe.
2. **Tests that rely on frame GC.** Some tests may expect frames to be collectable — e.g., `frame = nil; collectgarbage("collect"); assert(weakref:get() == nil)`. Pinning breaks weak-ref tests for frames. Audit `tests/` for weak references to frames.
3. **Frame re-creation memory cost.** AGENTS.md describes the orphan case where `CreateFrame` with an existing name produces a new id, leaving the old frame stranded. With pinning, the stranded old frame is now permanently retained too. Memory cost: ~200 bytes × the orphan count. Audit `register_new_frame` and `migrate_children_to_new_frame` for orphan-frequency baseline.

## Track 2: Freeze `_G` and `__secureenv` after bootstrap

### Insight

`_G` (66k) and `__secureenv` (28k) grow during bootstrap and are essentially stable afterwards. Built-ins, `Enum.*` tables, mixin definitions, global constants. Addon runtime code mostly reads these; when it writes, it writes to its own tables.

Snapshot at the bootstrap boundary, transitively pin every reached object, forbid mutation. GC skips traversal entirely.

### Why naive pinning of `_G` alone doesn't work

Pinning `_G` without pinning its values: `_G.UIParent` is a separate GC object. If `_G` isn't traversed, `UIParent` is seen as white and swept.

Correct approach: transitively pin every object reachable from `_G` in a single walk at freeze time, then forbid mutation. Without a barrier, pinned objects can't write references to non-pinned objects (target would be swept). Forbidding all mutation removes the need for a barrier and the need for any future scan.

### Design

```rust
impl Gc {
    /// Walks the object graph from `root`, pinning every reached table,
    /// function, closure, and userdata. After this call:
    ///   - All reached objects are permanently retained (no sweep).
    ///   - GC mark phase skips traversal of frozen objects.
    ///   - Writes to any frozen table raise a Lua error (initial policy).
    pub fn freeze_table(&mut self, root: GcRef<Table>);
}
```

Implementation reuses Track 1's `Pinned` flag, plus a `Frozen` flag that makes `raw_set` reject writes.

### Shadow-table policy (wow-sim side)

Addons do occasionally overwrite built-ins (`_G.print = myprint`). Wrap `_G` with a metatable proxy at freeze time:

```lua
-- _G_live is a regular Lua table, starts empty
-- _G_frozen is the pre-freeze snapshot (frozen via Gc::freeze_table)
-- _G proxy: reads check live first, then frozen; writes always go to live
```

`_G_live` participates in GC normally. It'll grow with addon-declared globals — probably a few hundred entries after full startup, not 66k.

Same treatment for `_G.__secureenv`.

### Phases

- [x] Phase 2a — Rilua: `Flag::Frozen` already present (shared flags byte with Pinned / SkipTraverse).
- [x] Phase 2b — Rilua: `Gc::freeze_table(root)` BFS over tables/closures/upvalues/userdata, landed in rilua commit `d0d9aec`.
- [x] Phase 2c — Rilua: `raw_set` write path raises "attempt to modify a frozen table"; stdlib `rawset` gets equivalent inline gate. (rilua 5993351)
- [x] Phase 2d — Wow-sim: `_G_live` shadow + proxy metatable installed around `freeze_table(_G)`, gated on `WOW_SIM_FREEZE_GLOBALS=1`. (wow-sim 1bace1dc)
- [x] Phase 2e — Wow-sim: same flow extended to `_G.__secureenv` via `freeze_root_with_shadow`. (wow-sim 2d4fcf38)
- [x] Phase 2f — Re-flamegraph. See "Measured results — Phase 2g" above. End-to-end measurement blocked on overwrite audit; single-run opt-in shows `traverse_table` relative share drops from 2.36% to 0.62% (~74% reduction), though ON-run total is ~3.6× smaller because the overwrite errors truncate Blizzard UI loading. Full combined measurement needs the audit to land so the opt-in can safely load the entire addon set.

### Risks

1. **Bootstrap is not a clean boundary.** Some Blizzard UI files load late and still register globals. Pick the freeze moment carefully: after Blizzard UI, before user addons. `WOW_SIM_NO_ADDONS` makes this trivial; full load needs a deliberate hook.
2. **`rawset(_G, key, value)` callers.** `rawset` bypasses the metatable proxy. Audit `src/` and `tests/` for `rawset(_G, ...)` and route those through the proxy or the live table.
3. **Reentrancy during freeze walk.** Use raw access only (`raw_get`, no metamethods).
4. **Weak tables inside `_G`.** Skip them during the freeze walk and don't descend into them; they stay on normal GC.
5. **Finalizers.** Confirm no `__gc` equivalents in the bootstrap graph (rilua doesn't expose `__gc` on userdata today — low risk).

## Track 3: Investigate `__rilua_frame_fields` and `__scripts`

These hold dynamic, addon-mutated state. They can't be frozen or pinned-transitively. They're the remaining ~68k entries (47k + 20k) on the GC scan path after Tracks 1–2 land.

Open questions:

- [ ] What's the actual scan cost of these two tables after Tracks 1–2? Re-flamegraph and measure. May be small enough to leave alone.
- [ ] What fraction of `__scripts` entries are added at bootstrap (Blizzard frames' OnLoad / OnShow / OnEvent) vs. runtime?
- [ ] Could `__rilua_frame_fields` be reshaped to keep only frequently-mutated entries in a "hot" table and stable entries in a "cold" frozen table? Most `EditMode` overrides, mixin overrides, and bootstrap-set custom properties never change.
- [ ] Could `__scripts` use a parallel-array layout (`Vec<(frame_id, handler_id, closure)>`) instead of a hash map? Cheaper to scan; addresses different access patterns.

Decision deferred until Tracks 1–2 land and we re-measure.

## Sequencing

1. **Track 0** (traverse_table hoist) — independent, ship first. Pure perf.
2. **Track 1** (pin frame backing tables) — smallest wow-sim change, biggest single-table win.
3. **Track 2** (freeze `_G` + `__secureenv`) — bigger wow-sim change (proxy logic), biggest combined win.
4. **Track 3** (investigate the rest) — only if there's still meaningful GC cost after 0–2.

## Alternatives considered

- **Pure generational GC (Lua 5.4 port)**: more general, but Tracks 1+2's freeze/pin approach is ~10× simpler and captures more of the entry-count savings. Frames-never-deleted invariant makes generational unnecessary for our workload.
- **Immortal-root flag without transitive pinning**: unsound — cascade sweeps descendants.
- **Mark-only-if-modified per entry**: requires full barrier tracking, which is essentially the generational machinery.
- **Bigger GC pause multiplier**: trades heap size for less frequent collection. Doesn't touch per-collection work. Orthogonal; can stack.
- **Move `__rilua_frame_refs` off the Lua heap**: rejected — values must be Lua values for `debug.getregistry().__rilua_frame_refs[id]` to work from addon Lua code.
