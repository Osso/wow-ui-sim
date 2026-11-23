# Widget System

Every UI element is a `Frame` struct stored in the `WidgetRegistry`. The `WidgetType` enum discriminates 18 widget kinds; creation establishes default children, strata inheritance, and parent-child linkage.

## Frame Struct (~140 fields, `src/widget/frame.rs`)

Key field groups:

**Identity & hierarchy** — `id: u64` (atomic counter), `widget_type`, `name: Option<String>`, `parent_id`, `children: Vec<u64>`, `children_keys: HashMap<String, u64>` (named child refs).

**Rendering order** — `frame_strata`, `frame_level: i32`, `alpha: f32`, `scale: f32`, `draw_layer`, `draw_sub_layer`. The `BLIZZARD` input token is ignored rather than modeled as a drawable strata tier. Retail probe snapshots record effective XML strata and `HasFixedFrameStrata()` before and after selected operations; they do not expose the original XML token or internal resolution mechanism.

**Input** — `mouse_enabled`, `mouse_motion_enabled`, `keyboard_enabled`, `propagate_keyboard_input`, `movable`, `resizable`. `SimState.focused_frame_id` tracks the focused EditBox; Lua `SetFocus`/`ClearFocus` commit that state before their gained/lost callbacks. Repeated/non-owner calls are no-ops, and loss-handler reentry may replace a requested new owner. Changed EditBox `SetText`/`SetFormattedText` commits text and clamps scalar caret/selection endpoints before synchronous `OnTextSet`; INFERRED (R88et-c): drain changed-text notifications after the outermost Lua call returns to host; layout/cursor timing is separate (see the owning contract below). Same-value assignment is a simulator no-op. At `0c5d5f047`, focused printable, Backspace, and Delete consume a nonempty selection through the same Frame helper as public `Insert`, committing text/caret/render caches before existing callbacks; rejected numeric input preserves selection. `1933cb68c` keeps keyboard overflow as a no-edit/no-callback user-selected policy. At `390e560f5`, positive `MaxLetters` truncates programmatic EditBox `SetText` and shared `SetFormattedText` before cache, cursor/selection, and callback work; same final text remains a no-op, and non-EditBoxes or unlimited limits are unchanged. Bounded proof covers 51 default-profile cases and 23 Forever-profile cases; totals overlap and are not distinct features. The Forever secret EditBox test in `tests/editbox_stub_family/max_letters.rs` proves simulator-only clipped secret readout, tainted `GetText`/secret `SetText` denial, atomic failed secret and ordinary-userdata writes, and plain replacement clearing secret origin. `924a44fbc` makes the existing combo-point constant `pub(crate)` without changing its value or behavior; fresh Forever check passes without warnings. Unicode scalars follow the simulator `GetNumLetters` model, not native markup/grapheme proof; `SetText` MaxBytes remains unresolved. `547907429` initializes XML `MaxLetters` and `MaxBytes` independently from explicit values or the most-derived inherited value in ordinary and runtime-template construction, including nested template children, before `OnLoad`; zero is explicit and no byte adjustment is added. [EditBox text position and selection](../../specs/editbox-text-position-selection.md) is the SSOT. Source-only `c8d79f3bf` adds synchronous insertion `OnChar`; notification timing is defined by the owning contract. Keyboard selection creation/navigation remain unmodeled. This dispatch is simulator behavior; existing mouse focus behavior is unchanged.

**FontString fields** — `text`, `font`, `font_size` (default 14.0), `font_outline` (None/Outline/ThickOutline), `text_color` (default gold), `justify_h/v`, `word_wrap`, `max_lines`.

**Texture fields** — `texture`, `color_texture`, `vertex_color`, state textures (`normal_texture`, `pushed_texture`, `highlight_texture`, `disabled_texture`) each with `_tex_coords` companions, `tex_coords`, `atlas_tex_coords`, `blend_mode`, `nine_slice_layout/atlas`, `horiz_tile/vert_tile`, `is_mask`.

**Widget-specific** — Slider, StatusBar, EditBox, ScrollFrame, Cooldown each have dedicated field groups. Model-family state is grouped in one lazy `Option<Box<ModelWidgetState>>`; absent payloads preserve getter defaults, while mutating methods allocate only when needed and remain globally callable.

### Wrath EditBox history and presentation

