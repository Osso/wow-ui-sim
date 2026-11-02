# Additional control, text and tooltip discrepancies

Static comparison at the [expanded-pass revisions and limits](additional-rendering-and-map.md). No fixture was executed. Historical return types, API availability and timing must be checked against the supported profile before implementation. Findings below describe actual data/control/render paths, not mere method presence.

## C4 — Password EditBox renders plaintext

**External:** `Engine/Source/Frame/CSimpleEditBox.cpp:2519-2531`, `UpdateVisibleText`, selects an asterisk buffer for display when password flag `0x8` is enabled, retaining the original stored text.

**Simulator:** `src/lua_api/frame/methods/widgets/editbox.rs:319-327` stores `editbox_password`. Actual EditBox rendering routes through `src/iced_app/quad_builders.rs:356-362` to `quad_builders_button.rs:463-479,523-564`, which emits `f.text` directly without checking the password flag.

**Counterexample:** `SetText("s3cr3t"); SetPassword(true)` retains `GetText()=="s3cr3t"` on both, but reconstruction displays six masks and simulator emits plaintext.

**Rework/regression:** mask display input without replacing stored text; use display metrics consistently for caret/selection. Assert visible glyph/pixel masking plus unchanged `GetText`, false/true transitions and caret behavior. ASCII fixture proves the basic omission; exact Unicode/IME masking policy remains unresolved, so do not blindly copy the reconstruction's byte-count masking.

## C5 — History getter reports occupancy instead of configured capacity

**External:** `Engine/Source/Frame/CSimpleEditBox_Scripts.cpp:527-532`, returns `m_historyLines`. `CSimpleEditBox.cpp:1492-1504`, uses that value as fixed history capacity, storing nothing when zero.

**Simulator:** `src/lua_api/frame/methods/widgets/editbox.rs:624-659` already separates `editbox_history` from `editbox_history_max`, but `GetHistoryLines` returns vector length. Zero capacity means unlimited in this model (`src/widget/frame.rs:432-434`), unlike the reconstruction. This explicit current model policy is a compatibility question, not proof of a universal defect.

**Counterexample:** `SetHistoryLines(3); AddHistoryLine("x"); GetHistoryLines()` yields reconstructed 3 versus simulator 1. Capacity and occupancy are different observable facts.

**Rework/regression:** resolve the target API's capacity/getter/zero policy; if historical behavior applies, return configured capacity and enforce its ring/history behavior. Test capacity 3 with zero, one and four additions and a capacity change after population; keep a separate explicit zero-capacity case. Do not derive history policy from text-length APIs merely because both use a zero default.

## C6 — Changing message visibility duration does not reset active message age

**External:** `Engine/Source/Frame/CSimpleMessageFrame.cpp:46-58`, assigns the new `m_timeLeft` to each active row when `SetTimeVisible` is called.

**Simulator:** `src/lua_api/frame/methods/widgets/message_frame/getters.rs:109-116` changes only shared duration. `src/lua_api/message_frame.rs:5-34` retains message timestamps; actual `src/iced_app/message_frame_render.rs:82-101` computes age from the message/override timestamp. The setter does not update the fade origin.

**Counterexample:** enable fading, set initial visibility 100 seconds, add a message at t=0, then at t=5 set visibility to 30. Reconstructed remaining time resets to 30, so fade starts near t=35; simulator retains original age and starts near t=30. Fade duration is additional; this is not an assertion that both remove the row instantly at those times.

**Rework/regression:** if selected by the supported profile, reset the appropriate active per-record fade origin/remaining lifetime rather than merely replacing a shared duration. Use controlled clock tests around both deadlines, multiple differently aged records, and fading-disabled controls. Do not infer complete message-queue timing parity from this one setter.

## H1 — SimpleHTML loses document structure, styles and hyperlink hit regions

