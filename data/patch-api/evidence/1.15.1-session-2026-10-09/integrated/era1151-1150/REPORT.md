# Independent Era 1.15.1 / 1.15.0 verification — 2026-10-09

PASS within SOURCE/portable and retained headless-alias scope; NOT native/full-UI/final acceptance.

Canonical cwd throughout direct Pyrun commands: `/home/osso/Projects/wow/wow-ui-sim`. HEAD stayed `17cc9b0f20869d34b5445adf87a89a27afd565be`. 1.15.1 artifacts/test match merge `9717f867db5bfa3381d2c194eff6b3957649472b` byte-for-byte by Git diff. No source edits, delegation, Cargo builds, network, push, deploy, broad checks, shared-parser tests or global duplicate scans.

## Exact coverage

| Scope | 1.15.1 | 1.15.0 | Proof |
|---|---|---|---|
| Own SOURCE suite | 8/8, exit 0 | 10/10, exit 0 | Fresh raw stdout/stderr JSON receipts |
| Original portable suite | 3/3, exit 0 | 3/3, exit 0 | Fresh portable receipts; copied SOURCE8/10 and default-register byte replay |
| Independently executed canonical-cwd portable controls | 3/3 | 3/3 | Separate no-Git/no-target/no-current-tools archives, empty PATH, absolute Python, copied SOURCE8/10, default bytes, ledger/log rejection and exact restoration |
| Original seals | 57/57 | 64/64 | SHA256 before/after checks |
| Separate receipt seals | 9/9 | 5/5 | SHA256 before/after checks |
| Runtime alias | Retained 1/1 at `71427714747ac3840d38dd5a1a88a8e905fe936d` | No runtime credit | Actual test/assertion review, unchanged inputs, original raw test result |
| Native/full addon/C_Seasons | UNPROVEN | UNPROVEN | Not executed or inferred |

Original 1.15.0 portable harness uses historical hardcoded cwd `/home/osso/.worktrees/wow-ui-sim-p1150-page`; its 3/3 passes do not alone prove removal of that cwd dependency. Independent controls here execute identical archived programs with explicit canonical cwd and empty PATH, and all three pass. Original immutable harness/evidence left untouched.

## Independently derived SOURCE accounting

Derived from raw files, not receipt count assertions alone; concrete rows/link identities and hashes in `independent-source-derivation.json`.

1.15.1: page577687/revision5998991, 835 bytes, 11 physical/9 nonblank rows; metadata4, substantive5 at lines4,5,6,10,11. Two headings, unexpanded navigation, three prose claims and four unexpanded links = seven contracts. Named enum addition and deprecated alias continuity carry no numeric value/signature. Unspecified Dragonflight10.2.5 subset stays separate from same-Era context. Frozen ledger remains zero runtime/model/native; later bounded headless execution does not rewrite history.

1.15.0: page564510/revision5950848, 915 bytes, 12 physical/10 nonblank rows; metadata4, substantive6 at lines4,5,6,7,11,12. Two headings, one unexpanded navigation, four prose claims and seven links = eleven contracts. Literal ALL Wrath3.4.3 inclusion is distinct from unspecified Dragonflight10.1.7-through10.2.0 SUBSET. C_Engraving is one added namespace, not callable inventory; functions/arguments/returns/rune transitions/security remain unspecified. No runtime/model/native credit.

SOURCE tests assert concrete lists, counts, identities, unknown-contract limits, omission rejection and fabricated-credit rejection. They do not execute a simulator or pretend unknown values are runtime equality. Runtime test independently asserts typed concrete numeric tuples, preventing nil==nil.

## Retained headless runtime proof reuse

Original argv:

```
cargo test --no-default-features --features client-era --test patch_1_15_1_alias official_public_build_deprecation_preserves_numeric_season_alias -- --exact --nocapture
```

Original cwd/target: `/home/osso/.worktrees/wow-ui-sim-p1151-page` / its `target`. Original log: `running 1 test`; named test `... ok`; `1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`, 0.10s, exit0. Original receipt/log copied to report scope without modifying originals.

