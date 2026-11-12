# GUID player identity query

`C_PlayerInfo.GUIDIsPlayer` has profile-specific contracts. Retail/PTR project explicit local, active populated party and target/focus unit-model identity/player-kind facts, not GUID syntax or offline/social catalogs. Forever uses the R39 native GUID parser: parsed Player kinds return true; other or failed parses return false. Cached `PlayerLocationDocumentation.lua` declares a required `WOWGUID`, one boolean result and `AllowedWhenUntainted`. Strict string/no-coercion and closed-world unknown handling below are simulator policy, not native validation evidence. See [API architecture](../lua-api.md).

## What it must do

### Retail and PTR

- [x] Authenticate the original secret argument before type validation or lookup; require an actual UTF-8 string and return exactly one boolean. Missing/nil/nonstring values error without coercion.
- [x] Return true for exact local identity and currently active populated party identities; target/focus use their explicit player-kind facts. Text prefixes never classify input.
- [x] Reflect live identity replacement, kind changes, removal, roster activation/population and independent environments without stale results.
- [x] Return false for unrepresented strings, including player-looking GUIDs and empty strings: “not represented as a player in current model state,” not native non-player classification.
- [x] Preserve authentic secret wrappers/caller taint: wrapped strings allowed when untainted, wrapped hits/misses/malformed values denied when tainted; ordinary strings remain usable.
- [x] Load the ordinary ObjectAPI addon closure and exercise original whole `PlayerLocation:CreateFromGUID`/`PlayerLocationMixin:IsValid`, with vendor source provenance. Represented players yield true; misses retain the unchanged account operand's observed result, not a native account-type claim. No account mocks or sliced consumers.

### Forever

- [ ] Register the query through the Game API installer. Indirect Glue installation or Lua copying remains UNKNOWN.
- [ ] Return true only for parsed Player GUIDs; return false for other parsed kinds and failed parses.
- [ ] Invalid GUID strings and numeric inputs return false. Numeric inputs convert to text before parsing.
- [ ] Missing/nil and non-string, non-number values raise the verified Usage text. Check secret access before type conversion and return one Boolean.
- [ ] Do not assert the inferred trailing-junk result or the returned Boolean's secret-publication status.

## How it works

- [Lua/API architecture](../lua-api.md)
- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_player_info_guid.rs` — Retail/PTR identity projection and Forever native GUID-kind parsing, with profile-specific registration.
- `src/lua_api/globals/unit_misc.rs` — existing represented identity projection.
- `src/lua_api/globals/unit_probes.rs` — existing target/focus player-kind facts.
- `src/lua_api/state_types/social.rs` — existing local identity constant.
- `tests/guid_is_player.rs` and `tests/forever_guid_is_player.rs` — profile-specific direct and consumer behavior.

## Tests asserting this spec

`tests/guid_is_player.rs` (Retail/PTR only): local/misses, target/focus replacement, kind/removal, party lifecycle, environment isolation, strict type, authentic secret boundary and two original ObjectAPI consumer cases. `tests/forever_guid_is_player.rs`: Forever Player-kind parsing, number conversion, Usage errors and secret-access ordering. Initial Retail RED at `76f405e1d`:0/9 PASS, exit101, eight missing raw-provider guards and one original whole-consumer exact-true mismatch. Ordinary closure/provenance reached; downstream query/security/account diagnostics unreached. Compile/fmt0 and source unchanged independently verified. At `d6d8de29a`, independent2929 confirms Retail9/9 and PTR9/9 PASS, two normal raw-query/startup markers with `[]`, fmt0/readability clean and four newly authenticated seals. Original whole consumer accepts represented players as true; NPC/unknown direct query is false while unchanged account operand and miss consumer return nil. Account provenance is Lua `=(string)`, not a modeled/native account classifier. Invalid UTF-8 byte rejection is source-inspected, not a separately executed regression.

Forever account GUID behavior remains covered separately by `tests/account_guid.rs`; it does not determine `GUIDIsPlayer` classification.

## Known gaps (current cycle)

- [ ] The Forever parser's trailing-junk result is INFERRED and untested. The returned Boolean's secret-publication status and indirect Glue installation remain UNKNOWN.
- [x] Genuine RED observed before producer; assertions unreached after missing-provider guards receive no behavioral credit.
- [x] Implement and verify Retail/PTR direct/consumer/startup checks, preserve diagnostics and account predicate provenance.
- [x] Fresh default library check, owning strict publication and current-validator gates: main-verified `d7a34be4a` check exit0, `a555f927a` strict case PASS and `d21ca3f64` validator retry9/9 plus actual validator exit0. [Current packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/guid-is-player-current-2026-10-12/README.md) owns detailed evidence; independent2939 retention/link/privacy PASS at19988.
- [x] Public retention/link/privacy and collision disclosure independently2939 verified at19988; raw packet unchanged.
- [ ] Original structured RED JSON NOT_RECOVERED: GREEN overwrote generic `proof.json`; original RED report/raw/auth receipt survive. Reconstruction is not byte-exact.
- [x] Saved d21 broad suite PASS; [broad packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/test-perf/evidence/fullsuite-d21-2026-10-12/README.md) owns phase counts, raw result/log/proof and selected outcomes, including GUID9 and original strict publication PASS. New packet retention verified by independent2944 checks/2945 semantic correction; original word-only checker FAIL preserved/rejected as noncontract; existing GUID retention independent2939 PASS unchanged. Historical533cFAIL/BA3FAIL unchanged; broad PASS is not full-handoff completion.
- [ ] GUI authenticated-user review, native/other-profile acceptance, credits and Pi provenance remain OPEN; no successful body-timing, cache-content, historical-equivalence or causal claim.

Initial publication1FAIL/exit1 and wrong101 worker guard, publication-before fixture-race/coherenceFAIL, and earlier validator8/9FAIL with standalone NOT RUN remain historical. Retry does not relabel them. Native, earlier-profile, external and whole-handoff acceptance remain OPEN.

## Out of scope

Native arbitrary-GUID classification/coercion/catalogs, prefix inference, new normal-session identities/seeding, account-type implementation, earlier-profile contracts, vendor/global overrides, native or full-handoff acceptance.