**External:** `Engine/Source/Frame/CSimpleHTML.cpp:372-410,423-495`, `SetText`, parses HTML/BODY and heading/paragraph/line-break/image nodes, with hyperlink buttons for parsed links. Plain-text behavior is a parse-failure path, not the whole renderer. `CSimpleHTML_Scripts.cpp:240-249`, refreshes hyperlink regions when enablement changes.

**Simulator:** `src/lua_api/frame/methods/text_attribute_event/text.rs:170-191` strips tag spans rather than creating document nodes. `src/lua_api/simple_html.rs:37-55` stores style preferences but no parsed document/link regions. Actual `src/iced_app/quad_builders.rs:352-354` routes SimpleHTML through generic FontString emission using `text_stripped`.

**Counterexample:** `<HTML><BODY><H1>Title</H1><P>See <A href="spell:123">Spell</A><BR/>next</P></BODY></HTML>` becomes `TitleSee Spellnext` in simulator's stripped stream; heading/paragraph/break distinctions and clickable link regions are lost. Exact reconstructed pixel positions were not measured.

**Rework/regression:** parse the supported document subset into styled layout and link/image regions, connect per-element settings and hyperlink enablement to it, and preserve text semantics. Test concrete heading/body font differences, line breaks, image geometry and click/hover targets—not just a stripped text getter or a true `GetHyperlinksEnabled` result. Unsupported modern HTML extensions must not be invented from this source.

## U1 — Tooltip textures use the wrong attachment and argument model

**External:** `WoW/Source/UIUtil/Tooltip_Scripts.cpp:1343-1374`, accepts filename plus optional UVs. `Tooltip.cpp:3773-3817`, attaches a texture to the last existing text row, with legacy row-count/texture-count prerequisites, and applies UVs.

**Simulator:** `src/lua_api/frame/methods/widgets/tooltip/line_frames.rs:245-280` accepts numeric/FileDataID-like input, ignores UV arguments and appends a new empty text/texture row. A filename path is silently ignored. FileDataID support itself may be a valid modern extension; replacing it with a legacy-only form is not the proposed fix.

**Counterexample:** after two text rows, add an existing filename texture with non-default UVs. Reconstruction associates it with the last row; simulator rejects that filename. For accepted numeric input, simulator adds a separate row rather than an inline attachment. This does **not** claim every tooltip icon is invisible: child Texture rendering was not fully traced, and absence from a text-only render projection is insufficient evidence.

**Rework/regression:** establish accepted argument forms and inline association for each target profile, preserve UV/size/row attachment in actual state and layout, and test a two-row tooltip's icon geometry/pixels. Include full/cropped UV controls. Do not copy legacy prerequisites/limits into modern profiles without evidence.

## U2 — Clearing an empty tooltip emits an extra callback

**External:** `WoW/Source/UIUtil/Tooltip_Scripts.cpp:187-191`, calls `ClearTooltip`; `Tooltip.cpp:895-954` records whether there were lines and invokes `OnTooltipCleared` only when that predicate was true.

**Simulator:** `src/lua_api/frame/methods/widgets/tooltip/line_data.rs:19-39` unconditionally fires the callback after clearing modeled data.

**Counterexample:** new empty tooltip, installed counter handler, `ClearLines()`: reconstructed counter stays zero, simulator increments. A populated tooltip invokes the callback on both paths.

**Rework/regression:** determine target-profile callback conditions, then dispatch only at the proper state transition. Assert empty/populated/repeated clears and callback-observed cleared state; preserve [existing owner lifecycle requirements](../specs/tooltip-owner-lifecycle.md), which do not establish native callback timing for every case. This is a historical callback discrepancy, not permission to rewrite ownership behavior.

## U3 — Double-column tooltip wrapping remains enabled when right text exists

**External:** `WoW/Source/UIUtil/Tooltip.cpp:1568-1584`, `AddLine`, forces `lineWrap=0` when nonempty right-column text is present, regardless of requested wrap.

