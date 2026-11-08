# Crafted-quality boundary: bounded SOURCE/spec/manual Rust gate

Verdict: PASS for source/spec/manual-readability scope only at HEAD `8b46433a4568e26458a15907ea79636e38ff65f9`. Runtime/feature acceptance remains OPEN. No production implementation is certified.

## Revision and scope
Five read-only `git -C /home/USER/Projects/wow/wow-ui-sim` invocations: rev-parse HEAD; diff-tree --no-commit-id --name-status -r <HEAD>; ls-tree -r <HEAD> -- <two paths>; show <HEAD>:<each path>. All exit 0; exact stdout/stderr retained in proof.json from saved tool output, not rerun. HEAD equals requested revision. Commit adds ONLY docs/specs/crafted-item-quality-info.md and tests/crafted_quality_api.rs, both mode100644 tracked blobs. Current file bytes equal their committed contents. No crafted-production changes in this commit. This is not a whole-worktree cleanliness claim.

## Source coverage matrix
| Boundary | Evidence | Proof level |
|---|---|---|
| Required omitted/nil contextual rejection | tests/crafted_quality_api.rs:7–28: public pcall, omitted and explicit nil, method/itemInfo context, non-denial classification | Authored, not executed |
| Secret nil ordering and recovery | tests/crafted_quality_api.rs:30–73: same secretwrap(nil), issecretvalue, untainted required error, stamped CraftedQualityTest closure, denial, blocked tainted unwrap, preserved caller taint, outer restoration, repeated validation and recovered nil payload | Authored, not executed; later assertions may be unreachable |
| Statement execution, not mock query | env.exec at test lines10/33; src/lua_api/env.rs:123–126 forwards to exec_rilua; env_rilua.rs:23–26,37–39 loads a statement chunk and calls VM; no public-query replacement/synthetic wrapper in tests | Source wiring only |
| Test discovery | Cargo.toml:9,97–99,178–180; tests/integration.rs:1; crates/test-harness/src/lib.rs:7; build.rs:29–66,441–449,462–483 discovers top-level .rs files, does not exclude crafted_quality_api.rs | Source reachability only, not compilation/discovery execution |
| Signature and exact result | Retail AND PTR cached TradeSkillUIDocumentation.lua:333–347 and TradeSkillUITypesDocumentation.lua:193–212; spec lines3,13,44 | Local declarations only |

TWO authored Rust tests; FIVE planned public method calls (two ordinary, three secret-nil). ZERO tests executed here, ZERO observed feature RED/GREEN. These are authored call counts, not runtime counts.

## VM, guard and cleanup grounding
Cargo.toml:26 pins rilua revision 842e4d3ff8592af45a5a93058eddf93419d29012. Existing local checkout /home/USER/.cargo/git/checkouts/rilua-fd5a0715e46b5888/842e4d3/src/table_security.rs:216–223 wraps the original value in VM secret userdata; :401–419 unwraps after can_access_secrets, denying with `secret access requires an untainted caller without revoked access`. Test fragment `requires an untainted caller` is a literal substring of that existing guard. No native exact-error or checkout authenticity claim.

Simulator src/lua_api/globals/security/secret_values.rs:30–55 registers host VM wrappers and calls wrap_secret/unwrap_secret. env_init/mod.rs:72–73 installs these for retail-12-1-0; Cargo.toml:123–124,153–154 includes that capability in both client-retail and client-ptr. Rilua stdlib/taint.rs:148–159 stamps the actual closure; :256–273 walks active call frames for getstacktaint; :302–306 enables taint mode. Tests assert preserved tainted caller after both protected failures and restoration of outer untainted access. Fresh independent WowLuaEnv instances isolate the cases. Taint cleanup is asserted, NOT observed passing; no runtime cleanup guarantee follows from this inspection.

Existing src/c_api/c_trade_skill_quality.rs:31–35 registers recipe queries, not this crafted query; runtime_surface_bootstrap.lua:64–78 may manufacture nil-returning compatibility callables. Public callability/nil misses are not modeled feature proof. An anticipated required-error assertion against that callable is source reasoning, NOT an observed RED.

## Spec boundary
All thirteen declared non-nil fields match exactly: quality:number; icon, iconSmall, iconInventory, iconMixed, iconAppear, iconDissolve, barFill, barBackground, barBackgroundCap, barHighlight, iconChat, iconQuestObjective:textureAtlas. Result info remains nullable. Required itemInfo:ItemInfo and AllowedWhenUntainted preserved. Method/selector diagnostics are simulator policy; authentication-before-validation is identified as interpretation, not native observation. Numeric IDs, numeric strings, item strings, hyperlinks and valid secret selector grammar remain UNKNOWN. Positive records, lifecycle/snapshots/isolation and whole original consumer remain planned/unexecuted; no recipe/catalog inference or production authorization added.

## Manual Rust readability
Read every changed Rust line under rust-readability skill, including embedded Lua. No specific readability violations found. No fabricated cognitive/cyclomatic scores, similarity percentages or automated metrics; no analyzer/formatter/check invoked. Focused behavioral assertions are not source-shape tests. No TODO/FIXME/HACK/XXX, empty catch, or commented-out implementation found in the authored changes. This does not certify compile/format/lint success.

## Admission and historical failures
Retail-first compile admission DENIED / ZERO workload launches is supplied boundary status, not a newly executed admission check. No build/test/format/check, binary/library/archive workload, source-tree authentication, vendor-cache write, service action, Bash, cwd tool or delegation occurred. Only the five explicitly allowed git reads ran as repo commands. PTR admission and behavioral execution remain unobserved.

Original handoff source/actor failures and historical proof remain historical: not erased, repaired, reclassified or promoted to passing by this gate. Original report at baseline 402ab5a88aa90a6a5fd877f8c906964d29a1cecc and audit's then-absent-target statements are not current HEAD assertions. Corrected-stage report retains unexecuted status, former stale helpers.rs unwrap location correction, and no source-authenticity claim. All existing files in both named handoff packets and original audit were hashed and checked unchanged during artifact creation.

Inspection-only tool incidents: find could not run because fd is unavailable; a follow-up Pyrun read of prior evaluation locals failed with name 'ledger' is not defined. Neither is feature RED or a build/test result. Original saved command output was read instead; git commands were not repeated.

## Artifact integrity
New report.md and proof.json exclusively created at requested verification path, mode0600, directory0700; file and directory fsync, full-byte readback and mode checks completed. Integrity evidence is not behavior/native acceptance. Exact hashes and command outputs reside in proof.json/tool readback. Only these two requested artifacts were written by this gate.
