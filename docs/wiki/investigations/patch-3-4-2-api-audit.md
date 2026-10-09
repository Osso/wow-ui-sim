# Patch 3.4.2 Wrath Classic API audit

Bounded frozen-source audit against `f0baf34d43ea6d6eb1ef0aef2a85efae9f7d55f5`, observed 2026-10-09. Warcraft Wiki page **355030**, revision **3422177**, timestamp **2023-08-29T19:24:02Z**, TOC **30402** identifies Wrath Classic. No network retrieval or linked page reconstruction.

## Literal coverage matrix

The [source inventory](../../../data/patch-api/sources/3.4.2-source-inventory.json) preserves original spelling/owner, direction and source line for every occurrence. The [ledger](../../../data/patch-api/sources/3.4.2-page-coverage.json) accounts for all **208 nonblank lines**: **155 inventory**, **2 substantive prose**, **51 metadata/table-context**. At source-audit time all 157 substantive rows were **UNPROVEN for runtime**, not diagnosed missing APIs. Historical accounting remains unchanged; the separate current bare-factory measurement below does not manufacture native compatibility gaps.

| Section | Added | Removed | Proof |
|---|---:|---:|---|
| Global API | 63 | 31 | Literal identities/directions; current publication/absence unmeasured |
| Widgets | 16 | 2 | Literal owners/members; signatures/transitions unspecified |
| Events | 4 | 4 | Literal names; payload/emission/order unspecified |
| CVars | 26 | 9 | Literal defaults and available metadata retained; current registration/defaults/effects unmeasured |

Eight page headers match parsed counts; **109 additions, 46 removals**. No FrameXML inventory, commands, explicit call signatures, return structures or enum changes on this revision. Linked APIs/diffs are not expanded. The apparently unusual `EditBox:OnUiMapChanged` and `EditBox:GetFogOfWarBackground*` ownership is retained literally, not repaired from another page.

The two prose statements are “many API changes” from retail 10.1.0 (unspecified subset, not all) and Settings panel adoption (no named API/interaction contract). Both remain UNPROVEN. TOC, links, build caption `3.4.1 (47612) → 3.4.2 (50375) Jul 7 2023`, headings and table markup are source context, not measured runtime facts. CVar `desc`, `default`, `scope`, `cat` fields remain exact, including omissions/spelling; removed-CVar defaults describe source metadata, not a currently available default.

## Implementation and unsupported proof boundaries

Named globals/widgets specify identities only, not arguments, results, errors, security or state transitions. Added events supply no trigger/payload contract; accepting their names would not prove emission. Removed entries authorize no current Classic retirement without current absence/same-line successor evidence. No guessed models, shims, fallback paths or runtime removals were introduced.

CVar metadata does establish literal source defaults and descriptions. It does **not** measure current Wrath defaults, account/character persistence, graphics/VRS effects, logging/jobs, WowLabs social state, guild death announcements, nameplate scaling or target selection. Adding generic storage/defaults merely to pass publication would not model these effects. Those registration/default checks and subsystem behavior remain explicitly queued; this bounded task is own source-fixtures only, not a failed runtime implementation. No “all behavior unspecified” claim is made for CVar metadata.

[ClientProfile](../../../src/client_profile.rs) already supports `client-wrath`, `Wrath`, cache `wrath`, configured interface **38001**. That differs from source **30402**; profile existence is not historical version/native parity. [Observation](../../../data/patch-api/evidence/3.4.2-session-2026-10-09/profile-observation.json) records **42 local files**, exclusively API documentation directories, including a zero-byte `.missing` marker. No SharedXML/full publisher cache or native client probe. Static profile/manifest snapshots are retained; no cache sync/modification, build, runtime load or replay attempted.

