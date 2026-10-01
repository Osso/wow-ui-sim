# Chat messaging lockdown

Bounded Retail 12.0.5 input and predicate for `C_ChatInfo.InChatMessagingLockdown()`. Input lives in `src/lua_api/state/sim_state.rs`; the real predicate lives in `src/c_api/c_chat_info.rs`. See [Lua API architecture](../lua-api.md) for environment/state context.

## What it must do

- [x] Match literal audit row `global api-C_ChatInfo-InChatMessagingLockdown-251`: `- ret2 = lockdownReason` ([retained register](../../data/patch-api/sources/12.0.5-register.json), source line 251).
- [x] Return exactly one required boolean, including fresh-state `false` and explicit `true`/`false`/`true` transitions; no second reason, even nil.
- [x] Read the per-environment Rust boolean `chat_messaging_lockdown`, independently of `player.in_combat`, across all four combinations.
- [x] Isolate input and query results between environments.
- [x] Preserve ordinary addon caller stack taint without claiming broader native security parity.

Cached contract: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua`, lines 293–300, declares `Name = "InChatMessagingLockdown"` and exactly one return `{ Name = "isRestricted", Type = "bool", Nilable = false }`. Its description is “Returns true if API security restrictions regarding chat messaging are in effect.” This documents the return contract, not the native restriction producer or transition policy.

**Bounded input policy:** Rust callers set a single per-environment bool. Default `false` is documented simulator input policy, not native-verified behavior. The input is gated by the existing `retail-12-0-5` epoch. No inference from combat, encounter, PvP, keystone, message, channel, or party state; no Lua/admin/test-only setter and no new state struct. The predicate reads this input directly and returns one ordinary boolean. No enforcement is implemented.

## How it works

- [Lua API environment/state architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` — explicit epoch-gated bool input.
- `src/lua_api/state.rs` — per-environment false initialization.
- `src/c_api/c_chat_info.rs` — state-backed predicate, pushing one `Val::Bool` and returning arity one.
- `src/c_api/mod.rs` — `retail-12-0-5` module and utility registration gates. `ensure_namespace` preserves other slots; channel/message/default providers do not overwrite this predicate. No concrete predicate fallback remains; generic lazy namespace behavior is unchanged.
- `tests/chat_messaging_lockdown.rs` — five grouped, automatically discovered Retail predicate fixtures; no new Cargo target.

## Tests asserting this spec

`tests/chat_messaging_lockdown.rs`: fresh exact-one false; explicit transitions; four combat/lockdown combinations; environment isolation; ordinary caller taint.

Parent filter: `cargo test --test integration chat_messaging_lockdown::` with Retail 12.0.5+ enabled. Saved parent RED at input commit `f777027be4721bb46728a32553cf15a5cf44fbcd`: compilation exit 0 (303.63s), then five failures at non-boolean query results, exit 101. Evidence: `/tmp/patch-12.0.5-batch30-red-build-result.json`, `/tmp/patch-12.0.5-batch30-red-run.log`, `/tmp/patch-12.0.5-batch30-red-run.json`. Saved parent GREEN metadata binds actual compiled/run revision `18b09cbf9787ed781399173be022b0866218384f`, not a later interleaved docs commit: compilation exit 0 (411.42s); five predicate PASS and two separate `c_chat_info_probes::` control PASS, both exit 0. Evidence: `/tmp/patch-12.0.5-batch30-green-build-result.json`, `/tmp/patch-12.0.5-batch30-green-runs.json`, and `green-run-0.log` / `green-run-1.log` under the same prefix. Both runs identify integration binary SHA-256 `455c53f291d206453255bca5bb452ec15d683394977b335808a44f801c842c5f`. Parent startup metadata at that revision records exit 0; saved summary is CLEAN, zero unique/occurrence errors (`/tmp/patch-12.0.5-batch30-green-startup-run.json`, `green-startup.log`). Startup is parent evidence, not independent execution.

## Reconciled batch30 bounded proof — 2026-10-01

Literal row `global api-C_ChatInfo-InChatMessagingLockdown-251` removes only `ret2 = lockdownReason`. Fixtures assert `select('#', C_ChatInfo.InChatMessagingLockdown()) == 1`; `reason == nil` alone would not prove absence of a second nil return. Saved RED reaches non-boolean assertions after compilation; it is not a demonstrated two-return RED. Saved GREEN supports explicit simulator input, exact arity, combat independence, environment isolation and ordinary caller taint only.

Independent report `/tmp/patch-12.0.5-chat-lockdown-predicate-independent-proof.md`, read fully, accepts only the explicit per-environment input and single-boolean predicate. Saved five predicate and two control PASS at producer `18b09cbf9`; saved parent startup exit 0, JSON `[]`, is ancillary evidence, not independent execution. Fresh fmt/check each ran once and exited 0 at `0d247625c`, with original predicate scope identical to producer. Ready-check inputs `8649fe072` and concurrent ready-check implementation are excluded; this is recorded source-scoped acceptance, not acceptance of current dirty code.

Only exact row251 gains **bounded-coverage**: **264 pending / 84 bounded / 14 partial = 362**. All source IDs, source SHA-256 and unrelated rows remain unchanged. RED observed non-boolean nil, not a two-return failure. No native producer/security, enforcement, message/channel, macro, all-profile, full-page or whole-audit credit.

## Known gaps (current cycle)

- [x] Independent bounded gate for the recorded predicate scope; see reconciled proof above.
- [ ] Native restriction producer, activation/reset/ordering policy, and broader security semantics remain open.

## Out of scope

- Named party/message/channel enforcement remains explicitly open; no messaging restrictions are implemented here.
- Native reasons and historical reason semantics remain explicitly open; the retained row removes the second return, not every concept of a reason elsewhere.
- Macro scope remains explicitly open; no macro evaluation or execution restrictions are implemented here.
- Other profiles, vendor Lua, security behavior, startup acceptance, and whole-audit closure are unchanged/unclaimed.
