# Duration text binding configuration

`C_DurationUtil.CreateDurationTextBinding` uses the existing table-backed binding model in `src/c_api/duration_text_binding.rs`. This change adds the documented `Assign` and `Copy` operations without changing constructor, formatting, clock, or update scheduling behavior. See [the 12.0.7 API audit](../wiki/investigations/patch-12-0-7-api-audit.md) for existing compatibility limits.

## What it must do

- [ ] `Assign(other)` validates both binding objects before mutation, copies configuration into the receiver, and returns no values. Self-assignment preserves configuration and identity.
- [ ] `Copy()` returns a distinct binding with independent configuration. Duration, font-string, clock, formatter, and color-curve object handles remain shared references.
- [ ] Copy format-component containers and records while retaining formatter handles. Later source component mutations must not alter the assigned or copied binding.
- [ ] Copy absent values as absent, clearing prior receiver configuration. Preserve enabled state, interval, modifier, expired/zero text, and color-curve property.
- [ ] Support Blizzard `CustomAuraButton:SetDurationText(..., {binding=...})` without replacing the source binding's display target.

## How it works

- [Numeric rule formatter](numeric-rule-formatter.md)
- [Aura option normalization](aura-container-options.md)

## Implementation inventory

- `src/c_api/duration_text_binding.rs` — binding factory and configuration copy model.
- `src/c_api/mod.rs` — module declaration.
- `src/lua_api/env_init/mod.rs` — initialization after existing bootstrap defaults and before secure-environment copying.

## Tests asserting this spec

- `tests/duration_text_binding_copy.rs` — five behavioral cases, including the actual aura initializer.
- `tests/numeric_rule_formatter.rs` — existing formatter-to-font-string binding behavior.

## Known gaps (current cycle)

- [ ] GREEN verification of the five assignment/copy cases is pending; their missing-method RED is recorded in `/tmp/pi-final-aura-primitives-current-red.*`.

## Out of scope

This retains the existing table-backed representation and best-effort formatting behavior. It does not establish native userdata representation, exact clocks, automatic scheduling, expiration policy, color-curve evaluation, or secret-value enforcement. No new behavior is claimed for those domains.
