# Patch 12.0.5 Probe Inventory
Probe-subfinding register for the retained 12.0.5 live-client audit. Machine status advances only with row-specific behavioral evidence or item-specific evidence for an explicit exception path; prior documented states remain visible separately.

## Content

**Coverage boundary:** these 38 native-probe subfindings are not full patch-page coverage. The retained page and expanded source audit (**IN PROGRESS**, no completed behavior claim) are documented in [[patch-12-0-5-api-audit]]; its inventory counts 244 consolidated delta rows / 186 subjects plus 118 chronological prose rows, including narrative, and 0 CVar entries. Best-supported implementation guesses must remain labeled and paired with concrete future probes, not native-verification claims. The broader audit's original six capability/future-probe slices and newer bounded batch4/batch5 development GREEN are tracked in [[patch-12-0-5-api-audit#Batch4 bounded development proof]]. Batch5 stat-output, UnitIsUnit, ignoreGCD and native-enabled Seconds/common-duration runs add no register credit; independent final/native proof remains pending.
- **Source:** `data/patch-api/sources/12.0.5-probes.json`
- **Source SHA-256:** `2d7671b7702eed71c5d8a3ae4e92595771f6c04bb58fe98bd771647ac26cddca`
- **Target:** retail build `12.0.5`
- **Rows:** 38 changed probe subfindings — 0 implemented, 33 best-effort, 4 evidence-required, 1 approved provenance-only exception-requested, 0 untriaged

