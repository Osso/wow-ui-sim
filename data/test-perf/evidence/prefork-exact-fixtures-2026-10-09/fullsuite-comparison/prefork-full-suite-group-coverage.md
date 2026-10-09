# Prefork full-suite aggregation: falsification and minimal regression boundary

## Corrected diagnosis

The 3de full-suite run did execute all four migrated cases. The saved immutable log at `/home/osso/Projects/wow/full-suite-results/3de87465828db7cc7f6f900d4b63e24f1b399825.log`, lines 14559–14572, shows the parent prefork summary (`2320 passed; 1 failed; 2321 total`), then four case `ok` lines and separate successful group summaries: chat 2/2, cast-bar 1/1, spellbook 1/1. This falsifies omission. The `2321` count is the parent libtest registry only; the three nested fixture runners print separate summaries, not merged into that count.

## Parser falsification

`tools/full_suite.py` runs `cargo test --test prefork_full_ui`; `run_step` captures stdout and stderr together as `output`, writes it to the saved log, and returns it. It stores the command's process exit and elapsed seconds separately. `PREFORK_FAILED` is `^test (\S+) \.\.\. FAILED` (multiline), and `summarize(output, pattern)` returns unique regex matches. Applied directly to the immutable full log, it returns exactly:

```text
['blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors']
```

It does not parse pass counts or child summaries. Therefore the JSON's single prefork failure is an accurate failure-name summary, while its `2321` total comes from no JSON field/parser at all: that number is only human-readable cargo output. No saved JSON/log mutation or rerun was performed.

## Minimal behavioral test

The narrow regression boundary for aggregation is a pure parser/helper test using a synthetic combined output fixture, with no Cargo invocation and no writes to saved results. Input should contain a failed parent summary, the four successful exact-case lines, and child summaries `2/2`, `1/1`, `1/1`. Assert:

1. failure extraction still yields only the parent garrison test;
2. reported parent total remains 2321;
3. fixture-child aggregate is 4 passed, 0 failed, 4 total, with 3 group summaries (2, 1, 1).

This establishes separation of parent count from nested child count and protects the required aggregate. Keep test data synthetic or a copied literal excerpt; never rewrite the saved result. The current parser has no aggregate field, so this test would require a small, separately authorized parser/report change. No such change was made here.

No tests/builds/reruns, code/vendor edits, or operations performed.
