# Party countdown

Bounded Retail 12.0.5 `C_PartyInfo.DoCountdown` observable request lifecycle contract. The provider is registered by `src/c_api/c_party_info.rs` and implemented in its C API-owned `countdown` module. See [Lua API environment/state architecture](../lua-api.md) and [event dispatch](../event-system.md). Parent reports compiled RED at `676e4c25a` (ten FAIL); parent GREEN and independent292 bounded acceptance are recorded below.

Cached generated `Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:156–170` declares required numeric `seconds`, one required `success:boolean`, `HasRestrictions=true`, and `SecretArguments="AllowedWhenUntainted"`. Cached `WorldStateInfoDocumentation.lua:10–39` declares synchronous events:

- `START_PLAYER_COUNTDOWN(initiatedBy:WOWGUID, timeRemaining:time_t, totalTime:time_t, informChat:bool, initiatedByName:string?)`.
- `CANCEL_PLAYER_COUNTDOWN(initiatedBy:WOWGUID, informChat:bool, initiatedByName:string?)`.

Both events declare `SecretInChatMessagingLockdown=true`. Generated declarations establish signatures and synchronous delivery, not server request semantics, malformed-input behavior or native error policy. Cached `Blizzard_FrameXML/Timer.lua:188–196` consumes these exact positions; its start helper uses `GetTime() + timeSeconds` (`:137–143`), and its cancel handler passes zero time/total to that helper. The handler's internal zero is **not** proof of native zero-as-cancel API semantics.

## What it must do

### Surface and event contract

- [ ] Gate this bounded contract by `retail-12-0-5`; discover fixtures in the existing grouped integration target, without a new Cargo target.
- [ ] An accepted request returns exactly one boolean, `true`. **INFERRED success policy:** all accepted local starts, replacements and cancellations succeed; generated docs establish the boolean return, not these success conditions.
- [ ] A positive accepted request synchronously emits exactly one START event before the API returns, with exactly five payload positions in generated order. **INFERRED local payload policy:** initiator is the real modeled `UnitGUID('player')`, remaining and total both equal requested seconds, `informChat=false`, and the optional name is populated from modeled `UnitName('player')`. No hardcoded production GUID/name or fabricated group-chat notification.
- [ ] Active cancellation synchronously emits exactly one CANCEL event before return, with exactly three payload positions in generated order, modeled player GUID/name and `informChat=false`. Populating the nilable name is local policy, not a stronger generated nilability contract.

### Inferred request lifecycle and validation

- [ ] **INFERRED lifecycle:** positive seconds replace any active request with one START and no intermediate CANCEL; test increasing and decreasing replacement order. Zero cancels an active request once. Zero without an active request succeeds without an event, including repeated cancellation and cancellation before the first start.
- [ ] The environment retains enough active-request state to distinguish replacement, active cancellation and idle cancellation. Start/cancel/start and cancel/start/cancel work in call order. Mutation or cancellation in one `WowLuaEnv` cannot affect another environment's active request or listeners.
- [ ] **STRICT INFERRED validation:** accept ordinary finite numbers in the inclusive range `0..Constants.PartyCountdownConstants.MaxCountdownSeconds` (cached maximum 3600), preserving fractions. Missing/nil, negative, above-maximum, NaN/infinite, numeric strings, booleans, table, function and frame userdata reject with a nonempty runtime error, without coercion, countdown events or loss/creation of an active request. Subsequent public cancellation proves rejection atomicity without requiring internal Rust fields.
- [ ] **CONSERVATIVE SECURITY POLICY:** reject real secret positive and zero seconds in secure and tainted callers, before effects. Preserve secrecy and caller taint. This is intentionally stricter than generated `AllowedWhenUntainted`; untainted-secret acceptance is **not modeled or native-verified**.

### Restriction axis

- [ ] Reuse the [shared chat messaging lockdown guard](chat-messaging-lockdown.md) before validation or effects. **INFERRED explicit-error policy:** locked start, replacement, active cancellation and idle cancellation report the shared nonempty lockdown error. Invalid locked calls report lockdown rather than argument validation; exact native wording/convention is unknown.
- [ ] Cover all four combinations of explicit lockdown and combat. Combat alone does not block requests; rejected replacement/cancel leaves the existing request cancellable after unlock. No countdown events occur while blocked.

Final retained [`prose-2026-03-31-169`](../../data/patch-api/sources/12.0.5-register.json), source line 169, says restrictions were “loosened to only apply when in chat messaging lockdown rather than in all combat.” This supports the restriction axis, not the inferred request/security/error policies. Countdown tests alone do not establish the aggregate row decision below; they do not prove native parity.

## How it works

- [Lua API environment/state architecture](../lua-api.md)
- [Event dispatch architecture](../event-system.md)
- [Chat messaging lockdown input contract](chat-messaging-lockdown.md)

## Implementation inventory

