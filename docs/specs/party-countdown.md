# Party countdown

Bounded Retail 12.0.5 `C_PartyInfo.DoCountdown` observable request lifecycle contract. The provider is registered by `src/c_api/c_party_info.rs` and implemented in its C API-owned `countdown` module. See [Lua API environment/state architecture](../lua-api.md) and [event dispatch](../event-system.md). Parent reports compiled RED at `676e4c25a` (ten FAIL); producer GREEN and independent acceptance remain pending.

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

Final retained [`prose-2026-03-31-169`](../../data/patch-api/sources/12.0.5-register.json), source line 169, says restrictions were “loosened to only apply when in chat messaging lockdown rather than in all combat.” This supports the restriction axis, not the inferred request/security/error policies. Countdown tests do not close the whole row or change audit accounting.

## How it works

- [Lua API environment/state architecture](../lua-api.md)
- [Event dispatch architecture](../event-system.md)
- [Chat messaging lockdown input contract](chat-messaging-lockdown.md)

## Implementation inventory

- `tests/party_countdown.rs` — ten epoch-gated fixtures, real Lua frame observers, public request effects and actual host-secret fixtures; automatically included by the existing grouped integration harness.
- `src/c_api/c_party_info.rs` — epoch-gated namespace registration owner.
- `src/c_api/c_party_info/countdown.rs` — C API-owned `CountdownRequest` stores requested seconds and modeled initiator snapshot. Validation precedes mutation; positive replacement commits the new record, cancellation clears it, and idle cancellation does neither. Both event strings are stack-rooted during existing synchronous dispatch; state is committed and the mutable borrow released before callbacks. Nested requests therefore observe the committed lifecycle.
- `src/c_api/c_chat_info.rs` — existing shared lockdown error guard; unchanged.
- `src/lua_api/state/sim_state.rs` and `state.rs` — `party_countdown_request: Option<CountdownRequest>`, initialized to `None`, independently owned by each environment. Start/replacement snapshots current modeled player identity; cancel snapshots the current requester, not stale initiator metadata. No new identity constants or clock origin are introduced.

## Tests asserting this spec

`tests/party_countdown.rs` covers exact return type/arity and synchronous payloads; modeled name; fraction/max boundaries; idle cancellation; both positive replacement orders; start/cancel call orders; malformed atomicity in fresh/active states; secret start/cancel with secure/tainted callers; two-environment isolation; locked fresh operations; active-request lockdown/combat 2×2 matrix and recovery.

Parent filter: `cargo test --test integration party_countdown::`.

**Proof ledger, 2026-10-01:** parent-established compiled RED at `676e4c25a`, same successful build recorded by `/tmp/patch-12.0.5-batch34-green-fixed-build-result.json`; parent reports run3 logs/manifests show ten FAIL because no producer existed. Producer slice runs formatting only, no builds/tests/checks/readability gate or native probes. Parent owns GREEN compilation/run and independent acceptance. Existing grouped harness discovery requires no other test/Cargo edits. Requirement checkboxes stay open pending behavioral proof.

## Known gaps (current cycle)

- [x] Parent established compiled behavioral RED before producer work: ten FAIL at `676e4c25a`, not compile errors.
- [ ] Parent proves the implemented bounded producer GREEN, then obtains independent acceptance.
- [ ] Native zero-as-cancel/replacement, duration validation, blocked failure convention, initiator/chat policy and `AllowedWhenUntainted` behavior remain unverified. These are explicitly inferred simulator policies.

## Out of scope

- Visual countdown expiry belongs to the existing local consumer using `GetTime`. No speculative recurring timer, tick producer, expiry event or post-expiry request-state contract is required by this slice.
- Optional cached Timer.lua integration is omitted: the inspected consumer allocates `StartTimerBar` and depends on animation/sound/UI helpers, beyond a minimal observer fixture. No fabricated callback overrides replace it.
- Native callback-error policy is not characterized here; no countdown-specific failure/rollback rule is invented.
- Native group permissions, remote countdowns, chat delivery, native lockdown activation/secrecy enforcement, legacy aliases, all-profile parity and whole-source acceptance are unproved.
- Loot code/tests/spec and audit accounting belong to other producers; this slice changes none. Wiki architecture links document this producer without audit promotion.
