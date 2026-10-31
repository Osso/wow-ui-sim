# CLI Commands

Two binaries: `wow-sim` (full UI load, GUI capable) and `wow-cli` (connects to running server or loads standalone for some commands).

**Build before running** — compilation takes 30s+; never use `cargo run` with a timeout. Build with `cargo build --bin wow-sim`, then run with `timeout 90`.

Build `wow-cli` with `cargo build --bin wow-cli --features tools` before using the bare `wow-cli` commands below; `tools` is a Cargo feature, not a runtime argument. See [utility builds](../../../README.md#utility-binary-builds).

GUI, screenshot, dump-tree, and run-tests start with four demo party members. Set `WOW_SIM_NO_PARTY=1` to start solo. See [party administration](../../admin-api/party.md).

## wow-sim Subcommands

### `lua-errors`

Check for Lua errors during startup. Outputs unique errors as JSON to stdout (all other output to stderr). Each JSON `message` keeps the normalized headline plus any traceback lines from the first occurrence. Run this after any stub or API change.

```bash
wow-sim --no-addons --no-saved-vars lua-errors 2>/dev/null
wow-sim lua-errors 2>/dev/null
```

Empty output = zero errors.

### `run-tests <addon>`

Run Lua-level tests from `Interface/AddOns/<addon>/tests/`. For internal logic, prefer `cargo test` instead. Do not run `run-tests Wowless` during normal development — it takes 60s+.

```bash
wow-sim --no-addons --no-saved-vars run-tests MyAddon
wow-sim --addons-path /tmp/test-addons run-tests MyAddon
```

For deterministic timing in addon tests, see the [manual clock example](../../admin-api/challenge-mode.md#phase-b-explicit-manual-clock).

Use [exclusive addon roots](../../../README.md#exclusive-addon-test-runs) to
prevent live addons from entering a test run. The guide covers repeated CLI
roots, the environment-only mode for existing scripts, bundled TestFramework,
and Docker/Action usage. See the [selection contract](../../specs/exclusive-addon-paths.md).

### `self-test`

Runs the Wowless compatibility test suite. Takes 60s+ and hangs waiting for async completion. Only use when debugging Wowless compatibility.

```bash
wow-sim --no-saved-vars self-test
wow-sim --no-saved-vars self-test --max-ticks 20000
```

### `screenshot`

Render the UI without starting the GUI. WebP is the default at standalone quality 15; `wow-cli screenshot` captures from a running simulator at live IPC quality 65. Explicit `--png` writes lossless PNG and forces `.png`. Without it, output is forced to `.webp`, even for `-o frame.png`. Encoding selection leaves source pixels, viewport (default 1600×1200), crop, color handling, and layout unchanged. Windows IPC remains unsupported. See the [output-format contract](../../specs/screenshot-output-format.md).

`wow-sim screenshot --quality <1-100>` sets the standalone WebP quality; `--png` ignores it. `--ui-scale <f>` (default 1.0) scales `UIParent` before the canvas size is set, so the managed containers are laid out for the scaled UI; the client's scale is `height / 768 × uiScale`, for example 1.6875 for 1440p at uiScale 0.9. Above 1.0 the render lookup draws the 2x atlas art; `WOW_SIM_HIRES_ATLASES=0` or `=1` overrides that choice. See [[ui-capture-fidelity]].

```bash
wow-sim --no-addons --no-saved-vars screenshot              # → screenshot.webp (1600×1200)
wow-sim screenshot -o frame.webp --filter AddonList         # Subtree only
wow-sim screenshot --width 1920 --height 1080
wow-sim screenshot --png -o frame.webp                     # → frame.png (lossless)
wow-cli screenshot --png -o live.webp                      # → live.png (running server)
wow-sim screenshot --width 3440 --height 1440 --ui-scale 1.6875 --quality 90
```

### `dump-tree`

Load UI and dump the frame hierarchy without starting the GUI. Shows stored dimensions (not computed layout).

```bash
wow-sim --no-addons --no-saved-vars dump-tree
wow-sim dump-tree --filter ScrollBar
wow-sim dump-tree --filter-key SpellBookFrame    # Full subtree of match
wow-sim dump-tree --visible-only
wow-sim dump-tree --delay 500                    # Wait 500ms after startup events
```

## wow-cli Subcommands

### `dump-tree`

Connect to a running `wow-sim` and dump computed (anchor-resolved) frame positions.

```bash
wow-cli dump-tree
wow-cli dump-tree --filter Button
wow-cli dump-tree --visible-only
```

### `lua` (REPL)

Interactive Lua REPL connected to a running simulator, or execute code directly.

```bash
wow-cli lua
wow-cli lua -e "print(GetScreenWidth())"
wow-cli lua -l                              # List running servers
```

### `audit-api`

Static analysis of `vendor/wow-ui-source/` to find gaps between what BlizzardUI references and what the simulator registers.

```bash
wow-cli audit-api --gaps --format plan      # Markdown checkboxes for PLAN.md
```

### `casc sync-blizzard-ui`

Populate `~/.cache/wow-ui-sim/blizzard-ui/<profile>` from the selected source: ordinary CASC, pinned PTR CDN ranges, or [separate pinned Wrath source](../../specs/client-profiles.md#separate-pinned-wrath-source-restoration). GUI startup syncs missing or stale runtime caches; repo symlinks/vendor checkouts are not runtime sources.

```bash
wow-cli casc sync-blizzard-ui
```

## Benchmark Binaries

### `bench_steady_state`

Settled-UI tick and draw CPU cost through the real GUI `update(ProcessTimers)` and shader `draw()` path, headless. Each frame forces a full OnUpdate interval, so the per-frame workload does not depend on window pacing, stale-tick dropping, or `perf` overhead. Draw covers the CPU side only (quad rebuild, texture loading), not GPU submission.

```bash
cargo build --release --bin bench_steady_state --features tools
target/release/bench_steady_state [--warmup 300] [--frames 1000] [--rounds 5]
```

Compare builds by `min p50` across rounds; single rounds vary up to ~2.6x with host load. Two back-to-back runs on 2026-09-25 agreed within 1% (tick) and 6% (draw). Use this, not separate GUI runs, for before/after perf claims.

### `bench_spellbook`

Spellbook open/close settle cost on the same GUI path. `--cycles N` repeats close/open N times after the first open (which also loads the spellbook addon) and reports p50/p90/mean of keypress+settle and draw for the repeated phases; two 20-cycle runs agreed within 2.4%. Use it for panel-interaction before/after claims.

```bash
cargo build --release --bin bench_spellbook --features tools
target/release/bench_spellbook --cycles 20
```

Host noise: this machine mixes Zen 5 (CPUs 0-3, 12-15, 5.16 GHz) and Zen 5c (3.29 GHz) cores, and runs are bimodal depending on which cores the scheduler picks. Interleave A/B runs, run several pairs, and discard pairs where both land in the slow regime; `taskset -c 1-3` helps only when other builds are not saturating those cores.

## Common Flags (wow-sim)

| Flag | Effect |
|------|--------|
| `--no-addons` | Skip third-party addons |
| `--addons-path DIR` | Exclusive third-party root; repeat for multiple roots |
| `--no-saved-vars` | Skip WTF SavedVariables (~18% of load time) |
| `--delay <ms>` | Delay after startup events (dump-tree/screenshot); followed by one OnUpdate tick carrying the slept time and the timers that came due, before `--exec-lua` runs |
| `--exec-lua "code"` | Execute Lua after startup; prefix with `@` to load file |
| `--class <NAME|N>` | Select the initial class by case-insensitive name or one-based index |
| `--spec <NAME|N>` | Select the initial specialization by case-insensitive name or one-based index; uses the startup class when `--class` is absent |
| `--race <NAME|N>` | Select the initial race by case-insensitive display name/file token or one-based simulator race-list index |
| `--debug-borders` | Red borders overlay |
| `--debug-anchors` | Green anchor-point dots |
| `--debug-elements` | Both overlays |

Class, race, and specialization selections are seeded before addons load, then reaffirmed after startup events and before `--exec-lua`. Spec-only input must be valid for the initial class and is resolved again against the current class at the late hook. Class/spec selections dispatch `PLAYER_SPECIALIZATION_CHANGED` for `player` once at that hook; race selections dispatch `UNIT_PORTRAIT_UPDATE` for `player`. A race-only selection does not dispatch a specialization event. They do not repeat `PLAYER_ENTERING_WORLD`, because a class or specialization change is not a world-entry lifecycle. Wrath, Era, and Anniversary reject `--spec` because these profiles do not model modern specialization selection. Unknown names or indices report valid choices and exit non-zero.

## Sources

- [benchmark.rs](../../../src/iced_app/benchmark.rs) — GUI-path benchmark harness

- [AGENTS.md](../../../AGENTS.md) — full CLI reference, Docker usage, environment variables
- [[casc-asset-cache]] — CASC-backed texture and Blizzard UI source caches

## See Also

- [[debug-tools]] — dump-tree and overlay details
- [[addon-compatibility]] — Docker CI usage for addon testing
- [[api-coverage]] — audit-api output and gap report
- [[casc-asset-cache]] — CASC cache locations and failure modes
