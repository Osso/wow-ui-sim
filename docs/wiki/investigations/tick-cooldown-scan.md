# Tick Cooldown Registry Scan

A settled GUI session spent ~46% of main-thread self time finding Cooldown widgets. The per-tick cooldown checks scanned the entire widget registry; the registry now indexes Cooldown widget IDs.

## Content

### Symptom

2026-09-24 release build, `--no-addons --no-saved-vars`, ~40s settled window, `perf` self time on the main thread:

| Symbol | Self |
|--------|------|
| `is_visible_active_cooldown_widget` | 24.8% |
| `App::compute_tick_interval` | 12.2% |
| `Vec<u64>` filter-collect | 7.7% |

Draw (quad rebuild + textures) was p50 3.8ms; ticks were p50 16.6ms under perf. Rendering was not the bottleneck. Host load was ~50 on 24 cores, so absolute times are inflated.

### Root cause

`has_active_cooldown_widget` and `active_cooldown_widget_ids` (`src/iced_app/app.rs`) iterated `WidgetRegistry::iter_ids()` — every frame, one hash lookup each — to find the few `WidgetType::Cooldown` widgets, even when none were active. Callers:

- `compute_tick_interval`, called from `subscription()` after every iced message (mouse moves, redraws)
- `drop_stale_timer_tick`
- `mark_active_cooldown_widgets_dirty`, once per timer tick

### Fix

`77089bb12`: `WidgetRegistry::register` maintains `cooldown_ids`; the cooldown checks iterate `cooldown_ids()` only. `register` is the sole insertion point, frames are never removed, and `widget_type` is never reassigned, so insert-time sync is complete. Re-registering an ID with a non-Cooldown type removes it from the index.

Tests (`src/iced_app/app_tests.rs`): active/slow/fast mod-rate tick intervals, hidden-parent cooldown stays idle, per-tick dirty marking including the final redraw after `Clear()`, and index tracking across re-registration.

### Profiling notes

- The release build has no frame pointers and `perf --call-graph dwarf` did not unwind; use `perf report --no-children -g none --sort dso,sym` for self time.
- A first profile was invalid: the main thread spent the whole window rebuilding the shared asset-resolver community listfile cache (see below).

### Shared listfile cache thrash (asset-resolver)

`~/.cache/asset-resolver/data/community-listfile.sqlite` is shared across projects, but freshness compares the recorded source CSV path and mtime. game-engine worktrees record their own `data/community-listfile.csv`; wow-ui-sim records `~/.cache/asset-resolver/data/community-listfile.csv`. Each project therefore rebuilt all ~2.1M rows on first lookup, on the GUI thread; a run killed before `COMMIT` restarted the rebuild next launch. Worktrees symlinking one CSV also thrashed, because the unresolved symlink path was recorded.

Fixed in asset-resolver `3088dee` (pinned by `c3e373f9c`): the cache file is `community-listfile-<fnv1a(canonical source path)>.sqlite`, and sources are canonicalized so symlinks share their target's cache. Verified: first headless run built the new cache (30.6s), second reused it (11.9s, file untouched). The legacy `community-listfile.sqlite` stays in use by game-engine builds that predate the fix.

### Idle texture warmup rescans (follow-up)

After the cooldown fix, ~8% of settled main-thread self time was `preload_visible_textures_for_tick`: every idle tick cloned every cached texture-request path into a `HashSet<String>`, sorted them, and probed each against the texture cache, even when all were loaded. Two emptiness checks (`remaining_texture_work_after_warmup`, `preload_pending_render_requests_for_tick`) cloned the same full path set just to test `is_empty()`.

- `7535af12e`: skip the warmup until cached strata change. Strata rebuild on ~87–97% of settled ticks (no-addons dirty set: `TabardModel`, `QueueStatusFrame`, `QueueStatusButton`, `MicroMenu`, `FramerateFrame`, a few anonymous textures), so this cut preload p50 but not the tail.
- `946d8255b`: `texture_warmup_unsettled_strata` bitmask — warmup collects paths only from strata rebuilt/reset since the last settle; emptiness checks use `has_cached_render_requests()`. CPU texture caches never evict, so skipped strata stay verified. Retries and resizes unsettle all strata.

Result: `HashSet<String>::insert` 4.3% → 0.13% self time; preload p90 1.51ms → 0.62ms. Tests in `src/iced_app/render/pending_texture_tests.rs` (full and partial-stratum rebuild) fail under mutation of either invalidation site. `on_update` rose between measurements across 29 unrelated commits; unattributed.

### Every-frame re-anchoring (follow-up)

Dirty-frame probing showed the same frames dirty on idle ticks: `MicroMenu`, `QueueStatusButton`, `QueueStatusFrame`, `FramerateFrame`, `HelpOpenWebTicketButton`. Blizzard's `GridLayoutFrameMixin:Layout()` returns before `MarkClean()` when grid settings are unchanged, so `MicroMenu.dirty` stays true and its `OnUpdate` re-runs `MicroMenuMixin:Layout()` every frame, re-anchoring those frames to identical points (Blizzard behavior; not patched).

`672c32aab`: anchor setters (`ClearAllPoints`, `ClearPoint`, `SetPoint`, `SetAllPoints`, `AdjustPointsOffset`) record the frame's anchors and rect before its first edit (`src/widget/registry/anchor_edits.rs`). When the dirty set is read, the frame is marked visually dirty only if its anchors changed or a freshly resolved rect differs — the rect check covers a same-anchor reapply after the target moved. Layout invalidation is unchanged. Tests: `src/loader/tests/anchor_render_dirty.rs`; the target-moved case fails under an anchors-only mutation.

`bench_steady_state` A/B (host load 18–20, noisy): draw min p50 2.60/1.24ms → 0.41/0.46ms; draw/tick ratio per round 1.08–1.14 → 0.44–0.67. Every measured frame still uploads a stratum: remaining idle dirt includes `TabardModel`, a cast-bar texture, an NPE texture, and `QueueStatusButtonIcon` `Show()`.

## Sources

- [app.rs](../../../src/iced_app/app.rs) — tick interval and cooldown checks
- [registry/mod.rs](../../../src/widget/registry/mod.rs) — `cooldown_ids` index
- [app_tests.rs](../../../src/iced_app/app_tests.rs) — behavior coverage

## See Also

- [[on-update-dirty]] — other per-tick dirty/handler costs
- [[rendering-pipeline]] — draw path measured alongside
- [[casc-asset-cache]] — asset-resolver cache layout
