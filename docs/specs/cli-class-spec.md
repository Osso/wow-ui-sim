# Startup and CLI class, race, and specialization selection

`wow-sim` accepts class, race, and specialization selections for post-startup scripts and UI capture. See the [CLI reference](../wiki/reference/cli-commands.md).

## Initial class, race, and specialization

GUI startup loads `player_class` and `player_race` from simulator preferences;
headless commands (`dump-tree`, `lua-errors`, `run-tests`, `screenshot`, ...)
ignore those preferences and start from the defaults (Paladin and Human), so
their results do not depend on GUI choices. Explicit `--class` and `--race`
values override both. All supplied selections are validated and applied to the
player model before any addons load, without emitting selection events during
this seeding step. A spec-only `--spec` is seeded only when the initial class
owns it. WTF account, realm,
and character selection does not determine these values.

GUI and headless startup share this model. GUI selectors and talent texture
preload read its values without reapplying saved class/race settings. ReloadUI
retains the current model, including later GUI and scripted changes.

The post-startup hook reaffirms explicitly requested values before `--exec-lua`.
It dispatches `PLAYER_SPECIALIZATION_CHANGED` for class/spec selections and
`UNIT_PORTRAIT_UPDATE` for race selections, without repeating world entry.
Spec-only selection is resolved against the class present at that hook. For
example, `--spec Holy` seeds Paladin Holy and later selects Priest Holy if
startup handlers changed the class; `--spec Fire` with an initial Paladin seeds
nothing and selects Mage Fire if startup handlers made the player a Mage. A spec
that is invalid for the class at the hook exits with status 2.

## What it must do

- [x] Accept case-insensitive class names and one-based indices from `CLASS_LABELS`.
- [x] Accept case-insensitive specialization names and one-based indices from the selected class's specialization catalog.
- [x] Accept race display names or file tokens case-insensitively, and one-based simulator race-list indices.
- [x] Apply race-only selections without changing class/spec or dispatching a specialization event.
- [x] Resolve a specialization without `--class` against the class at the post-startup execution point, seeding it early only when the initial class owns it.
- [x] Apply saved GUI class/race preferences only to GUI launches.
- [x] Seed all supplied selections before addon initialization, validating the entire selection before changing the model.
- [x] Reject invalid values with valid choices and a nonzero exit status.
- [ ] Reject specialization selection on Wrath, Era, and Anniversary.
- [x] Apply class/spec selections through `A_Admin`, then dispatch `PLAYER_SPECIALIZATION_CHANGED` with `player` before `--exec-lua`.
- [x] Do not repeat `PLAYER_ENTERING_WORLD` for a class or specialization change.
- [x] Monk Mistweaver and Windwalker indices resolve to their matching specialization identities in the Blizzard specialization content frame.

## How it works

- [CLI reference](../wiki/reference/cli-commands.md)

## Timing and precedence

Addon file initializers, OnLoad handlers, and startup events see the requested
initial context. Class-specific setup can therefore choose the requested class
on its first initialization.

Startup handlers can change the model. The retained late hook restores explicit
class/race selections and applies the requested specialization. Without
`--class`, it uses the class left by those handlers. Notification remains after
listeners have loaded; `SetSpec` already emits a specialization event when its
index changes, so the hook does not add a duplicate event.

## Implementation inventory

- `src/bin/wow_sim/startup.rs` resolves and seeds initial class/race/spec before loading addons.
- `src/iced_app/app.rs` initializes class/race selectors from the resolved model.
- `src/bin/wow_sim/class_spec.rs` resolves and applies the selected context.
- `src/bin/wow_sim/main.rs` accepts and schedules the flags.
- `src/lua_api/env.rs` and `src/lua_api/env_rilua.rs` execute the scheduled hook.
- `crates/sim-data/data/specializations.rs` returns class specializations in `order_index` order.

## Tests asserting this spec

- `tests/cli_class_spec.rs` checks class and specialization pairs through `--exec-lua` and exercises the Blizzard Monk specialization content frame.
- `src/bin/wow_sim/class_spec.rs` tests class/spec/race resolution, deferred race changes, and notification behavior.
- `src/bin/wow_sim/main.rs` tests race-flag parsing alone and with class/spec.
- `src/bin/wow_sim/startup_player_tests.rs` checks all three selections during addon initialization, atomic validation, late overrides/notifications, spec-only resolution (including a spec outside the initial class), headless isolation from GUI preferences, and reload preservation.
- `src/iced_app/app_tests.rs` checks that GUI preferences preserve the resolved class/race and that selectors display them.

## Known gaps (current cycle)

- [ ] Unsupported-profile CLI execution and seven-profile all-target checks are pending.

## Out of scope

Character, realm, and account selection are separate from class, race, and specialization state.
