# Dungeon score rarity

The simulator selects a colour from the pinned profile cache with the native 70688 first-match range scan. The [contract](../../specs/dungeon-score-rarity.md) identifies the decoded PlayerCondition gates and simulator defaults.

## State and evidence

`SimState.passed_time_events` is a per-environment set. The initial `{121, 248}` snapshot is a best guess, not evidence of live server state. Condition 96877 reads membership of 248; 96879 requires 121 and the absence of 248. The client snapshot producer is unknown. Missing cache data is an error and must not be replaced by built-in rows.

The Retail GroupFinder Mainline TOC selects `Mainline/LFGList.lua`. Its original search-result tooltip renders supplied scores. The 1000-rating consumer test failed against the former highest-condition selection and passed after the scan revision. Direct tests also exercise the supplied event snapshots and range boundaries. This evidence does not establish PTR runtime parity or native snapshot contents.

## Sources

- [Dungeon score rarity contract](../../specs/dungeon-score-rarity.md) — native basis, cache and known gaps.
- `tests/dungeon_score_rarity.rs` — original-consumer and supplied-state tests.

## See Also

- [[consumer-selection-and-red-proof]] — selection and RED proof requirements.
