# Ambiguate context boundary

Current follow-up: [rilua capability rows](rilua-capability-rows.md) now covers bounded row416 secret-fullName acceptance/output secrecy with host primitives. B74 evidence below remains historical exact415 proof, not proof of that later implementation.

B74 covers only retail 12.0.5 source occurrence `global api-PlayerScript Ambiguate-415`: argument 2 is `NeverSecret`. [Retained changes](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 415–416, separate this annotation from row 416's `AllowedWhenTainted` change. Cached retail `Blizzard_APIDocumentationGenerated/PlayerScriptDocumentation.lua:22–35` declares non-nil cstring `fullName` and `context`, with `NeverSecret` on context. Architecture: [Lua API](../lua-api.md).

## What it must do

### Declared boundary

- [x] Reject actual VM host-secret NUM, BOOL, and STR contexts in secure and addon callers. This exercises row 415 only; no secret first argument is supplied.

### Bounded simulator requirements, not native-verified semantics

- [x] Reject secret context before processing fullName, including when fullName is nil or a public table; include deterministic `Ambiguate argument #2 must not be secret` marker. Declaration does not establish validation order or error text; these are authored requirements.
- [x] Preserve caller stack taint on rejection and on subsequent successful public calls, restoring the secure caller after an addon caller returns.
- [x] Leave context wrappers secret and live across collection, rejection, and recovery; preserve rooted wrapper identities.
- [x] Preserve established public mapping: context `none` retains fullName; every other public string context uses `string.match(fullName, '^(.-)%-.+$')` or retains fullName when unmatched. This is simulator behavior, not a native context enumeration.
- [x] Preserve that mapping for `short`, `none`, no realm, multiple hyphens, and trailing hyphen. Public addon `short` and `none` calls retain caller taint.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- Producer `9441f7c7c` promotes the public transform to a real module and removes the old fallback. Private-epoch context-secret rejection precedes argument 1 processing; the old Lua exact body and inverse boolean `false` are retained. Inventory is supplied producer evidence, not independent acceptance.
- `tests/ambiguate_context.rs` — actual-provider boundary and public mapping assertions, gated by `retail-12-0-5`; no additional integration target.

## Tests asserting this spec

- `tests/ambiguate_context.rs`: 18 secret rejection cases = three wrapper payload types × two caller contexts × valid fullname/nil/table. Each includes caller-taint assertions, collection, wrapper liveness/identity, and public short/none recovery.
- Same file: four public-control cases, secure short/other string contexts and none, plus addon short and none. Concrete names cover no realm, multiple hyphens, and trailing hyphen.

## B74 producer checkpoint — 2026-10-03

Supplied actual RED `d11`: build exit0, zero diagnostics, **828.602250s**; **22 cases: 4 PASS / 18 FAIL**, **21.161436s**. Earlier combined E0277 compilation ran no tests and is not behavioral RED. This compiled RED and producer `9441f7c7c` supersede the historical inputs-only checkpoint, not the acceptance gates.

Historical producer checkpoint awaited GREEN/check/Forever acceptance; superseded only by bounded acceptance below. Separate worker roots and revision-scoped proof do not establish current whole-HEAD equivalence.

## B74 independent bounded acceptance — 2026-10-03

Main accepts independent625 for **exact415 argument-2 NeverSecret only** at `9441f7c7c`: **24 fresh Retail PASS = 22 new cases + 2 inert controls**, with **2 separate Forever public controls**. Saved GREEN build exits0/zero diagnostics in567.778806s; focused run8.058839s. Pinned Retail startup exits0 with `[]` in25.001060s; scoped fmt/check exit0, check zero diagnostics in580.000887s including lock waits. Forever build exits0 in492.975528s with **8 unrelated warnings retained**; no warning-free Forever claim. Proof: `/tmp/patch-12.0.5-b74-independent-proof.{md,json}`.

Only satisfied bounded requirements above are checked. Error/order, public mapping and recovery remain authored simulator semantics, not native evidence. Wrappers are compared for secrecy, identity and liveness, not decoded payloads or native nominal types. Forever proves public/inert controls only, not secret-context inverse behavior.

Accounting `604a3eea9`, independently accepted644 PASS, promotes **415 only**: **171 pending / 170 bounded / 14 partial / 7 metadata; 362 ordered IDs / 80 capabilities**. Other361 rows and prior79 capabilities/order/source hashes remain unchanged. Accounting SSOT artifacts: `/tmp/patch-12.0.5-batch74-accounting-validation.{md,json}`. B75's39/39 accounting and scoped named-body equivalence remain valid at the older172/169 checkpoint; not current whole-tree equivalence.

Dirty-combined evidence proves only five owned Rust hashes and scoped LoC fixture equivalence. Source/check-to-current whole-tree equivalence is **false**; intervening main/build-host commits are unowned, not cleared. Initial Python launcher never reached Cargo; terminal-result `.spawn()` incident occurred after completed check, whose saved worker/status/streams prove exit0 without duplicate execution. Historical RED, E0277, check-launch incidents, globalfmt/process limits and full-suite failures remain. No row416, secret argument1, output-secrecy, native/all-profile or full-suite GREEN credit. [Audit](../wiki/investigations/patch-12-0-5-api-audit.md#batch74--ambiguate-inputs-only-checkpoint) and [separate LoC acceptance](spell-book-loss-of-control-outputs.md#test-only-loc-fixture-repair--2026-10-03).

## Known gaps (current cycle)

- [x] Obtain independent scoped GREEN/check/Forever acceptance for the producer; acceptance is bounded to exact415 and the saved revision/scope above.

## Out of scope

- Row 416 `AllowedWhenTainted` remains unmodeled: no secret fullName acceptance, decoding, or argument-1 policy claims.
- Output secrecy and native parity: neither follows from context `NeverSecret` or caller-taint retention.
- Context enumeration, new types, coercion, name policy, new globals, shared secret-policy changes, other profiles, and unrelated APIs: not required by exact row 415.