- `tests/party_countdown.rs` — ten epoch-gated fixtures, real Lua frame observers, public request effects and actual host-secret fixtures; automatically included by the existing grouped integration harness.
- `src/c_api/c_party_info.rs` — epoch-gated namespace registration owner.
- `src/c_api/c_party_info/countdown.rs` — C API-owned `CountdownRequest` stores requested seconds and modeled initiator snapshot. Validation precedes mutation; positive replacement commits the new record, cancellation clears it, and idle cancellation does neither. GUID/name payloads are stack-rooted during existing synchronous dispatch; state is committed and the mutable borrow released before callbacks. Source ordering permits nested requests to observe committed state; dedicated reentrancy behavior is untested.
- `src/c_api/c_chat_info.rs` — existing shared lockdown error guard; unchanged.
- `src/lua_api/state/sim_state.rs` and `state.rs` — `party_countdown_request: Option<CountdownRequest>`, initialized to `None`, independently owned by each environment. Start/replacement snapshots current modeled player identity; cancel snapshots the current requester, not stale initiator metadata. No new identity constants or clock origin are introduced.

## Tests asserting this spec

`tests/party_countdown.rs` covers exact return type/arity and synchronous payloads; modeled name; fraction/max boundaries; idle cancellation; both positive replacement orders; start/cancel call orders; malformed atomicity in fresh/active states; secret start/cancel with secure/tainted callers; two-environment isolation; locked fresh operations; active-request lockdown/combat 2×2 matrix and recovery.

Parent filter: `cargo test --test integration party_countdown::`.

**Proof ledger, 2026-10-01:** parent-established compiled RED at `676e4c25a`, same successful build recorded by `/tmp/patch-12.0.5-batch34-green-fixed-build-result.json`; parent reports run3 logs/manifests show ten FAIL because no producer existed. Producer slice runs formatting only, no builds/tests/checks/readability gate or native probes. Parent owns GREEN compilation/run and independent acceptance. Existing grouped harness discovery requires no other test/Cargo edits. Requirement checkboxes reflect the original production milestone; parent GREEN and independent bounded acceptance are recorded below.

## Reconciled batch35 parent proof — 2026-10-01

Saved `/tmp/patch-12.0.5-batch35-green-{build-result,runs,startup-run}.json` and build/run-{0,1,2,3}/startup logs establish producer `27a840b348f8391e2fd7794d2b4f96c4aec19460`: integration no-run build exit0 in **251.83s**; **10 countdown + 12 loot + 6 ready-check + 7 ping = 35 PASS**, four exit0 selected runs. Integration SHA256 `5e22d2e2f5f6d9531e5ea3651f0dac3aeb6ff1d26a9c574031526e9cc80371d0`. Countdown proof covers the ten fixtures above; controls are selected, not a broad suite.

