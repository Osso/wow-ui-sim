# Cataclysm 15595 follow-up

Static comparison only; no reconstruction code was executed, configured, built or imported. [Audit interpretation](README.md) and [security limits](security.md) apply. This supplement does not change simulator requirements or implementation.

## Provenance and evidence levels

| Source | Pinned revision | Declared target |
|---|---|---|
| `~/Repos/wowdecomp15595` | `959b0b85c5f55c565328d7c3ca33027a65e41327` | Root `README.md:3,9–16`: Cataclysm 4.3.4, build 15595, x86/x64 |
| `~/Repos/wowdecomp` | `0e4e9fe0a4a9fa08027c5eed4b2c5e873739cfd3` | 3.3.5a, build 12340 |
| Simulator | `5c3b7bd54227d036bc81244bec0ef920b4a6c0a2` | Shared implementation at follow-up baseline |

15595 `README.md:39–52` defines ordinary paired offsets as **x86/x64 addresses in 15595**, not old/new client addresses. `%` prefixes identify 13623 carryovers; `#` prefixes identify 12340 carryovers. An unprefixed address is the author's attribution, not independent binary verification. Matching text across these related reconstructions is not two independent native observations. The older registration inventory remains the coverage denominator; this is not a full 15595 export inventory.

## C10 — EditBox text notifications cross different mutation/update boundaries