**Simulator:** `src/lua_api/frame/methods/widgets/tooltip/line_data.rs:117-179` stores the caller's wrap flag unchanged for double lines; actual `src/iced_app/tooltip.rs:392-420,497-564` uses it for left-column layout.

**Counterexample:** a constrained tooltip with a long left string, nonempty right string and final wrap argument true can wrap in simulator while the reconstruction disables wrapping for that row.

**Rework/regression:** resolve the supported profile's two-column overflow policy and apply it in layout; test long-left/nonempty-right, empty-right and single-column controls with concrete rendered dimensions. RGB triplet argument positions are **already matched**: the external colour helper reads three consecutive components from the starting argument, so the earlier suspected colour-parsing defect was rejected.

## F1 — `SetAllPoints` bypasses protected mutation and named-target validation

**External:** `Engine/Source/Frame/CScriptRegion_Scripts.cpp:550-594`, checks `ProtectedFunctionsAllowed`, resolves names/tables, and errors on missing/self/cyclic targets before applying anchors. `CScriptRegion.cpp:104-109`, resolves a name through the actual script-object lookup rather than substituting the parent.

**Simulator:** `src/lua_api/frame/methods/button_anchor_hierarchy/anchors.rs:651-691` checks cycles, but omits the caller-protection guard used by adjacent `SetPoint`; unrecognized string targets fall through to the parent instead of resolving/erroring. This is separate from L1's target-kind eligibility.

**Counterexamples:** mutate a protected frame via `SetAllPoints` in an insecure blocked context; reconstruction returns without anchor mutation, whereas the inspected simulator method has no matching authorization check. Passing an unknown named target errors in the reconstruction but substitutes the parent in simulator. The exact explicit-nil/screen-root geometry policy is not asserted here.

**Rework/regression:** enforce the target's mutation authorization and name-resolution/error policy before touching anchor dependencies. Test blocked mutation preserves prior points, missing-name errors, valid named/table targets and cycle controls. Retain actual modern forbidden-aspect/layout requirements rather than substituting only a legacy security check.

## F2 — Move/resize starts do not establish the full script-driven interaction

**External:** `Engine/Source/Frame/CSimpleFrame_Scripts.cpp:1336-1392`, errors on immovable/non-resizable frames and starts a single move/resize transaction using current cursor/frame mouse position when no transaction is active.

**Simulator:** `src/lua_api/frame/methods/misc/drag_input.rs:141-165` silently ignores invalid preconditions and otherwise only sets per-frame flags. `src/iced_app/mouse.rs:87-105` processes movement only when `active_drag_frame` already exists; the start methods do not establish it. Actual dragging/resizing helpers do exist (`mouse_drag.rs:20-67,104-151`), so this is **not** a claim that all pointer dragging is absent.

**Counterexamples:** calls on non-movable/non-resizable frames return normally instead of erroring. Start a script-driven move outside an existing drag transaction and then supply pointer motion: setting the flag alone does not select an active drag frame. Multiple independently flagged frames also do not model the source's single interaction start policy.

**Rework/regression:** connect public script-driven start/stop methods to the input model's transaction/cursor baseline and enforce agreed preconditions/exclusivity. Test through actual pointer dispatch, including existing OnDragStart flows, starts outside that flow, correct sizing corner and stop/user-placement behavior. Do not implement duplicate movement logic beside the working drag helpers.

## Unresolved legacy surface and policy leads

Direct legacy `SetBackdrop`/backdrop-colour behavior differs from the original client: the source has 2D backdrop methods while simulator Rust has no `SetBackdrop` registration and classic colour methods are no-ops (`src/wrath/frame_methods.rs:42-52`). Modern Blizzard mixins/templates and per-profile API removal must be traced before declaring an actual supported-profile method missing. This is not excluded 3D rendering, but remains a profile/exposure lead rather than a counted new requirement. `RegisterAllEvents` event-set semantics and effective `SetClampedToScreen` behavior also remain incompletely compared.
