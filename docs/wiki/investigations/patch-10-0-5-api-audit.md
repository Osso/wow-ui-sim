# Patch 10.0.5 API page audit

Page 75033, revision 742761 (February 1, 2023, 00:52:10 UTC), retrieved October 7, 2026 UTC. Branch `p1005-page` starts from master `c6501877b`; master already contains the 10.0.7 sweep. Default retail carries 12.1.0, not reconstructed 10.0.5.

## Source accounting

93 inventory occurrences: 75 added, eight removed, ten changed. Global API 49 added/four removed/eight changed, events 17/one/two, CVars nine/three. Every published header count matches. Widgets says None. All 23 chronological later registers from 10.0.7 through 12.1.0 apply; none changes the final publication expectation of a 10.0.5 row.

160 unique source IDs: 93 inventory + 67 extract occurrences. Ledger statuses: 17 bounded-coverage, 49 partial-development-green, 87 audit-pending, seven metadata-only. Pending comprises 27 exact publication gaps and all 60 substantive extract occurrences. Bounded coverage means current absence or CVar queries only; partial green means published function/event registration only. Neither establishes signatures, populated DTOs, security, payloads or historical/native parity. No current CVar default mismatches were observed.

Existing parser/extractor already handles this page's level-two inventories and Structures exit. No tool changes were needed. The retained extract includes both Summary statements and the full enum/constant/DTO tail. [Literal scout](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-extract-scout.json) accounts for every nonblank extract occurrence with line, source ID and specific proof boundary. Item 17 asynchronous data unavailability/non-fired callback, saved-instance return 14, enum identities/flag positions, exact constants and populated field changes remain unproved; no subsystem test is credited without occurrence-specific evidence.

## Bounded closures

Discovery 63 OK / 30 gaps. Final 66 OK / 27 gaps. Three unused C_TradeSkillUI namespace autostubs are retired: ContinueRecast, GetRecipeRepeatCount and HasRecipesTracked. A separate RETIRED_10_0_5_MEMBERS constant uses the existing retail epoch module gate; classic profiles do not load it.

[Qualified and bare cached Lua searches](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-removal-consumers.json) find no matches for any of the three members. BNSendSoR was already absent; no new global retirement is required. [Initial whole src/tests scan](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-whole-caller-scan.txt) is empty. [Final scan](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-final-whole-caller-scan.txt) includes all files with --hidden --no-ignore and retains untruncated bare-name matches, including embedded Lua, function references and guards. Matches are only the new retirement assertions/classic preservation test and the retirement constant; no pre-existing retail integration or prefork callers need migration.

Repeated bare lookup and one unmodified cached Game prefork prove absence and successor publication. Nine existing tracked-recipe tests preserve separate normal/recraft buckets, transitions and event payloads. Mists behavior assertions preserve all three legacy lookup members. No Blizzard deprecation wrapper, vendor Lua, cache file, existing registration or model was deleted or monkey-patched.

## Retained gaps

[Per-ID review](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-gap-review.json) assigns every discovery failure once: three closed and 27 retained. Fourteen Trading Post catalog/drag/freeze/purchase/refund/timer/telemetry APIs need real producers; existing empty-catalog startup defaults explicitly document the missing model in `src/lua_api/workarounds/temporary/perks_program_defaults.rs:1-5`. No new placeholder was added.

Remaining contracts need favorite-customer catalogs, complete mount display variants, item-use eligibility, PlayerInfoCharacterData, PvP unlock/rated-match penalties, first-craft history, remote inspected talent serialization, currency-specific trait reset and layout-added transitions. [Native declaration excerpts](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-native-contracts.json) and [whole-tree/declaration scans](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-gap-source-scans.json) retain exact file:line references. Similar local APIs are not aliases for these contracts.

IsUsingGamepad has a modeled implementation only under client-wowforever (`globals/register.rs:186-187`, `state/sim_state.rs:18-19`); retail has no active-device-style producer. IsUsingMouse also lacks one. Capability, frame-enable flags and cursor/button state are not evidence of which device is actively used; neither global is fabricated.

## Verification

Runtime/test revision `263574c2f`. All 24 publication sweeps run alone through the integration target (autotests=false), and match exact existing fixtures. The initial direct --test patch_10_0_5_publication_sweep command had no registered target and produced no runtime evidence; the corrected integration-filter discovery was expected RED. Bare retirement RED fails at ContinueRecast, then GREEN after the modeled namespace retirement. New cached Game prefork and Mists legacy assertions pass; nine successor regressions pass. Existing extractor/parser fixtures pass 21 + seven.