`Frame.editbox_history` is `Vec<Option<String>>`: `None` is an unused slot and `Some("")` is a stored empty entry. A shared index is maintained by [history methods](../../../src/widget/editbox_history.rs). Wrath bindings use physical slots; other profiles retain their existing push/length/clear behavior. The [owning contract](../../specs/wrath-widget-labels.md) separates required Retail/PTR preservation and len-based storage-accounting evidence from native parity.

Wrath [direct arrows](../../../src/lua_api/key_dispatch/wrath.rs) bypass OnArrowPressed, reuse the common text-assignment path after history lookup, and leave callbacks outside the state borrow. XML attributes are applied base-to-derived before lifecycle scripts. Password rendering retains raw text/scalar cursor state while [presentation input](../../../src/widget/editbox.rs) produces byte-count stars and the corresponding display cursor. Wrapped visual-line/preferred-column behavior and word movement are not modeled in this slice.

[Tooltip stored-size investigation](../investigations/wrath-tooltip-stored-size.md) distinguishes parent-fill query geometry from retained texture dimensions; current Retail XML defaults remain unverified.

### CreateTexture draw-layer bounded runtime status

[Owning runtime status](../../specs/texture-draw-layer-validation.md#current-bounded-runtime-status) records 10 Retail/PTR outcomes, two separate startups, retained check/format caveats and OLD480 Mists observation. Public retention, fresh earlier-profile and native/render/full-parent acceptance remain open. The source checkpoint below remains historical.

### CreateTexture draw-layer source checkpoint — 2026-10-13

`6e085fce9` validates supplied ordinary draw-layer names for Retail/PTR before Texture construction and registration. VectorGraphics bypasses the new validation; earlier profiles retain their existing path. [Owning contract/status](../../specs/texture-draw-layer-validation.md) separates complete preproducer shared-fixture normal-Lua RED from authored, uncompiled Rust cases (`ab0`, formatting `6bdb02b06`). Producer GREEN remains pending; no native/render/full-parent acceptance is established.

### Actual texture file dimensions — bounded runtime proof

`271793bc6` adds Texture `GetFileWidth`/`GetFileHeight` in [texture adapters](../../../src/lua_api/frame/methods/widgets/texture/atlas.rs). Existing secret-origin readability is checked before resolving actual source metadata through a CPU-only TextureManager with configured addon roots; widget geometry is not the metadata source. No new widget state or GPU dependency. The temporary manager does not retain metadata between getter calls. [Texture-file spec](../../specs/texture-file-dimensions.md) owns source/ID/atlas policies and remaining coverage; [API evidence boundary](lua-api.md#current-pet-hover-and-follower-instance-models--bounded-runtime-proof) records historical six-case RED and sealed Retail `e3978fd48196` six-case GREEN, exit0. Local-file metadata, clearing, geometry independence and existing secret-read policy pass at that old revision; numeric-ID/atlas actual metadata and native parity remain unverified. `b003ec183` later contains both getters to Texture and gives WorldFrame a distinct cached index preserving other entries. [Receiver checkpoint](../investigations/method-dispatch-refactor.md#texture-receiver-containment--2026-10-12) owns default-fixture changes and current bounded receiver proof.

### Message history and Translation target state — 2026-10-16

`269ed6f17` extends existing `MessageFrameData` entries with separate access-ID/extraData metadata alongside line ID; it does not introduce a widget type. Both widget names share Rust storage, but Wrath plain MessageFrame no longer exposes scrolling-history methods. Its cached method index removes this message module's non-native own methods while retaining shared methods; unrelated universal-index leaks are outside this lane. XML intrinsic promotion reattaches the scrolling index. Retail method exposure and color tuples remain unchanged. Wrath ScrollingMessageFrame unfiltered/filtered reads return text/accessID/typeID/extraData, with zero selecting total history for reads but exact zero-ID removal. Native AddMessage positions do not depend on backFill being false; backFill ordering is outside this repair. Wrath MessageFrame additionally queues one insertion per update and stores per-row visible/fade timers; the update stage advances those timers before OnUpdate-handler dispatch, and rendering projects linear alpha. ScrollingMessageFrame retains timestamp fading. The [Wrath widget contract](../../specs/wrath-widget-labels.md) owns scheduling and acceptance limits. The [C11 contract](../../specs/message-frame-access-history.md) owns the Wrath-only transfer fixture and execution scope.

Translation writes existing target `anim_offset_x/y`, consumed by layout, with per-playback baseline restoration shared with Path motion. It does not mutate owner/sibling dimensions or turn Scale/Rotation configuration into visual effects. [Transform contract](../../specs/animation-transforms.md) owns the bounded unit-scale NONE-looping policy and receipts; [[lua-api]] links producer/storage paths. No native/render/full-lifecycle acceptance follows.

### VertexColor animation target state

The implemented texture-target path writes `Frame.vertex_color`; [animation state](../../../src/lua_api/animation.rs) stores per-playback baselines, and [application/restoration](../../../src/lua_api/frame/methods/button_anchor_hierarchy/animations/vertex_color.rs) updates the selected texture. Receiver repair `7ec9c049a` attaches four raw endpoint methods only to the `VertexColor` table at `CreateAnimation`, removing shared widget-index registration without changing this target state or introducing a widget category. Main reports seven receiver-stage PASS; later `a2a9606f9` extracts the attachment helper without changing target state. [Contract](../../specs/vertex-color-animation.md) and [existing animation section](lua-api.md#vertexcolor-animation--bounded-runtime-proof) own the evidence boundary: historical 6/6 GREEN at `080833f1d` and 64-extras receiver-leak RED remain unchanged; native discovery captures remain unchanged; the default method-comparison fixture separately gains two intended Texture file-getter extras. [Integrated checkpoint](../investigations/integrated-source-and-factory-proof-2026-10-09.md#five-producer-and-receiver-source-checkpoint--2026-10-12) distinguishes receiver-stage proof from current bounded GREEN/startup PASS, pending default check and unrun broader gates. No rendering or native acceptance is implied.

### ColorSelect RGB callback boundary

`3397eeec9` makes changed `SetColorRGB` commit RGB while retaining alpha, then dispatch `OnColorSelect(self, r, g, b)` through registered scripts and hooks; same RGB remains a no-event call. `69e4bd61d`/`579770559` load the six XML texture slots across ordinary, inherited-template, and runtime-template paths. Final bounded proof at `e586e6d30` covers 35 behavioral cases, including the unchanged Retail ColorPicker setup/hex-entry consumer with zero recorded Lua errors. Native HSV-only/alpha-only callbacks, reentry, GPU parity, and non-Retail profiles remain unverified. [ColorSelect RGB callback](../../specs/colorselect-rgb-callback.md) is the SSOT.

### Scripted Button click boundary

Public Lua `Button:Click()` first rejects a `ScriptedInput` forbidden aspect. It then toggles a CheckButton before the enabled/same-button recursion guard. A permitted click dispatches every registered `PreClick`, `OnClick`, and `PostClick` binding in order with `(self, mouseButton, down)`; omitted arguments become `LeftButton` and `false`. Handler errors reach the error handler without stopping later bindings or phases, and cleanup releases the recursion guard. Tests-only `8ff05b19a` is RED: six failures; two new and two existing controls pass. Its fixtures cover normal scripts and `HookScript`, not intrinsic-binding order. `/tmp/cross-version-button-click-proof.md` records the bounded evidence; independent post-change verification is pending. Wowless corroborates Button phase order, arguments, and error continuation but has no CheckButton override. Solarity corroborates toggle-before-guard but propagates handler errors. This does not claim native or physical-mouse-click parity. [Scripted Button clicks](../../specs/button-script-click.md) defines the contract.

### Button enabled-state callback boundary

At `1d989b0a3`, `Enable`, `Disable`, and `SetEnabled` retain their committed-state-before-callback, changed-state-only dispatch, and same-value no-op behavior, but route `OnEnable`/`OnDisable` through the existing ordered binding lookup: XML intrinsic precall, normal script plus hooks, then intrinsic postcall. Handler errors still report and later bindings continue. Tests-only `960c20b32` is RED: two intrinsic-binding dispatch cases fail; the normal hook-only control passes, so normal hooks were never omitted. XML `GetScript` binding queries establish that the tested precall/postcall bindings are registered. Intrinsic `HookScript` restrictions are unchanged. No native all-profile or vendor/UI claim. [Button enabled-state callbacks](../../specs/button-enabled-callbacks.md) is the bounded contract; independent post-change verification is pending.

### ScrollFrame script-binding boundary

`1c66baffc` registers XML `OnHorizontalScroll`, `OnVerticalScroll`, and `OnScrollRangeChanged` during ordinary and runtime-template construction. `2cc079507` dispatches registered precall, normal/hooks, and postcall bindings after existing commits, replacing normal-only lookup plus unregistered `*_Intrinsic` property calls. The unchanged EventScrollFrame XML declares intrinsic precalls whose methods emit callback-registry events. Original RED `1e4706e2a` / `c46c6f76d` was three missing registrations plus one bogus-property invocation, not dispatch-only. Final `c82ed91af` proof validates 60 unique relevant cases, including ordinary XML and cached EventScrollFrame paths with zero recorded Lua errors. `0c913d2ed` supplies inline body aliases before execution: `offset`, `xrange`/`yrange`, and `delta`, without consuming `...` or changing method/function bindings. Historical tests-only `6b63be464` was RED in three wrong-value cases. The [inline-scroll contract](../../specs/xml-scroll-arguments.md) now retains saved three-case PASS, separate from source equivalence and the enclosing suite FAIL. The three new alias cases are error-clean; the prior 46 behavioral passes had five FauxScrollFrame diagnostics, attributed to `function=""` resolving `_G` to a table rather than shared environment or ScrollBox state. `3e67e7b6e` has bounded default-profile GREEN proof: ordinary normal and explicit runtime precall/postcall clearing preserve other bindings, nonempty behavior, and cached FauxScrollFrame behavior across 47 test instances. The prior five table-call diagnostics are gone; one intentional sentinel diagnostic remains. Intrinsic-default selection, whitespace-only classification, and `method=` precedence are source-inspected only; native/full-profile behavior is unclaimed. Cached consumers use `offset`/`delta` (`FauxScrollFrameTemplateLight`) and `yrange` (Mists CharacterCreate). No physical-input, native, or vendor/UI claim. [ScrollFrame script bindings](../../specs/scrollframe-script-bindings.md), [Inline XML scroll arguments](../../specs/xml-scroll-arguments.md), and [XML empty script-function clearing](../../specs/xml-empty-script-clearing.md) are the bounded contracts.

### ScrollFrame requested-offset boundary

`SetHorizontalScroll` and `SetVerticalScroll` store the requested offset without range clamping. Commit `e9b72b107` preserves existing changed-state callback delivery after storage and same-offset suppression; `ee25b7d62` supplies the regression cases. Cached client observations record round-trips for vertical `-50` and horizontal `999`; no fresh probe ran. RED 0/2 becomes independent GREEN six ScrollFrame cases and 15 shared widget controls, with format/check/readability passing; `/tmp/cross-version-scroll-offset-verification-ledger.md`. That proof excludes renderer movement, covered separately below; implicit range-refresh timing remains unverified. [ScrollFrame offsets](../../specs/scrollframe-offsets.md) is the contract and scope.

### ScrollFrame child-ownership boundary

`07afe68ae` clears the current designation and detaches its old child with shared `reparent_widget` before assigning a different child; nil takes the same clear path. The helper invalidates old-child layout and presentation before replacement assignment, while reassigning the same child avoids detachment. Tests-only `3eea04853` is RED for two ownership failures with one same-child control passing; independent post-change proof is pending. Local Wowless `data/uiobjects/ScrollFrame/SetScrollChild.lua` corroborates only the unparent-before-replace model, not native callbacks or anchor-reset behavior. Unrelated custom Lua `ScrollChild` properties remain unchanged. [ScrollFrame child ownership](../../specs/scrollframe-child-ownership.md) is the bounded contract.

### ScrollFrame presentation boundary

Commits `08c43a536` and `4c39f4a5a` leave stored anchors and logical layout immutable, then apply a shared presentation translation at each ScrollFrame edge whose `scroll_child_id` names the current descendant. The designated subtree moves once per crossed viewport, including externally anchored descendants; nested viewports inherit outer translation but apply their own offset only below their own scroll child. The transform uses the crossed scroll child's effective scale. [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) is the bounded render/input contract. `ed69bd136` records three RED cases. Final independent verification passes 52 scoped cases, including scaled-child quads, nested cached hits and both 20,000-frame depth controls; format/check pass. See `/tmp/cross-version-scroll-presentation-final-verification-ledger.md`. Live GPU pixels and all UI/native behavior remain unclaimed.

`b6bb2f710` applies that same translation to public `Region:IsMouseOver` bounds after its logical-rect, visibility, and mouse-enabled guards and before bounds/margin comparison. `GetRect` stays logical and optional margins remain preserved. This adds neither viewport clipping nor intersection semantics to the query; native behavior remains unverified. Independent verification passes the bounded query scope; [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) records its 35-case and format/check evidence separately from the 52-case presentation proof. Cached MapCanvas queries its viewport rather than a scrolled child, so it is not evidence of this mismatch.

Ownership when `SetScrollChild` replaces or clears its designation is outside this presentation-only boundary: [ScrollFrame child ownership](../../specs/scrollframe-child-ownership.md) covers the pending detach contract. It makes no native callback or anchor-reset claim.

## Bound Slider thumb layout — 2026-10-15

Updated bounded source136 model proof: [owning contract](../../specs/slider-bound-thumb-layout.md), [central receipt](../../baselines/README.md#verified-scoped-receipts--2026-10-15). This does not cover the unbuilt current import change or native/cross-profile/full-parent acceptance.

`2fc87822e` adds engine geometry for sized, unanchored Textures designated as their parent Slider's thumb. `src/layout_slider_thumb.rs` preserves dimensions/scale, centers the cross-axis and positions contained travel by value: right horizontally, down vertically. Contained endpoints, oversized-thumb centering and collapsed/non-finite normalization are simulator policy, not native measurements.

Query and render admission share positive finite resolved-rectangle eligibility; binding identity alone is insufficient. Value/range/orientation/binding/size mutations invalidate layout, including geometry queried synchronously inside `OnValueChanged`. Explicit anchors remain authoritative; ordinary, formerly bound and unsized unanchored textures remain unresolved.

`623294944` strengthens callback proof: capture `GetCenter()` inside `OnValueChanged`, then assert the observed new position after `SetValue` returns, outside protected dispatch. A precondition establishes movement from above to below midpoint; callback failures cannot silently satisfy the outer assertion. This is a test change, not new runtime callback behavior.

Historical pre-fix GPU evidence records zero thumb pixels and `THUMB_NO_ANCHOR_RED` stderr. The [combined feedback GPU receipt](../../rendering-pipeline.md#combined-feedback-gpu-receipt--2026-10-15) now records 432 magenta pixels (18×24), a green final-assertion marker and no exec-Lua errors at sealed source head `623294944`. Independent saved-capture verification PASS; existing startup errors remain. The [owning contract](../../specs/slider-bound-thumb-layout.md) retains broader public Lua/Rust and mutation gates. No native endpoint parity, clean startup or full-test GREEN is claimed.

## WidgetType Enum (18 types, `src/widget/mod.rs`)

Frame, Button, FontString, Texture, EditBox, ScrollFrame, Slider, CheckButton, StatusBar, Cooldown, Model (stub), ModelScene (stub), PlayerModel, ColorSelect, MessageFrame, SimpleHTML, GameTooltip, Minimap.

[[simple-html]] describes the bounded structured block/font-role/anchor implementation; plain GetText projection remains separate. [Owning contract](../../specs/simple-html-structured-content.md) and [proof limits](../../baselines/approved-batch-2026-10-16.md) exclude full HTML/native acceptance.

`from_str()` maps WoW aliases: "ItemButton" → Button, "ScrollingMessageFrame" → MessageFrame.

## WidgetRegistry (`src/widget/registry.rs`)

```rust
pub struct WidgetRegistry {
    widgets: HashMap<u64, Frame>,
    names: HashMap<String, u64>,
    render_dirty: Cell<bool>,
}
```

`get_mut()` sets `render_dirty`. `take_render_dirty()` atomically checks and clears. `would_create_anchor_cycle()` is a BFS reachability check used by `SetPoint`.

### Public parent-cycle boundary

Lua `SetParent` keeps its protected-state and forbidden-aspect guards before it calls the shared `methods_hierarchy::would_create_parent_cycle` helper. That helper walks the proposed parent's `parent_id` ancestry and rejects a self or descendant edge before animation reparenting or `reparent_widget` mutates the registry. Commit `818fe8d59` has RED evidence for both invalid graphs and GREEN evidence for those two rejections plus ordinary reparent, nil-parent, same-parent, and region-enumeration controls (4/4). The helper assumes an acyclic existing parent chain; it neither validates every malformed model graph nor covers other parent writers. The simulator error is tested only as containing `cycle`, not as native WoW wording.

### Public SetParent visibility boundary

After the existing protected-state, forbidden-aspect, and cycle guards, public Lua `SetParent` compares the moved frame's effective visibility before and after hierarchy mutation. When it changed, `676cda72e` recursively dispatches `OnShow` or `OnHide` children first without mutating any frame's local `visible` flag. Locally hidden descendants receive no callback. Reentry validates the expected parent and effective visibility before each callback, so a redirected reparent suppresses stale delivery; an erroring child binding still allows the parent binding to run. `/tmp/cross-version-reparent-final-targeted.log` is GREEN 6/6, including those cases and existing self/descendant cycle controls. Independent verification after `1fcdb52b2` passes 19 scoped cases plus format/check/readability; exact scope is recorded in `/tmp/cross-version-reparent-visibility-proof.md`. Wowless is source corroboration, not a native probe; other parent writers remain outside this public method boundary.

### Storage accounting

`Frame::storage_estimate_bytes()` accounts for inline state, registry/index capacities, and owned collection data. The lazy model payload contributes its boxed allocation and owned capacities only when present. The settled full-game fixture has 45,002 frames and estimates **213,058,036 bytes**, below the unchanged **230,000,000-byte** budget; the prior inline model state estimated **239,185,088 bytes**.

## Default Children (`src/lua_api/globals/create_frame.rs`)

**Button/CheckButton** — 4 Textures (NormalTexture Artwork, PushedTexture Artwork, HighlightTexture Highlight additive, DisabledTexture Artwork) + 1 FontString (Text at Overlay). All textures fill the button via TOPLEFT+BOTTOMRIGHT anchors. `mouse_enabled = true`.

**Slider** — FontStrings: Low, High, Text; Texture: ThumbTexture.

**ItemButton** — 6 Textures (icon, searchOverlay, IconBorder, IconOverlay, IconOverlay2, ItemContextOverlay) + 2 FontStrings (Count, Stock).

**GameTooltip** — Inserts tooltip data into `SimState.tooltips`; sets strata to TOOLTIP.

## Strata Before/After Observations

- The retail capture recorded `HasFixedFrameStrata() == false` for the tested XML strata literals, including `PARENT`.
- During XML `OnLoad`, a direct `PARENT` child and its actual parent both reported `DIALOG`; an explicit literal sibling reported `LOW`. That equality does not identify why values match.
- Under an actual `DIALOG` parent, the base `HIGH` instance reported `HIGH`, the derived literal `LOW` instance reported `LOW`, and the derived `PARENT` instance reported `HIGH`.
- After parent `SetFrameStrata("LOW")` and, separately, reparenting to a `LOW` parent, every tested non-fixed direct child and grandchild reported `LOW`, including explicit XML `MEDIUM` fixtures. These snapshots do not establish an internal propagation mechanism.
- Runtime-fixed retail behavior using `SetFixedFrameStrata(true)` remains unproven.

## Button Text Rendering and Three-Slice Issue

Button text is emitted twice: once by `build_button_quads()` as part of the non-region frame (renders before all child regions), and once by the child `Text` FontString at Overlay draw layer (renders after Artwork textures).

Three-slice buttons (ThreeSliceButtonTemplate) define their background as child Textures at BACKGROUND draw layer. These render after the button frame but before the child FontString — meaning the step-1 text gets covered. The child FontString should fix this, but its default CENTER anchor gives it zero width, so it renders nothing.

**Fix**: replace the child Text FontString's single CENTER anchor with fill-parent anchors (`add_fill_parent_anchors()`), so `resolve_multi_anchor_edges` computes proper width from the parent button bounds.

## Clean-room control and anchor follow-ups — 2026-10-15

[Bounded Mists/Era/Anniversary owner matrix](../../baselines/README.md#mists-shared-controls--2026-10-16) owns separately accepted saved execution, provenance and diagnostics; [Anniversary receipt](../../baselines/README.md#anniversary-shared-controls--2026-10-16) adds the same39 logical cases with exact recipe-name equality and unchanged original-CVar guards. Broader gates remain open. Accepted PTR/Forever evidence is unchanged.

User functional requirements define shared simulator policies for Wrath and Retail, not native parity; no reconstructed implementation is a source. Later bounded [PTR](../../baselines/README.md#ptr-shared-controls--2026-10-16) and [Forever](../../baselines/README.md#forever-shared-controls--2026-10-16) receipts supplement the historical matrix below without current-whole-source, startup or native acceptance. Owning specs retain algorithm/behavior details:

| Scope | Owning contract | Current proof |
|---|---|---|
| C1 step assignment | [Slider callbacks](../../specs/slider-value-callback.md#clean-room-step-assignment-follow-up) | Saved Retail `05f1b4a62`: five unique PASS/exit0, all assertions reached; independent saved-runtime audit PASS. Saved Wrath `39d38`: C1 5 PASS; [Wrath checkpoint](../../specs/editbox-text-position-selection.md#saved-bounded-wrath-checkpoint) owns pending audit/hash qualifications. No current-HEAD/all-profile acceptance. |
| C2 cursor geometry | [EditBox geometry](../../specs/editbox-text-position-selection.md#cursor-geometry-follow-up) | Earlier saved layout/preservation/key proof retained in owner. Saved `03769bb00` XML binding and genuine cached Retail consumer independently accepted; [shared receipt](../../specs/editbox-text-position-selection.md#saved-xmlscroll-and-genuine-consumer-acceptance) owns reached counts, historical RED resolution and buffering audit. Wrath guard intact/shared acceptance OPEN; ESC diagnostic unfixed, no broad C2 GREEN. |
| C3 highlight endpoints | [EditBox selection](../../specs/editbox-text-position-selection.md#tests-asserting-this-spec) | Saved Retail source `3fa90ab7b`: four unique PASS/exit0, all assertions reached; independent saved audit PASS. Historical three valid REDs and wrong fixture oracle remain separate. Saved Wrath `39d38`: C3 4 PASS; shared criterion open pending current relevant-fix/profile proof and runtime audit. |
| L1 protected-recipient targets | [Anchor target policy](../../specs/protected-anchor-targets.md) | Saved Retail `05f1b4a62`: five unique PASS/exit0, all assertions reached; independent saved-runtime audit PASS. Saved Wrath `39d38`: L1 5 PASS, runtime audit pending; current-HEAD/all-profile criteria remain open. |
| C8 range lifecycle | [Slider range owner](../../specs/slider-value-callback.md#c8-changed-range-callbacklifecycle-candidate) | Saved Retail `d16876fcc`: five unique PASS/exit0 with independent saved-runtime/build/seal verification. Saved Wrath `39d38`: C8 5 PASS, runtime audit pending. Earlier Retail RED retained; current-HEAD/all-profile acceptance open. |
| C7 active button font | [Button font owner](../../specs/button-active-font-state.md) | Saved Retail `3a530`: corrected module 12 PASS/0 FAIL (three C7, nine controls). Independent Retail validation COMPLETE. Saved Wrath module 1 PASS/11 SETUP FAIL (all three C7 blocked by missing TOC), not font RED; [owner](../../specs/button-active-font-state.md#saved-wrath-setup-boundary) retains boundary. T6 excluded. |
| C9 range update pass | [ScrollFrame timing owner](../../specs/scrollframe-script-bindings.md#c9-range-update-boundary) | Earlier bounded C9 proof retained. Saved `03769bb00` independently accepts broader scroll preservation including `d374c9520` timing-reconciled XML fixtures; exact payload/global assertions intact. [Shared receipt](../../specs/editbox-text-position-selection.md#saved-xmlscroll-and-genuine-consumer-acceptance) owns totals, expected sentinel and historical REDs. No shared-Wrath/full-parent closure. |

[Shared Wrath checkpoint](../../specs/editbox-text-position-selection.md#saved-bounded-wrath-checkpoint) owns saved C2/XML/C9/preservation results, pending audit5795 and integration-hash recovery qualification. [Separate source0184775 receipt](../../specs/editbox-text-position-selection.md#saved-wrath-byte-limit-and-bare-font-execution--source0184775) now accepts bounded keyboard28/EditBox15/bare-C7 3 PASS, exit0, with independent build/seal/runtime audits and retained diagnostics/warning; historical `39d38` byte-limit RED/cache failures and audit qualifications remain. Later [bounded Wrath guard-retirement acceptance](../../specs/editbox-text-position-selection.md#wrath-nil-field-guard-retirement--accepted-bounded) supersedes empty-cache/unexecuted-consumer status and the original cursor-startup blocker; unrelated startup errors remain. Saved build/seal consistency is not current-code runtime acceptance. Owning specs retain exact boundaries; no new native, current-HEAD startup or full-parent completion claim.

## Region alpha numeric range

Stored alpha is `f32`; ordinary Retail/PTR numeric validation precedes narrowing and mutation. [Owning contract/status](../../specs/region-alpha-numeric-range.md) records five unique cases/profile, two separate startups, legacy observation and remaining acceptance limits.

## Visibility

`Show()` fires `OnShow` recursively on visible children. `IsVisible()` walks the parent chain; `IsShown()` checks only the frame's own `visible` flag.

## Sources

- [Wrath widget contracts](../../specs/wrath-widget-labels.md) — profile-gated history, presentation, tooltip and MessageFrame producers
- [Tooltip stored-size investigation](../investigations/wrath-tooltip-stored-size.md) — false oracle and separate cached-anchor failure
- [Message-frame access history](../../specs/message-frame-access-history.md) — filtered history contract and receipts
- [Animation transforms](../../specs/animation-transforms.md) — Translation presentation state and bounded receipts

- [Slider value callbacks](../../specs/slider-value-callback.md) — shared step follow-up and pending proof
- [Protected-recipient anchor targets](../../specs/protected-anchor-targets.md) — shared eligibility policy and pending proof

- [Bound Slider thumb layout](../../specs/slider-bound-thumb-layout.md) — geometry, admission, invalidation and pending proof

- [Region alpha numeric range](../../specs/region-alpha-numeric-range.md) — ordinary numeric contract and bounded retained status

- [FrameStrataProbe](../../../docs/addons/FrameStrataProbe/README.md) — retail XML `PARENT`, template comparison, and before/after operation observations
- [widget-system.md](../../widget-system.md) — Frame struct, lazy model payload, WidgetType, WidgetRegistry, default children, strata
- [frame.rs](../../../src/widget/frame.rs) — Frame storage and model-state accessors
- [frame_size.rs](../../../src/widget/frame_size.rs) — registry storage estimate and boxed payload accounting
- [button-text-rendering.md](../../button-text-rendering.md) — three-slice rendering order problem and fix
- [EditBox focus callbacks](../../specs/editbox-focus-callbacks.md) — Lua focus-transition contract and limits
- [EditBox text position and selection](../../specs/editbox-text-position-selection.md) — byte offsets, selected edits, and proof limits
- [SetParent parent cycles](../../specs/set-parent-cycles.md) — public cycle-rejection contract and limits
- [Visibility script dispatch](../../specs/visibility-script-dispatch.md) — public reparent visibility callbacks and proof limits
- [ScrollFrame offsets](../../specs/scrollframe-offsets.md) — requested-offset, callback, and range boundary
- [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) — presentation-only descendant translation and bounded proof
- [ScrollFrame child ownership](../../specs/scrollframe-child-ownership.md) — designated-child replacement and clearing boundary
- [Inline XML scroll arguments](../../specs/xml-scroll-arguments.md) — inline callback locals, preserved varargs, and bounded saved proof
- [Scripted Button clicks](../../specs/button-script-click.md) — public click lifecycle and bounded evidence
- [Button enabled-state callbacks](../../specs/button-enabled-callbacks.md) — transition callback binding order and pending verification
- [ColorSelect RGB callback](../../specs/colorselect-rgb-callback.md) — RGB callback boundary and pending consumer/loader proof

## See Also

- [[wrath-tooltip-stored-size]] — XML parent-fill versus stored texture dimensions
- [[layout-system]] — uses Frame.anchors to compute screen positions
- [[rendering-pipeline]] — dispatches quad emission per WidgetType
- [[event-system]] — Frame.registered_events, script handler storage, and focus callback dispatch
- [ScrollFrame offsets](../../specs/scrollframe-offsets.md) — public offset setter contract and proof status
- [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) — presentation-only scroll-child subtree contract
- [ScrollFrame child ownership](../../specs/scrollframe-child-ownership.md) — pending designated-child ownership contract
- [[lua-api]] — Lua method surface
- [EditBox text position and selection](../../specs/editbox-text-position-selection.md) — selected keyboard-edit boundary
- [Scripted Button clicks](../../specs/button-script-click.md) — public Button/CheckButton click lifecycle
- [Button enabled-state callbacks](../../specs/button-enabled-callbacks.md) — public enabled-state transition callbacks
- [ColorSelect RGB callback](../../specs/colorselect-rgb-callback.md) — changed-RGB callback and pending loader evidence
