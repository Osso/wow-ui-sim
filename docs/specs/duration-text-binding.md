# Duration text binding configuration

`C_DurationUtil.CreateDurationTextBinding` creates userdata handles backed by configuration in `src/c_api/duration_text_binding.rs`. The pinned `DurationTextBindingObjectAPIDocumentation.lua` declares `ObjectType = "Userdata"`. Documented `Assign` and `Copy` operations retain existing constructor, formatting, clock, and update scheduling behavior. See [the 12.0.7 API audit](../wiki/investigations/patch-12-0-7-api-audit.md) for existing compatibility limits.

## What it must do

- [ ] Expose the existing binding factory on Forever and Retail-family 12.0.7+. Keep color-curve methods limited to Forever and Retail-family 12.1+; earlier profiles do not gain a modeled binding factory. Rust profile selection controls availability without changing `GetBuildInfo`.

- [x] `Assign(other)` validates both binding objects before mutation, copies configuration into the receiver, and returns no values. Self-assignment preserves configuration and identity.
- [x] `Copy()` returns a distinct binding with independent configuration. Duration, font-string, clock, formatter, and color-curve object handles remain shared references.
- [x] Copy format-component containers and records while retaining formatter handles. Later source component mutations must not alter the assigned or copied binding.
- [x] Copy absent values as absent, clearing prior receiver configuration. Preserve enabled state, interval, modifier, expired/zero text, and color-curve property.
- [x] Support Blizzard `CustomAuraButton:SetDurationText(..., {binding=...})` without replacing the source binding's display target. Its `securecopy(options)` must copy ordinary option tables while retaining the binding handle; copied or forged tables are not binding objects.

### Best-effort representation policy

- [x] Binding handles are userdata: `rawget` and `rawset` reject them. Retained handles remain usable and preserve identity through explicit collection.
- [x] A copied binding retains its duration, clock, font-string, formatter, and color-curve references after caller references and the source binding are released. Mutating a shared resource remains observable through the retained copy.

These are simulator policies, not claims about native object layout or garbage collection. Configuration is independently owned by each binding; external resource handles remain shared. Existing assignment/copy tests cover configuration independence and receiver validation.

## How it works

- [Numeric rule formatter](numeric-rule-formatter.md)
- [Aura option normalization](aura-container-options.md)

## Implementation inventory

- `src/c_api/duration_text_binding.rs` — binding factory and configuration copy model.
- `src/c_api/mod.rs` — module declaration.
- `src/lua_api/env_init/mod.rs` — initialization after existing bootstrap defaults and before secure-environment copying.

## Tests asserting this spec

- `tests/duration_text_binding_copy.rs` — configuration/copy cases plus `duration_binding_userdata_copy_retains_resources_through_collection` in the grouped integration target.
- `src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_1_duration_binding_reference_lifetime_and_identity` — retained identity and duration access; userdata expectation replaces the stale table expectation.
- `tests/numeric_rule_formatter.rs` — existing formatter-to-font-string binding behavior, including Forever.
- `src/c_api/duration_text_binding.rs::tests::duration_binding_availability_preserves_client_versions` — profile availability, modern-method boundary, and formatted FontString output.
- Existing copy/configuration and native CustomAuraButton tests also run on Forever; unrelated native aura dependencies remain separate failures, not reasons to weaken these assertions.

## Known gaps (current cycle)

Forever availability RED: `/tmp/ellesmere-forever/batch-numeric_rule_formatter.stderr` records a nil binding before formatting. Availability sharing and broadened regressions await parent-run compilation/GREEN. This slice changes no formatting, clock, color, scheduling, or copy semantics.

The representation-retention test passed on `client-retail` at `9a8189612` (one focused integration test). This proves the chosen handle/reference policy only; the existing copy/configuration tests were not rerun for this slice.

Focused proof at `92675f08d`: all six assignment/copy cases passed in `/tmp/pi-aura-followup-green.*`, including the actual `CustomAuraButton` initializer and secure-option copy. The earlier failed table-backed identity boundary is retained in `/tmp/pi-aura-three-models-green.*`; the source-backed userdata handle fixes it without accepting forgeable table markers.

Actual addon/SavedVariables startup returned `[]`, exit 0 after the separate dispel-filter, controlled-player token, and curve-userdata fixes (`/tmp/pi-accepted-final-startup.*`). This binding slice does not make broader native-fidelity claims.

## Out of scope

This retains existing best-effort formatting behavior. Native finalization, invalidation, metatable shape, ownership, and GC equivalence are not established; no finalizer or invalidation policy is added. Exact clocks, automatic scheduling, expiration policy, color-curve evaluation, and secret-value enforcement remain outside this representation proof.