Saved normal startup exits0 in10.03s, stdout `[]`, CLEAN0 unique/occurrences; wow-sim SHA256 `721c2b00b2220ff21362e0ea11af372cb437c84be91b84f722860988dda4e00e`. Saved artifacts inspected only, not rerun. **Independent292 bounded PASS**, reconciled below; earlier [loot fmt/check](party-loot-method.md#independent-bounded-acceptance--2026-10-01) predates countdown producer and is not a countdown gate.

Historical batch35 checkpoint: row `prose-2026-03-31-169` remained pending before the chronological correction below. Preserve **264 pending / 84 bounded / 14 partial = 362**, retained IDs/source hash. Future row decisions must specify the exact bounded explicit-input predicate across countdown, ready check, ping and loot: lockdown blocks requests; combat alone does not. No native lockdown activation/secrecy, network or permission-enforcement credit. Lifecycle/security/error policies above remain inferred.

## Known gaps (current cycle)

- [x] Parent established compiled behavioral RED before producer work: ten FAIL at `676e4c25a`, not compile errors.
- [x] Parent saved bounded producer GREEN: countdown10 and selected controls25 PASS, startup0 `[]`.
- [x] Independent292 bounded acceptance: saved35PASS, fresh producer fmt/check0; native/full-row unproved.
- [ ] Native zero-as-cancel/replacement, duration validation, blocked failure convention, initiator/chat policy and `AllowedWhenUntainted` behavior remain unverified. These are explicitly inferred simulator policies.

## Out of scope

- Visual countdown expiry belongs to the existing local consumer using `GetTime`. No speculative recurring timer, tick producer, expiry event or post-expiry request-state contract is required by this slice.
- Optional cached Timer.lua integration is omitted: the inspected consumer allocates `StartTimerBar` and depends on animation/sound/UI helpers, beyond a minimal observer fixture. No fabricated callback overrides replace it.
- Native callback-error policy is not characterized here; no countdown-specific failure/rollback rule is invented.
- Native group permissions, remote countdowns, chat delivery, native lockdown activation/secrecy enforcement, legacy aliases, all-profile parity and whole-source acceptance are unproved.
- Loot code/tests/spec and audit accounting belong to other producers; this slice changes none. Wiki architecture links document this producer without audit promotion.

## Independent bounded acceptance — 2026-10-01

Full `/tmp/patch-12.0.5-countdown-independent-proof.md` accepts producer `27a840b348f8391e2fd7794d2b4f96c4aec19460`: saved compiled RED ten behavioral FAIL at `676e4c25a`; saved **10 countdown + 12 loot + 6 ready + 7 ping = 35 selected PASS**, startup exit0 `[]`. Fresh independent default fmt/check exit0 at clean producer, 24.34s/49.34s, no warnings; unchanged before/after hashes. Ledger/log prefix `/tmp/patch-12.0.5-independent-countdown-e56e57a9c5f6`, results/before/after JSON and fmt/check logs. No commands rerun for reconciliation.

Countdown source SHA256 `7287aca4399c7b155f89b6125bfa58155e7083b75171a405d492e75cbdd6e79c`; fixture SHA256 `914fce81bb420959ddb34ac6ec392b51b07b7eab07867f3e238325409b50047f`. Five scope hashes remain producer-identical at final `d656bf0377a871d37dcc5a5429b8306bb705ef24`; later cooldown fixture commit is **compile active, not acceptance**. Saved binary hashes agree with independent inspection. Historical RED executable was overwritten; RED remains saved evidence.

Lifecycle/replacement/zero cancellation, strict finite validation/fractions, success/error conventions, requester/chat payload and conservative security remain **inferred simulator policies**. Secret rejection is stricter than `AllowedWhenUntainted`. Native event secrecy/lockdown production, group permissions/network, profiles and visual expiry remain unproved. Callback reentrancy and forced-GC dispatch have source-order/rooting observations only, not dedicated behavioral probes. GUID/name payloads are rooted; no countdown-specific event-name rooting claim.

One nonblocking readability finding deferred: `countdown.rs:46`, inline `3600.0` should name the documented maximum. No zero-findings claim. New production max cognitive4/cyclomatic6; publish body27 lines. Existing owner-file issues excluded.

### Exact row169 decision — bounded predicate acceptance

Row `prose-2026-03-31-169`: “The recent restrictions to countdown, ready check, ping and loot method APIs have been loosened to only apply when in chat messaging lockdown rather than in all combat.”

| Family | Bounded modeled predicate coverage | Proof / exclusions |
|---|---|---|
| Countdown | `C_PartyInfo.DoCountdown`, explicit lockdown/combat 2×2, atomic rejection/recovery | Ten PASS; lifecycle/security inferred |
| Ready check | Public start/confirm and legacy ReadyCheck guard before effects; recovery without combat transition | [Ready ledger](chat-lockdown-ready-checks.md#reconciled-batch31-parent-proof--2026-10-01), six fixtures/22 selected PASS |
| Ping | `SetRestrictPings` blocks mutation iff explicit lockdown; combat independent, getter readable | [Ping ledger](party-ping-restrictions.md#reconciled-batch32-parent-proof--2026-10-01), seven fixtures/29 selected PASS; **not ping actions/delivery** |
| Loot method | `SetLootMethod` lockdown/combat 2×2, state/event preservation, getters readable | [Loot ledger](party-loot-method.md#independent-bounded-acceptance--2026-10-01), 19 unique PASS/31 executions; bounded roster |

Historical decision: row169 was retained pending at the earlier countdown milestone. Its ping action/delivery prerequisite was assistant-invented, not established by the chronological source. That rationale is withdrawn; prior counts remain historical snapshots. Current parent acceptance promotes only row169 to **bounded PREDICATE coverage**, using all four independent proofs above. This is not native or exhaustive API-family completion.

March 25, 2026 source line74 explicitly names `C_PartyInfo.SetRestrictPings` alongside countdown, ready checks and loot setting in the combat restriction list. March 31 source line169 changes the condition to chat messaging lockdown. Ping restriction-setting therefore supplies bounded evidence for the changed predicate; actual ping action/delivery is not a prerequisite. `HasRestrictions` annotations alone do not establish the historical API inventory or require additional `C_Ping` operations.

**Unknown/unproved:** native lockdown producer/activation/reset, security/taint and `AllowedWhenUntainted` parity, native blocked-error convention, permissions/network behavior, exhaustive API inventory and all-profile parity. No `C_Ping` action, sending, receiving or delivery coverage is claimed.

Related exact row168 removes recent restrictions on `AddPrivateAuraAnchor`, `RemovePrivateAuraAnchor`, `SetPrivateWarningTextAnchor`, `RemovePrivateAuraAppliedSound`; `AddPrivateAuraAppliedSound` remains restricted during encounters/M+/PvP. Countdown predicate does not cover these removals or encounter enforcement. [Warning proof](private-warning-text-anchor.md#independent-bounded-acceptance--2026-10-01) remains bounded; rows168/405 pending.

Current accounting after `91b7d7bb6`: **261 pending / 87 bounded / 14 partial** → **260 pending / 88 bounded / 14 partial = 362**. Only row169 changes status; all IDs, unrelated rows and source text SHA remain unchanged. Register file SHA256 `eaea58ae8adf215587cea6de12349b3586fcb2520a4c8aefd4d7cee5406046ed`. Documentation/accounting only; no tests/build/delegation/source changes.