| Symbol | Machine Status | Documented Status | Category | Direction | Detail |
|---|---|---|---|---|---|
| `AnimScriptProbe.HandlerMatrix` | best-effort | resolved | script support | changed | Frame, AnimationGroup, and all nine Animation subtypes match the documented matrix: unsupported HasScript calls succeed and return false, while SetScript rejects those handlers. |
| `AttributeDispatchProbe.ScalarRepeat` | best-effort | resolved | attributes | changed | Repeated scalar SetAttribute dispatches OnAttributeChanged; explicit false is preserved and delivered as false. |
| `AttributeDispatchProbe.PanelPulse` | best-effort | best-effort | panel lifecycle | changed | Repeated ShowUIPanel pulse followed by CloseAllWindows preserves the expected panel-stack behavior. |
| `CoreBehaviorProbe.ForbiddenState` | best-effort | resolved | forbidden frames | changed | Normal addon frames remain un-forbidden; SetForbidden does not create forbidden state. |
| `CoreBehaviorProbe.ForbiddenConstructor` | best-effort | resolved | forbidden frames | changed | CreateForbiddenFrame is nil on retail, so the probe does not enter its creation or EnumerateFrames branch. |
| `CoreBehaviorProbe.UnitEventFilter` | best-effort | resolved | events | changed | Valid RegisterUnitEvent registration works; invalid unit filters are dropped while the event remains registered. |
| `CoreBehaviorProbe.AttributeWildcardFalse` | best-effort | resolved | attributes | changed | Wildcard GetAttribute preserves explicit stored false. |
| `CoreBehaviorProbe.AttributeWildcardValues` | best-effort | resolved | attributes | changed | Wildcard true/string values and one-, two-, and three-argument lookup behavior are preserved. |
| `CoreBehaviorProbe.RaiseLowerLevel` | best-effort | resolved | frame ordering | changed | Raise/Lower affect same-level ties but cannot overtake a higher frame level. |
| `CoreBehaviorProbe.MouseFocusOrder` | best-effort | best-effort | mouse focus | changed | GetMouseFoci/GetMouseFocus shape and hit ordering after Raise/Lower. |
| `DevToolsDumpProbe.FrameArrayDump` | best-effort | resolved | frame identity | changed | tinsert(frame, foo), frame slot contents, and DevTools_Dump output metadata. |
| `FrameIdentityProbe.IdentitySlot` | best-effort | resolved | identity | changed | Frame slot [0] contains the identity userdata token. |
| `FrameIdentityProbe.SurrogateDispatch` | best-effort | resolved | identity | changed | Replacing [0] redirects protection and method dispatch; [1] alone does not. |
| `FrameIdentityProbe.DuplicateFreshness` | best-effort | resolved | identity | changed | Duplicate named frames receive fresh Lua objects, identity tokens, and custom-field state. |
| `HookScriptBindingProbe.IndexedHooks` | best-effort | resolved | scripts | changed | The normal binding slot succeeds and chains; explicit slots 0 and 2 return false and GetScript reports nil for those slots. |
| `IsProtectedProbe.LegacySetters` | best-effort | resolved | protection | changed | Legacy Protect and SetProtected methods are absent and calls fail. |
| `IsProtectedProbe.SecureTemplate` | best-effort | resolved | protection | changed | Secure-template buttons report protected state while ordinary frames do not. |
| `IsProtectedProbe.DescendantAnchorPropagation` | best-effort | resolved | protection | changed | The directly protected root returns true/true; its child, grandchild, frames anchored to the root or child, and the root-keyed anchored frame remain false/false. |
| `JustifyProbe.FrameFontStrings` | best-effort | resolved | FontString layout | changed | Direct unanchored frame-layer FontStrings receive the observed default anchors and justification. |
| `JustifyProbe.ButtonText` | best-effort | resolved | FontString layout | changed | Implicit ButtonText FontString behavior matches the probe matrix. |
| `JustifyProbe.SizeVariants` | best-effort | resolved | FontString layout | changed | No-size, width-only, height-only, and width+height variants are captured. |
| `JustifyProbe.ExplicitAnchors` | best-effort | resolved | FontString layout | changed | TOP/BOTTOM/LEFT/RIGHT/TOPLEFT controls distinguish missing from partial anchoring. |
| `JustifyProbe.EditBoxRegions` | best-effort | resolved | FontString layout | changed | EditBox FontStrings, including sized and inset variants, are captured. |
| `JustifyProbe.MessageRegions` | best-effort | resolved | FontString layout | changed | MessageFrame and ScrollingMessageFrame owner/region behavior and TextInsets effects. |
| `ProtectedRetailProbe.PlainFrame` | best-effort | resolved | protection | changed | Plain frame protection/forbidden state and legacy setter behavior. |
| `ProtectedRetailProbe.XmlProtected` | best-effort | resolved | protection | changed | XML protected=true frame state and setters. |
| `ProtectedRetailProbe.SecureStore` | evidence-required | unsafe | protection | changed | Retained Store frames are forbidden, legacy setters are absent, and IsProtected errors; exact forbidden/secret-return enforcement is unsafe to guess. Await authoritative live evidence or a correct modeled implementation. |
| `ScaleEventProbe.OrderedEvents` | best-effort | resolved | scale events | changed | DISPLAY_SIZE_CHANGED, UI_SCALE_CHANGED, and relevant CVAR_UPDATE ordering. |
| `ScaleEventProbe.SameSizeDuplicatePair` | evidence-required | impossible | scale events | changed | Same-size maximize/restore duplicate event pairs remain open: the simulator receives no window-state transition signal, so correct behavior is not modeled. Await an observable window-state input or other authoritative evidence. |
| `SetAtlasProbe.InvalidArguments` | best-effort | resolved | texture atlas | changed | nil, no-argument, boolean, numeric, empty, and unknown atlas inputs. |
| `TextureSetTextureProbe.PathFdid` | best-effort | resolved | texture | changed | UI-Panel-Button-Up path assignment and retained FDID 130828. |
| `TextureSetTextureProbe.Clear` | best-effort | resolved | texture | changed | SetTexture(nil) and no-argument clearing behavior. |
| `XmlFrameLevelProbe.BareAndFixed` | best-effort | resolved | XML frame level | changed | Bare frameLevel versus fixedFrameLevel=true semantics. |
| `XmlFrameLevelProbe.ParentReparent` | best-effort | resolved | XML frame level | changed | Parent-level changes, unfixed-child propagation, and Lua SetFrameLevel reparenting. |
| `XmlFrameLevelProbe.Flags` | best-effort | resolved | XML frame level | changed | HasFixedFrameLevel and IsUsingParentLevel observations. |
| `XmlFrameLevelProbe.RawCaptureProvenance` | exception-requested | impossible | provenance | changed | Behavior is tested, but the raw SavedVariables capture does not exist and cannot be reconstructed locally. Approved provenance-only exception; behavior remains independently regression-tested. |
| `StoreForbiddenProbe.DropdownPopulation` | evidence-required | unsafe | Store lifecycle | changed | Retained `StoreDropdown_SetDropdown == nil`; population, reuse, text/check, callback, and protection behavior were never observed. Await authoritative live evidence or a correct modeled implementation. |
| `StoreForbiddenProbe.ForbiddenDescendants` | evidence-required | unsafe | Store lifecycle | changed | Store descendant forbidden/protected state remains open: the retained file lacks the `/sfp` manual descendant scan, so correct behavior is not modeled. Await authoritative live evidence or a correct modeled implementation. |

Bounded [secret `%s` spec](../../specs/secret-string-formatting.md) and [[patch-12-0-5-api-audit#Secret-string formatting bounded runtime proof]] add independent runtime development proof only; new-pin simulator integration and native probes remain pending. No credit or classification change to these 38 subfindings.

## Machine state totals

- implemented: 0
- best-effort: 33
- evidence-required: 4
- exception-requested: 1 (approved provenance-only)
- untriaged: 0

## Sources

- `data/patch-api/sources/12.0.5-probes.json` — categorized probe subfindings and preserved documented state metadata.
- `docs/wiki/investigations/patch-12-0-5-api-audit.md` — broader patch audit context.

## See Also

- [[patch-12-0-5-api-audit]] — broader patch audit context.
- [[patch-api-audit-manifest]] — register schema and validation contract.
