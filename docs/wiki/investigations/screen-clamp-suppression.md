# R88n.5r-q1/q2 pre-edit gap table
Base 5401e68f89e49e1fe847b9ff0aaedaf6068cfed0, dot/clamp-suppression-r88n5r.

| VERIFIED fact | Existing source | Status / bounded finding |
|---|---|---|
| Four clamp methods published | drag_input.rs:30-31; bounds.rs:15-17 | MATCH |
| Dedicated per-region suppression bit; resolver enabled AND not suppressed | src/layout.rs:703-710; src/widget/frame.rs:209 | MISSING: neutral global function always false, no dedicated region bit |
| Effective IsClampedToScreen; setter compares effective request | drag_input.rs:221-245 | CONTRADICTED: getter reads stored bit, setter always writes/marks dirty |
| Generic construction flags false, scale1 | frame_defaults.rs:111 and scale defaults | MATCH enabled/scale; MISSING suppression state |
| XML supplied bool independent of presence | direct.rs:752-767; XML Option<bool> | MATCH |
| Suppression writer sets/clears from Boolean; dirties/queues unless already both | registry/mod.rs:730-735 | MISSING writer; existing dirty-set queue deduplicates IDs, no suppression state transition |
| Dependency state/dependents updated before suppression writer | registry/anchor.rs:97-115 | MISSING dedicated native propagation state; existing code tracks only targets/reverse index |
| SetPoint / SetAllPoints CAN reach dependency propagation | button_anchor_hierarchy/anchors.rs; registry/anchor.rs | MISSING native propagation-state writer; native Boolean predicate UNKNOWN |
| ClearAllPoints clear path; ClearPoint clear path when last anchor gone | anchors.rs:205-257 | MISSING suppression/dependency-state clear route |
| SetOwner indirect through anchors, not direct flag mutation | tooltip owner/anchor paths | Existing anchor operations present; MISSING common native propagation state |
| Accepted moving/sizing/stopping CAN reach anchor machinery | drag_input.rs:142-198 | MISSING native anchor movement setup/rectangle-to-anchor transitions; cannot infer suppression=is_moving |
| SetUserPlaced changes only placement | drag_input.rs:201-211 | MATCH |
| Insets convert public dimensions to native units; getter reverse | bounds.rs:57-86 | MISSING explicit native-unit representation; existing storage in public layout units. Native conversion factor not supplied |
| Clamp scaled insets against screen not owner | layout.rs:688-701,722-735 | MATCH in simulator public-coordinate representation; native conversion algebra needs evidence |
| Left,bottom,right,top corrections; oversize right/top wins | layout.rs:737-758 | CONTRADICTED: oversize resets x/y=0, bypasses inset bounds. Per-axis finite ordinary order otherwise equivalent |
| Conditional anchor/dimension rounding before clamp | layout.rs:226-242,307-316,389-390,644-645; registry/pixel_scale.rs:29-35 | MATCH order for existing rounded inputs; exact native anchor-coordinate vs offset and derived-dimension rounding arithmetic not supplied |
| No extra clamp rounding before commit | layout.rs clamp/commit pipeline | MATCH: clamp adds none |
| Successful geometry commit then callback; notifications may defer | state_render.rs rect resolve/size changes | Existing commit/notifications present; separate inset pre-store notification state MISSING |
| SetClampedToScreen unchanged effective request does not write/queue; changed policy checked | drag_input.rs:221-233 | CONTRADICTED: unconditional write/mark; protected-state policy existing, native enable-policy mapping not established |
| Accepted enabled change queues, does not sync resolve | drag_input.rs:232; registry/mod.rs:730-735 | MATCH queue-only after change; conditional/dedup guard needs correction |
| Insets store without comparison/invalidation/queue/resolve | bounds.rs:71-86 get_mut_visual | MATCH ordinary store; conditional pre-store notification callback MISSING |
| Notification callback conditional on saved access/object state; propagation distinct from rect queue | bounds.rs:71-86 | MISSING explicit notification input/state. Actual predicate/traversal not supplied |
| Required bool truthiness and required four dimensions with numeric strings; zero-value setters | drag_input.rs:223; bounds.rs:73-76 | Setter arity MATCH; bounds missing args default0 CONTRADICTED; required bool errors/Usage need exact readers |

