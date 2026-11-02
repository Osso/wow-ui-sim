# Additional rendering and minimap discrepancies

Expanded pass against simulator `2da20225ba147dda579dd01164917cc16f0618ed`; external revision remains `0e4e9fe0a4a9fa08027c5eed4b2c5e873739cfd3`. These are **static source comparisons**, not executed/native confirmations. [Historical-profile and security limits](README.md) apply. Source paths below are relative to the respective repository root.

## T4 — FontString alpha gradients never reach glyph rendering

**External:** `Engine/Source/Frame/CSimpleFontString_Scripts.cpp:137-157`, calls `SetAlphaGradient(start,length)` and returns its outcome. `Engine/Source/Frame/CSimpleRender.cpp:1094-1100,1628-1629`, stores the gradient and applies it to an existing or subsequently created text block. `Engine/Source/Gxu/GxuFontString.cpp:697-750`, changes geometry alpha across the gradient span rather than merely retaining setter arguments.

**Simulator:** `src/lua_api/frame/methods/misc/alpha_gradient.rs:54-70` records `alpha_gradients[0]`, clamps length and always returns true. A source-wide `alpha_gradients` search finds initialization and API storage/getters/clear only, no glyph/render consumer. Existing `src/loader/tests/frame_interaction.rs:266-310` and `tests/patch_11_2_0_publication_fixes.rs` exercise stored surface state, not rendered fading.

**Source-derived fixture:** render a long FontString before and after `SetAlphaGradient(2,4)`. The reconstruction modifies glyph geometry alpha; the inspected simulator glyph path receives no gradient state, predicting unchanged pixels. No pixel comparison was performed. Exact fade quantization and character/vertex indexing require further target-profile evidence.

**Rework:** carry gradient state into actual glyph emission, including retained state after text recreation. Resolve return behavior separately for profiles retaining the historical contract; current modern gradient APIs are not defined by this old wrapper. Do not mistake a working `HasAlphaGradient` roundtrip for rendering support.

**Regression:** deterministic text render compares pre-span, within-span and post-span pixel alpha; changing text/font after the setter preserves the requested effect. Include empty text/invalid spans only after their native contract is established. Keep modern additional gradient APIs separate from the historical two-argument FontString contract.

## T5 — Font mutation claims success without validating a usable font

**External:** `Engine/Source/Frame/CSimpleFontString_Scripts.cpp:285-320` validates positive height and reports success only if `SetFont` succeeds. `Engine/Source/Frame/CSimpleRender.cpp:1228-1266`, `CSimpleFontString::SetFont`, calls `TextBlockGenerateFont` before changing the existing handle/height; failed generation returns 0 with prior font intact. The failure chain is explicit: `Engine/Source/Services/TextBlock.cpp:333-379` → `Engine/Source/Gxu/GxuFontMain.cpp:862-898` → `Engine/Source/Gxu/GxuFontFont.cpp:217-239` → `Engine/Source/Gxu/GxuFontFace.cpp:72-112`; an uncached name whose file cannot be read yields no face/handle, not a default-font success.

The public `SetTextHeight` wrapper at `CSimpleFontString_Scripts.cpp:490-509`, rejects nonpositive height **before** calling the core setter, and is registered at `:737`. The unvalidated internal setter at `CSimpleRender.cpp:1271-1277` does not bypass that public Lua boundary.

**Simulator:** `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330-398` accepts any non-nil font path, stores path/size/outline and returns true without loading/validating the font. `:433-447` permits zero/negative size and aliases `SetTextHeight` to that assignment. The getter can therefore advertise an invalid requested font/height regardless of the renderer's eventual resource resolution.

**Source-derived counterexamples:** begin with a valid font; request a fresh, uncached fixture path absent from the asset sources with positive height. The traced reconstruction failure path preserves the old font, while simulator setter code reports true and changes getter state. The simulator font system can draw a default family for an unknown requested path (`src/render/font.rs`); this is a return/getter/resource-identity discrepancy, not proof of invisible text. `SetTextHeight(0)` errors in the registered reconstructed Lua wrapper but stores zero in the simulator. No fixture was executed.