Shared [publication classifier](../../../tests/common/publication_sweep.rs) accepts Retail/MistsClassic/ClassicEra, not WrathClassic; [generator CLI](../../../tools/gen_patch_wikitext_register.py) has the same client-line limitation. Existing pure section parsing correctly captures this inventory, so owned historical copies replay it into a separate `patch-source-inventory/v1`. No default-retail register or falsely passing retail sweep is created. This describes historical source-audit state. The bounded current change adds shared test-only Wrath classification and a cache-independent factory sweep, not a generator flag or publisher-loaded measurement; coordinator owns the latter if required.

## Actual successor and integration order

Actual **3.4.3 Wrath Classic** source is retained and identity/hash checked in own reference evidence: page 152751/revision 5983024, TOC 30403. Its audit at `48ab8e1c3` is completed but unmerged, and enumerates no explicit API members. Consequently no named 3.4.2 supersession can be derived from it. `later_registers` stays empty; a separate historical successor record queues integration **after 3.4.3**. Retail 10.1.0/10.1.7 references and Cata 4.4.0 navigation are not Wrath member-level proof. Successor empty inventory creates no positive runtime/publication credit.

## Historical source proof

Identity, exact returned content and both manifest hashes were validated before owned copies. Wikitext **12,105 bytes**, SHA-256 `284871b95a3150e0dd0383ae4070ad1c067c1eee874e83d5bb44ebbf12118f0e`; response **12,697 bytes**, SHA-256 `2b81231fe3cea60671d8c61715cbef6c152f06ee2a3fb127eacb578aa9e6fd1a`. [Pin](../../../data/patch-api/evidence/3.4.2-session-2026-10-09/source-pin.json) preserves the exact manifest row. Plaintext uses existing `--text-only --canonical-patch-navigation` flags. Inventory parsing requires no opt-in changes.

[Validator](../../../data/patch-api/evidence/3.4.2-session-2026-10-09/validate.py) derives counts, statuses and source rows from own serialized files and historical parser/extractor copies, not fixed global receipts, live caches or current shared tools. It validates literal identity, complete inventory reproduction, every nonblank line, all CVar fields, proof limits, profile and actual successor. Seals protect owned historical inputs/logs; trust anchor is tracked Git, not the mutable seal file itself. Historical snapshots remain observations at capture time, not assertions about replay-time host state.

[Tests](../../../data/patch-api/evidence/3.4.2-session-2026-10-09/test_source_accounting.py) reject serialized missing/changed rows, directions, defaults, headers, invented proof, wrong clients/successors and native/runtime credit. Targeted RED first failed missing derived accounting against a temporary empty validator. GREEN **10/10** passes at `4429022ea`, covering exhaustive per-row deletion/credit controls and literal tampering. Historical replay and recorded-flag extraction pass at `f9905c3a3`. Both disk-seal controls fail with exit 1 at the exact changed ledger/GREEN-log seal; original bytes/hashes restored. Current evidence plus owned sources is under 0.6 MB, with 23 sealed inputs. Later receipt/docs/seal-entry additions preserve all tested semantic input bytes; no redundant fixture rerun. Python formatted manually (ruff/black unavailable); no Rust changes or Cargo formatter invoked. Exact scope/revision/command/log is retained in [proof ledger](../../../data/patch-api/evidence/3.4.2-session-2026-10-09/source-proof.json).

No shared tools/runtime/vendor changes, push, merge, delegation, cwd switch or provider/model/retry changes. Main owns broader gates and integration. Source-accounting tests are not simulator/native compatibility tests.

## Current bare simulator factory measurement

Measurement base `990dae19c30b7b71c0ae3e1a4dfcd0a444546d84`, branch `p342-source`, 2026-10-09. Standalone [target](../../../tests/patch_3_4_2_factory.rs) uses `WowLuaEnv::new` and the unchanged 155-row source inventory directly. Build features `sound,gui,casc,client-wrath` compile successfully; no headless workaround, full Game initialization, cached publisher loading or native probe needed. Profile is **Wrath, interface 38001, 3.3.5-era simulator architecture**, not historical source **30402**.

