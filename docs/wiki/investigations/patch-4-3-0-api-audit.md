# Patch 4.3.0 API audit

Pinned pageid **167555**, revision **1639407**, timestamp **2021-08-22T03:08:50Z**; supplied HTTP 200 response. Historical retail only. Default generator handles both inventories; existing `--legacy-api-tables` extractor retains navigation only. No parser/runtime/vendor changes.

## Coverage matrix

| Scope | Accounted | Remaining / proof boundary |
|---|---:|---|
| Global additions/removals | 76 + 7 = 83 occurrences | Both source headers match generator counts |
| Current retail publication/absence | 58 observations meet expectation | 25 exact publication gaps, individually reasoned in ledger |
| Prose/signatures | Zero statements | One metadata-only navigation row; linked API pages not reconstructed |
| Modeled behavior / native parity | Zero newly proved | No output, security, gameplay or historical-signature credit |
| Retirement | Seven source removals already absent | Zero runtime retirements; complete consumer scans preserved |

[Ledger](../../../data/patch-api/sources/4.3.0-page-coverage.json) contains **84 IDs**: 58 bounded publication-only, 25 audit-pending, one metadata-only. These numbers describe retained observations; validation derives totals from register/results/ledger and reviewed gap IDs rather than freezing them in code.

## Gap assessment

Legacy transmogrification needs pending item/slot eligibility, pricing, cursor and transaction state; modern outfit state is not its contract. Missing loot needs its own collection, not existing loot rolls. Guild absence/replacement/rename lacks authoritative lifecycle; an existing permission flag is insufficient. Map-level ranges, auction detail-column formatting, weapon sheath state, base spell cooldown metadata, CUF-profile load state, client executable architecture, selected-resolution catalog and specialization unlearning all lack established backing contracts. `UnitPowerBarTimerInfo` is currently nil-only, not a timer collection to count. Each missing name has its own precise ledger reason and cached consumer reference where present. **Zero behavioral gaps closed; 25 publication gaps retained.** No shims or speculative implementations.

## Discovery and development proof

Own prefork discovery at `2ffd97c5e` intentionally fails against the initial empty known-gap list and saves all 83 observations. Reviewed exact-gap sweep at `df53a1b9e` passes **1/1**; observations are unchanged. Seven original removals are absent without edits. Later retail supersession changes other expectations; cached Blizzard deprecation wrappers remain unmodified and receive only publication/absence credit.

[Consumer scan](../../../data/patch-api/evidence/4.3.0-session-2026-10-09/consumer-scan.json) covers entire cached retail Lua/XML and `src/`/`tests/` Rust/Lua/XML, including Lua strings. All source identities are globals, so qualified and bare spellings coincide. Exact file/line hits and tree digests are retained; all seven removed names have zero hits. This is not native-client absence proof. No global/method was changed; no caller migration or Classic behavior change was needed.

Actual 5.0.1 and newer retail registers are included; Classic 5.5.x is excluded. **Queued 4.3.4 placeholder is first** in `later_registers`, for coordinator replacement. Its source/register is unavailable here: possible supersession remains unassessed, not invented.

[Evidence](../../../data/patch-api/evidence/4.3.0-session-2026-10-09/) contains exact command logs and compact revision/tree/hash receipts. Own accounting/source/log-seal fixtures pass **8/8** at `eff4fbe2e`; targeted publication negative changes gaps **25 → 26** and fails exactly as required. [Command ledger](../../../data/patch-api/evidence/4.3.0-session-2026-10-09/p430-command-ledger.md) records exact scopes. Three compressed historical Git tree/blob identity manifests preserve root identities without requiring original Git objects; these are identity receipts, not historical compilation/source-byte replay. Tree identity/rejection fixtures pass **2/2** at `e06de50ae`; combined focused fixture proof **10/10**, with no redundant broad rerun. Every artifact remains below 5 MB. Standalone `validate.py` is supplied for coordinator replay but was not invoked as a final gate. No check/lint/type/readability/coverage, all-publication sweep, smoke/full-suite or final gate run; coordinator owns acceptance. No push/merge/deploy/delegation.

## Sources

- [Pinned source](../../../data/patch-api/sources/4.3.0-api-changes.wikitext), [pin](../../../data/patch-api/evidence/4.3.0-session-2026-10-09/source-pin.json) and [response](../../../data/patch-api/evidence/4.3.0-session-2026-10-09/source-response.json).
- [Publication spec](../../specs/patch-4-3-0-publication-sweep.md).
- [Binding prompt](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/page-agent-prompt.md).

## See Also

- [[patch-5-0-1-api-audit]] — first actual retail successor; queued 4.3.4 awaits integration.
- [[patch-audit-validator-portability]] — historical proof identity conventions.
