# Mount item relations

Current Retail/PTR `C_MountJournal.GetMountFromItem(itemID)` queries an explicit simulator-owned item-to-mount relation. Cached API declarations establish one required numeric input, a nullable numeric mount ID return, and `AllowedWhenUntainted` secret arguments. Catalog contents, ownership filtering and dressability rules are not established by those declarations.

## What it must do

- [x] An unseeded environment returns exactly one nil for an unmapped numeric item ID. No normal-session relation seeding or inferred spell/name/link mapping.
- [x] A supplied mapping returns its exact numeric mount ID without requiring an ordinary journal record or ownership. Replacements/removals are live; separate environments remain independent.
- [x] Missing/nil and nonnumeric inputs fail the required-number contract. Item links and numeric strings are not this API's numeric argument. Simulator policy reuses existing supplied-relation identities: finite integral signed32-bit values may match, while fractional/nonfinite/out-of-range numbers miss without truncation or saturation. This is not a native range claim.
- [x] Authenticate the original VM secret argument before type validation or lookup. Secure callers can query wrapped numeric inputs; tainted callers cannot inspect wrapped inputs, including malformed values. Ordinary numbers remain usable from tainted callers. Preserve wrapper identity and caller taint.
- [x] Preserve existing mount spell lookup; do not transfer `GetMountFromSpell`'s separate `SpellIdentifier`/`AllowedWhenTainted` contract.
- [ ] Preserve earlier-profile surfaces; API exclusion is source-inspected only, earlier-profile runtime preservation unverified. Shared-state compile refresh is independently verified, not runtime proof.

## How it works

- [Lua/C API surface](../lua-api.md) — runtime integration context.
- [Retained public packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/mount-item-relations-2026-10-12/README.md) — authoritative bounded query, preservation and consumer evidence.

## Implementation inventory

- `src/lua_api/state/sim_state.rs` — explicit environment-owned `mount_item_relations` field.
- `src/lua_api/state.rs` — empty default; no normal catalog population.
- `tests/mount_from_item.rs` — initial/expanded direct queries and mapped-secret recovery coverage.
- `tests/mount_item_player_choice.rs` — original whole-template baseline and live relation contrast; returns test-owned supplier/result observations alongside the boolean, independent of vendor print routing and without guessing classification.
- `src/c_api/c_mount_item.rs` — current-profile secret-authenticated supplied-relation query.
- `src/c_api/mod.rs` and `src/c_api/registration.rs` — Retail/PTR module and namespace wiring.

## Tests asserting this spec

`tests/mount_from_item.rs` has three initial empty-result, required-number and authentic secret/taint cases, four supplied-state/no-catalog, replacement/removal, environment-isolation and numeric-non-aliasing cases, and one mapped-secret hit/denial/recovery control. The additional mapped-secret case has no separate prior RED credit. Initial `0808b3cd0` execution independently verified0PASS/3FAIL at missing-API guards; downstream assertions were not reached. Empty field/default scaffolding follows that genuine RED. Expanded `3cc6d36af` execution independently verified0PASS/7FAIL at missing-API guards after fixture setup. Query producer follows that actual RED. Eight query cases per Retail/PTR pass; the eighth mapped-secret case first executed GREEN, with no separate prior RED.

## Known gaps (current cycle)

- [ ] Earlier-profile runtime preservation remains unverified.

### Current bounded evidence — 2026-10-12

The [final query report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/mount-item-relations-2026-10-12/final-current-report.md) records historical `7487c8bec`: 28 new +24 reused =52 scoped test executions, zero test failures. Retail/PTR normal probes each reached the actual marker followed by `[]`, exit0. Fresh default library check at that stage passed; the [current observation report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/mount-item-relations-2026-10-12/observation-current-report.md) retains its applicability via an empty production/query/spell/normal/check-scope diff. Current formatting exits0. Six inherited dependency-manifest deprecations remain; nil/missing-requirement diagnostics are retained, not assertion failures.

Independent observation at `609033535` is bounded PASS: four new consumer executions, eight original invocations and 88 raw rows. Current scope is 4 new +48 reused query/spell executions =52: eight query and sixteen spell-preservation cases per profile. Earlier consumer executions remain historical, not double-counted. Runtime units are terminal; CLI/check/fmt operations are separate from test counts. Independent2865 packet retention/link/privacy PASS at `587a22e38`:303 copies/9,996,495 bytes plus13 snapshots/411,010 bytes,20 links/four anchors,zero privacy findings.

Each profile executes the unsliced original PlayerChoice live contrast first: false→true→false for absent, supplied `6948→401`, then removed relation; standalone baseline follows with false. Original ItemButton `SetItem(6948)` and vendor method provenance/input/stage observations are retained. Actual predicate observations are raw registered function / nil return type / nil value, Lua `=(string):72–74`, matching the existing unmodeled namespace fallback. This proves supplied-relation consumer dependence in this simulator context, not modeled/native dressability or native Hearthstone classification. No vendor predicate replacement or normal-session relation seeding.

The original raw-console observation FAIL in the final report remains unchanged; corrected test-owned observation transport supplies the later PASS. Initial RED3 and expanded RED7 failed at missing-API guards; downstream assertions were unreached. Eighth mapped-secret query and consumer contrast first executed GREEN, with no invented RED credit. [Earlier-profile compile refresh](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/mount-shared-state-profile-check-2026-10-12/README.md) independently verifies five existing checks at `dceb4c751`; shared-state compile compatibility only. Earlier-profile API exclusion remains source-only; no blanket earlier-profile runtime preservation acceptance. Native/catalog/range/classification/ownership, external and full-handoff acceptance remain open.

## Out of scope

Native catalog/range/classification rules, ownership inference and normal relation population: absent source authority or user seeding authorization. Native/external/full-handoff acceptance remains separate and open; this query slice cannot close the parent goal.

## Current bounded GUI publication — 2026-10-12

[New90-second WoW API publication only; not Pi/LLM provider diagnostics](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/live-retail-gui-providers-2026-10-12/README.md) · [Independent publication gate; logged event, not pixels; intentional124 expiry, not assertion failure or service repair](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/live-retail-gui-providers-2026-10-12/live-provider-report.md). [Owned-process/socket cleanup, existing display preserved;15 copies/39,405 bytes retention independently2905 PASS at5e1b7bab4; native/visual/authenticated-remote/user acceptance OPEN](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/live-retail-gui-providers-2026-10-12/README.md).

