# WoW UI Simulator

A World of Warcraft UI simulator for addon testing and UI rendering. Supports headless test workflows plus frame-tree and screenshot output — no WoW client required.

## Community

Join the wowless Discord: <https://discord.gg/rTwWcfJXuz> — we have a `#wow-ui-sim` channel there.

## UI Gallery

| Main Screen | Spellbook |
|---|---|
| ![Main screen](docs/gallery/main-screen.png) | ![Spellbook](docs/gallery/spellbook.png) |

| Character Panel | Reputations Panel |
|---|---|
| ![Character panel](docs/gallery/character-panel.png) | ![Reputations panel](docs/gallery/reputations-panel.png) |

| Blacksmithing | Guild Panel |
|---|---|
| ![Blacksmithing](docs/gallery/blacksmithing.png) | ![Guild panel](docs/gallery/guild-panel.png) |

| Store | Achievements Panel |
|---|---|
| ![Store](docs/gallery/store.png) | ![Achievements panel](docs/gallery/achievements-panel.png) |

| World Map | Housing Panel |
|---|---|
| ![World map](docs/gallery/worldmap.png) | ![Housing panel](docs/gallery/housing-panel.png) |

| Talents Panel | Damage Meter |
|---|---|
| ![Talents panel](docs/gallery/talents-panel.png) | ![Damage meter](docs/gallery/damage-meter.webp) |

| Game Menu |
|---|
| ![Game menu](docs/gallery/game-menu.webp) |

## GitHub Action

Run your addon's test suite in CI:

```yaml
- uses: osso/wow-ui-sim@12.0.5
  with:
    addon: MyAddon
```

The action ref matches the WoW interface version it ships against
(e.g. `@12.0.5`). Use `@master` to track unreleased changes.

Tests live in `Interface/AddOns/MyAddon/tests/*.lua`:

```lua
test("frame name matches", function()
    local f = CreateFrame("Frame", "MyFrame")
    assertEquals("MyFrame", f:GetName())
end)
```

