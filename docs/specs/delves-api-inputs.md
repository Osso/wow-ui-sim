# Delves API inputs

`C_DelvesUI.HasActiveDelve`, `RequestPartyEligibilityForDelveTiers` and `GetCurioLink` use explicit per-environment host inputs. The 12.0.5 [Global API source](../../data/patch-api/sources/12.0.5-api-changes.txt) and [register](../../data/patch-api/sources/12.0.5-register.json) identify rows 257, 258, 262, 263 and 265:

| Source ID | Delta |
|---|---|
| `global api-C_DelvesUI-GetCurioLink-257` | arg1 type number → SpellIdentifier |
| `global api-C_DelvesUI-GetCurioLink-258` | arg2 NeverSecret added |
| `global api-C_DelvesUI-HasActiveDelve-262` | mapID argument removed |
| `global api-C_DelvesUI-HasActiveDelve-263` | AllowedWhenUntainted argument annotation removed |
| `global api-C_DelvesUI-RequestPartyEligibilityForDelveTiers-265` | gossipOption renamed mapID |

Cached retail `Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua` declares: lines 372–380, `HasActiveDelve()` returns one nonnil boolean, with no arguments or SecretArguments annotation; lines 454–464, `RequestPartyEligibilityForDelveTiers(mapID: number)` requires nonnil input, `SecretArguments = 'AllowedWhenUntainted'`, with no declared returns; lines 45–59, `GetCurioLink(spellID: SpellIdentifier, rarity: CurioRarity)` requires both inputs, marks rarity `NeverSecret = true`, declares `SecretArguments = 'AllowedWhenTainted'`, and returns one nonnil cstring. Declarations are not native execution evidence.

## What it must do

### Active delve

- [x] Return exactly one public boolean from `SimState.has_active_delve`, default false; read host changes live.
- [ ] Never read, validate, authenticate or convert undeclared extra arguments. Public invalid values and authentic secret number/string extras leave the result unchanged for untainted and tainted callers.
- [x] Preserve secret wrappers and caller taint; no former map constant selects active state.

### Eligibility request

- [x] Authenticate arg1 with VM `unwrap_secret` before validation and state mutation. Untainted callers may pass a secret number; tainted callers passing a secret are denied, including secret strings that would fail numeric validation.
- [x] Require a nonnil finite integral number; missing, nil, wrong type, fractional and nonfinite values error without changing the previous request.
- [x] Store the accepted map ID in `SimState.last_delve_eligibility_map_id`, initially None; each accepted request replaces the last map and returns zero values.
- [x] INFERRED storage policy: accept integral values within signed i32 range, including zero and negative values; do not impose map-catalog membership. Reject out-of-range values before mutation.
- [x] Preserve taint and wrapper secrecy; tainted public requests remain usable after denial.

### Curio link

- [x] Resolve public numeric IDs and explicitly seeded, case-insensitive aliases through `read_public_spell_identifier_at`; seeded numeric aliases take precedence over numeric identity.
- [ ] Read `(resolved spell ID, rarity)` from `SimState.curio_links`, initially empty. Return exactly one public string identical to the host record, without fabricating link text or mutating records/aliases.
- [x] Read replacements and removals live. INFERRED miss policy: unknown aliases, absent spells and absent rarities raise an error because the cached return is nonnil; this is not native-verified missing-curio behavior.
- [x] Reject authentic secret arg2 for every caller before any conversion or lookup, retaining secrecy and caller taint; denial remains effective after full GC with rooted wrappers.
- [ ] INFERRED validation/order policy: validate rarity as a nonnegative finite integral u32 number before resolving arg1. No CurioRarity enum membership validation. Wrong type, nil, missing, fractional, nonfinite and out-of-range rarity errors.
- [ ] INFERRED inherited identifier boundary: strict public UTF-8 strings or finite integral u32 numbers only; reject secret identifiers for both caller classes. This does not implement the cached AllowedWhenTainted permission for arg1.
- [x] Recover with public inputs after failures; retain environment isolation for all three host fields.

## How it works

- [Lua API and shared state](../lua-api.md)
- [C API signature audit](../c-api-signature-audit.md)

## Implementation inventory

- `src/lua_api/globals/missing_surface/delves_ui.rs` — existing registration and three state-backed producers; placement retained because this bounded slice cannot edit registration owners or create a C API module.
- `src/c_api/c_spell.rs` — existing strict public identifier reader and host alias resolution.
- `src/lua_api/state/sim_state.rs` — three host field declarations (integration insertion delegated to main session).
- `src/lua_api/state.rs` — false/None/empty defaults (integration insertion delegated to main session).

## Tests asserting this spec

- `tests/delves_api_inputs.rs` — nine behavioral tests: live active state, ignored secret extras, observable eligibility/arity, malformed request nonmutation, authenticated secret requests/denial, exact curio links/aliases/live state, invalid public curio inputs, NeverSecret/GC/taint boundary, environment isolation.
- `tests/delves_ui.rs` — existing entrance test now expects former map argument not to activate a delve.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs and producer landed together in `a0e23199d`. RED with the producers withheld from the working tree: 0 PASS / 9 FAIL plus the changed `delves_ui::` expectation. GREEN: 10/10; the combined run was 396 PASS / 1 FAIL, the failure being `c_system_api::test_c_console_get_all_commands_empty` on an untouched console command count. `cargo fmt --check` exit0; startup `lua-errors` `[]`.

Main accepts an independent GPT-6.1-sol review: **ACCEPT WITH QUALIFICATIONS** (report SHA256 `05e8b7e825db92730ba365323823df0beb7f693ce661837e2e932414e00f8a19`, scratchpad-only), own rerun 10/10 exit0. Row 258 earns the rarity `NeverSecret` policy only. Four requirements stay unchecked as partly untested (tainted invalid extra, initially empty map, validation order and rarity bounds, malformed UTF-8). Checked requirements are bounded simulator proof on the tested fixtures, not native parity. No `cargo check`, broad suite or older-profile run.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows 262, 263, 265, 257, 258 under new capability `delves-api-inputs`; **107 capabilities/362 IDs; 77 pending /239 bounded /13 partial /33 metadata**.

## Known gaps (current cycle)

- [ ] `curio_links` is empty by default and a miss errors, so the two cached `Blizzard_DelvesCompanionConfiguration.lua` chat-link callers (lines 399 and 676) raise until a host seeds records; before this change they received a fabricated link. Established from source, not reproduced in the UI.
- [ ] Model ownership remains in existing missing_surface placement under the exclusive file boundary; moving ownership into `src/c_api/` is not performed by this slice.

## Out of scope

- Row 260 is specified separately in [Tiered entrance PDEID](tiered-entrance-pdeid.md); all other Delves methods remain outside this input slice.
- Native acquisition of curio links or active-delve state, native link encoding, eligibility results/tooltips, response timing and events: no backing evidence or request authorization.
- Native secret spell identifier permission, missing-record behavior, numeric domain limits and exact error wording: unverified; inferred policies above are simulator bounds only.
- Older-profile/native parity and accounting updates: not asserted or changed here.
