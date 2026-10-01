# Chat messaging lockdown

Bounded Retail 12.0.5 input and pending predicate fixtures for `C_ChatInfo.InChatMessagingLockdown()`. Input lives in `src/lua_api/state/sim_state.rs`; predicate publication is deliberately absent from this input-only change. See [Lua API architecture](../lua-api.md) for environment/state context.

## What it must do

- [ ] Match literal audit row `global api-C_ChatInfo-InChatMessagingLockdown-251`: `- ret2 = lockdownReason` ([retained register](../../data/patch-api/sources/12.0.5-register.json), source line 251).
- [ ] Return exactly one required boolean, including fresh-state `false` and explicit `true`/`false`/`true` transitions; no second reason, even nil.
- [ ] Read the per-environment Rust boolean `chat_messaging_lockdown`, independently of `player.in_combat`, across all four combinations.
- [ ] Isolate input and query results between environments.
- [ ] Preserve ordinary addon caller stack taint without claiming broader native security parity.

Cached contract: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua`, lines 293–300, declares `Name = "InChatMessagingLockdown"` and exactly one return `{ Name = "isRestricted", Type = "bool", Nilable = false }`. Its description is “Returns true if API security restrictions regarding chat messaging are in effect.” This documents the return contract, not the native restriction producer or transition policy.

**Bounded input policy:** Rust callers set a single per-environment bool. Default `false` is documented simulator input policy, not native-verified behavior. The input is gated by the existing `retail-12-0-5` epoch. No inference from combat, encounter, PvP, keystone, message, channel, or party state; no Lua/admin/test-only setter and no new state struct. No predicate registration or enforcement in this change.

## How it works

- [Lua API environment/state architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` — explicit epoch-gated bool input.
- `src/lua_api/state.rs` — per-environment false initialization.
- `tests/chat_messaging_lockdown.rs` — five grouped, automatically discovered Retail predicate fixtures; no new Cargo target.

## Tests asserting this spec

`tests/chat_messaging_lockdown.rs`: fresh exact-one false; explicit transitions; four combat/lockdown combinations; environment isolation; ordinary caller taint.

Parent filter: `cargo test --test integration chat_messaging_lockdown::` with Retail 12.0.5+ enabled. No compilation or tests run for this input-only checkpoint. Actual RED remains parent-owned: a missing callable predicate or lazy nil-return stub should fail the query/arity assertions; a callable single-false stub should fail true-input fixtures. Neither absence nor observed RED is claimed from source inspection. GREEN requires a separately implemented producer; all checklist items remain unverified.

## Known gaps (current cycle)

- [ ] Parent compile and actual RED, followed by separate predicate producer and bounded GREEN proof.
- [ ] Native restriction producer, activation/reset/ordering policy, and broader security semantics remain open.

## Out of scope

- Named party/message/channel enforcement remains explicitly open; no messaging restrictions are implemented here.
- Native reasons and historical reason semantics remain explicitly open; the retained row removes the second return, not every concept of a reason elsewhere.
- Macro scope remains explicitly open; no macro evaluation or execution restrictions are implemented here.
- Other profiles, vendor Lua, security behavior, startup acceptance, and whole-audit closure are unchanged/unclaimed.
