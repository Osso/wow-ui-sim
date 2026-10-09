# Historical retail Patch 2.3.0 bounded SOURCE audit

Frozen page **44655**, revision **6055877**, timestamp **2024-06-04T05:03:50Z**; audited 2026-10-09 in `p230-source` at base `614402d56f726ec34cedea2f28a9609cdd653462`. Original retail history, not Burning Crusade Classic 2.5, Wrath Classic 3.4 or Era. Only source accounting; no runtime/native/model observations or edits.

## Literal coverage matrix

[Ledger](../../../data/patch-api/sources/2.3.0-page-coverage.json): **163 nonblank raw rows**, 16 blank rows, **132 inventory occurrences**, **137 signature/command-limit rows**, **68 prose rows**, **11 literal headings**, **82 unexpanded API links**. Raw rows: 150 UNPROVEN / 13 metadata (navigation, 11 headings, editorial diff/string-dump comment). No numerical inventory-count headers or explicit CVars exist on this page; zero CVars does not establish a native absence claim.

| Explicit source occurrence | Count | Proof boundary |
|---|---:|---|
| Globals | 116 | 34 labeled occurrences plus 82 consolidated links; duplicate identities preserved, not unique-function count |
| Widget methods | 10 | Includes both tooltip rename endpoints; ScrollFrame automatic recalculation is separate prose/reference evidence |
| Events | 1 | `INSPECT_TALENT_READY`: availability/update prose, no complete payload/dispatch/lifecycle proof |
| Slash commands | 5 | Four new identities and changed `/cancelform`; `/dismount` and `/stopcasting` remain references, not inferred changes |

Directions **28 added / 18 changed / 2 removed / 2 noted / 82 listed**. `GetSendMailPrice` and `GetInboxHeaderInfo` NOTE behavior is retained without fabricating addition/removal. `GameTooltip:SetTrackingSpell` → `GameTooltip:SetTracking` retains both endpoints. `UDPATED` and missing NEW hyphen on cursor getter preserved literally. Consolidated links use **listed**, not inferred added: the editorial comparison mentions `Special:Diff/4997529` and a 2.2.0 string dump; neither is expanded or measured here.

**Signature boundary:** 50 literal API call fragments (including examples/references), 82 linked-identity unknown signatures and five literal command-line limits = 137 rows. Named return prefixes, optional-bracket syntax, spaces before talent parentheses and argument alternatives remain exact; empty parentheses in an update do not reconstruct the previous full argument list. `complete_signature=false` means no complete native-contract proof, not absence of explicit call syntax or named outputs. Source describes two boolean outputs for auction query/reversal, sort return types/domains and nil missing sort; those clauses remain in prose as well as literal prefixes. No publication execution.

| Source section | Exact retained contract | UNPROVEN boundary |
|---|---|---|
| Casting/macros | Latency/fallthrough; pet toggle, last enemy/friend, exact target, instant cancelform; modifier→mod, button→btn, actionbar→bar, equipped→worn, stance→form and SELFCAST example | Parser dispatch, targets, pet state, timing and conditional evaluation |
| Frames/support | Automatic scroll-child rect/event recalculation; event registration/enumeration, formatted text/allocation, string height, explicitOnly false/zero behavior, cursor position, tracking rename | Real widget/event state, ordering, metrics, allocation and native correctness |
| APIs/GC | Message window despite errors disabled; pickup/count/use flags, book/slot spell state/action equivalences, helpful/harmful exceptions, macro item/link/spell/rank; out-of-combat GC and reuse advice | Backing item/spell/macro/display state and collector schedule |
| Auctions | getAll versus 50-item query, 15-minute throttle; two boolean outputs; indexed sort order/types/nil; clear/prepend without apply, explicit apply and quantity sorting | Auction data, throttle time, ordering/reversal/apply transitions; existing parameters not reconstructed |
| Mail/talents/bug | Compose slots vs received indices, attachment pricing/count/retrieval/removal; truthy inspect unit selection and readiness event; UnitName absent-unit regression | Source attachSlot/attachIndex wording discrepancy retained; no server lifecycle, corrected UnitName tuple, full event payload or native execution |

Every substantive raw row, inventory/prose/signature/link row has **UNPROVEN** status and no capabilities. Overlapping ledgers are source views, never additive runtime coverage. Linked pages and navigation template remain unexpanded; no extra source fetched or attributed.

## Provenance and parser decision

Before copying any source, validated [manifest](../../../data/patch-api/evidence/2.3.0-session-2026-10-09/frozen-manifest.json) row, [registry](../../../data/patch-api/evidence/2.3.0-session-2026-10-09/frozen-registry.json) hash, 101-page count/1.0.0 endpoint, response page/revision/timestamp and exact returned body. Body **12134 bytes**, SHA-256 `2d87459c71de044511f67565c26c981385f01477d8a4195eca593cf20b4cf283`; response **12776 bytes**, SHA-256 `7052cd94b09d2c1a2fca15d16973b2962f27218a5170c2759069bc38cd4c9dc8`. Registry SHA-256 `e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c`.

