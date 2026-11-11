# ScriptObject full lockdown

VERIFIED R18 predicates distinguish ScriptObject full lockdown from ordinary forbidden flags, aura access restrictions, and combat protection. Completed Lua call outcomes are INFERRED from static analysis, not a live native reproduction. The simulator models the bounded creation rules and five measured API consumers; it does not guess unknown writers or lifecycle paths.

## State and creation

VERIFIED: an object is fully locked when its explicit full-lockdown flag is set **or** its access-allowed flag is missing. Base construction sets access allowed. At child creation, the native path copies full lockdown when the parent is fully locked and removes the child's access-allowed flag when the parent lacks it. INFERRED: the resulting child rejects the measured APIs. Do not infer propagation to existing children or reparent transitions.

VERIFIED: `ScopedModifier/@fullLockdown` sets the creation default to true or false for its scope, then the loader restores the prior default. INFERRED: objects created in the true scope receive full lockdown. Parent inheritance applies independently, so a child of a fully locked parent remains fully locked under a false creation default. The native engine can mask the requested default; the mask conditions are UNKNOWN and are not modeled.

VERIFIED local cache and manifest evidence: Retail `UI.xsd:1266` declares `fullLockdown` as a boolean with default `false`; `Blizzard_AuthChallengeUI.xml:5` opens a `fullLockdown="true"` scope. The committed Retail manifest lists the XML at lines 871–872 and XSD at line 3392. The WowForever manifest lists them at lines 882–883 and 3723. The cached Retail and WowForever AddOn TOCs say `UseSecureEnvironment: 1` and `AllowLoad: Both`. These file bytes establish the local pinned inputs checked here, not the identity of a client's live session.

## API boundary

VERIFIED: `IsForbidden()` reports true for ordinary forbidden flags or full lockdown; it cannot distinguish the two states. `SetForbidden()` changes only ordinary forbidden flags and does not set full lockdown or clear access-allowed state.

VERIFIED R18 lists these APIs as using the locked-down-disallowed resolver:

- `DurationTextBinding:SetFontString(fontString)`
- `FontString:CanWordWrap()`
- `FontString:CanNonSpaceWrap()`
- `FontString:ClearAlphaGradient()`
- `Frame:CreateFontString(...)`

INFERRED completed Lua outcome: each rejects a fully locked object for secure and tainted callers with `Attempt to access fully locked-down object.`. A tainted caller is also rejected by the ordinary forbidden check where the API applies that separate check. The XML loader uses a private constructor passed only to its generated chunk; it does not call the public `Frame:CreateFontString` resolver when it creates a FontString inside a full-lockdown scope.

## Verification

The bounded Retail and WowForever integration runs passed 7/7 each, including the original AuthChallengeUI and DurationTextBinding consumers. Six profile all-target checks, the Wrath compile, both library suites, the Retail filtered suites, and the manifest tests passed with zero compiler warnings. See the [lane report](../../../handoff/full-lockdown-model/REPORT.md) for exact results and receipts.

## Evidence limits

UNKNOWN: live native reproduction; whether arbitrary AddOns may use `fullLockdown`; the engine's masking policy; complete writer and API inventories; the source of a missing access-allowed flag; teardown behavior; propagation to existing children; and reparent behavior. R18 found no Lua mutator, but that search does not prove none exists. R18's cached Forever XML example was itself marked unverified for build identity.

## Sources

- [ScriptObject full-lockdown contract](../../specs/script-object-full-lockdown.md)
- [Duration text binding](../../specs/duration-text-binding.md)
- [Pinned Retail manifest](../../../data/blizzard-ui-files/retail.txt) — AuthChallengeUI and UI.xsd entries at lines 871–872 and 3392.
- [Pinned WowForever manifest](../../../data/blizzard-ui-files/wowforever.txt) — AuthChallengeUI and UI.xsd entries at lines 882–883 and 3723.
- Cached XSD and UI source: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_SharedXML/UI.xsd:1266` and `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_AuthChallengeUI/Blizzard_AuthChallengeUI.xml:5`.
- `/home/osso/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`, R18 Q1–Q3 — native predicates, creation paths, checked APIs, and UNKNOWN labels.

## See Also

- [[lua-api]] — frame methods and C API boundaries.
- [[widget-system]] — frame state and parent/child representation.
