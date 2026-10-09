# Patch 4.1.0 API audit

Pinned historical retail page 426651, revision 4094671, timestamp 2011-07-24T19:40:04Z; supplied locally October 9, 2026. Source body retained; HTTP 200 reported in supplied handoff. No network acquisition or Classic successor credit.

## Source format

Existing flags produce an empty register for the page's `API changes` / `Event changes` headings. New `--cataclysm-change-bullets` is opt-in: NEW groups, nested references, two-name removals, changed combat-log event pairs and padded template names retain their source lines. Default extraction retains the entire page, so no extractor change is needed. Publication is not historical signature/output proof. Accounting and targeted development evidence follow in a separate coherent commit.

## Sources

- [Pinned source](../../../data/patch-api/sources/4.1.0-api-changes.wikitext).
- [Source receipt](../../../data/patch-api/evidence/4.1.0-session-2026-10-09/source-response.json).
- [Spec](../../specs/patch-4-1-0-publication-sweep.md).

## See Also

- [[patch-5-0-1-api-audit]] — first already-integrated later retail page.
- [[patch-5-0-4-api-audit]] — next substantive retail successor.

## Bounded retirement

`GetPetHappiness` is incorrectly registered in retail despite explicit 4.1.0 removal. Full cached-retail/source/tests whole-word scans are retained in [retirement scans](../../../data/patch-api/evidence/4.1.0-session-2026-10-09/retirement-scans.json). Zero cached consumers; three test references become Classic-only and a retail raw/lookup absence case is added. Qualified and bare identities are identical for this unnamespaced global. Registration/handler are gated only under `client-retail`; pet state and non-retail behavior remain intact. No Blizzard wrapper or cache edit. Unit RED reproduces publication; retail and supported Mists GREEN results are recorded below. `Transform` and `END_REFUND` have cached bare matches, so no additional retirement changes.

## Capability matrix — historical implementation evidence

| Statements/cases | Closed/accounted | Still missing | Proof |
|---|---|---|---|
| Publication inventory | 81 occurrences, 50 current-retail publication/absence observations | 31 mismatches: 27 global occurrences, four combat-log event occurrences | Own prefork 1/1; results and exact known-gap set |
| `GetPetHappiness` removal | Retail raw/lookup absence; Mists default/seeded state unchanged | Native historical pet lifecycle unproven | Retail 1/1; Mists 2/2; full cached/src/tests pre-change scans |
| Retained full-page extraction | 86 nonempty rows; 78 metadata/cross-reference rows | Eight substantive prose rows | Data ledger; no behavioral credit from repeated API references |
| Combat-log signatures | Four named occurrences, both events on source lines 4/86 | hideCaster position 3 and following-argument shift native contract | Source only; current retail intentionally rejects script registrations |
| Negative publication control | 31 → 32 gaps, exact fabricated global | No missing-case acceptance | Required exit 1; committed fixture unchanged |

[Ledger](../../../data/patch-api/sources/4.1.0-page-coverage.json): **171 IDs = 81 inventory + 86 full-extract + 4 signature**; **50 bounded-coverage / 78 metadata-only / 43 audit-pending**. Callable no-ops, defaults and vendor deprecation wrappers receive publication-only credit, never modeled behavior. Precise per-row gap notes identify missing producer/state or unpinned input/output contracts; no speculative aliases, constants or shims added.

[Command ledger](../../../data/patch-api/evidence/4.1.0-session-2026-10-09/command-ledger.json) pins revisions, commands, scopes, exits and retained logs. Development proof at `4f806105b`: parser fixture 1/1 at `84564092e`, own prefork 1/1, retail pet absence 1/1, supported Mists pet tests 2/2, negative exit 1. Headless attempts fail in pre-existing GUI-dependent harness construction; supported profiles subsequently pass. Formatting ran before code commits. No check/lint/readability/coverage/broad/all-publication/smoke/full-suite/final gate was run; main owns acceptance. Six inherited iced manifest warnings retained and unsuppressed. Native 2011 client receipts are absent.

## Integration boundary

Current integration uses actual retail [4.2.0](patch-4-2-0-api-audit.md), [4.3.0](patch-4-3-0-api-audit.md) and [4.3.4](patch-4-3-4-api-audit.md) successors before 5.0.1 onward. The sole overlap is 4.3.0's `IsIPv6Available` removal: current publication accounting becomes **51 matches / 30 gaps** across the same 81 occurrences. This is removal/supersession credit only, not an IPv6 backing model, historical signature proof or native parity. Historical 50/31 observations and 171-ID ledger remain frozen at their capture boundary.

Main reports 69 current sweeps pass. The Mists check failed because a file was missing during main's rebase; retry, main runtime build, portability and independent verifier remain pending. Historical targeted Mists results do not close that current acceptance boundary. Native parity, full-suite and CI are unproven; no all-green acceptance claim.

## Own historical receipt preservation

Dedicated [validator](../../../data/patch-api/evidence/4.1.0-session-2026-10-09/validate.py) derives publication, extraction, signature, gap/status and command-summary counts from sealed inputs. [Compact pins](../../../data/patch-api/evidence/4.1.0-session-2026-10-09/historical-inputs.json) archive 78 deduplicated blobs in three selected Git trees, including the exact 64 real later-retail registers at development time. Gzip archive is 333,799 bytes; 23 source/evidence inputs are SHA-sealed, each under 5 MB. Original commit objects are not needed for receipt replay. The selected trees are not a complete native runtime replay and never prove current-head acceptance. No shared validator or prior audit receipt is modified. Own RED fixture requires missing validator implementation. [Targeted GREEN receipt](../../../data/patch-api/evidence/4.1.0-session-2026-10-09/validator-development-proof.json) at `b6b86d223`: one unittest passes clean replay in a copied no-`.git` fixture, rejects source and own-log tampering at exact seals, restores both byte-identically, and confirms clean replay again. This is development proof, not main-thread final acceptance. Registers/extracts reproduce byte-identically with frozen flags against archived parser/extractor blobs; counts are derived, not receipts hard-coded into the validator.

Historical validation now uses explicit [frozen ledger](../../../data/patch-api/evidence/4.1.0-session-2026-10-09/historical-page-coverage.json) and [gap fixture](../../../data/patch-api/evidence/4.1.0-session-2026-10-09/historical-known-gaps.json) sidecars with exact hashes recorded at `216568b5e`. Original 23 manifest seals, archive and logs remain unchanged; historical negative control remains 31 → 32. Current acceptance evidence is main-owned and separate from historical replay.

`GetPetHappiness` is a real legacy non-`C_*` global over preserved pet backing state, not a `C_*` namespace or compatibility shim. Retail publication is removed; non-retail, including Mists, retains that state-backed behavior. See [Lua API classification](../../lua-api.md).
