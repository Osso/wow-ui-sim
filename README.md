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

## Docker

Published addon-test images and CI/release workflows remain separate from native development.

```bash
# Run addon tests
docker run --rm \
  -v ./MyAddon:/app/Interface/AddOns/MyAddon \
  ghcr.io/osso/wow-ui-sim run-tests MyAddon

# Run with all Blizzard addons loaded
docker run --rm \
  -v ./MyAddon:/app/Interface/AddOns/MyAddon \
  ghcr.io/osso/wow-ui-sim --no-saved-vars run-tests MyAddon
```

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
