# Independent bounded 3.1.0 integration verification — 2026-10-09

## Verdict
PASS for assigned bounded source/publication/radians checks at `614402d56f726ec34cedea2f28a9609cdd653462`. This is not native parity, all-publication, startup-error, full-suite, CI, or final integration acceptance.

Read/followed verify and rust-readability skills; read wiki index before audit/spec. Audited changes since `f95eed96e`. No repository edits, commits, network, operational mutations, delegation, Bash, broad tests or full-suite submission.

Parent explicitly moved command execution to stable `/home/osso/.worktrees/wow-ui-sim-p310-source`, same starting revision as canonical. Every native gate stayed on that revision, with empty tracked-input diff at start/end and `scope_unchanged=true`. Later lightweight accounting fixture ran separately on canonical at `6a84c2b09e38d37e4822bb0b27c414d9219b6efb`.

## Fresh command proof

All invoked through Pyrun argv helpers with explicit cwd and `/tmp/p434-acceptance-runner.py`. Full stdout/stderr, argv, env overrides, timestamps and exact start/end revisions retained in `/tmp/p310-independent-<name>-result.json`; full combined logs in corresponding `.log`; aggregate receipts `/tmp/p310-independent-command-receipts.json`. Long native commands spawned in background and inspected without rerun.

| Receipt name | Exact child argv | Result |
|---|---|---|
| fmt | `/usr/bin/cargo fmt --check` | exit 0; empty output |
| retail | `/usr/bin/cargo test --offline --test prefork_full_ui -- patch_3_1_0 --nocapture` | exit 0; `2 passed; 0 failed; 2 total` |
| source | `python3 -B tools/test_patch_3_1_0_source.py` | exit 0; `Ran 3 tests in 0.229s`, `OK` |
| validator | `python3 -B tools/test_patch_3_1_0_validator.py` | exit 0; `Ran 2 tests in 0.443s`, `OK` |
| historical | `python3 -B data/patch-api/evidence/3.1.0-session-2026-10-09/validate.py` | exit 0; exact summary below |
| default-check | `/usr/bin/cargo check --offline` | exit 0; finished dev profile in 1m 43s |
| mists-check | `/usr/bin/cargo check --offline --no-default-features --features sound,gui,casc,client-mists --tests` | exit 0; finished dev profile in 2m 26s |
| integrated-source | `python3 -B tools/test_patch_3_1_0_source.py` | canonical `6a84c2b09`; exit 0; `Ran 3 tests in 0.580s`, `OK`; unchanged start/end scope |

Retail environment override exactly `P310_SWEEP_OUT=/tmp/p310-integrated-results.json`; all other overrides `{}`. Native logs each contain six inherited iced manifest lint-key deprecation warnings and their one aggregate summary; zero other warnings. Full outputs read, not just exit codes.

The second source invocation was explicitly requested after a relevant fixture/accounting change, not a redundant rerun. It reproduces original historical snapshots, immutable implementer-current snapshots, and integrated observations/live ledger independently. Native Cargo commands were not repeated. Canonical includes unrelated Cargo/test additions after `614402d56`; native evidence is scoped to the stable revision, not asserted as whole-canonical verification.

## Actual observation and successor boundaries

Fresh `/tmp/p310-integrated-results.json`: **110 observations, 52 matches, 58 gaps**. Both GetPlayerFacing occurrences observe `raw=function; lookup=function`. Exact delta against sealed implementer-current observations: **one row**, retained in `/tmp/p310-independent-observation-delta.json`.

- `wt-global-api-IsPlayerResolutionAvailable-135`: actual 4.0.1 removal `wt-global-api-IsPlayerResolutionAvailable-67` changes expectation from published to absent; actual `raw=nil; lookup=nil`, now match. Publication-only successor closure, no behavioral/model credit.
- `wt-global-api-GetInstanceLockTimeRemaining-128`: 3.2.0 row is **changed**, not removed; expectation stays published, `superseded_by=null`; actual `raw=nil; lookup=nil`, still gap. No invented closure.
- Radians case `patch_3_1_0_player_facing_reads_radians_state`: configured `0`, `FRAC_PI_2`, `PI` return exactly unchanged and exactly one Lua result. Separate publication case also passes. These are the two actual passing cases, not 110 behavioral tests.
- Canonical integrated ledger independently counted **521 IDs: 54 bounded / 140 pending / 327 metadata**; 52 publication matches plus two narrowly bounded radians fragments. This reconciles later integration with immutable older 51/59 observations.

## Immutable evidence and default bytes