Negative control changes only C_Mail.SetOpeningAll added → removed in a scratch register: exactly that new failure, no resolved failures, 27 → 28, expected exit 101. [Result](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-negative-result.json).

Cargo fmt/fmt --check, default check, Mists check --no-default-features --features sound,gui,casc,client-mists --tests and separate retail build pass. Mists check has zero non-vendor warnings; six existing iced manifest deprecations and their summary remain unsuppressed. Retail is rebuilt after Mists verification. Bounded startup exits zero and returns `[]`. Changed Rust manually audited: one flat retirement constant/call, bounded lookup assertions, a flat sweep specification and classic/cached tests; no changed-line readability violations.

[Proof ledger](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-proof.json) retains commands, exact revisions/scopes, exits and hashes. Cargo output is captured once, saved and searched, never rerun to recover logs. Complete logs are retained as gzip files with compressed and decompressed hashes. Documentation/data-only follow-ups do not invalidate compiled runtime proof. Local targeted evidence is not native or independent acceptance.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.0.5 | 93 | 66 | 27 | 0 |
| 10.0.7 | 70 | 44 | 26 | 0 |
| 10.1.0 | 129 | 93 | 36 | 0 |
| 10.1.5 | 101 | 69 | 32 | 0 |
| 10.1.7 | 48 | 34 | 14 | 0 |
| 10.2.0 | 150 | 120 | 30 | 0 |
| 10.2.5 | 59 | 45 | 14 | 0 |
| 10.2.6 | 220 | 200 | 20 | 0 |
| 10.2.7 | 104 | 68 | 36 | 0 |
| 11.0.0 | 495 | 329 | 166 | 0 |
| 11.0.2 | 34 | 22 | 12 | 0 |
| 11.0.5 | 48 | 38 | 10 | 0 |
| 11.0.7 | 98 | 70 | 28 | 0 |
| 11.1.0 | 116 | 97 | 19 | 0 |
| 11.1.5 | 125 | 89 | 36 | 0 |
| 11.1.7 | 48 | 40 | 8 | 0 |
| 11.2.0 | 162 | 135 | 27 | 0 |
| 11.2.5 | 163 | 118 | 45 | 0 |
| 11.2.7 | 508 | 414 | 94 | 0 |
| 12.0.0 | 1010 | 989 | 21 | 0 |
| 12.0.1 | 225 | 222 | 3 | 0 |
| 12.0.5 | 363 | 352 | 11 | 0 |
| 12.0.7 | 174 | 171 | 3 | 0 |
| 12.1.0 | 778 | 773 | 5 | 0 |

[Artifact validation](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-validation-result.json) passes at `95dbd6db4`: all 160 IDs/source/scout hashes, exact fixtures/later expectations, negative control, 146 preserved inputs, both extract modes, 24 register reproductions and 43 hashed logs. Reproduce with the retained evidence-directory `validate.py`; later reason/documentation-only changes preserve these proof scopes.

All 24 registers regenerate byte-identically. [Preservation](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-preservation.json) proves all 146 pre-existing source/register/ledger/gap inputs unchanged. [Before](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-extract-before.json) and [after](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-extract-after.json) checks preserve every prior extract success/failure under both example modes. Pre-existing 12.0.5/12.0.7/12.1.0 capture failures remain untouched, not labeled reproducible. Own extract reproduces under both modes.

No full suite, agents/model CLIs, push, merge, sibling target reuse or canonical working-file/vendor/cache changes. Every command uses explicit cwd `p1005-page` and its own target; only prescribed worktree-creation Git metadata was changed through the canonical path while cwd was the empty destination. The untracked PLAN.md task queue stays local.

## Sources

- [Provenance](../../../data/patch-api/sources/10.0.5-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/10.0.5-page-coverage.json)
- [Publication contract](../../specs/patch-10-0-5-publication-sweep.md)
- [Register reproduction](../../../data/patch-api/evidence/10.0.5-session-2026-10-07/p1005-register-reproduction.json)

## See Also

- [[patch-10-0-7-api-audit]] — publication/accounting template.
- [[patch-10-1-0-api-audit]] — gap/native-contract and extract proof boundaries.
- [[client-profiles]] — retail/classic gates.