**Historical baseline gap report superseded within the reported shaped/deferred implementation scope.** The [exchange checklist](README.md#exchanged-implementation-checklist) owns cross-host Retail/Wrath results and the still-open genuine Wrath consumer. Later implementation and exact mixed-origin/reentrant corner-case coverage were not independently inspected here; the trace/fixtures below retain their original baseline attribution.

**Kind:** existing contract conflict for changed setters/keyboard edits; missing notification capability for public `Insert`. The difference is concrete in source, but historical evidence alone does not authorize replacing the [current EditBox contract](../specs/editbox-text-position-selection.md).

**Reconstructed producer:** 15595 `Engine/Source/Frame/CSimpleEditBox_Scripts.cpp:273–297` routes Lua `Insert` to `Insert(...,1,0,0)` and `SetText` to the setter. `CSimpleEditBox.cpp:1963–2081`, mutates text/cursor metadata, marks text/cursor dirty, and invokes `OnChar` synchronously when `runScripts` is enabled. `:2371–2387`, makes changed `SetText` call `Insert(...,0,0,1)`, then synchronous `OnTextSet`; same-value text returns early. Accepted characters use insertion (`:2223–2235`); deletion also marks text dirty (`DeleteSubstring`, `:593–684`).

**Delivery:** `CSimpleEditBox.cpp:2579–2609`, snapshots text dirtiness, refreshes visible text/highlight/cursor, then runs `OnTextChanged`. `:1786–1799` reads `userInput` from dirty bit `0x8`. Layer update calls this processor at `:2724–2727`; key-down processing also flushes pending state before handling its key (`:2761–2778`), as does mouse-down (`:2987–2994`). Therefore this is deferred to **dirty-state processing**, not an unconditional next-frame guarantee. The mouse-down call is not a post-key-handler flush.

**Older counterpart:** 12340 `CSimpleEditBox_Scripts.cpp:303–326`; `CSimpleEditBox.cpp:2049–2171` insertion, `:2318–2333` character input, `:636–727` deletion, `:2468–2483` setter, `:2685–2715` dirty processing, `:2829–2832` layer update, `:2868–2885` pre-key flush, `:3094–3102` mouse-down flush. Both source trees show the same bounded delivery model.

**Simulator difference:** `src/lua_api/frame/methods/text_attribute_event/text.rs:74–97` dispatches changed setter `OnTextSet`, then `OnTextChanged(false)` before returning. `src/lua_api/key_dispatch.rs:337–357,392–406` dispatches accepted keyboard edit notifications immediately after mutation/character scripts. Public `Insert` at `src/lua_api/frame/methods/widgets/editbox.rs:777–796` only updates text/cache/caret and produces neither `OnChar` nor a pending changed notification. Callback acceptance is not a producer. C2 separately owns missing cursor geometry production.

### Coalescing and mixed origins

For ordinary non-reentrant handlers, multiple accepted mutations before one dirty-state flush produce **one** `OnTextChanged`, observing final text. The source does not queue one notification per edit or OR together all edit origins:

| Mutations before the flush | Pending `userInput` |
|---|---|
| Keyboard insertion → changed `SetText` | `false` |
| Changed `SetText` → keyboard insertion | `true` |
| Multiple ordinary keyboard insertions | `true` |
| Lua `Insert` | `true` despite being called from Lua |
| Same-value `SetText` after an edit | Previous bit retained |

Insertion sets `0x8` for `script=0` and clears it for `script=1` (`15595 :2014–2020`; `12340 :2101–2109`). Scripted deletion alone does not clear an existing bit; non-script deletion sets it (`12340 :716–722`). Thus a generic “last API invocation wins” rule would be wrong. Formatting, numeric validation and markup may reject a path before insertion's bit update; no claim that every attempted call replaces the origin bit. Reentrant edits from cursor/text handlers need separate tests: the bit is read at callback delivery and cleared after the changed callback, not stored in an immutable queued payload.

**Discriminating fixtures / proposed regressions:** attach logging callbacks and a final-text getter. In one Lua chunk call `SetText("a")`, then `SetText("b")`: source-derived prediction is synchronous `OnTextSet` twice, no changed callback before the chunk returns, then one changed callback with `"b", false` at a flush; current simulator emits two changed callbacks synchronously. Separately call `Insert("x")`: reconstruction runs `OnChar` synchronously and later changed with `true`; simulator produces neither. Exercise mixed keyboard/setter edits through real input dispatch, ensuring no key/mouse flush intervenes; assert final text, callback count, origin and geometry-before-changed ordering. Include same-value setters, deletion, rejected input and callback-induced edits. These are proposed executions, not observed results.

**Required rework if adopted:** unify accepted edit paths around actual pending text/cursor state and source-matched flush boundaries; keep character and setter callbacks distinct. Do not synthesize cursor geometry inside setters merely to preserve current changed-callback timing. Existing user-selected length-limit policy and modern-profile contracts remain separate decisions. Neither reconstruction's EditBox table registers `SetFormattedText`; simulator exposes its shared setter path, so its native availability/timing is not established by this comparison.

## C11 — ScrollingMessageFrame loses access-ID filtered counts

**Open in the [exchange checklist](README.md#exchanged-implementation-checklist):** peer read-only triage corroborates the argument/model gap, not implementation or a supported-profile contract. C12/T7/T8 below likewise have no implementation proof from this exchange. Their simulator traces remain baseline-scoped.

**Kind:** capability. 15595 `Engine/Source/Frame/CSimpleMessageScrollFrame_Scripts.cpp:381–386` forwards the optional access ID to `GetNumMessages`; core `CSimpleMessageScrollFrame.cpp:131–149`, returns total count for zero, otherwise scans retained entries for matching `m_accessID`. Older wrappers/core are `CSimpleMessageScrollFrame_Scripts.cpp:397–403` and `CSimpleMessageScrollFrame.cpp:148–166`. These are counts of retained messages, not just visible lines.

**Simulator:** `src/lua_api/frame/methods/widgets/message_frame/mod.rs:34` registers the getter; `getters.rs:49–60` always returns total stored messages and ignores the argument. `add.rs:69–95` reads only text/RGBA/message_id, and `src/lua_api/message_frame.rs` stores no independent access-ID field. Thus simply filtering the existing message ID would conflate distinct metadata.

**Fixture:** insert three messages with access IDs `11,22,11`, using `AddMessage("a",1,1,1,7,false,11,101)`, `AddMessage("b",1,1,1,7,false,22,102)` and `AddMessage("c",1,1,1,7,false,11,103)` (historical text/RGB/typeID/backFill/accessID/extraData slots; 15595 wrapper `:170–230`); assert unfiltered/zero count `3`, filter `11` → `2`, filter `22` → `1`, absent filter `33` → `0`. The source-derived simulator prediction is `3` for every query. Confirm the target profile's AddMessage argument contract before treating the historical producer as native-equivalent; a missing producer/model is part of the finding, not a fabricated executed repro.

**Required rework / regression:** preserve independent access-ID metadata through insertion, retention and removal, then filter retained records for nonzero queries. Test different message/type and access IDs, truncation, clear and unfiltered controls. No runtime/native proof; modern exposure remains unverified.

## C12 — Disabled checked button hides its only checked texture

**Kind:** capability. 15595 `Engine/Source/Frame/CSimpleCheckbox.cpp:78–110`, hides both checked textures, then selects the disabled texture only when it exists and the button is disabled; otherwise it selects the ordinary checked texture. Older `CSimpleCheckbox.cpp:97–133` has the same selection. This is a specific missing-disabled-slot case, not a claim that ordinary disabled-checked rendering is wholly absent.

**Simulator:** `src/lua_api/frame/methods/widgets/slider/checkbutton.rs:68–98` makes ordinary checked visibility `enabled && checked`, independently of whether a disabled-checked texture exists. `src/lua_api/frame/methods/button_anchor_hierarchy/textures.rs:22–59` uses the same visibility rule on texture assignment. The selected child is therefore hidden when checked+disabled even if it is the only configured checked texture. These are active rilua state/visibility paths, not the legacy mlua methods file.

**Fixture:** create a visible CheckButton with only an explicit checked Texture, disable it, then `SetChecked(true)`; inspect texture visibility and rendered mark. Reconstruction shows the ordinary mark; simulator hides it. Control: supplying a disabled-checked Texture selects that texture instead; unchecked hides both. Also test checking before disabling and removing/replacing the disabled slot. These are proposed tests, not observed pixels.

**Required rework / regression:** apply the selected target's texture-slot precedence consistently in checked-state, enable/disable and texture-assignment consumers, asserting texture identity/visibility as well as render output. Historical reconstruction supports this missing-slot rule; no modern-profile/native acceptance is claimed. Existing broad rejection of a generic disabled-CheckButton gap does not cover this narrower case.

## T7 — FontString justification accepts invalid input instead of rejecting it

**Kind:** boundary. 15595 `Engine/Source/Frame/CSimpleFontString_Scripts.cpp:525–545`, errors for non-string or unrecognized horizontal justification; older counterpart `:541–565` does likewise. The getter/registration is present in both trees (`15595 :518–521,737–738`; `12340 :533–536,740–741`).

**Simulator:** `src/lua_api/frame/methods/text_attribute_event/mod.rs:267–268` registers setter/getter. `text/style.rs:260–298` quietly returns for non-string input and stores `TextJustify::from_wow_str` for strings; `src/widget/frame_types.rs:264–273` maps an unknown string to `Left`. Real text consumers use justification in `src/render/text.rs:59,87–102`; this is not inert private storage.

**Fixture:** start at `RIGHT`, call `SetJustifyH("NOT_A_JUSTIFICATION")` under `pcall`, then read `GetJustifyH`. Reconstruction predicts error and unchanged `RIGHT`; simulator predicts success and `LEFT`. Repeat with a table argument: error versus silent no-op. Valid LEFT/CENTER/RIGHT are controls.

**Required rework / regression:** validate before mutating, preserving prior alignment on rejected calls if adopting this boundary. Assert error/result state through public API; do not infer vertical-only keywords are horizontal-valid simply because the simulator shares an enum parser. Supported-profile native validation remains unverified.

## T8 — FontString text-color defaults and clamping differ

**Kind:** boundary. Both FontString wrappers call `FrameScript_GetColor` (`15595 CSimpleFontString_Scripts.cpp:366–375`; `12340 :388–398`). Its actual helper is in **`Engine/Source/Frame/CSimpleModel_Scripts.cpp`**, not FrameScript.cpp: 15595 `:27–53`, and 12340 `:28–54`. It consumes three positional RGB numbers, clamps each to `[0,1]`, and defaults omitted/nonnumeric alpha to one. Missing/nonnumeric RGB goes through Lua numeric conversion to zero. It does **not** establish a color-table overload.

**Simulator:** `src/lua_api/frame/methods/text_attribute_event/text/style.rs:111–142` defaults failed RGB conversion to one and stores raw components using `Color::new`; `src/widget/frame_types.rs:32–35` does not clamp. Getter `style.rs:165–195` exposes stored values. Renderer saturation would not repair this getter difference. SimpleHTML's separate text-type branch is outside this comparison.

**Fixtures:** `fs:SetTextColor()` yields black/opaque from reconstructed helper versus white/opaque from simulator; `fs:SetTextColor(2,-1,0,1)` yields normalized `(1,0,0,1)` versus `(2,-1,0,1)` through the getter. Use numeric tolerance for all reconstructed getter components, including endpoints: byte-to-float conversion uses an approximate multiplier, so these tuples are not exact floating-point equality claims. Both fixtures are source-derived, not executed. Reject the earlier untraced color-table hypothesis: helper choice alone does not prove table support.

**Required rework / regression:** settle target-profile conversion/default/range semantics, then validate/project stored components before rendering. Assert public getter and rendered color separately, with valid RGB, optional alpha and malformed-input controls. This does not duplicate shared FontObject propagation (T6).

## Bounded non-findings and remaining leads

| Aspect | Disposition / evidence |
|---|---|
| FontString spacing public methods absent | **Rejected.** `widgets/mod.rs:85–90` registers EditBox-named method collection into shared base table; `widgets/editbox/registration.rs:47–48` supplies spacing methods. `src/lua_api/methods/frame_metatable.rs:83–94` does not filter them from FontStrings. `widgets/editbox.rs:729–763` stores/reads spacing and refreshes FontString measurements. File placement is not a type restriction. Narrow finite roundtrip is modeled; exact units, glyph geometry and invalid-input policy are not thereby matches. |
| CheckButton `GetChecked`, message `AtTop`/`AtBottom` exact return type | Both reconstructions return `1/nil`; simulator returns booleans (`slider/checkbutton.rs:101–114`; `message_frame/scroll.rs:47–69`). Record **UNRESOLVED historical return-policy difference**, not an instruction to regress modern boolean contracts. |
| Checked texture getters | Active getters return existing child identity or nil (`button_anchor_hierarchy/textures.rs:63–112`). Replacement/destruction/invalid-argument parity is not closed; do not credit whole-method equivalence. |
| Message scroll offset / `GetCurrentScroll` | 15595 `CSimpleMessageScrollFrame.cpp:82–88,729–751` derives ring-relative state; simulator `message_frame/scroll.rs:83–110` stores a direct offset. `message_frame/mod.rs:54–68` registers `GetScrollOffset`, not the legacy name. Units, exposure and out-of-range policy remain unresolved rather than equating differently named getters. |
| Message retention | Ring versus Vec is not itself a defect. Full-capacity top insertion and off-bottom append still need observable retained-message/viewport comparison; no accepted retention mismatch here. |

## Frame lifecycle and resize follow-up

| Aspect | Paired reconstruction / simulator evidence | Disposition and regression boundary |
|---|---|---|
| `SetMinResize` / `SetMaxResize` preserve the opposite bound pair | 15595 `CSimpleFrame_Scripts.cpp:1055–1072,1086–1103`; 12340 `:1215–1231,1248–1265`. Simulator `misc/bounds.rs:126–145` stores separate min/max fields; `src/iced_app/mouse_drag.rs:104–145` consumes both when resizing. | **CHECKED_MATCH only for independent bound composition.** Configure min then max and reverse the order; getter must retain all four values. Error/coercion, historical getter names, source unit conversion and actual resize/native geometry remain outside this match. Existing simulator tests: `src/loader/tests/frame_state.rs:225–255`, inspected not run. |
| `SetDontSavePosition` / `GetDontSavePosition` | 15595 `CSimpleFrame_Scripts.cpp:1127–1145`, flag `0x80000`, and `CSimpleFrame.cpp:411–417`; 12340 scripts `:1292–1311`. Simulator `misc/render_layers.rs:68–79` stores/reads `dont_save_position`; no persistence consumer was closed on either side. | **UNRESOLVED** semantic save/load effect and historical `1/nil` versus boolean return policy. A roundtrip does not prove layout omission after serialize/reload. Do not invent a persistence defect without tracing the reconstruction consumer and supported simulator persistence contract. |
| `StopMovingOrSizing` shared transaction | 15595 `CSimpleFrame_Scripts.cpp:1225–1234` stops only the active top-level frame; `CSimpleTop.cpp:148–155,318–336` clears/starts shared transaction. Older top `:57–64,194–212`. Simulator `misc/drag_input.rs:141–177` changes local flags; real motion additionally consumes `active_drag_frame` in `src/iced_app/mouse.rs:84–108`. | **UNRESOLVED**, extending existing F2 rather than new finding. Test real drag A, stop unrelated B, ensure A continues; then stop A and ensure motion ends. Reconstructed script-start fixture remains blocked on missing producer parity; private flag mutation is not acceptable end-to-end proof. |
| Clamp inset geometry | 15595 `CSimpleFrame_Scripts.cpp:1003–1041`, core `:476–488`; 12340 scripts `:1179–1199`, core `:492–506`; simulator `misc/bounds.rs:57–82`. | Storage exists; geometry consumer not closed on either side. No new mismatch or whole-method match. Nonzero asymmetric insets and real edge dragging require a source-backed expected rectangle first. |

Simulator `misc/*` paths in this table are under `src/lua_api/frame/methods/`. The five resize/dont-save/stop registrations now have bounded coverage notes; clamp rows remain unchecked for behavioral effect.

## Cross-version checks on existing questions

| Existing aspect | 15595 evidence | Disposition |
|---|---|---|
| C7 button font precedence | `Engine/Source/Frame/CSimpleButton.cpp:491–514`; lock refresh `:51–55` | Pushed applies offset but does not force normal font; disabled selects disabled font, otherwise active highlight layer selects highlight font. Supports prior bounded trace; nil-font policy/live inheritance remain separate. |
| C9 explicit scroll-child refresh | `Engine/Source/Frame/CSimpleScrollFrame_Scripts.cpp:166–172`; `CSimpleScrollFrame.cpp:223–234` | Lua marks dirty; layer update calls range calculation. Same deferred boundary, not a new finding or universal frame-delay guarantee. |
| Swapped indented-wrap registration | `Engine/Source/Frame/CSimpleFontString_Scripts.cpp:652–675,745–746` | Newer reconstruction retains getter/setter registration inversion. It does not resolve the older anomaly or justify inverting simulator APIs. |

These confirmations do not increase the finding count or establish current-client parity. The [coverage ledger](coverage.md) owns compared-aspect counts and remaining gaps.