**Rework:** make font changes transactional against the actual asset/font loader and the supported profile's height validation. Commit path/height/flags only after successful font creation; align return/error behavior and `GetFont` with the resource actually used. Do not silently substitute a different font while reporting the rejected request as installed.

**Regression:** valid font→missing/corrupt asset leaves prior getter and rendered text intact; valid asset updates both. Check invalid heights and repeated valid assignment without losing existing text. Tests must observe return/state/render coherence, not assert which internal loader was called.

**Applicability:** reconstructed 3.3.5a failure/height contract; modern flags and font APIs may differ. Resource existence must use the simulator's supported asset sources, not require the original client's MPQ layout.

## T6 — Shared FontObject mutations do not update existing FontStrings

**External:** `Engine/Source/Frame/CSimpleFontable.cpp:20-37`, links an attached fontable to its Font object and applies initial attributes. `Engine/Source/Frame/CSimpleFont.cpp:145-155`, `UpdateObjects`, walks those attachments on later mutations. Font `SetFont`, `SetTextColor` and `SetSpacing` invoke it (`Engine/Source/Frame/CSimpleFont_Scripts.cpp:143-166,182-194,273-289`). `Engine/Source/Frame/CSimpleFontStringAttributes.cpp:68-99,103-135` projects changed categories; local FontString setters clear inheritance bits for their overridden font/colour/spacing categories.

**Simulator:** Font is a Lua table. `src/lua_api/globals/font_strings_collection/fonts.rs:102-116,181-195,234-244` mutates table fields without notifying attached strings. `src/lua_api/frame/methods/button_anchor_hierarchy/font_strings.rs:283-318,414-485` reads and applies snapshots to widget fields and stores the object reference. Rendering/measurement uses those widget fields, not a live dereference of the Font table. Returning the original object from `GetFontObject` therefore does not establish a live dependency.

**Counterexample:** attach existing FontStrings A/B to shared Font F. Change F's font/height, colour or spacing: the reconstructed attachment updates eligible A/B fields, while simulator snapshots stay unchanged. Give only A a local colour override, then change F's colour again: A should retain its override while B follows F under the historical contract. This is a source-derived fixture, not executed output.

**Rework/regression:** preserve a live association and per-category local-versus-inherited provenance, then project actual Font changes to eligible existing strings and invalidate their measurement/rendering. Reassignment must detach from the old Font; local overrides must survive updates in other categories. Test two strings, three mutation categories, one local override, reassignment and exact `GetFontObject` identity in a single grouped fixture. Existing creation-time inheritance and [spacing snapshot/independence tests](../specs/fontstring-spacing.md) are not proof of later shared mutation. That owning contract explicitly leaves live propagation unverified; resolve target-profile behavior before extending it. Font-to-Font, EditBox and button consumers need their own traces; do not extrapolate this bounded FontString result to every fontable.

## M1 — Minimap texture methods do not drive its 2D overlays

**External:** `WoW/Source/Ui/MinimapFrame_Scripts.cpp:92-131,212-230` implements player texture size and asset changes. `WoW/Source/Ui/MinimapFrame.cpp:425-432,536-539` creates/anchors the player Texture and rotates it from facing. POI/blip resources feed separate render paths (`:2573-2575,2741-2792`), not just setter bookkeeping.

**Simulator:** `src/lua_api/frame/methods/map_frames.rs:420-459,596-614` stores texture names. `src/wrath/frame_methods.rs:52-62` implements player texture width/height as no-ops. `src/iced_app/quad_builders_textures.rs:583-603` emits only the fixed `minimap-placeholder` image and applies a mask; it does not consume the stored player/blip/POI textures. A source-wide `minimap_player_texture` search finds state/copy/test paths but no overlay renderer. The active mask **is** consumed, so the old assertion that `SetMaskTexture` is entirely a no-op is rejected.

**Counterexample:** on a legacy profile exposing these setters, replace the player arrow with an asymmetric supplied texture and request a different width/height. Getter/state bookkeeping may change, but the actual minimap output has no corresponding arrow update; reconstruction owns a real child texture with those changes.

