# Retail Patch 5.4.7 API audit

Warcraft Wiki pageid **549368**, current revision **5298169** (2014-05-31), refetched October 8, 2026. Substantive API page, neither redirect nor stub. The API page states diff build **5.4.7.17956**; it does **not** state an interface number. Its linked patch page (pageid 491086, revision 6854922) states **TOC 50400**, release **February 18, 2014**. This is retail Mists, not the separate 2025–2026 Classic 5.5.x line.

## Coverage matrix

| Source scope | Accounting | Proof boundary |
|---|---|---|
| Three function additions | All three publication observations OK after retail supersession | BNGetFriendInviteInfoByAddon absent after 6.0.2; BNSendGameData deprecated in 12.0.0 but unmodified cached forwarding wrapper remains; specialization function published |
| Six event additions | Four OK, two exact gaps | RegisterEvent acceptance/absence only; no source-era lifecycle or payload parity |
| CHAT_MSG_ADDON sender arg4 | Pending concrete mismatch | Legacy synthetic echo copies outbound recipient into arg4, not an inbound name-realm sender; namespace outbound intent is not network delivery |
| CHAT_MSG_WHISPER author arg2 | Pending missing producer | Outbound chat logs without an incoming whisper producer; injected consumer fixtures do not model author normalization |
| Other CHAT_MSG_* realm names | Pending source uncertainty | Author explicitly did not check other events; no universal normalization inferred |
| Diff captions and headings | Seven metadata IDs | Build/context only; no runtime credit |

Ledger: **19 unique IDs** = nine inventory + eight retained extract lines + two table captions. Statuses: seven bounded publication rows, five pending rows (two inventory, three prose), seven metadata rows. Default generator with `--legacy-api-tables --prose-api-links` and extractor with `--legacy-api-tables` suffice; no tool behavior changes.

## Meaningful backing behavior

`BNSendGameData` remains an unmodified cached Blizzard wrapper forwarding to `C_BattleNet.SendGameData`, discarding its result. The backing model records environment-local outbound intent using an online nested game-account ID. New prefork coverage tests acceptance, concrete prefix/payload/recipient preservation, void return shape, and rejection after that same account goes offline. Native transport, incoming BN_CHAT_MSG_ADDON delivery, throttling and historical 2014 limits are not claimed.

`GetSpecializationNameForSpecID` already reads the real specialization catalog. Reuse `patch_11_1_0_specialization_names_use_catalog_identity` for concrete valid/unknown IDs; do not duplicate it or confuse availability with catalog behavior. Existing namespace regression `p1200_rest_battle_net_outbound` supplies an additional backing boundary. No runtime implementation or vendor behavior changes are needed for these bounded contracts.

## Problematic cases

- `CHARACTER_UPGRADE_ABORTED` and `CHARACTER_UPGRADE_STARTED` are rejected by the strict current retail event catalog. The source-era character-boost start/abort lifecycle and service producer are not modeled. Adding names alone would manufacture publication without meaningful behavior; modern store completion callbacks are not evidence of these old transitions.
- The three chat-prose limits above remain explicit. Fixing them requires an inbound sender/author identity and delivery model, not mutating Blizzard handlers, using recipient strings as senders, or injecting arbitrary events to make a test pass.
- Registerable AUTH_CHALLENGE_FINISHED, BN_CHAT_MSG_ADDON and PRODUCT_DISTRIBUTIONS_UPDATED receive publication-only credit. No auth, remote delivery or purchase lifecycle parity follows.

## Retirements and integration ordering

**No removals on this page; no retirement candidates or runtime retirements.** Existing BNGetFriendInviteInfoByAddon and CHARACTER_UPGRADE_COMPLETE absences follow pinned 6.0.2 removals; no new deletion. BNSendGameData's cached fallback remains untouched.

Own sweep starts with one-line **5.4.8**, then **6.0.1** placeholders, followed by 6.0.2, 6.1.0 and the remaining retail registers. No 5.5.x Classic registers. Queued branch inventories and overlapping function entries are recorded using `git ls-tree` at pinned master/p548/p601 revisions; queued files are read from Git, not modified or used as live inputs.

Discovery/backing/caller scans use untruncated whole-word **`/usr/bin/grep -RnwE`** outputs saved in the session directory. Bare symbols include callback uses (`pcall(Name, ...)`) and guards (`and Name then`). These are discovery evidence, not clearance to retire anything. No retirement scan exemptions are granted.

## Verification

Initial committed discovery finds exactly two publication gaps. All five Python fixture scripts pass. **54 registers and 51 saved extracts reproduce**; inherited failures for 12.0.5, 12.0.7 and 12.1.0 remain exactly unchanged. Every prior source blob and extraction-mode result is preserved against pinned master **080f26f1c**.

At source/test scope `7ba500c07`, all publication sweeps plus classifier factory pass **55/55**; own sweep/cached sender **2/2**, cached Battle.net consumer cases **3/3**, existing specialization and namespace backing cases **1/1 each**. Python fixtures pass **84/84**. Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` and `cargo fmt --all --check` pass, with zero non-vendor warnings (seven unchanged iced vendor-manifest warnings). Negative control changes exactly one observation and increases gaps **2 → 3**. Local validator passes and rejects own-log tampering; exact bytes restored. Clean/future-audit gate runs at sealed validator revision `517ca44b1`. No full integration suite. No shared/runtime path changed, so an addons-enabled startup comparison is not triggered; this audit makes no fresh startup-output claim.

All shared validation inputs are historical Git blobs; register/sweep sets use the pinned `historical_registers`/`historical_sweep_tests` helpers, queued inventories use `git ls-tree`, and counts come from retained files. Immutable seals cover only this session; no ignored or uncommitted input, live shared comparison, or checkout-path equality. [Command ledger](../../../data/patch-api/evidence/5.4.7-session-2026-10-08/p547-command-ledger.md) records exact commands, revisions, exits and invalidation boundaries. No `__pycache__` generated.

## Sources

- [Spec](../../specs/patch-5-4-7-publication-sweep.md).
- [Pinned API wikitext](../../../data/patch-api/sources/5.4.7-api-changes.wikitext), [provenance](../../../data/patch-api/sources/5.4.7-api-changes.provenance.json), [register](../../../data/patch-api/sources/5.4.7-wikitext-register.json), [ledger](../../../data/patch-api/sources/5.4.7-page-coverage.json).
- [Fetch, linked TOC source, scans and receipts](../../../data/patch-api/evidence/5.4.7-session-2026-10-08/).
- [Gap reasons](../../../data/patch-api/evidence/5.4.7-session-2026-10-08/p547-gap-review.json), [validator](../../../data/patch-api/evidence/5.4.7-session-2026-10-08/validate.py).

## See Also

- [[patch-6-0-2-api-audit]] — later retail supersession and reproduction template.
- [[patch-audit-validator-portability]] — pinned historical inputs and clean/future checkout gate.
