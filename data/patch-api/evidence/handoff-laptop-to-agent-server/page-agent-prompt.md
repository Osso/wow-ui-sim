# Page-agent prompt template

Replace `<PATCH>` (e.g. 8.2.0), `<P>` (e.g. 820), `<NEWER>` (the next newer page, merged or
in flight) and `<NEWER_STATE>`.

---

Repo: ~/Projects/wow/wow-ui-sim (WoW UI simulator, Rust, rilua). Read CLAUDE.md and
docs/wiki/index.md first and follow CLAUDE.md strictly (never edit Interface/AddOns/Wowless*,
Blizzard UI cache, vendor files; no monkey-patching Blizzard Lua; C_* code in src/c_api/;
placeholders only under src/lua_api/workarounds/{temporary,permanent}).

Task: audit the Warcraft Wiki "Patch <PATCH>/API changes" page exactly the way <NEWER> was
audited. (If no such page, report it and stop.) Worktree from master:
`git -C ~/Projects/wow/wow-ui-sim worktree add ~/.worktrees/wow-ui-sim-p<P>-page -b p<P>-page master`.
Templates: docs/wiki/investigations/patch-<NEWER>-api-audit.md; data/patch-api/sources/<NEWER>-*
(note provenance generator_flags/extractor_flags); evidence dirs under
data/patch-api/evidence/; tests/patch_<NEWER>_publication_sweep.rs (sweeps are
`prefork_full_ui_case!` items run by `cargo test --test prefork_full_ui -- <filter>`);
tools/gen_patch_wikitext_register.py (+ tests); tools/extract_patch_non_inventory.py
(+ tests); tests/common/publication_sweep.rs. All later registers on master supersede.
<NEWER_STATE: if <NEWER> is not on master yet, put a one-line placeholder at the start of
your later-register list for the <NEWER> register (main thread adds it at integration) and
report possible <NEWER> supersessions.>

Validator guidance: derive gap counts, status totals, register sets, results files and sweep
summaries from fixture/results/sources files at validation time — never hard-code values or
receipts that go stale when another page merges.

CRITICAL scope: EVERY command must explicitly run with cwd ~/.worktrees/wow-ui-sim-p<P>-page,
using that worktree's own target dir (no CARGO_TARGET_DIR to siblings; never write relative
output paths that could resolve elsewhere). Never modify the canonical checkout or other
worktrees. Verify the branch is p<P>-page before the first commit. No push, no merge, no
spawning agents/CLIs. Commit each coherent step.

Rules: prioritize complete publication accounting; fix only cheap, clearly meaningful gaps
with real modeled behavior; record precise reasons for the rest. Retail retirements must be
gated so classic profiles (mists/wrath/era/anniversary) are unaffected — older APIs often
still exist in classic clients; add any list to src/c_api/patch_retired_members.rs as a
separate const and update its header patch list. Keep shared-file edits small and additive;
new generator/extractor behavior must be behind a new opt-in flag (check existing flags
first). Keep all saved extracts reproducible with their recorded extractor flags
(12.0.5/12.0.7/12.1.0 were already non-reproducible) and all registers byte-identical with
recorded flags. Before retiring anything listed as removed, grep current cached retail
Blizzard Lua (~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns) with namespace-qualified
whole-word AND bare-name searches; if current retail Blizzard code uses it, do NOT retire it —
record a retained gap with file:line. Retirements must not delete Blizzard deprecation
wrappers. When you retire or change any global/method: rg the WHOLE tests/ and src/ trees
(no truncation; Lua strings in Rust, `pcall(Name, ...)`, `and Name then`); migrate retail
callers, gate classic-only tests, run every affected integration and prefork test. Default
retail carries the 12.1.0 surface.

Verification (targeted only): all publication sweeps
(`cargo test --test prefork_full_ui -- publication_sweep`), new behavioral tests, tests
touching changed surfaces, negative control, `cargo fmt`, Mists
`cargo check --no-default-features --features sound,gui,casc,client-mists --tests` zero
non-vendor warnings, `wow-sim --no-addons --no-saved-vars lua-errors` returning []. Update
docs/wiki (patch-<PATCH-with-dashes>-api-audit.md, index.md, log.md).

Final report: page revision, row counts, sweep table, gaps closed/remaining, ledger counts,
verification results, commits, possible <NEWER> supersessions.
