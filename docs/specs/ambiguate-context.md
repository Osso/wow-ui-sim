# Ambiguate context boundary

B74 covers only retail 12.0.5 source occurrence `global api-PlayerScript Ambiguate-415`: argument 2 is `NeverSecret`. [Retained changes](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 415–416, separate this annotation from row 416's `AllowedWhenTainted` change. Cached retail `Blizzard_APIDocumentationGenerated/PlayerScriptDocumentation.lua:22–35` declares non-nil cstring `fullName` and `context`, with `NeverSecret` on context. Architecture: [Lua API](../lua-api.md).

## What it must do

### Declared boundary

- [ ] Reject actual VM host-secret NUM, BOOL, and STR contexts in secure and addon callers. This exercises row 415 only; no secret first argument is supplied.

### Bounded simulator requirements, not native-verified semantics

- [ ] Reject secret context before processing fullName, including when fullName is nil or a public table; include deterministic `Ambiguate argument #2 must not be secret` marker. Declaration does not establish validation order or error text; these are authored requirements.
- [ ] Preserve caller stack taint on rejection and on subsequent successful public calls, restoring the secure caller after an addon caller returns.
- [ ] Leave context wrappers secret and live across collection, rejection, and recovery; preserve rooted wrapper identities.
- [ ] Preserve established public mapping: context `none` retains fullName; every other public string context uses `string.match(fullName, '^(.-)%-.+$')` or retains fullName when unmatched. This is simulator behavior, not a native context enumeration.
- [ ] Preserve that mapping for `short`, `none`, no realm, multiple hyphens, and trailing hyphen. Public addon `short` and `none` calls retain caller taint.

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

**GREEN, check, and Forever independent acceptance remain PENDING.** Async default and Forever workers use separate roots; proof must identify its source revision/scope, not imply current whole-HEAD equivalence. Behavior boxes remain unchecked until independent verification. No native parity, row416 credit, count promotion, or full-suite clearance. [Audit checkpoint](../wiki/investigations/patch-12-0-5-api-audit.md#batch74--ambiguate-inputs-only-checkpoint).

## Known gaps (current cycle)

- [ ] Obtain independent scoped GREEN/check/Forever acceptance for the producer; supplied implementation and RED do not establish passing behavior.

## Out of scope

- Row 416 `AllowedWhenTainted` remains unmodeled: no secret fullName acceptance, decoding, or argument-1 policy claims.
- Output secrecy and native parity: neither follows from context `NeverSecret` or caller-taint retention.
- Context enumeration, new types, coercion, name policy, new globals, shared secret-policy changes, other profiles, and unrelated APIs: not required by exact row 415.