Existing shared parsers target different labels/headings and cannot fully retain this mixed page. Page-owned `accounting.py` is explicit opt-in by invoking that script; no shared parser/extractor flags or behavior changed. Both historical tools are exact base copies. Text uses retained `extract_text(..., canonical_patch_navigation=True)`; raw ledger is authoritative, including angle-bracket command parameters that stock rendered text strips as markup. No default-byte change or text-based reconstruction.

## Development proof and successor limits

Owned fixture RED: empty accounting failed five assertions. Further targeted RED caught incidental `/dismount` promotion and missing two NOTE identities; corrections are page-owned. Initial pre-correction 5/5 GREEN is developmental, not final proof. Final owned fixtures cover mixed labels, exact prefixes/options, literal raw/header/prose/link rows, **593 individual row omissions**, invented status/capabilities/profile/runtime/native/model/foreign-register credit and ordinary parenthetical words not being APIs. Targeted **6/6 GREEN** only; [source proof ledger](../../../data/patch-api/evidence/2.3.0-session-2026-10-09/source-proof.json) retains commands, exact tested working-scope hashes, base revision and logs. Those bytes equal source commit `766a7343c05f90692ce66be5ab996f08ba7d749f`; [capture receipt](../../../data/patch-api/evidence/2.3.0-session-2026-10-09/capture.json) binds that revision. Tests ran on recorded uncommitted scope before this commit, not as a fresh post-commit execution. RED logs are retained; initial stub code was not archived for historical RED replay. No broad/final gate claim.

Pinned actual retail successors in the frozen registry are **2.4.0, 2.4.2, 3.0.2, 3.0.3, 3.0.8**. Queue identities only; `later_registers=[]`, no overlap/retirement/native claim. Main owns successor integration and native/final gates. No current-runtime investigation is claimed; no backing-state contract was measured, no runtime proposal or edit made.

## Historical replay status

Owned validator derives accounting and verifies exact serialized register/ledger/text using only frozen own inputs. [Portable receipt](../../../data/patch-api/evidence/2.3.0-session-2026-10-09/portable-proof.json) records original and copied replay **exit 0** at `ffda99b11db3d971f58f6a31eb77a1afbc21b04b`. Copied root has no `.git`, `target` or source cache, process PATH `/nonexistent`; absolute Python executable and every command's own root cwd recorded. Both summaries match; **22 historical seals** cover 677816 bytes. Serialized native-credit ledger tamper and fabricated GREEN-log receipt each **exit 1 at the exact artifact seal**, followed by exact byte restoration and SHA-256 equality. All copied member hashes equal trusted original mapping.

[Seal mapping](../../../data/patch-api/evidence/2.3.0-session-2026-10-09/seals.json) SHA-256 `d6a4749886f3f72ce4086050b6ee67e03283867b10afdbeefab481336441f63b`; ephemeral tar **727040 bytes**, SHA-256 `4924f3e02192113692414fecc6affce0a13163bda545ac0cfa81cf5f57775acf`. Mapping plus copied member hashes allow reconstruction; ephemeral tar is not committed. [Control log](../../../data/patch-api/evidence/2.3.0-session-2026-10-09/portable-controls.log) SHA-256 `54c5b962cd127308720cd2dcaff4f1d81ba015bc2a8a10e1a51f652593c6b930`. Separate receipt seals bind mapping/log/receipt without changing 22-input historical seals. Later documentation/receipt additions do not invalidate tested historical bytes. No self-authenticating-seal trust claim: trusted recorded hashes bind historical bytes, not arbitrary replacement of validator plus seal map.

Proof is SOURCE serialization only: no runtime/publication/native/model replay, broad/check/lint/type/readability/coverage/startup/final gate, delegation or operations. `PLAN.md` checkboxes are session state, left uncommitted under PLAN policy.

## Sources

- [Raw wikitext](../../../data/patch-api/sources/2.3.0-api-changes.wikitext), [rendered mirror](../../../data/patch-api/sources/2.3.0-api-changes.txt), [occurrence register](../../../data/patch-api/sources/2.3.0-wikitext-register.json).
- [Owned evidence](../../../data/patch-api/evidence/2.3.0-session-2026-10-09/), [tracked spec](../../specs/patch-2-3-0-source-accounting.md).

## See Also

- [[patch-3-1-0-api-audit]] — original-retail literal fragment/source-accounting template.
- [[patch-3-4-0-api-audit]] — frozen registry/seal/replay template; separate Classic history, not a successor.
