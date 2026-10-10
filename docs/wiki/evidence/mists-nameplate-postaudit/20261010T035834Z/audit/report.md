# Saved Mists nameplate compile + execution audit

Sanitized derived retention: identity/proof-boundary correction only. Original local evidence unchanged. `hashmanifest.json` describes original consumed evidence and original audit outputs; `retention-manifest.json` binds these retained derivatives.

**PASS, bounded:** saved compile exit 0; main's saved execution 4/4 tests pass, exit 0. Eight Modern/Classic style cases inside four tests, not eight independently enumerated tests. No native or full-profile acceptance claim.

## Identity and integrity

Epoch `/home/osso/.local/state/wow-ui-sim/verification/mists-nameplate-current-postaudit/20261010T035834Z`; submission revision `acfe6322f91cb936db2c41f263370e815dcdf37f`. Revision correction: `53863d18e` is not this submission revision. The submission records `acfe6322f91cb936db2c41f263370e815dcdf37f`; no alternate source-epoch identifier is established. Source maps match before/after: **3839 tracked path hashes**, not a whole-input seal. Relevant current fixture/resolver/Cargo files match recorded hashes; read-only git show of revision fixture also matched SHA256 `2c731f0f6629072113e6e8eac2876f81fcc3683990cf25ff083c7b92064b5a6f` (exit 0). Equality is tracked-scope compile-window evidence, not current-HEAD proof, an execution-window source snapshot or an execution-time cache seal. Relevant-file matches are historical audit-time observations, not a current checkout attestation by this retention task.

Compile argv: `["/usr/bin/cargo", "test", "--offline", "--locked", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--test", "mists_nameplate_scale", "--no-run", "--message-format=json"]`; cwd `/home/osso/Projects/wow/wow-ui-sim`; jobs 8. Default features disabled; executable features exactly `casc,client-mists,gui,rodio,sound`. Boot ID unchanged. Cargo stdout: 739 JSON records: 663 compiler artifacts (655 fresh, 8 non-fresh), 74 build-script records, 1 warning, 1 successful build-finished. Compile stderr has seven warning lines (repeated target summaries, not seven unique diagnostics). Warning: `src/lua_api/handler_timing.rs:12`, unused `is_enabled`; this is not a warning-free build.

Seven executable slots have build-end SHA256 seals matching Cargo target/path/features/profile records:

| Slot | SHA256 |
|---|---|
| `panel-visual-metrics` | `f986e9c785b9ca3a77a070306acdc439f9844024c6ba0efb4b2cdbccadd0fc9f` |
| `bench_talents` | `f206587ed8fdcd982ad38433025ea4693c77df61751df729f362dbe21e942ae6` |
| `bench_spellbook` | `39de114d4266782f9fce15b66c2dfeff3245fbe49dfd12de935b0b4b9a3351bb` |
| `mists_nameplate_scale` | `a510a1f341d43353f0881513318c4aaa12c11a8a3b788eda13f088da003580b3` |
| `wow-sim` | `9cd6abdc8f3c61a4311028af44a1f3ed8cd1a223522cb03b5ade43884de99f8b` |
| `bench_steady_state` | `94eb64a3f1dd5222557361cdf8a54c5181129c3de0d0cf5d9308de9defcef51b` |
| `wow-cli` | `d3820b8eca92813009b2e6175abf9032a9970742f472d78a521db8062804eb93` |

All seven executable records are non-fresh. Shared slot paths are not identity; seals bind this epoch. Only selected `mists_nameplate_scale` was executed by the saved invocation; other slot seals do not imply runtime acceptance. Selected path `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_nameplate_scale-7b616f0b74b8ec9e` and hash match build-end seal and execution invocation. Recorded post-execution hash unchanged; stream hashes independently recomputed from retained files and match result receipt. No new executable run or current-slot attestation performed.

## Saved execution and coverage

Main invoked `["/usr/bin/timeout", "90", "/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/mists_nameplate_scale-7b616f0b74b8ec9e", "--nocapture", "--test-threads=1"]`, `WOW_SIM_NO_SOUND=1`, `2026-10-10T04:07:41.667354+00:00` → `2026-10-10T04:07:41.979497+00:00`. Exact output:

`test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s`

| Test | Cases and boundary |
|---|---|
| `nameplate_small_scale_selects_modern_and_classic_style` | Modern/Classic Small five-component scale tuples: (.75,.8,.8,.75,1) / (.8,.8,.8,.8,1). |
| `nameplate_unsupported_size_99_uses_medium_scale_for_both_styles` | Size 99 selects source Medium fallback, five ones, both styles. |
| `nameplate_explicit_base_dimensions_override_small_scale_for_both_styles` | Explicit base (128,32) overrides Small-derived dimensions, both styles. |
| `nameplate_options_publish_small_style_scales_and_explicit_size` | Both styles publish scale/options and explicit native/script size; controlled collaborators only. |

Fourth test asserts horizontal .75/.8, vertical/classification .8, health heights 16/8, cast height 8, Classic flags false/true, border widths 0/102.4 and heights 0/12.8. Enemy/friendly class colors true/false; level/selection follows Classic; mouseover color is supplied yellow identity for Classic and nil for Modern. Native size publication count 1; native/script size (128,32). Shared ApplyFrameOptions counter 2 is aggregate dispatch evidence, **not proof of one call per unique plate or real frame state**. Instance-local `SetupClassNameplateBars` is a no-op isolation stub; removed construction-only assertion is absent. Raw vendor scale/dimension/update source remains the exercised implementation.

