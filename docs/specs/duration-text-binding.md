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

## Bounded secret-duration handoff — informed guesses

These are explicitly **simulator guesses**, not Forever-client probe results. The user cannot run Forever probes. Native aura code passes wrapped timing into duration objects and hands those objects to text bindings; the output/access policy below is chosen to avoid exposing plain timing to addon callbacks.

- A binding reports secret state when its input is a VM-owned secret wrapper or a duration with secret timing. Copy/Assign retain the same input handle and therefore its secrecy; defaults replace it with ordinary duration state.
- Formatting a secret input requires an untainted caller. Remaining-time reads and callback errors propagate; they are not replaced with `"0"` or prior/default text. Plain-input formatting keeps its existing behavior.
- Userdata `FormatNumber` callbacks receive a wrapped number. Function/table formatters retain their existing duration-object/input argument, not a decoded numeric value. Callback closure taint is not cleared.
- Native NumericRuleFormatter accepts only authenticated wrapped numbers through rilua's checked unwrap, then applies its existing finite validation and rounding model. Plain numbers retain their existing path. Returning a plain formatted result is permitted only after the secret argument's untainted authorization check; this is not a native-return-secrecy claim.
- An untainted binding caller may receive plain formatted text. `UpdateFontString` wraps that text before invoking the widget setter, preserving the [secret-origin widget readout boundary](aura-secret-display.md). A tainted caller cannot format/update the secret binding or read its secret-origin text through the covered widget getter.
- The secret branch captures bootstrap numeric/string conversion functions and private host wrap/unwrap callbacks. Later addon replacements must not receive plaintext timing through these conversions.

The existing formatter choice/clock/update model is not redesigned. General secret arithmetic, arbitrary userdata coercion, and native output-secrecy equivalence are not established. Native default SecondsFormatter behavior remains the existing bounded model, not newly claimed native formatting parity.

## Automatic engine updates — inferred timing policy

- Enabled bindings with a writable FontString update on the engine tick after frame OnUpdate handlers, so native aura assignment/configuration takes effect without explicit `UpdateFontString` calls.
- New bindings, configuration writes, Assign/Copy, defaults, and re-enabling request an update on the next tick. Otherwise `updateInterval` measures accumulated engine elapsed time since the previous automatic update. Disabled bindings do not update; missed intervals are not replayed in a burst.
- Duration-object clocks still determine the displayed remaining duration. The binding's existing default manual clock does not drive scheduler cadence. These timing choices are simulator policy, not native-probe results.
- Scheduling state is private and independent from copied configuration. The existing weak-key binding registry must not retain otherwise unreachable handles or their display targets.
- The engine invokes a privately retained bootstrap callback without clearing taint. Secret conversion captures, formatter callback taint, and secret-origin FontString read restrictions remain in force. A propagated binding-update error is sent to the existing Lua error handler; other bindings and later engine-tick/GC work still run. The failed update does not replace its label text. Existing plain-input function/table formatter error suppression is a pre-existing limitation, unchanged by this slice.

Development RED: `/tmp/forever-addon-audit/binding-tick-red-8ubmhouy/` records six failing automatic-update cases before scheduler implementation, including the real native CustomAuraButton initializer/assignment. At `748e36687`, those six targeted tests pass. A seventh error-isolation RED at `/tmp/forever-addon-audit/binding-tick-error-red-corrected-lk6be6q3/` shows a propagated formatter error aborting the tick. Final seven-test GREEN and independent acceptance remain pending.

## How it works

- [Numeric rule formatter](numeric-rule-formatter.md)
- [Aura option normalization](aura-container-options.md)

## Implementation inventory

- `src/c_api/duration_text_binding.rs` — binding factory, configuration copy model, weak scheduling metadata, and private engine callback.
- `src/lua_api/on_update.rs` — automatic binding updates after frame OnUpdate dispatch.
- `src/c_api/mod.rs` — module declaration.
- `src/lua_api/env_init/mod.rs` — initialization after existing bootstrap defaults and before secure-environment copying.

## Tests asserting this spec

- `tests/duration_text_binding_tick.rs` — automatic native CustomAuraButton display, manual duration-clock progression, engine interval boundaries, disable/re-enable/configuration invalidation, independent copy/reset schedules, weak lifetime, and tainted callback/conversion boundaries.

- `tests/duration_text_binding_copy.rs` — configuration/copy cases plus retained resources; the `duration_binding_secret_` cases cover duration → formatter → FontString, tainted readout, wrapped-number input, malicious userdata callback capture, and unsuppressed callback failure. New secret cases await integrating compilation/GREEN.
- `tests/numeric_rule_formatter.rs::numeric_rule_formatter_secret_numbers_require_untainted_caller` — authenticated input, arbitrary userdata/wrapped-string rejection, and tainted-caller rejection; integrating GREEN pending.
- `src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_1_duration_binding_reference_lifetime_and_identity` — retained identity and duration access; userdata expectation replaces the stale table expectation.
- `tests/numeric_rule_formatter.rs` — existing formatter-to-font-string binding behavior, including Forever.
- `src/c_api/duration_text_binding.rs::tests::duration_binding_availability_preserves_client_versions` — profile availability, modern-method boundary, and formatted FontString output.
- Existing copy/configuration and native CustomAuraButton tests also run on Forever; unrelated native aura dependencies remain separate failures, not reasons to weaken these assertions.

## Known gaps (current cycle)

The availability RED recorded a nil binding before formatting. At `9476efcf5`, `secure-chain-tests-ledger.json` records the Forever native binding/copy group 7/7 and `c_api::duration_text_binding::tests::` 1/1 after profile availability sharing. This proves factory publication and existing binding behavior, not formatter threshold parity, broader GUI aura display, or new formatting/clock/color/scheduling semantics.

The representation-retention test passed on `client-retail` at `9a8189612` (one focused integration test). This proves the chosen handle/reference policy only; the existing copy/configuration tests were not rerun for this slice.

Focused proof at `92675f08d`: all six assignment/copy cases passed in `/tmp/pi-aura-followup-green.*`, including the actual `CustomAuraButton` initializer and secure-option copy. The earlier failed table-backed identity boundary is retained in `/tmp/pi-aura-three-models-green.*`; the source-backed userdata handle fixes it without accepting forgeable table markers.

Actual addon/SavedVariables startup returned `[]`, exit 0 after the separate dispel-filter, controlled-player token, and curve-userdata fixes (`/tmp/pi-accepted-final-startup.*`). This binding slice does not make broader native-fidelity claims.

## Out of scope

This retains existing best-effort formatting behavior. Native finalization, invalidation, metatable shape, ownership, and GC equivalence are not established; no finalizer or invalidation policy is added. Exact native clock/scheduling parity, expiration policy, color-curve evaluation, and general secret-value enforcement remain outside this proof. The bounded automatic engine scheduling and explicitly guessed secret-duration handoff above are included.
