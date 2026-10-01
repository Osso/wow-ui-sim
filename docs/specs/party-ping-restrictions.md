# Party ping restrictions

Bounded Retail 12.0.5 `C_PartyInfo.GetRestrictPings` / `SetRestrictPings` contract. This input/test/spec slice adds an explicit per-environment numeric enum input and seven pending fixtures only; neither method is registered or implemented here. The future provider belongs to existing `src/c_api/c_party_info.rs`, not a new Lua workaround or backing abstraction. See [Lua API state architecture](../lua-api.md).

## Evidence

Final retained [`prose-2026-03-31-169`](../../data/patch-api/sources/12.0.5-register.json), source line 169, states: “The recent restrictions to countdown, ready check, ping and loot method APIs have been loosened to only apply when in chat messaging lockdown rather than in all combat.” Superseded March 25 prose is not this contract.

Profile runtime cache `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua`:

- Line 348: `GetRestrictPings` returns one required `RestrictPingsTo` enum, `restrictTo`.
- Line 579: `SetRestrictPings` takes one required `RestrictPingsTo` enum, `restrictTo`, declares `HasRestrictions = true` and `SecretArguments = "AllowedWhenUntainted"`, and declares no returns.

Current cached `Blizzard_CompactRaidFrames/Mainline/Blizzard_CompactRaidFrameManager.lua:164–180` compares `GetRestrictPings()` with the selected enum, calls `SetRestrictPings(enum)`, and resets a repeated dropdown selection to `Enum.RestrictPingsTo.None`. Four radio options are None=0, Lead=1, Assist=2, TankHealer=3. The reset is caller logic, not setter behavior. Legacy unused mixin code at line 1289 passes `GetChecked()`; its boolean call is not evidence of accepted boolean input and is not emulated.

## What it must do

- [ ] Return exactly one required numeric enum from this environment's explicit `party_ping_restriction` input. **INFERRED SIMULATOR DEFAULT:** a fresh environment starts at None=0; native initial/reset policy is unknown.
- [ ] Public setter accepts all four enum values, stores the selection, and returns zero values. Repeating a setter selection preserves it; an explicit None argument resets it. Public changes in one environment cannot affect another.
- [ ] With explicit [chat messaging lockdown](chat-messaging-lockdown.md) true, reject the setter before mutation; getter remains readable. With lockdown false, setter succeeds. Cover all four combinations of combat and lockdown; combat itself neither blocks nor activates the restriction.
- [ ] **SIMULATOR POLICY:** blocked calls fail fast with an explicit nonempty runtime error detectable by `pcall`; exact native error convention/text is unknown. Future producer reuses existing `reject_chat_messaging_lockdown` rather than inferring lockdown from combat.
- [ ] **STRICT SIMULATOR VALIDATION POLICY:** missing/nil, fractional, unknown numeric, string, boolean, table and function enum arguments reject atomically, without coercion. This is not native malformed-input characterization.
- [ ] **CONSERVATIVE SIMULATOR SECURITY POLICY:** reject actual secret enum arguments in both untainted and tainted callers without unwrapping, mutation, declassification or clearing/replacing caller taint. Cached `AllowedWhenUntainted` semantics are **not modeled** by this stricter policy; untainted-secret acceptance remains a known gap.

Input and fixtures are gated by `retail-12-0-5`; no native filtering, permissions, event or producer claim follows from them. All requirements remain unchecked: no compilation or execution in this slice.

## How it works

- [Chat messaging lockdown input contract](chat-messaging-lockdown.md)
- [Lua API environment/state architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` — public primitive `u8` input `party_ping_restriction`, explicitly identified as C_PartyInfo backing state.
- `src/lua_api/state.rs` — initializes the epoch-gated input to inferred None=0.
- `tests/party_ping_restrictions.rs` — seven autodiscovered fixtures in existing grouped integration target; no new Cargo target.
- `src/c_api/c_party_info.rs` — existing C API provider identity; unchanged, ping methods still absent.
- `src/c_api/c_chat_info.rs` — existing lockdown guard for future setter reuse; unchanged.

## Tests asserting this spec

`tests/party_ping_restrictions.rs`: fresh numeric getter/arity; direct explicit input; four public enum selections/zero setter returns/caller-controlled reset; environment isolation; lockdown atomicity/readability on both combat axes; malformed argument atomicity; actual secret argument rejection with preserved caller taint.

Parent filter: `cargo test --test integration party_ping_restrictions::`.

**Proof ledger, 2026-10-01:** source inspection only. Parent owns compiled RED; no build, test, check, broad suite or native probe run here. Fixtures assert providers exist before testing rejection, so a missing function cannot masquerade as successful atomic validation.

## Known gaps (current cycle)

- [ ] Compile and execute the seven fixtures as RED before producer implementation.
- [ ] Implement/register both methods in the existing C API provider; reuse the existing lockdown guard before mutation, then prove the bounded contract.
- [ ] Native `AllowedWhenUntainted` secret acceptance and native malformed/blocked error conventions remain unverified.

## Out of scope

- **Ping behavior:** actual ping filtering/content, group-role permissions, event production and native ping-system state/reset are not implemented or claimed.
- **Remaining named party actions:** countdown and loot-method restrictions remain open; separate [ready-check work](chat-lockdown-ready-checks.md) is not changed or credited here. This slice cannot close the whole final prose row.
- **Native lockdown producer:** activation/reset, encounter/M+/PvP inputs and ordering remain open; explicit test input is not a native producer.
- **Channels/macros:** channel join/leave/mutation, messaging and macro execution restrictions remain open.
- **Other-owned work:** ready-check/predicate code, their specs and audit accounting remain unchanged by this slice; no whole-audit or all-profile acceptance.
