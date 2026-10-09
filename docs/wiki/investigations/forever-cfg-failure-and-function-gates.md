# Forever cfg failure and function gates

## Exact failed repair boundary

[Original independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-cfg-independent-report.md) verifies canonical `0b5c69d6f`, copied as `811a0491b` in the source worktree. Enabling the two module declarations removes E0433 but reveals **RED E0609**: `addon_messages.rs` references Retail-only `bnet_custom_message`; `c_combat_log.rs` references Retail-only `combat_log_restricted`. Exact Forever source-model invocation exits 101; zero tests executed. Module availability does not establish function-body/backing-field availability.

| Gate | Retained proof at failed repair | Remaining boundary |
|---|---|---|
| Forever source model / UnitName | Build RED E0609; zero assertions | Configured-name behavior unproven |
| Forever addon senders | Source assertions inspected only | Success/rejection, ordered logs and no inbound echo unproven |
| Removed combat-log getter | Source assertions inspected only | Public/raw/legacy absence unproven at runtime |
| Retail-only restriction function | Registration remains Retail-only, static evidence | No runtime Forever non-exposure proof |
| Retail regression | Focused `--lib` check exit 0; aggregate test E0432 disabled iced | Zero Retail runtime assertions; formatting exit 0 is separate |

The report preserves exact argv, cwd, revisions, times, compiler warnings and failure sites. [Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-cfg-independent-retention.json) binds byte-identical copies of report, full streams, receipts, equivalence and proof-scope hashes. Post-command hashes remain post-command evidence, not invented pre-command snapshots.

## Follow-up: bounded compile/state PASS

Canonical `1044215d0`, copied as `2218c0f41` in the source worktree, gates Retail-only BNet bodies/constants/import and `register_restriction` to their existing Retail callers and state. No state or registrations enabled. [Independent follow-up](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-bodies/forever-bodies-independent-report.md) records exact Forever source-model command exit 0, **1/1**: before test-dispatched login, `UnitName('player')` reads Ada then Grace while a separate environment retains Linus. Both E0609 errors gone. Fresh formatting and changed-attribute readability pass; prior Retail library check reused because every added predicate evaluates true there and enabled bodies are unchanged.

[Full retained receipts](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-bodies/main-retention.json) keep this GREEN epoch separate from E0433/E0609 RED. Seven library, one binary and six vendor warnings remain unsuppressed; newly observed Forever warnings do not establish regressions. Chat/removal API execution, real pre-login lifecycle, native tuple/security/identity and loaded UI remain UNPROVEN. Aggregate-only API tests cannot supply headless proof: disabled iced blocks their harness, and the sender helper also requires Retail BNet. No broad acceptance.

## Standalone execution target

`patch-tests/forever_runtime_contracts.rs` is registered as the Forever-only `forever_runtime_contracts` target. It constructs direct `WowLuaEnv` environments without GUI/BNet dependencies and checks ordered addon/logged/whisper records, independent-environment isolation, invalid-prefix no-append behavior, and the three explicitly removed public/raw/legacy combat-log getter lookups. These assert existing simulator contracts, not native/security/transport behavior. No runtime implementation or Retail restriction registration changed. At this development epoch, target execution and independent verification receipts were pending; the earlier proof epochs above remain unchanged. Fresh standalone proof follows separately.

## Independent standalone proof epoch

[Retained independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/report.md): standalone **3/3**, actual execution at `045e396b0c6f717c2d962b97cdf46b736a3328ce`, compiled scope equivalent to requested `f611a6752a14b4d660902417c59e14073de9ac31`. [Equivalence and hash scope](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/equivalence.json) and [runtime receipt](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/runtime-test.receipt.json) retain exact revision, argv, CWD, environment, timestamps and unchanged pre/post inputs. Later SOURCE merges are not latest prefork-runtime acceptance.

| Gate | Fresh bounded proof | Remaining boundary |
|---|---|---|
| Ordered records / environment isolation | Three success results 0; exact ordered `(addon, ACE, one, PARTY, empty)`, `(addon_logged, BUG, plain text, PARTY, empty)`, `(addon, ACE, three, WHISPER, Bob-Realm)`; second direct environment log empty | One-way isolation observation, not bidirectional mutation or delivery |
| Invalid prefixes | Both senders return 1; accepted control record retained; one shared final whole-log snapshot equals pre-call snapshot | No intermediate per-call snapshot; unchanged shared helper statically appends only on SUCCESS, supporting no append for either rejection |
| Specific removed getter | Raw `C_CombatLog.GetCurrentEventInfo`, public member lookup, raw legacy global `CombatLogGetCurrentEventInfo` each nil | No generic namespace absence or Retail restriction runtime inference |
| Formatting / readability | `cargo fmt --check` exit 0; manual full changed-Rust readability audit passes | Automated metric tool unavailable; no metric values claimed |

**14 warning diagnostics retained:** six vendor manifest deprecations, seven library warnings, one binary warning; no suppression or warning-free claim. [Full runtime stderr](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/runtime-test.stderr) preserves sites. Prior Retail library check is reused type evidence only, not fresh runtime restriction proof. No native parity, security/taint, transport, inbound delivery/no-echo, real login, loaded Blizzard UI, BNet or broad acceptance credit. No prefork migration-pass claim.

[Separate byte-hash retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-runtime/main-retention.json) covers 14 original files / 933,071 bytes, all byte-identical, including original retention seal, report, receipts, comparisons and full streams. Original seals and historical RED/unproven epochs are not rewritten.

## Sources

- Independent failure, follow-up and standalone reports and byte-hash manifests linked above — distinct bounded epochs.

## See also

- [Narrow repairs](narrow-validator-discovery-and-forever-cfg-repairs.md).
- [Integrated source/factory proof](integrated-source-and-factory-proof-2026-10-09.md).
