# XML and dispatch: source-backed rework and rejected leads

[Revision, proof and profile limits](README.md) apply. These are static source comparisons, not native measurements. No repository/client code was executed.

## X1 — Non-frame virtual templates overwrite definitions without trust precedence

**External evidence:** `Engine/Source/FrameXML/FrameXML.cpp:366-388`, `FrameXML_StoreHashNode`. Top-level virtual nodes reach this function at `:541-549`, independent of node kind. The duplicate-name policy is:

| Existing definition | Incoming definition | Reconstructed outcome |
|---|---|---|
| Trusted | Trusted or tainted | Keep existing; duplicate warning |
| Tainted | Tainted | Keep existing; duplicate warning |
| Tainted | Trusted | Replace existing |

Trust is stored metadata controlling replacement, **not part of the registry key**.

**Simulator difference:** `src/loader/xml_file.rs:107-117,472-506` registers virtual Texture, FontString, Animation and AnimationGroup with name+XML, not declaration provenance. `src/xml/template.rs:446-449,468-472,675-679` and `src/xml/animation_templates.rs:11-14` unconditionally insert into their respective maps. Later definitions win. Frame templates have a separate `declaration_taint` mechanism and are not included in this finding.

**Observable fixture:** a trusted virtual Texture `T` has width 10; a later addon declares `T` with width 20. Instantiate a texture inheriting `T`: reconstructed policy retains width 10 and warns; simulator registry retains the later definition. Repeat both declaration orders and same-trust collisions for each relevant template kind, using observable instantiated properties.

**Required change:** carry actual declaration provenance into non-frame registration and enforce the target client's collision policy before replacement. Do not infer trust from template name. Preserve separate template-kind resolution and existing frame-script declaration-taint behavior. Establish modern-profile policy rather than silently globalizing the historical rule.

**Regression:** load duplicate definitions through the real trusted/addon XML loader boundary; instantiate the winning template and assert concrete size/property plus appropriate duplicate diagnostics. Include trusted→tainted, tainted→trusted and same-trust cases. Current `HashMap::insert` behavior is established by inspection; no end-to-end collision fixture was executed here.

## X2 — Failed includes have inconsistent continuation and diagnostic behavior

**External evidence:** `Engine/Source/FrameXML/FrameXML.cpp:56-76`, `FrameXML_LoadXML`, records file-open/parse errors; `:458-500,557-569`, `FrameXML_ProcessFile`, recursively processes Includes, ignores the recursive success integer, continues subsequent siblings and merges accumulated `CStatus` diagnostics. Missing `file` attribute is itself an error. `Engine/Source/FrameScript/FrameScript.cpp:851-907`, `FrameScript_ExecuteFile`, explicitly records missing Lua file errors and returns 0 on read failure.

**Simulator difference:** `src/loader/xml_file.rs:433-454` logs a failed Lua Include through tracing but returns a successful file count `Ok(1)`. `src/loader/lua_file.rs:20-28` returns an error for an unreadable file, so this includes missing Lua paths. For XML Includes, errors instead propagate to the `?` in the outer element loop at `xml_file.rs:40-53`, preventing later siblings from being processed. Top-level XML errors become warnings in `src/loader/addon.rs:631-649`; this does not restore skipped siblings. Runtime Lua error handling can additionally invoke the existing Lua error handler; this entry does not claim tracing is the only channel for every Lua failure.

**Observable fixtures:**

1. Outer XML includes a missing/unparseable XML file, then a valid sibling script setting a marker. Reconstruction records the include error and still sets the marker; simulator exits that XML processing loop first.
2. Outer XML includes missing `absent.lua`, then the marker script. Simulator continues but counts one loaded Lua file and only traces the include read failure rather than returning that failure in loader warnings; the reconstruction records an error status and continues. The reconstruction has no matching Lua-file-count API; the count problem is a separate simulator reporting inconsistency, not an assertion about original-client counters.

**Required change:** aggregate include failures without abandoning independent later siblings, retain file/context diagnostics in the loader's returned diagnostic channel, and do not count failed Lua reads/loads as successful. Preserve legitimate filtering of profile-excluded entries (`Ok(0)`), existing runtime error handling and intended continue-after-Lua-error behavior. Do not turn every include error into an immediate outer-file abort.

**Regression:** real temporary addon XML plus missing Lua, missing XML and malformed nested XML fixtures, each followed by a marker-producing sibling. Assert marker execution, returned diagnostic context and accurate successful-file count. Avoid relying on source-shape assertions or tracing-text searches instead of loader output/state.

**Applicability:** reconstructed 3.3.5a file-status/continuation evidence. Current profile-specific diagnostic policy remains to be resolved; no loader fixture was run here.

## Include depth — unmeasured boundary, not a demonstrated mismatch

Cross-file XML `<Include>` depth is separate from XML element nesting and Lua call-stack depth. `Engine/Source/FrameXML/FrameXML.cpp:427-498` recursively invokes `FrameXML_ProcessFile` for an included XML file, with no explicit include-depth/path-cycle guard in the inspected path. `Engine/Source/FrameXML/XMLTree.cpp:160-199,290-310` handles per-file element trees through Expat; it does not establish the Include recursion limit.

Simulator `src/loader/xml_file.rs:21-53,433-454` similarly recurses into `load_xml_file` without a depth/cycle check there. `src/loader/load_addon_trace.rs:54-69` increments/decrements `xml_load_addon_depth` for load-scope reporting, not a maximum-depth limit. Both traced paths are recursive; that does **not** mean unlimited depth succeeds or that the original client lacks every possible outer guard.

No numeric maximum, safe depth, cycle failure mode or native parity was measured. Future behavioral coverage should distinguish a finite `A→B→…` Include chain, `A→A`/`A→B→A` cycles, repeated noncyclic includes and deep elements within one file, recording the actual client's diagnostic/termination boundary. Do not add a simulator cap solely because a counter exists, or infer native limits from the reconstruction's compiler/platform stack size. No depth probe or repository code was run here.

## Leads not accepted as simulator defects

| Lead | Why not promoted |
|---|---|
| Stamp callbacks using owning frame's addon taint | The reconstructed executor accepts per-script taint, not a rule equating every handler to frame ownership. Simulator closure taint, script installation, declaration-taint handling and rilua execution must be traced together. Absence of a dispatch-time setter alone does not prove missing taint. No faithful counterexample established. |
| Frame table/slot-0 identity must match a native pointer | Both expose tables, but internal representation differences do not establish an addon-visible defect. Public raw operations, forged identity and debug environments need target-profile behavior evidence. Preserve [the representation caveats](../../AGENTS.md#frame-representation-and-debug-fields). |
| Frame-template trust collisions | Frame declaration metadata already exists; this audit did not establish complete collision behavior. X1 is deliberately limited to the four non-frame registries inspected. |
| Implement modern encounter/housing/secret APIs from this source | 3.3.5a cannot establish later API contracts. No claims about modern C_* gaps follow from their absence here. |
| Reproduce full client networking, patching or 3D | Outside this 2D simulator audit. The [security report](security.md) describes execution capabilities, not implementation requirements. |
