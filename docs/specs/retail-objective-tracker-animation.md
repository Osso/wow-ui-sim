# Retail original Objective Tracker animation

## Scope

Retail alone no longer applies `temporary/objective_tracker_quest_header.rs`.
Other profiles retain their existing workaround; parity is not inferred.
This removes an original-Lua surface override, not a new native animation model.

The selected `Blizzard_ObjectiveTracker.toc` loads Module.lua/XML and
QuestObjectiveTracker.lua/XML without a Retail family exclusion. The XML creates
`QuestObjectiveTracker` inheriting `ObjectiveTrackerModuleTemplate`, whose header
inherits `ObjectiveTrackerModuleHeaderTemplate`. The original
`ObjectiveTrackerModuleHeaderMixin:PlayAddAnimation` (Module.lua:779-781) calls
`self.AddAnim:Restart()`. Module.xml:56-63 defines timed alpha and translation
animations. The temporary surface normalization instead immediately called Stop
and forced terminal region alphas. That contradicts this original consumer.

The fix omits the entire temporary Retail surface patch: forced visibility,
text/color fallback, terminal alphas, and mixin wrappers. It does not edit Blizzard
files, replace consumer methods, add timing defaults, or claim native frame-clock
or animation-final-alpha equivalence. If original animation/layout primitives
still differ, they require separate evidence and tests rather than another
surface override.

## Consumer proof and limits

`tests/objective_tracker_original_animation.rs` is Retail-only and optional.
It uses the discovered current-profile addon closure, validates its selected TOC
and exact Lua/XML entries, checks the real XML-created header and original
PlayAddAnimation identity/source, then supplies only ordinary layout fields
(displayable state, height, collapsed flag). No fake widget or animation sink,
API substitute, sliced vendor source, or vendor override is used.

The original header starts its real XML AddAnim. The ordinary post-event hook
must leave it playing. Stock fails exactly that assertion; bypassing only the
Retail workaround passes. The test does not advance time or assert final alpha.
The second iteration checks reapplication preservation: the post-event hook's
existing one-shot marker means it does not run its patches a second time.

A second test preserves collapsed/empty presentations. Its collapsed iteration
exercises the hook; the empty iteration follows the one-shot marker and therefore
is only an empty-state/reapplication preservation observation, not independent
proof of the empty branch on first application. Neither is a RED selector.
Existing synthetic workaround unit tests now apply only to non-Retail profiles;
they preserve legacy behavior and are not native animation evidence.

Observed Retail proof: original consumer RED 1 failed / 1 preservation passed;
GREEN 2 passed. Both builds returned build-finished success:true with zero
compiler warnings. These are targeted integration results, not full-suite or
seven-profile gate completion. Full SHA/source-tree association and command,
binary hashes, toolchain metadata, JSON build output and runtime logs are kept
in the untracked task receipts. Tests were committed after execution without
changing the bytes used by RED or GREEN.

## Remaining inventory B candidates

This slice does not complete lt-2897. QuestCache/QuestUtil fallbacks need a normal
Retail original-load-gap reproduction (real ObjectAPI replaces QuestCache).
Wrong fallback enum values need a real consumer that reaches fallback, not
artificial deletion of generated constants. QuestLog's parent guard needs the
original WorldMap parenting/event ordering diagnosis before removal. Encounter
color signatures omit a declared trigger, but no selected pinned Retail original
color-call consumer was found and native validation/storage behavior remains
unknown. All NATIVE questions and KEEP seams in synthesis B remain open.