Actual compiled test `patch-tests/patch_1_15_1_alias.rs:12-49` initializes real WowLuaEnv. Before unchanged official Lua: IsPublicBuild=true, raw SeasonOfDiscovery type=number/value2, Placeholder absent. After direct unchanged pinned Deprecated_1_15_1.lua: both raw names numeric2, equal=true. No mock, flag injection or pre-test enum write. Lua SHA256 `aa34e2aa7b244efde499f8eb4767cb3b29bc7cbb4a583947c90996256ce512b3`; retained pin Gethe commit `967711a33ee9d3db2e7262e0bc0b49f4ef5a0013`, not independently fetched over network.

Full tracked-tree comparison finds no non-doc/non-evidence changes between original runtime revision and canonical HEAD. 1,674 source/build/profile/test/fixture inputs have identical revision blob hashes with canonical SHA256 inventory in `runtime-reuse-input-hashes.json`; Cargo.toml, Cargo.lock, build.rs, .cargo, client_profile.rs, Era manifest, actual test and official Lua included. Rilua pinned revision `842e4d3ff8592af45a5a93058eddf93419d29012` unchanged. Explicit client-era features=[]; GUI/CASC/sound and retail capability bundles disabled. Scoped working diff against HEAD empty. No missing relevant input proof; original 1/1 valid for the same narrow scope. No redundant Cargo command solely for merge milestone.

This does NOT establish native enum numeric2, pre-vendor publication timing, full Blizzard addon execution, C_Seasons realm state, non-public branch, or generalized seasonal behavior. Existing-state equality after directly executing the official Lua is the entire runtime claim.

Warnings retained: six simulator library warnings (unused PROVENANCE_SCHEMA, unused new, remove_missing_marker, ensure_known_asset_cached, unread encoding_key_hex, unused cooldown methods); one wow-sim unused SavedVariablesManager import; six vendor manifest lint-name deprecations. No warning-clean claim and no new Cargo target/build.

## Same-Era successor reconciliation

All canonical own source/ledger/spec artifacts exist for 1.15.2..1.15.9. All 16 frozen successor copies across 1.15.1 and 1.15.0 match own canonical pins/raw source/responses exactly. Own SOURCE receipt existence/integration is separate from historical `inflight`/`queued`/`integrated-canonical-not-applied` snapshots, which remain unchanged and applied=false/native_proof=false. No successor semantic closure/native credit inferred. Exact identities and statuses retained in `successor-reconciliation.json`; integration history in `successor-history.json`.

## Scoped Rust quality

Only new executable Rust here is `patch-tests/patch_1_15_1_alias.rs`; other .rs additions are immutable configured-input/test snapshots, not new runtime implementation. 1.15.0 introduces no executable Rust. `rustfmt --check --edition 2024 /home/osso/Projects/wow/wow-ui-sim/patch-tests/patch_1_15_1_alias.rs`: exit0, empty stdout/stderr. Manual full changed-file readability review: no violations; no nesting/parameter overload/suppression, concrete shared-state precondition→official execution→postcondition. Details/hash in `scoped-readability.json`.

## Boundary and artifacts

During final status another operation staged 1.14.4 evidence and produced conflicts in docs/wiki/index.md and docs/wiki/log.md. Those unrelated shared-checkout changes were not touched; HEAD still17cc9b0f2 and scoped runtime/evidence diff empty. Initial untracked .code-index.db also not touched.

Raw commands include argv, explicit cwd, revision, time, non-secret execution-relevant env/overrides, stdout/stderr/exit. Python direct suites: `1.15.1-source.json`, `1.15.0-source.json`; original portable: corresponding `*-portable.json`/`*-portable-receipts.json`; independent canonical-cwd subprocesses: `own-portable-raw-streams.json`. Original and final SHA256 reports: `*-seals.json`, `final-seal-recheck.json`. Git tree inventories/diffs and full retained runtime log included. Disposable copied archives remain only under this /tmp report directory.

Main owns final/native/full integration gates. This report verifies the bounded requested evidence, not completion of those gates.
