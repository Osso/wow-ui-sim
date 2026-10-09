# Historical retail Patch 3.2.0 API audit

Pinned supplied page 499137, revision 4812266, timestamp 2020-09-04T21:33:02Z. Response SHA-256 `0b2cee11a2b724abaaf682a32b87e4501911dc30807329a2496534666ec1b6a2`; 4,948-byte wikitext SHA-256 `5a6674fd08ffd867e894196ed1abf2a599a3a25d32d5db5819387dcda0ad9f71`. Identity, timestamp, byte content and both manifest hashes validated before writing audit artifacts. No network refresh.

## Source format

Opt-in `--wrath-retail-change-bullets` retains labelled function/event/quoted-array occurrences and literal source signatures. Source spelling `GetDifficutlyColor` remains unchanged. `UnitIsPlusMob` is **changed**, not removed: source says “updated or removed” and explicitly questions removal. Undocumented functions receive changed publication expectations, not invented addition dates. Default whole-page extraction fails on `{{Reflist|2}}`. New opt-in `--numbered-reflist` preserves its column count and unexpanded-citation boundary without changing default behavior.

Queued retail successor slots are 3.3.0, 3.3.3, 3.3.5 and 4.0.1, in that order. Actual 4.1.0 onward is used now. Wrath Classic 3.4.x is a separate history, not a successor.

## Capability matrix — original development boundary

| Source/cases | Accounted | Unsupported | Proof |
|---|---|---|---|
| Labelled API/event/array inventory | 43 occurrences; 30 bounded publication/absence matches | 13 reviewed current-retail mismatches | Own targeted prefork GREEN 1/1 after empty-gap RED |
| Full-page extraction | 74 nonempty rows; 27 editorial/reference rows | 47 substantive statements, including repeated inventory qualifications | Existing extractor plus numbered-reference opt-in; no prose behavior credit |
| Call signatures | 24 literal parenthesized occurrences | Historical argument/return contracts remain unproven | Source only; per-row precise limits |
| Developer slash tools | Four literal claims: `/dump`, `/eventtrace`, `/framestack`, `/reload` | Handler/output/lifecycle behavior untested | Separate command IDs, not console-command aliases |
| Negative publication control | Same 43-row boundary | Exactly one fabricated missing identity | Required failure; gaps 13 → 14 |

[Ledger](../../../data/patch-api/sources/3.2.0-page-coverage.json): **145 IDs = 43 inventory + 74 extract + 24 signatures + 4 command claims**; **30 bounded / 88 pending / 27 metadata**. Sixty-eight actual later-retail registers are frozen separately from the four queued slots. Later removals permit current absence or unchanged loaded Blizzard deprecation aliases; neither counts as a backing model.

**No backing-model gap closed.** Missing legacy quest-map index/POI semantics, recurring-calendar sequences, instance-lock extension state, GM response workflow and Wintergrasp eligibility lack established complete contracts. Existing player XP/flying/mail/GUID/cast models receive publication-only credit; this page does not establish remote-player identity, enemy interruptibility, Dalaran subarea transitions or unloaded-mail totals. `GetAbandonedQuestName` is not silently aliased to the distinct existing `GetAbandonQuestName`.

`GetAbandonQuestItems` remains a mismatch against a later retail removal because real legacy registration survives. No removal mutation was made. Source-listed removed `GetDifficutlyColor` and `QuestDifficultyColor` are already absent; uncertain `UnitIsPlusMob` is not retired. Whole cached-retail/src/tests [scans](../../../data/patch-api/evidence/3.2.0-session-2026-10-09/retirement-scans.json) cover 4,044/1,653/2,160 files, zero matches for these three literal identities (qualified and bare searches coincide for unnamespaced symbols). No Classic, vendor, Wowless or runtime code changed.

## Original receipt replay

[Historical ledger](../../../data/patch-api/evidence/3.2.0-session-2026-10-09/historical-page-coverage.json) and [known gaps](../../../data/patch-api/evidence/3.2.0-session-2026-10-09/historical-known-gaps.json) are immutable sidecars, not current closure fixtures. [Compact archive](../../../data/patch-api/evidence/3.2.0-session-2026-10-09/historical-blobs.json.gz) retains 81 original source/parser/sweep/register blobs (285,933 bytes compressed). [Manifest](../../../data/patch-api/evidence/3.2.0-session-2026-10-09/historical-inputs.json) seals archive plus 22 external original inputs/logs. Validator derives inventory, signatures, prose, gaps, status totals, later-register sets and command summaries from these sealed inputs. It never reads Git, target or current checkout files; old 4.1.0 register/extract are bounded unchanged-output controls, not all-page reproduction credit.

[Command ledger](../../../data/patch-api/evidence/3.2.0-session-2026-10-09/command-ledger.json) records commands, revisions, scopes, exit codes and retained logs. Own source fixtures pass individually; prefork RED discovers exactly thirteen gaps, GREEN passes 1/1, negative fails with the fabricated identity only. Cargo formatting passes. Six inherited iced manifest warnings remain unsuppressed. No broad tests/check/readability/profile/startup/full-suite/final acceptance run; main owns integration and those gates. Portable fresh-process controls pass **3/3 targeted accounting fixtures** at `106630826`; [development proof](../../../data/patch-api/evidence/3.2.0-session-2026-10-09/validator-development-proof.json) seals the GREEN log, exact fixture, validator and original manifest. Clean replay uses a copied fixture with no Git/target/tools and empty PATH. Unarchived current ledger/gap replacement is ignored; serialized response, own sweep log, historical ledger and historical gap mutations fail at their exact seals, restore byte-identically and replay clean. Original archive, manifest and historical receipts are unchanged. This is owned development proof, not main's integration/final acceptance.

## Sources

- [Pinned source](../../../data/patch-api/sources/3.2.0-api-changes.wikitext).
- [Source pin](../../../data/patch-api/evidence/3.2.0-session-2026-10-09/source-pin.json).
- [Spec](../../specs/patch-3-2-0-publication-sweep.md).

## See Also

- [[patch-4-1-0-api-audit]] — first integrated later retail register.