Historical pre-edit questions (answered by q3; implementation deferred #1169):
1. Predicate producing suppression Boolean from changed anchor state, dependent combination/traversal, and one concrete original consumer call sequence with expected transitions (including unchanged StartMoving).
2. Exact public-to-native dimension conversion and anchor-coordinate/dimension rounding arithmetic; is current algebra in public units equivalent? Current offset rounding is not independently proof that final anchor coordinates are rounded.
3. Exact conditional inset pre-store notification predicate (saved access + object state), target/dependent/ancestor propagation, and enable-policy predicate. Until supplied, these must remain explicit neutral host seams, not unconditional setters/movement toggles.

## Implemented unambiguous subset (candidate, not gate receipt)

The model now has a dedicated per-region bit and explicit Boolean writer,
effective clamp reads and compare-before-write, set-backed queued dependent
geometry updates. The identified anchor clear routes and distinct propagation-state
predicate are VERIFIED by q3; implementation is deferred by agreement (#1169),
not wired directly to the suppression bit. Generic construction clears
the bit. Retail overflow uses ordered later-edge precedence; other profile
behavior is unchanged. Required ordinary setter argument/return contracts are
covered. No renderer or tooltip-only adjustment is introduced.

Four `screen_clamp_state::` selectors use explicit host suppression input to
verify conditional effective-state/queue behavior, unchanged user-placement
behavior, already-dirty dependent invalidation, and setter contracts. They do not prove the deferred public propagation producer. The original
Retail GameTooltip helper source is checked; the new oversize selector feeds two
different widths and checks exact right/top pinning, unchanged size and unequal
outputs, rejecting a constant/no-op clamp. No test execution is claimed here.

Native q3 answers (#1168) VERIFY propagation, conversion/rounding and
notification/policy behavior. These producers were deferred in the q1/q2 merge
by #1169 and implemented in the q3 follow-up below.

## Sources

- lt-5c7c native comment1105, R88n.5r-q1/q2; #1168 q3 and #1169 deferral.
- [[layout-system]] and [layout reference](../../layout-system.md).

Review clarification: requesting propagation clear is not proof of an unconditional
suppression=false write. The native writer runs when propagation state changes;
that distinct state is not yet modeled. Direct public clear-bit wiring was withheld
rather than inferred, including the corresponding initially proposed test.

## Test-harness portability

The cloud verification kernel lacks the optional `/proc/<pid>/task/<pid>/children`
entry. The prefork conformance child-discovery helper falls back only on ENOENT
to scanning numeric `/proc` entries and matching the PPID in `stat` (field 4,
the second token after the final `)`). The fast path, deadline, child count,
and full conformance admission remain unchanged. This is a test-harness
portability fix, not tooltip behavior or native clamp evidence.


## q3 follow-up (lt-883d, Retail only)

The complete q3 native answer is tracked in #1176, scoped to Retail
12.1.5.70688. #1181 documents the remaining q4 evidence gaps. See
[layout reference](../../layout-system.md#screen-clamping) for the current
public contract, exact f32 unit operations, comparison and queue rules.

- **VERIFIED behavior:** anchor dependencies inherit propagation; changed
  unguarded nodes write state first, visit their ordered dependents, and then
  call the verified suppression writer. The false/guarded shared coordinate root
  is represented as a geometry target, separate from UIParent. Accepted
  StartMoving and ordinary active-target settlement request that root's state.
- **VERIFIED behavior:** native-unit inset stores and public getters use
  distinct f32 conversion order; rounding touches displacement and explicit
  dimensions before correction, not resolved target edges or derived spans.
  No renderer snapping is claimed. This model bridges existing public-unit
  displacements/dimensions through that helper rather than changing all
  unrelated geometry storage.
- **VERIFIED behavior:** the inset pre-store notification is separate from
  rectangle dirtiness and follows its saved-access, prevention and prior-mark
  predicates. The distinct geometry callback runs before its dependents.
  The restricted-region policy rejects changed true and raises the diagnostic
  rather than yielding a nil or successful no-op. Retail's old generic
  protected/combat gate for this setter is contradicted and removed; other
  profiles and general script-object access behavior are unchanged.
- **INFERRED simulator representations / UNKNOWN native origins:** the
  fallback rounding scale guard is positivity, not a claimed native threshold;
  movement special contexts, saved access, ancestor eligibility/gate and first
  boundary marker, and restricted-region state are separate explicit host
  inputs, neutral by default. The constructor does not establish any loaded
  owner's or UIParent's propagation. StartSizing is outside this change.
- **Conditional original proof:** the selected Retail GameTooltip helper
  drives cursor and right owners, followed by ClearAllPoints and a point to
  UIParent, with supplied true and false targets. The all-false sequence is
  **INFERRED** and conditional on the fixture. Movement, API, guard/equal
  return, rounding and independent notification tests exercise the other
  bounded behavior without modifying Blizzard Lua.