## Raw cache inputs and exclusions

Fixture reads/executes complete constants then NamePlates Lua through profile cache resolver, without rewriting vendor bytes. Audit-time cache identities:

| Input | Role | SHA256 |
|---|---|---|
| `Blizzard_NamePlates/Blizzard_NamePlateConstants.lua` | raw Lua read and executed | `dc032e268b108b90e3319e5f68415cc2e8a17031acad6da2e6341bfad9f9f04c` |
| `Blizzard_NamePlates/Blizzard_NamePlates.lua` | raw Lua read and executed | `4aa4a84ab2da7420698c642e2c5335070f7c5cbcb33be0395d2db8b2f54c4299` |
| `Blizzard_APIDocumentationGenerated/NamePlateConstantsDocumentation.lua` | enum grounding only; not executed by fixture | `67666eeb52447346132c0cbe1b692937d83a4a358a9544bcc75c418912379692` |

Root `/home/osso/.cache/wow-ui-sim/blizzard-ui/mists/AddOns`. Manifest entries confirmed at lines [2313], [2318], [314]; manifest SHA256 `88893b67f75cc61d5a465ece771ca194e79c193305dfae48be484c52b1a9f0cd`. These hashes match earlier source audit identities but were read **during audit**, not sealed at execution time. Runtime success establishes files loaded; no exact execution-time byte attestation is invented. Controlled enum tables, size/style CVar providers, class-color booleans, empty option destinations, yellow object identity, engine/frame callbacks, and explicit base dimensions are fixture inputs—not native defaults. Enum documentation is grounding, not loaded fixture code.

Submission exclusions: external path dependencies; uncaptured data inputs; inherited environment; runtime cache/addon files outside source scope; untracked index. No full dependency/input closure. Also unproven: native client parity/CVar registration/defaults, all other sizes/styles, OnLoad/event lifetime, dimensions without overrides, class resource bars, actual frame/render state, full Mists suite/profile. No source/backend changes or extra tests made.

## Historical failures retained

Original revision `903711da8b9d6472f718e407af350c576fe4a7e0` at `/home/osso/.local/state/wow-ui-sim/verification/mists-hidden-cast-build/20261010T030705Z`: original nameplate target **0 pass/2 fail, exit 101**; currency target **0 pass/2 fail, exit 101**. Four OS-code-2 NotFound source-reader failures (obsolete TBC nameplate/Cata TokenUI paths) precede intended downstream assertions. Raw streams privately consumed, hashes match their receipts, original files unchanged. Current four-test success does not erase these records, turn them into behavioral RED, or imply currency/full-profile closure. Older source-only report remains historical, superseded only for this exact target's compile/execution gate.

## Resource observations only

Both compile epochs used jobs 8, CPU quota `800000 100000` and memory cap `17179869184` bytes (16 GiB), unchanged before/after.

| Epoch | Revision | Submission→Cargo end | Submission→outcome end | Recorded cgroup peak |
|---|---|---|---|---|
| `/home/osso/.local/state/wow-ui-sim/verification/mists-nameplate-current/20261010T034853Z` | `be6d677fece6118b500b15d38eb6e829d6960aed` | 23.720628s | 25.026939s | 8844390400 bytes (8.236980 GiB) |
| `/home/osso/.local/state/wow-ui-sim/verification/mists-nameplate-current-postaudit/20261010T035834Z` | `acfe6322f91cb936db2c41f263370e815dcdf37f` | 44.935463s | 45.980029s | 10066337792 bytes (9.375008 GiB) |

Cgroup lifetime peaks include service activity; not isolated compiler RSS or reset-window peaks. Different source revisions/cache warmth/workload/load and timing boundaries prevent controlled speedup attribution. No jobs-12 measurement exists here. User chose **12 jobs for next work**; choice recorded only, not applied. No cap raise, benchmark rerun, build/test, backend, delegation, or operational mutation.

## Full-stream consumption and hash manifest

All retained compile/execution stdout and stderr (both compile epochs; current runtime; original four-failure receipts) read fully privately, not truncated for analysis. Safe counts:

| Stream | Bytes | Lines | SHA256 |
|---|---|---|---|
| `compile.stdout` | 570887 | 739 | `1349e63184e079e3874f60f3932d199df5649c2764171c5c6e8baccfa531a40e` |
| `compile.stderr` | 1275 | 9 | `9a54a58ca5c89428eccba480525452c1686813724eccd13f3c19fe39c0efa2cb` |
| `execution/stdout` | 415 | 9 | `a4b843141020f973d963b6407df59c994a12ddca30d498fb622529505e47674e` |
| `execution/stderr` | 2557 | 44 | `dd5e4bc13fb5314d100b0fdca4e338b39415da1b365e3cc202df3b835e0e6100` |

`aggregate.json` holds exact checks, slots, counts, inputs, exclusions and resource records. `hashmanifest.json` covers consumed evidence and generated report/aggregate; excludes itself to avoid circular hashing. Historical input bytes preserved; raw payloads not reproduced.
