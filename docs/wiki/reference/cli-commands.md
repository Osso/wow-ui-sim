# CLI Commands

Two binaries: `wow-sim` (full UI load, GUI capable) and `wow-cli` (connects to running server or loads standalone for some commands).

**Build before running** — compilation takes 30s+; never use `cargo run` with a timeout. Build with `cargo build --bin wow-sim`, then run with `timeout 90`.

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
```

### `self-test`

Runs the Wowless compatibility test suite. Takes 60s+ and hangs waiting for async completion. Only use when debugging Wowless compatibility.

```bash
wow-sim --no-saved-vars self-test
wow-sim --no-saved-vars self-test --max-ticks 20000
```

### `screenshot`

Render the UI to a WebP image without starting the GUI. Text is not rendered (layout/texture debugging only). Always saves as lossy WebP at quality 65.

```bash
wow-sim --no-addons --no-saved-vars screenshot              # → screenshot.webp (1024×768)
wow-sim screenshot -o frame.webp --filter AddonList         # Subtree only
wow-sim screenshot --width 1920 --height 1080
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

Extract Blizzard UI source files from the local WoW CASC install into `~/.cache/wow-ui-sim/blizzard-ui`. GUI startup runs this automatically when the repo symlink/vendor checkout is missing.

```bash
wow-cli casc sync-blizzard-ui
```

## Common Flags (wow-sim)

| Flag | Effect |
|------|--------|
| `--no-addons` | Skip third-party addons |
| `--no-saved-vars` | Skip WTF SavedVariables (~18% of load time) |
| `--delay <ms>` | Delay after startup events (dump-tree/screenshot); followed by one OnUpdate tick carrying the slept time and the timers that came due, before `--exec-lua` runs |
| `--exec-lua "code"` | Execute Lua after startup; prefix with `@` to load file |
| `--debug-borders` | Red borders overlay |
| `--debug-anchors` | Green anchor-point dots |
| `--debug-elements` | Both overlays |

## Sources

- [AGENTS.md](../../../AGENTS.md) — full CLI reference, Docker usage, environment variables
- [[casc-asset-cache]] — CASC-backed texture and Blizzard UI source caches

## See Also

- [[debug-tools]] — dump-tree and overlay details
- [[addon-compatibility]] — Docker CI usage for addon testing
- [[api-coverage]] — audit-api output and gap report
- [[casc-asset-cache]] — CASC cache locations and failure modes
