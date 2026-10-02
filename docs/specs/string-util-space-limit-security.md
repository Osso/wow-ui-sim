# StringUtil space-limit argument security

Batch50 covers only `C_StringUtil.RemoveContiguousSpaces(text, maxAllowedSpaces)` argument 2, exact Retail 12.0.5 source row328. Existing [ASCII-space contract](contiguous-ascii-spaces.md) and provider remain unchanged. See [Lua architecture](../lua-api.md) and [patch audit wiki](../wiki/investigations/patch-12-0-5-api-audit.md).

## What it must do

### Retained source facts and documented ASCII behavior

- [x] Preserve exact row `global api-C_StringUtil-RemoveContiguousSpaces-328`: retained [patch text](../../data/patch-api/sources/12.0.5-api-changes.txt), lines327–328, says `C_StringUtil.RemoveContiguousSpaces` / `+ arg2 NeverSecret`; [register](../../data/patch-api/sources/12.0.5-register.json), lines2150–2159, records that delta only. It does not establish secret arg1 acceptance, result secrecy or caller permissions.
- [x] Preserve required string text, numeric `maxAllowedSpaces`, and one string `trimmedText`, recorded in the [12.0.0 register](../../data/patch-api/sources/12.0.0-register.json), lines10761–10784.
- [x] Retain documented ASCII-space run truncation. A representative public matrix at limits0/1/2 preserves tabs, newlines, NUL, UTF-8 nonbreaking-space bytes and an invalid UTF-8 byte; output is byte-exact, arity1, an actual unwrapped public VM STRING.

Cached later Retail documentation at `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/StringUtilDocumentation.lua` says “Returns a string with all contiguous occurrences of ASCII space characters truncated.” It declares required `stringView` text, required numeric `maxAllowedSpaces` with `NeverSecret = true`, and one required string result. Its namespace function metadata says `SecretArguments = "AllowedWhenTainted"`. This later cache supplies documentation context, not native 12.0.5 runtime evidence or an additional row328 delta.

### INFERRED simulator policies, not native-verified semantics

- [x] Reject actual host-VM secret NUMBER limits with known payloads0/1/2/1e100 on ordinary valid public text, under secure and stamped-tainted callers. The large public counterpart succeeds unchanged, so rejection is not invalid numeric-range evidence.
- [x] Validate/reject arg2 even when public text is empty or contains no ASCII spaces; all four secret NUMBER limits still reject in both caller contexts.
- [x] Reject actual host-secret STRING with numeric-like payload `1`, and actual secret wrappers around a globally rooted Frame and Table. Establish Frame identity through `GetObjectType() == 'Frame'`, not an assumed userdata representation.
- [x] Preserve globally/list-rooted wrapper identity and secrecy across forced GC and rejection; preserve original object identity, sentinel inputs, frame alpha/marker, table fields and caller stack taint. Every rejection permits meaningful public ASCII-text recovery in the same caller context.
- [x] Retain existing strict required actual-NUMBER, finite, nonnegative, integral limit validation; public numeric string, boolean, Table, Frame, negative, fractional, NaN and infinities reject in both contexts. No new validation policy or exact-error-text contract.
- [x] Public calls remain usable under stamped taint without clearing/changing it. This is a public simulator control, not proof of native `AllowedWhenTainted` secret permissions.

## How it works

- [Lua architecture](../lua-api.md)
- [Retail 12.0.5 audit](../wiki/investigations/patch-12-0-5-api-audit.md)
- [Patch audit manifest system](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/c_api/c_string_util.rs`: existing Retail 12.0.0+ registration, strict limit validation and ASCII-byte run truncation; unchanged by this slice.
- `tests/string_util_space_limit_security.rs`: eight Retail 12.0.5-feature fixtures, automatically grouped by the existing integration harness; no new Cargo target.

## Tests asserting this spec

`tests/string_util_space_limit_security.rs`:

1. `secret_number_limits_reject_on_ordinary_valid_text_in_both_contexts`
2. `empty_text_cannot_short_circuit_secret_number_limit_rejection`
3. `space_free_text_cannot_short_circuit_secret_number_limit_rejection`
4. `secret_numeric_string_limit_rejects_without_coercion_in_both_contexts`
5. `wrapped_actual_frame_and_table_limits_reject_in_both_contexts`
6. `forced_gc_preserves_rooted_secret_identity_state_and_public_recovery`
7. `public_three_limit_matrix_preserves_non_space_bytes_and_single_public_string`
8. `retained_public_strict_limit_policy_rejects_and_recovers_in_both_contexts`

Existing `tests/c_api_surface.rs` owns broader algorithm fixtures; these tests call the actual C API without replacing it, do not unwrap/read private payloads, and require a function before attempted rejection to exclude missing-registration false positives.

## Known gaps (current cycle)

- [x] Parent accepted independent389 exact row328 bounded development. [Combined acceptance SSOT](private-aura-sound-removal.md#independent-bounded-acceptance--2026-10-02) owns accounting, gates and limits; earlier pending checkpoints are historical. Existing provider validates/rejects opaque secret limits, without payload authentication.
- [ ] Native runtime rejection/coercion/range/error and precise byte-edge parity remain unverified. Concrete controls preserve the existing simulator policy only.

## Out of scope

Secret arg1 acceptance, secret-result propagation, and native `AllowedWhenTainted` permissions are UNKNOWN and deliberately not asserted. No producer/algorithm, registration, module, Cargo, simulator-state implementation, vendor, PLAN changes. Combined proof and bounded accounting live in the linked sound spec. No native probe, build, test execution, checks, lint, readability/coverage gates, operations, deployment or delegation in this docs reconciliation.