**Rework:** connect supported 2D player/POI/blip overlay state, texture sizing and facing to minimap rendering. Separate absent world/map content producers from working API-to-overlay behavior. This does **not** request a 3D world renderer, native networking, or reconstruction of an entire terrain engine.

**Regression:** bounded GPU fixtures use supplied 2D map/overlay data, change arrow asset/dimensions/facing and assert visible output; test blips/POIs with concrete supplied positions. Existing `src/loader/tests/minimap_specialized.rs:58-125` checks state/reset, not those pixels. Respect modern profile removal gates for legacy setters.

## M2 — Minimap pings are raw coordinate storage, not position-derived state/events

**External:** `WoW/Source/Ui/MinimapFrame_Scripts.cpp:235-310`, converts click coordinates using minimap dimensions, view radius, player location and rotation, stores a world position and derives subsequent relative coordinates from current player state. `WoW/Source/Ui/MinimapFrame.cpp:2165-2215`, `SetPingPosition`, signals `MINIMAP_PING(unit,x,y)` after resolving the sender.

**Simulator:** `src/lua_api/frame/methods/map_frames.rs:641-664` merely records/replays the supplied `(x,y)` pair. It emits no event and the getter never consults player position, facing, zoom/radius or minimap dimensions. A source-wide `MINIMAP_PING` search finds event-name validity entries but no producer.

**Counterexample:** ping an offset on a non-unit-sized minimap and read its position: reconstruction returns derived relative coordinates, not the raw supplied pixel offsets. Move the supplied player position afterwards; the same world ping changes relative location in the reconstruction but remains the original pair in the simulator. A registered `MINIMAP_PING` callback receives no locally produced notification from this simulator method.

**Rework:** model ping world position/sender and view transformation, derive getters from current state, and dispatch the matching local event. Route supplied remote ping state through the same model if the simulator supports it. Do not add live network traffic; the original party-message send is not necessary to reproduce the local Lua contract.

**Regression:** explicit map dimensions, player location/facing/view radius and known click point; assert event payload plus getter, then movement/rotation/zoom transformations. Include no-active-player behavior separately. Existing ping roundtrip test proves the simulator's current storage, not this coordinate/event behavior. Exact modern units and payload policy remain to be established.

## M3 — Minimap zoom is disconnected from client view/CVar state

**External:** `WoW/Source/Ui/MinimapFrame.cpp:3543-3567`, selects distinct indoor/outdoor zoom, clamps to five, marks view updates and writes `minimapInsideZoom` or `minimapZoom` when changed. Getter reads the active environment's zoom; six levels are exposed.

**Simulator:** `src/lua_api/frame/methods/map_frames.rs:616-639` clamps into six levels but writes a private per-frame `__minimapZoom` field only. It does not synchronize either CVar or a view-radius/update model. The minimap renderer is the same static image independently of that field.

**Counterexample:** start with a distinct outdoor zoom CVar value, then call `Minimap:SetZoom(3)` with a different current zoom. Reconstruction synchronizes the corresponding CVar/view; simulator's inspected setter changes only its private getter field. Indoor/outdoor switching has no corresponding pair of zoom states in this path.

**Rework:** select a supported-profile view/zoom model as the source of truth and derive `GetZoom` from it. Synchronize the relevant CVars/environment selection only if that historical coupling applies to the selected profile. Preserve the existing six-level count/clamping where the target contract agrees. Do not implement new terrain generation as a substitute for missing state coupling.

**Regression:** supplied indoor/outdoor states with different stored zooms; setter updates active zoom/CVar, environment transition selects the other zoom, and view-dependent consumers use the changed radius. The six-level count itself matches and is not a new gap. Historical CVar/indoor policy requires target-profile confirmation.

## Historical surface lead, not counted as a demonstrated modern defect

`CSimpleFontString_Scripts.cpp:332-338`, registers `GetFieldSize` returning 8191; no occurrence exists in simulator `src/`. This is an absent historical method, but its required availability on each supported profile has not been established. Do not infer a universal 8191-byte text limit: the reconstructed FontString constructor sets `m_textMaxSize=0`, and `SetText` can allocate dynamically (`CSimpleRender.cpp:1039-1053,1281-1314`).