| Literal section | Added match / mismatch | Removed match / mismatch | Proof level |
|---|---:|---:|---|
| Global API (94) | 20 / 43 | 14 / 17 | Raw publication plus ordinary lookup; no calls/contracts |
| Widgets (18) | 11 / 5 | 2 / 0 | Factory object method lookup; no native class/state parity |
| Events (8) | 4 / 0 | 0 / 4 | Registration accepts any nonempty name; weak discrimination |
| CVars (35) | 12 / 14 | 9 / 0 | Current value/default queried; metadata/effects not modeled |

**155 observations: 72 publication-direction matches, 83 strict mismatches.** The [current fixture](../../../data/patch-api/3.4.2-factory-known-gaps.json) pins the exact 83 IDs. These are not “83 native missing APIs”: among 43 added-global mismatches, 27 have raw nil / lookup function, 11 both nil, five raw-absent parents / lookup function. Sixteen removed globals have raw nil / lookup function; `GetAddOnMetadata` remains raw/lookup function. Simulator lookup behavior is retained, not replaced with production fixes or stubs.

All 18 widget owners construct and probe without errors; 13 method-direction matches and five added-method nil lookups. Literal `EditBox` owners remain unchanged. This proves simulator factory access only, not native widget identity, arguments, results or transitions. All eight event names register, including four source removals; the independent nonsense-name control also registers. Added-event matches cannot establish native event existence, emission, payload or removal enforcement.

For 26 added CVars, 14 lack current value/default, 12 are published: 11 defaults match, **`TargetAutoLock`: source `1`, observed value/default `0`**. The classifier intentionally records default differences separately from publication; 15 recorded default differences include the 14 absent registrations. Nine removed CVars return nil for both. Source `default`, `desc`, `scope`, `cat` metadata remains preserved on every current-ledger source row; no persistence, scope or subsystem-effect credit. Thus 83 publication mismatches plus one additional published-default mismatch cover 84 distinct source rows, not an undifferentiated native gap count.

`WrathClassic` matches only `ClientProfile::Wrath`; runtime cross-line control rejects retail inventory and ignores foreign retail/Mists removals while retaining observable control publication. Empty same-line successor contributes no override; actual 3.4.3 has zero explicit rows, so the measured inventory uses no successors. No retail/Cata supersession is inferred.

[Current evidence](../../../data/patch-api/evidence/3.4.2-factory-2026-10-09/) retains first classifier RED, all 155 discovery observations, strict-gap discovery RED, GREEN observations, a current per-occurrence measurement ledger and exact proof commands. Historical source ledger and all 23 seals remain untouched. Two substantive summary claims remain UNPROVEN; linked contracts, full Blizzard UI and native Wrath Classic remain unmeasured. Main owns integration, broad/profile/readability/final gates; no production API/vendor edits, push, merge, deploy or delegation.

## Aggregate profile compilation boundary

Main default-retail publication and Mists `--tests` checks reproduced E0432: generated aggregate test modules included the Wrath factory file even though its standalone Cargo target required `client-wrath`. Its import of the Wrath-only factory helper therefore failed under other profiles. A file-level `client-wrath` guard excludes the complete test module at that aggregation boundary; no runtime method, historical receipt or Wrath measurement changes. Follow-up compilation proof is separate from the retained original 6/6 factory run. Current profile checks/CI remain pending; failed commands are retained, not suppressed.

## Sources

- [Frozen wikitext](../../../data/patch-api/sources/3.4.2-api-changes.wikitext), [plaintext](../../../data/patch-api/sources/3.4.2-api-changes.txt), [source inventory](../../../data/patch-api/sources/3.4.2-source-inventory.json), [ledger](../../../data/patch-api/sources/3.4.2-page-coverage.json).
- [Spec](../../specs/patch-3-4-2-source-accounting.md), [owned evidence](../../../data/patch-api/evidence/3.4.2-session-2026-10-09/).

## See Also

- [[patch-3-4-3-api-audit]] — actual Wrath successor, queued unmerged integration; not member-level supersession.
- [[client-profiles]] — profile/cache selection distinct from historical native parity.
