# Controls and layout: source-backed rework

**Static comparison only.** All entries inherit the [pinned revisions, historical-profile caveat and proof limits](README.md). No fixtures were run. External paths are relative to `~/Repos/wowdecomp`; simulator paths are relative to this project.

## C1 — Slider value assignment ignores its step

**External evidence:** `Engine/Source/Frame/CSimpleSlider.cpp:65-78`, `SnapValueToStep`; `:248-273`, `SetValue`; `:276-287`, `SetValueStep`. Assignment clamps, snaps relative to the minimum/base, then applies endpoint corrections. Changing the step re-applies assignment to an existing value. The reconstructed endpoint corrections are more than simple rounding followed by clamping.

**Simulator difference:** `src/lua_api/frame/methods/widgets/slider.rs:30-37` only stores `slider_step`; `:138-147` only clamps assigned values. The programmatic assignment path does not use the step and changing the step does not re-snap an existing value. Dragging behavior is a separate path and must not be inferred from this finding.

**Observable examples:** set range `[0,10]`, step `2`, value `3`: reconstruction stores/callbacks `4`; simulator stores/callbacks `3`. Set value `3` before changing step to `2`: reconstruction re-applies snapping; simulator leaves `3`. For step `4`, value `9`, the source's endpoint correction produces `10`, not merely nearest-step `8`.

**Required change:** apply the target client's base-relative quantization and endpoint policy in programmatic Slider assignment; step changes must re-evaluate existing values and emit callbacks only when the resulting value changes. Do not apply Slider snapping to the shared StatusBar value path. Establish modern-profile policy before adopting historical endpoint rules or defaults.

**Regression:** assert stored value and `OnValueChanged` payload for the examples, nonzero/negative minima, exact endpoints and unchanged-value controls; change step after a value is present. Existing `tests/widget_slider.rs` exercises ranges/callbacks and drag-step flag roundtrips, not proof of this assignment boundary. [Callback contract](../specs/slider-value-callback.md) remains the owning simulator requirement.

**Applicability:** concrete shared-source discrepancy against reconstructed 3.3.5a; not native proof for any supported modern profile.

## C2 — EditBox accepts `OnCursorChanged` but does not produce it