`/tmp/p310-independent-artifact-comparisons.json` records concrete hash/size comparisons: original **36/36 seals unchanged**, current **21/21 seals unchanged**, and **151/151 prior register/extract output hashes unchanged** against `unchanged-output-proof.json` (base `c17f1b4bb`). Final canonical seal recheck also finds zero mismatches. Original manifest SHA256 `a33186896972499e4f01b635994f0530f7e573566adde9849716c92d2c6371cd`; current `f8345b9ed1070995b3fe4d0ef8faa7747d23be74908e569ee62ad7ae8b60f6c4`.

Byte identity is not broad execution/reproduction proof. Existing source fixture freshly demonstrates default → opt-in → default exact byte restoration. Historical replay independently checks original generator default behavior and exact original register/extract reconstruction. Current tools contain unrelated opt-in successor parsers absent from the frozen archive; extractor source is not byte-identical to that archive (`extractor_unchanged=false` in comparisons), while all 151 retained output hashes match. No broad generator reruns made.

Historical validator exact stdout:

```json
{"archived_files":128,"extract_rows":189,"ledger_rows":521,"ledger_statuses":{"audit-pending":145,"bounded-coverage":49,"metadata-only":327},"modeled_closures":0,"negative_gaps":62,"pending_successors":["3.2.0","3.3.0","3.3.3","3.3.5","4.0.1"],"pinned_revisions":5,"publication_gaps":61,"publication_matches":49,"publication_rows":110,"raw_nonblank":189,"retained_commands":7,"sealed_files":36,"signature_fragments":33}
```

Validator fixtures freshly pass relocation without Git/target, synthetic future/current isolation, and six exact input tamper/rejection/restoration cases on disposable copies. Original 49/61 observations and negative 62 remain unchanged; current negative 60 is retained historical implementer evidence, not a fresh integrated negative-control run.

## Runtime wiring, nullable/radians/profile review

- `src/lua_api/globals/real/player_facing.rs:13-17`: getter reads `player.facing`, maps `None` to one `Val::Nil`, `Some(f64)` to one `Val::Num`, returns `Ok(1)`. No normalization, clock, fallback or model-widget coupling. Nullable contract established by source inspection; assigned fresh retail model case exercises configured numbers, not nil/native defaults.
- `src/lua_api/state_types/character_world.rs:228-230,268-269`: existing `PlayerState` derives Default and nullable scalar becomes shared across profiles; initial None is simulator policy. This does not create a separate orientation type or input producer.
- `src/lua_api/globals/real/mod.rs:56` and `globals/register.rs:192-193`: module and real registration compiled for retail/WowForever only. Classic profiles do not gain this real registration. Mists `--tests` compilation passes; no Mists runtime/native parity inferred.
- `src/lua_api/globals/admin.rs:111-115` and player_facing.rs:19-32: setter remains WowForever-only, finite-number-or-nil validation unchanged. Existing `tests/player_facing.rs` is WowForever-gated and covers nil, unwrapped values, invalid input preservation and texture OnUpdate; inspected, not freshly run.
- Current src/tests whole-word search finds getter registration, setter mutation and targeted new state case; unrelated widget facing fields remain separate. Sealed consumer evidence finds Blizzard PlayerScriptDocumentation.lua:666 name declaration and existing WowForever test callers, not a production Lua movement producer. Existing scans are explicitly historical, not current-cache completeness.
- `build.rs:56-60,116-121` discovers tests/generates prefork registry; successful actual named cases demonstrate the new test file is wired, not orphaned.

## Rust readability and artifact review

Manually audited every changed Rust line in `real/mod.rs`, `real/player_facing.rs`, `globals/register.rs`, `state_types/character_world.rs` and full new `tests/patch_3_1_0_publication_sweep.rs` against rust-readability. No in-scope readability violations identified: scalar getter/registration short, state mutation explicit, concrete constants named, feature boundaries explicit, test list below test-length bound. No new suppression, TODO, FIXME, HACK or XXX (each added-line count 0). Existing mandated public WoW getter naming retained, not treated as a renaming finding. No placeholders or runtime shims added; assigned model implementation reuses existing real state/getter.

## Remaining limits / parent ownership

No native 2009/PTR parity or security/default/coercion claim. Talent/aura/hover/item/macro contracts remain limited as audit states; 31 fragments and 51 prose rows remain unmodeled. Broader affected-caller runtime tests, other profile gates, startup lua-error gate, all-publication/full-suite/CI and final integration remain parent-owned. Requested default/Mists checks above are now complete specifically at `614402d56`. No unrelated fixes performed.
