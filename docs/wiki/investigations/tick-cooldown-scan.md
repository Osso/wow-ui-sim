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

`~/.cache/asset-resolver/data/community-listfile.sqlite` is shared across projects, but freshness compares the recorded source CSV path and mtime. game-engine worktrees record their own `data/community-listfile.csv`; wow-ui-sim records `~/.cache/asset-resolver/data/community-listfile.csv`. Each project therefore rebuilds all ~2.1M rows on first lookup, on the GUI thread; a run killed before `COMMIT` restarts the rebuild next launch. Fix belongs in asset-resolver; not yet addressed.

## Sources

- [app.rs](../../../src/iced_app/app.rs) — tick interval and cooldown checks
- [registry/mod.rs](../../../src/widget/registry/mod.rs) — `cooldown_ids` index
- [app_tests.rs](../../../src/iced_app/app_tests.rs) — behavior coverage

## See Also

- [[on-update-dirty]] — other per-tick dirty/handler costs
- [[rendering-pipeline]] — draw path measured alongside
- [[casc-asset-cache]] — asset-resolver cache layout