A minimal end-to-end example is at
[Osso/test-wow-addon](https://github.com/Osso/test-wow-addon) — a TOC,
one Lua file, a `tests/` folder, and a workflow that calls this action.

## Native development

Normal builds and runs use the shared desktop/local host selection:

```sh
python3 scripts/build-host.py --run
python3 scripts/build-host.py --build-host local --run
python3 scripts/build-host.py --check
python3 scripts/build-host.py --test --test integration FILTER -- --nocapture
scripts/start-profile.sh live
```

Use local while gaming on desktop. Raw Cargo stays local. See [native workflow, profile flags, and runtime proof limits](docs/remote-builds.md).

## Release packages

GitHub releases provide Retail, Mists, and WoW Forever packages for Linux x86-64, Windows x86-64, and macOS Apple Silicon. Windows packages include a ZIP and an installer; macOS packages use a DMG. Forever package names include `forever`, and its installer and macOS app use separate identities.

Forever packages include selected spell data. For a local Forever build, run `wow-cli casc sync-forever-spells` before starting the simulator. It uses the installed pinned Forever client when available, or downloads the pinned DB2 files from Blizzard's CDN. See the [spell data contract](docs/specs/forever-spell-data.md).

Retail and PTR dungeon score colours require a generated cache. Run `wow-cli casc sync-dungeon-score-rarity` for the active profile before running local tests or using those APIs. Retail release packages and Docker images include the cache. See the [rarity contract](docs/specs/dungeon-score-rarity.md).

## Docker

Published addon-test images and CI/release workflows remain separate from native development. The Dockerfile builds Retail by default; use `docker build --build-arg WOW_UI_SIM_PROFILE=wowforever .` for a Forever image with its spell data.
Docker builds use `.cargo/config.ci.toml` for the default x86-64 CPU target;
local builds keep `target-cpu=native` in `.cargo/config.toml`.
Before publication, `.github/scripts/verify_docker_image.py IMAGE` checks
`--version` and addon tests under QEMU's Nehalem CPU model as well as natively.
It requires `qemu-user-static` (Ubuntu package); use `--qemu PATH` to select
another static `qemu-x86_64` executable. Emulation runs offline with a read-only
fixture and does not require privileged containers or host binfmt changes.

```bash
# Run addon tests
docker run --rm \
  -v ./MyAddon:/app/Interface/AddOns/MyAddon \
  ghcr.io/osso/wow-ui-sim run-tests MyAddon

# Run with all Blizzard addons loaded
docker run --rm \
  -v ./MyAddon:/app/Interface/AddOns/MyAddon \
  ghcr.io/osso/wow-ui-sim --no-saved-vars run-tests MyAddon

# Render a screenshot with Mesa software Vulkan (no GPU required)
mkdir -p screenshots
docker run --rm -v "$(pwd)/screenshots:/out" \
  ghcr.io/osso/wow-ui-sim screenshot -o /out/screenshot.webp
```

For deterministic timing in addon tests, see the [manual clock example](docs/admin-api/challenge-mode.md#phase-b-explicit-manual-clock).

The image does not include WoW game assets; screenshots may omit textures unless
assets are available to the container.

## Exclusive addon test runs

Default discovery combines `WOW_SIM_ADDONS_PATH`, local and bundled addon
folders, and the selected WoW installation. Setting `WOW_SIM_ADDONS_PATH`
alone does **not** isolate a test run.

Use repeatable CLI roots to replace those candidates:

```sh
wow-sim --addons-path /tmp/test/AddOns --addons-path /tmp/dependencies/AddOns run-tests MyAddon
```

For scripts that already set `WOW_SIM_ADDONS_PATH`, use the environment-only
mode. No script change is needed:

```sh
WOW_SIM_ADDONS_PATH_EXCLUSIVE=1 WOW_SIM_ADDONS_PATH=/tmp/test/AddOns wow-sim run-tests MyAddon
# A script that sets WOW_SIM_ADDONS_PATH internally:
WOW_SIM_ADDONS_PATH_EXCLUSIVE=1 WOW_SIM_BIN=/path/to/wow-sim ./run-tests.sh
```

`true` also enables exclusive mode; `0`/`false` leave default discovery active.
CLI roots take precedence over environment selection. Each CLI value is one
addon root, not an individual addon folder. Use absolute paths: the simulator
changes its working directory to the executable directory before loading.
Missing roots are omitted; an empty exclusive selection does not restore the
default candidates.

Blizzard UI still loads from its profile cache. `run-tests` finds the bundled
TestFramework explicitly, without loading other bundled addons. Exclusive mode
also restricts test-file lookup and `C_AddOns.LoadAddOn`; it does not search
ambient `Interface/TestAddOns`. It is not a filesystem or Lua sandbox.

### GitHub Action and Docker

The [composite action](action.yml) retains its existing default discovery.
For an exclusive Docker run, mount the addon and its dependencies under a
dedicated root, then pass that root to the image:

```sh
docker run --rm -v ./test-addons:/test-addons \
  ghcr.io/osso/wow-ui-sim --addons-path /test-addons run-tests MyAddon
```

The root must contain `MyAddon/MyAddon.toc` and all required addon dependencies.
TestFramework does not need to be copied into that root. See the
[selection contract](docs/specs/exclusive-addon-paths.md).

## Utility binary builds

Integration tests build `wow-sim` only unless utility features are requested.
The other five binaries require `--features tools`: `wow-cli`,
`panel-visual-metrics`, `bench_spellbook`, `bench_steady_state`, and
`bench_talents`. The two existing GUI-gated benches also require `gui`.
Use `cargo build --bin wow-cli --features tools` or
`cargo build --bins --features tools` for utility builds.
The release aliases and `scripts/build-host.py --bin wow-cli --features tools` enable this
feature automatically. Docker and full-suite test selection do not need it.
Build before using bare `wow-cli` commands; do not pass `--features tools` to the executable itself. For direct runs, use `cargo run --bin wow-cli --features tools -- <args>`.
See [the build contract](docs/specs/integration-bin-selection.md).

Historical replay: add `--features tools` before `--` to the recorded Cargo utility commands in [patch audit milestone tests](docs/specs/patch-api-audit-manifest.md), [Mists world-map startup](docs/wiki/investigations/mists-world-map-startup.md), and [Windows port builds](docs/wiki/investigations/windows-port-build.md); original evidence is unchanged.

## Writing Tests

Test files go in `tests/` under your addon directory. Available assertions:

| Function | Description |
|---|---|
| `assertEquals(expected, actual)` | Strict equality |
| `assertNotEquals(expected, actual)` | Not equal |
| `assertTrue(value)` / `assertFalse(value)` | Truthy/falsy |
| `assertNil(value)` / `assertNotNil(value)` | Nil checks |
| `assertError(fn)` | Function throws an error |
| `assertType(expected, value)` | `type(value) == expected` |
| `assertAlmostEquals(expected, actual, tolerance?)` | Float comparison (default 0.001) |
| `assertContains(haystack, needle)` | String substring or table value |
| `assertStartsWith(str, prefix)` | String prefix |
| `assertEndsWith(str, suffix)` | String suffix |
| `assertMatches(str, pattern)` | Lua pattern match |
| `assertCount(expected, table)` | Table element count |
| `assertTableEquals(expected, actual)` | Deep table equality |
| `assertTableContains(table, subset)` | Table contains subset (deep) |

Async tests for timers and callbacks:

```lua
async_test("timer fires callback", function(done)
    C_Timer.After(0, function()
        done(function()
            assertTrue(true)
        end)
    end)
end)
```

## Admin API

The simulator exposes an `A_Admin` Lua namespace for controlling state in tests (player identity, combat, party, buffs, zone, economy, etc.). See [docs/admin-api/](docs/admin-api/README.md).

## License

GPL-3.0-only
