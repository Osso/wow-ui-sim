# 11.2.7 non-inventory extract scout

Source: page 649551, revision 6726771, captured 2026-10-06. [Retained plaintext](../../sources/11.2.7-api-changes.txt) is rendered from retained wikitext, not an expanded MediaWiki template extract. Linked resources are not expanded. All 19 nonblank source rows assigned: ten contractual candidates stay audit-pending; nine editorial/source rows stay metadata-only. No behavioral or historical parity credit is awarded by this scout.

## Ranked bounded follow-up batches

| Batch | Exact source IDs | Producer/consumer investigation | Required observable proof |
|---|---|---|---|
| B01 chat reorganization | `prose-undated-005` | `src/lua_api/globals/chat_frame_util.rs`, temporary chat-window defaults, cached ChatFrame/ChatFrameEditBox mixins. Existing ordinary helpers are not proof that the entire migration is modeled. | Cached chat frame send/edit/open/filter transitions with concrete messages; enumerate retained old/new function identities before testing. Separate cached delegate behavior from simulator publishers. |
| B02 store/choice DTOs | `structures-AccountStoreItemInfo-017`, `structures-AccountStoreItemInfo-018`, `structures-PlayerChoiceInfo-023`, `structures-PlayerChoiceInfo-024` | Account-store item DTO in `src/lua_api/globals/missing_surface/account_store.rs`; choice DTO in `src/c_api/c_player_choice.rs` already emits `requiresSelection`. Source parent rows remain independent audit targets. | Nonempty item/choice fixtures; concrete `mode` and `requiresSelection` values, host mutations, fresh snapshots, absent-record outcomes. Existing field publication alone is insufficient. |
| B03 Battle.net class DTO | `structures-BNetGameAccountInfo-019`, `structures-BNetGameAccountInfo-020` | `src/c_api/c_battle_net.rs` game-account producer; inspect `classID` host fields and account lookup paths. | Two game accounts with different concrete classes, mutation/snapshot isolation, missing account result; exercise actual cached consumer if it selects by class. |
| B04 difficulty DTO | `structures-DifficultyInfo-021`, `structures-DifficultyInfo-022` | Locate real `DifficultyInfo` producer and cached difficulty selector; `isUserSelectable` cannot be inferred from difficulty-name existence. | At least one selectable and one unselectable populated difficulty; host transition and missing difficulty; cached selector obeys the flag. |
| B05 housing summary | `prose-undated-004` | Broad summary, not a bounded API contract. Current `src/c_api/c_housing.rs` documents local state, blueprint/market/catalog subsets; 38 `C_Housing` publication gaps plus editor/selection gaps are recorded separately. | Decompose into supported acquisition, ownership/charter/travel, selection and placement contracts before proposing behavioral credit. No blanket Housing capability or invented service state. |

## Metadata assignments

| Source ID | Classification |
|---|---|
| `source-context-001` | Patch navigation/title |
| `source-context-003` | Summary heading |
| `source-context-007` | Resources heading |
| `source-context-008` | TOC 110207, build context |
| `source-context-009` | Official patch-note link |
| `source-context-011` | External source-diff links |
| `source-context-014` | Consolidated inventory heading |
| `source-context-015` | 11.2.5 → 11.2.7 build comparison, December 16, 2025 |
| `source-context-016` | Structures heading |

## Limits

No Notes/Blue posts/Enums subsection exists in this retained revision. The separate Commands inventory and unheaded Global API table belong to the publication register, not these supplemental rows. Two housing scriptobject factories fail under the sweep fixture; supported acquisition/native model proof remains missing. Two collision-bound preference methods remain within the intentionally unsupported 3D domain. The earliest supported retail epoch is 12.0.0; this task does not reconstruct an 11.2.7 client.

## Evidence

- [Per-occurrence gap review](p1127-gap-review.json): all 135 initial non-OK rows, 14 fixed, 121 retained.
- [Publication observations](p1127-sweep-result.json): 508 rows, 387 OK; no signatures/output/security claim.
- [Coverage ledger](../../sources/11.2.7-page-coverage.json): exact inventory/extract union, development scope only.
