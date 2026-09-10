# Cooldown numeric method coverage

Ordinary simulator state behavior for cooldown methods in `src/lua_api/frame/methods/widgets/cooldown.rs`. The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) defines API declarations; these tests do not establish native timing or security semantics.

## What it must do

- [ ] Direct `Clear` resets start, duration, and display duration to zero, and rate to one.
- [ ] Direct `SetCooldown` stores start/duration/rate; omitted rate becomes one.
- [ ] Direct `SetCooldownDuration` preserves start, replaces duration/rate, and defaults omitted rate to one.
- [ ] Direct `SetCooldownUNIX` preserves the supplied numeric start without epoch conversion; omitted rate becomes one.

The existing simulator computes display duration as nonnegative duration × 1000, independently of rate. Tests deliberately record that model rather than infer native units or elapsed-time behavior. The large numeric UNIX fixture proves literal storage only, **not native UNIX-to-frame-time conversion**.

Pinned base and target both declare these four methods. PTR adds protection annotations to all four and changes the duration argument types of `SetCooldown`/`SetCooldownDuration` from `DurationSeconds` to `Seconds`; `SetCooldownUNIX` retains numeric arguments. Both profiles declare rate default one.

## How it works

- [Lua API architecture](../lua-api.md)
- [Duration core and proxy integration](duration-core.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/cooldown.rs` — numeric setters, clearing, and query methods; unchanged by this coverage slice.
- `tests/cooldown_widget.rs` — direct-method coverage and existing expiration/threshold/proxy regressions in the grouped integration target.

## Tests asserting this spec

All rows below run through `cooldown_widget::` on PTR and retail. Existing coverage is reused, not duplicated.

| Method | Test | Observable coverage |
|---|---|---|
| `Clear` | `cooldown_clear_resets_timing_and_rate` | Configured timing → zero timing, rate one |
| `SetCooldown` | `cooldown_set_cooldown_stores_timing_and_defaults_rate` | Fractional start/duration, explicit/default rate, display duration |
| `SetCooldownDuration` | `cooldown_set_duration_preserves_start_and_defaults_rate` | Preserved start, replaced duration/rate, display duration |
| `SetCooldownUNIX` | `cooldown_set_unix_stores_literal_start_without_epoch_conversion` | Literal large numeric start, explicit/default rate, display duration |
| `SetCooldownFromExpirationTime` | `cooldown_widget_methods_persist_runtime_state` | Expiration 20, duration 8 → start 12, display 8000 |
| `GetMinimumCountdownDuration` | `cooldown_widget_methods_persist_runtime_state` | Default zero and round-trip 2500 |
| `SetMinimumCountdownDuration` | `cooldown_widget_methods_persist_runtime_state` | Stores 2500 |
| `SetCountdownAbbrevThreshold` | `cooldown_widget_methods_persist_runtime_state` | Stores 5 |

`SetCooldownFromDurationObject` has separate existing real-proxy, error-propagation, and zero-option coverage in the same module; see [duration core](duration-core.md).

## Known gaps (current cycle)

- [ ] Native units, epoch conversion, numeric coercion, and `Seconds`/`DurationSeconds` equivalence remain unverified.
- [ ] Protected-call, taint, secret-value, and restricted-access behavior remain unverified.

## Out of scope

Production changes, native timing corrections, security enforcement, rendering effects, and audit-status updates. This slice adds behavioral coverage only; parent audit owns later credit.
