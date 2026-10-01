# Party countdown

Bounded Retail 12.0.5 `C_PartyInfo.DoCountdown` observable request lifecycle contract. Tests/spec only: the provider belongs to `src/c_api/c_party_info.rs`; this slice adds no production implementation. See [Lua API environment/state architecture](../lua-api.md) and [event dispatch](../event-system.md). Requirements below remain unverified until parent-owned compiled RED and producer GREEN.

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
- `src/c_api/c_party_info.rs` — existing namespace registration owner; production changes are parent-owned and absent from this slice.
- `src/c_api/c_chat_info.rs` — existing shared lockdown error guard; unchanged.
- `src/lua_api/state/sim_state.rs` — existing per-environment lockdown/combat inputs; no new countdown field is assumed by tests.

## Tests asserting this spec

`tests/party_countdown.rs` covers exact return type/arity and synchronous payloads; modeled name; fraction/max boundaries; idle cancellation; both positive replacement orders; start/cancel call orders; malformed atomicity in fresh/active states; secret start/cancel with secure/tainted callers; two-environment isolation; locked fresh operations; active-request lockdown/combat 2×2 matrix and recovery.

Parent filter: `cargo test --test integration party_countdown::`.

**Proof ledger, 2026-10-01:** ten fixtures authored; no build, test, check, native probe or behavioral RED executed in this slice. Formatting only. Parent owns compiled RED after commit; producer implementation and GREEN remain separate work. Existing grouped harness discovery requires no other test/Cargo edits.

## Known gaps (current cycle)

- [ ] Parent compiles and executes these ten fixtures as behavioral RED before producer work; compile errors alone are not RED.
- [ ] Parent implements and proves the bounded producer GREEN, then obtains independent acceptance.
- [ ] Native zero-as-cancel/replacement, duration validation, blocked failure convention, initiator/chat policy and `AllowedWhenUntainted` behavior remain unverified. These are explicitly inferred simulator policies.

## Out of scope

- Visual countdown expiry belongs to the existing local consumer using `GetTime`. No speculative recurring timer, tick producer, expiry event or post-expiry request-state contract is required by this slice.
- Optional cached Timer.lua integration is omitted: the inspected consumer allocates `StartTimerBar` and depends on animation/sound/UI helpers, beyond a minimal observer fixture. No fabricated callback overrides replace it.
- Native callback-error policy is not characterized here; no countdown-specific failure/rollback rule is invented.
- Native group permissions, remote countdowns, chat delivery, native lockdown activation/secrecy enforcement, legacy aliases, all-profile parity and whole-source acceptance are unproved.
- Loot code/tests/spec and wiki/audit updates belong to other producers; this slice changes none.
