# Patch 4.3.0 development proof ledger

Only owned-worktree targeted development tests and formatting. Coordinator owns final acceptance. Counts derive from saved data; prose totals below describe recorded runs, not later integration.

| Command / input | Exact scope | Result | Later invalidation |
|---|---|---|---|
| Default register generator; extractor `--patch 4.3.0 --legacy-api-tables --text-only` | Source revision 1639407, SHA in source pin; register committed 2ffd97c5e | 76 added, seven removed; one context row | None; own Python source-reproduction fixture passes |
| `cargo fmt` | Own newly added Rust sweep before 2ffd97c5e | Exit 0 | No subsequent Rust changes |
| `cargo test --test prefork_full_ui -- patch_4_3_0 --nocapture` | 2ffd97c5e; initial empty gap fixture; discovery.proof.json | Expected RED, exit 1; 25 new gap IDs | Known-gap fixture updated at df53a1b9e; discovery remains historical |
| Same filter, normal inputs | df53a1b9e; reviewed.proof.json | GREEN, exit 0; 1/1 exact-gap case; all 83 observations saved | No later runtime/register/sweep/gap-fixture edits |
| Same filter, scratch register (`P430_SWEEP_REGISTER`) | eff4fbe2e; negative.proof.json | Expected RED, exit 1; missing-publication injection changes gaps 25 → 26 | Normal inputs unchanged; no normal rerun needed |
| Four accounting-negative fixtures | df53a1b9e + uncommitted validator stub/test fixtures; accounting-red.log | Expected RED: missing validation raises no assertion | Stub replaced before eff4fbe2e; not a committed-code acceptance claim |
| `python3 tools/test_patch_4_3_0_accounting.py` | eff4fbe2e; accounting-green.proof.json | GREEN, 8/8; dynamic counts/resolution, missing row, duplicate ID, native-credit rejection, source reproduction, log tamper | Later pin-only addition does not alter these functions |
| Historical tree-entry tamper fixture | eff4fbe2e + uncommitted tree-pin stub/test; tree-pin-red.log | Expected RED: altered tree identity not rejected by stub | Stub replaced in next coherent commit |

Original Cargo build emitted six existing `iced-wgpu-patched` manifest deprecation warnings and its summary. No warning suppression or vendor edits. No claim of check/lint/type/readability/coverage, broad publication/full suite, startup smoke, native behavior, CI-green, or final validation gate.