**Historical baseline gap report superseded within the reported implementation scope.** See the [exchange checklist](README.md#exchanged-implementation-checklist) for cross-host Retail/Wrath results. Genuine Wrath scrolling-consumer proof remains open and its guard retained; the baseline trace below is not a current-tip missing-producer claim.

**External evidence:** `Engine/Source/Frame/CSimpleEditBox.cpp:288-299`, `RunOnCursorChangedScript`, passes four geometry numbers; `:1630-1700`, `UpdateVisibleCursor`, calculates cursor geometry and calls that producer. `SetCursorPosition` at `:818-826`, dirties cursor layout rather than synthesizing an immediate geometry callback.

**Simulator difference:** `src/lua_api/frame/methods/widgets/editbox.rs:151-160` changes cursor position without producing this event. A source-wide search found `OnCursorChanged` in script registration/validation (`src/event/mod.rs:142,238`; `src/lua_api/frame/methods/text_attribute_event/events.rs:409`) but no actual EditBox dispatch producer. `src/wrath/post_load.lua:6-27` already guards `ScrollingEdit_OnUpdate` when cursor geometry fields are absent; that workaround is downstream evidence, not a substitute for the missing producer.

**Observable fixture:** bind `OnCursorChanged` on a visible EditBox, enter multiline text, position the cursor, and allow actual layout/update processing. The reconstructed cursor-layout path supplies `(x,y,width,height)`; simulator has no producer to notify cursor-following scroll/overlay consumers.

**Required change:** compute/commit cursor geometry through the simulator's real text/layout path and dispatch `OnCursorChanged(self,x,y,width,height)` at the corresponding update boundary. Fixing the producer should retire the Wrath guard only after its original consumer-ordering failure is covered. Do not invent geometry in `SetCursorPosition` merely to trigger a callback, or copy legacy coordinate conversion into every profile.

**Regression:** exercise a scrolling EditBox with its real `OnCursorChanged`/`OnUpdate` consumer ordering, programmatic cursor movement and keyboard movement; assert concrete geometry values and scrolling behavior after a tick. Preserve UTF-8 byte/scalar conversion. `tests/methods_anchor.rs:58-68` checks handler acceptance, not production. [Cursor/selection](../specs/editbox-text-position-selection.md) and [focus](../specs/editbox-focus-callbacks.md) specs own existing behavior. Exact native timing/coordinates remain unverified.

## C3 — Reversed/negative `HighlightText` endpoints select the wrong range

**External evidence:** `Engine/Source/Frame/CSimpleEditBox_Scripts.cpp:368-385`, defaults start/end to `0/-1`. `CSimpleEditBox.cpp:728-751`, `HighlightText`, clamps start, maps negative end to `-1`, then uses text length whenever end precedes start. It does not exchange endpoints.

**Simulator difference:** `src/lua_api/frame/methods/widgets/editbox/selection.rs:7-25,37-45` converts offsets and swaps reversed endpoints. Negative end offsets become the beginning before normalization, not an end-of-text sentinel.

**Observable examples with `abcdef`:**

| Operation, then `Insert("X")` | Reconstructed selection/result | Simulator selection/result |
|---|---|---|
| `HighlightText(4,2)` | `[4,6]` → `abcdX` | `[2,4]` → `abXef` |
| `HighlightText(2,-1)` | `[2,6]` → `abX` | `[0,2]` → `Xcdef` |

These outputs are derived from source, not observed executions.

**Required change:** if adopting the historical contract for a supported profile, distinguish the negative-end sentinel and end-before-start behavior before converting to internal scalar positions. Keep existing UTF-8 byte-offset/scalar-storage semantics; do not replace them with raw byte mutation.

**Regression:** assert subsequent `Insert` text for these cases, omitted end, in-order endpoints, empty selection, and multibyte text. `tests/editbox_stub_family.rs:255-355` covers adjacent selection/UTF-8 behavior, not evidence for the two boundaries above. Current owning selection spec does not establish these historical corner cases as universal native behavior.

## L1 — Protected frames can use region targets that the reconstruction rejects

**External evidence:** `Engine/Source/Frame/CSimpleRegion.cpp:46-48`, `CanBeAnchorFor`, rejects a protected recipient. `CLayoutFrame.cpp:827-844`, recursively checks the target's active anchor dependencies; `:869-885`, `SetPoint`, returns before mutation if the target is ineligible. `CScriptRegion_Scripts.cpp:464-548` exposes the Lua call into that path. Caller authorization and target eligibility are separate checks.

**Simulator difference:** `src/lua_api/frame/methods/button_anchor_hierarchy/anchors.rs:422-459,612-638` checks caller/combat restrictions, forbidden-aspect inheritance and cycles, then changes dependencies/anchor state. It does not apply a protected-recipient/Texture-or-FontString target rule or its indirect-chain equivalent. `shared.rs:39-61` resolves identities without that eligibility policy.

**Observable fixture:** out of combat in trusted context, give protected frame `P` a valid baseline anchor. Attempt to anchor it directly to Texture `T`, then to ordinary frame `M` already anchored to `T`. Reconstruction rejects both without replacing P's point; simulator's inspected mutation path accepts them. Controls: unprotected recipient→T and protected recipient→eligible ordinary frame. An ordinary insecure frame alone is **not** the disallowed target demonstrated by this evidence.

**Required change:** after resolving target-profile semantics, evaluate target eligibility before removing old dependencies or changing anchors. Include recipient protection, region kind and active indirect anchor chains. Preserve existing caller-security, cycle and forbidden-aspect checks; they are not equivalent rules.

**Regression:** assert unchanged prior `GetPoint`/resolved geometry for both rejected cases and successful controls. Establish protection via supported public template creation, not a test-only field toggle. Native protection propagation and modern applicability remain unverified.

## Rejected or unresolved candidates

| Candidate | Disposition |
|---|---|
| Stale inherited alpha/scale/strata after `SetParent` | Rejected: `src/lua_api/frame/methods/methods_hierarchy.rs:22-43` propagates these properties. No missing-propagation claim. |
| Generic disabled CheckButton checked texture absence | Rejected for configured disabled slots: selection and enable/disable refresh exist. [C12](cataclysm-15595.md#c12--disabled-checked-button-hides-its-only-checked-texture) isolates the distinct missing-disabled-slot case. |
| Generic UTF-8 cursor handling, ordinary focus transfer, animation pause/explicit stop | Already modeled with existing tests/contracts; no new omission established. |
| Animation zero-duration progress and multi-loop callback cadence | Lead only. Source raw division versus simulator guarded progress does not establish a target-client defect; keep existing contract pending stronger evidence. |
| General hit testing, StatusBar, ScrollFrame, anchor math | Bounded inspection did not isolate a concrete discrepancy; not a parity verdict. |
