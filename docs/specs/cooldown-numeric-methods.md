# Cooldown numeric method coverage

Ordinary simulator state behavior for cooldown methods in `src/lua_api/frame/methods/widgets/cooldown.rs`. The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) defines API declarations; these tests do not establish native timing or security semantics.

## What it must do

- [x] Direct `Clear` resets start, duration, and display duration to zero, and rate to one.
- [x] Shared `Clear` dispatch selects the existing Cooldown or MessageFrame/ScrollingMessageFrame handler by widget type; message history clearing and `ClearText` remain intact.
- [x] Direct `SetCooldown` stores start/duration/rate; omitted rate becomes one.
- [x] Direct `SetCooldownDuration` preserves start, replaces duration/rate, and defaults omitted rate to one.
- [x] Direct `SetCooldownUNIX` preserves the supplied numeric start without epoch conversion; omitted rate becomes one.

- [ ] `GetCooldownTimes` returns start and duration in milliseconds, while setters and internal storage use seconds.
- [ ] `GetCooldownDuration` returns duration × 1000 × modRate; `GetCooldownDisplayDuration` returns milliseconds independently of rate.
- [ ] RuneFrame's `(start + duration) / 1000` end-time comparison matches the seconds supplied to `SetCooldown`.

Pinned Forever `FrameAPICooldownDocumentation.lua:22,37` specifies the duration-query units and rate distinction. `GetCooldownTimes` has only numeric return declarations there; unchanged `Blizzard_UnitFrame/Mainline/RuneFrame.lua:250–252` explicitly documents milliseconds and divides the sum by 1000. Display duration remains nonnegative duration × 1000. The large numeric UNIX fixture proves literal storage only, **not native UNIX-to-frame-time conversion**.

Pinned base and target both declare these four methods. PTR adds protection annotations to all four and changes the duration argument types of `SetCooldown`/`SetCooldownDuration` from `DurationSeconds` to `Seconds`; `SetCooldownUNIX` retains numeric arguments. Both profiles declare rate default one.

## How it works

- [Lua API architecture](../lua-api.md)
- [Duration core and proxy integration](duration-core.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/cooldown.rs` — numeric setters, clearing, and query methods.
- `src/lua_api/frame/methods/widgets/mod.rs` — single shared `Clear` registration dispatching to existing widget handlers; unsupported widget types raise an explicit error.
- `tests/message_frame.rs` — MessageFrame/ScrollingMessageFrame history clearing, reuse, and isolation from other widgets.
- `tests/cooldown_widget.rs` — direct-method coverage and existing expiration/threshold/proxy regressions in the grouped integration target.

## Tests asserting this spec

All rows below run through `cooldown_widget::` on PTR and retail. Existing coverage is reused, not duplicated.

| Method | Test | Observable coverage |
|---|---|---|
| `Clear` | `cooldown_clear_resets_timing_and_rate` | Configured timing → zero timing, rate one |
| `SetCooldown` | `cooldown_set_cooldown_stores_timing_and_defaults_rate` | Fractional start/duration, explicit/default rate, display duration |
| `SetCooldownDuration` | `cooldown_set_duration_preserves_start_and_defaults_rate` | Preserved start, replaced duration/rate, display duration |
| `SetCooldownUNIX` | `cooldown_set_unix_stores_literal_start_without_epoch_conversion` | Literal large numeric start, explicit/default rate, display duration |
| `GetCooldownTimes` / `GetCooldownDuration` | `cooldown_set_cooldown_stores_timing_and_defaults_rate` | `SetCooldown(12.5, 6.25, 2.5)` → `12500,6250`, rate-adjusted `15625`, display `6250`; RuneFrame end-time arithmetic |
| `SetCooldownFromExpirationTime` | `cooldown_widget_methods_persist_runtime_state` | Expiration 20, duration 8 → start 12, display 8000 |
| `GetMinimumCountdownDuration` | `cooldown_widget_methods_persist_runtime_state` | Default zero and round-trip 2500 |
| `SetMinimumCountdownDuration` | `cooldown_widget_methods_persist_runtime_state` | Stores 2500 |
| `SetCountdownAbbrevThreshold` | `cooldown_widget_methods_persist_runtime_state` | Stores 5 |

`SetCooldownFromDurationObject` has separate existing real-proxy, error-propagation, and zero-option coverage in the same module; see [duration core](duration-core.md).

At test revision `943b21255`, `cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> cooldown_widget:: -- --nocapture` reports **10 passed, 1 failed** on each of `ptr` and `retail`. Only `cooldown_clear_resets_timing_and_rate` fails. All three other new tests and seven existing tests pass. No production changes were made.

At `060dbc0ff`, targeted grouped integration tests pass on both PTR and retail: **11 cooldown tests and 30 message-frame tests per profile**. PTR cooldown proof from `e954c7ba6` remains valid: the subsequent change only corrected the new message-frame test's query name. Both message-frame types retain history clearing, independent-widget state, reuse after clearing, and `ClearText` behavior.

Commands use `cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> <filter> -- --nocapture` with filters `cooldown_widget::` and `message_frame::`.

## Known gaps (current cycle)

The shared-metatable `Clear` collision is repaired by one widget-type dispatcher. Cooldown and message-frame registrations no longer overwrite each other. No per-widget metatable redesign or method-allowlist change is involved.
- [ ] Compiled GREEN for the output-unit correction remains pending. Existing-binary RED returns `12.5,6.25` from `GetCooldownTimes` and `6.25` from `GetCooldownDuration`; display `6250` already passes. Evidence: `/tmp/ellesmere-forever/cooldown-output-units-ledger.json`. Earlier passing counts above predate this correction.
- [ ] Epoch conversion, numeric coercion, and `Seconds`/`DurationSeconds` equivalence remain unverified.
- [ ] Protected-call, taint, secret-value, and restricted-access behavior remain unverified.

## Out of scope

Timing changes beyond the demonstrated output-unit conversions, security enforcement, rendering effects, unrelated shared-method collisions, and audit-status updates. Unsupported-widget error behavior is simulator policy, not a native compatibility claim; parent audit owns later credit.
