# Era and Anniversary engraving availability

The [availability contract](../../specs/era-engraving-availability.md) uses an environment-local supplied reader. On 2026-10-28 the user decided that unsupplied availability is false in the stock non-season Era and Anniversary scenario; supplied values still take precedence.

## Root cause and selection

The temporary Classic startup defaults installed `IsEngravingEnabled() return false` only on Era/Anniversary. Pinned declarations on both profiles declare a bool result but establish no inactive default. The existing [Mists engraving inputs](mists-engraving-inputs.md) already represent availability as `Option<bool>`; the shared C API registrar now publishes that same reader on all three profiles through `client_profile::ACTIVE`. See [shared publication and Rust input supply](mists-engraving-inputs.md#shared-availability-and-input-supply) for the profile inventory and absence of a Lua admin setter; its earlier Era/Anniversary unset behavior is superseded by the user-decided stock default. Native availability values remain unknown.

Both Era and Anniversary select vanilla/classic game types (`src/toc/mod.rs`), Family Classic and Game Vanilla. Both pinned `Blizzard_CharacterFrame.toc:18,22` select Vanilla PaperDoll Lua/XML. `Vanilla/PaperDollFrame.xml:716,722` creates RuneFrameControlButton and binds original OnLoad; `PaperDollFrame.lua:950–954` reads availability and hides the control on false. Settings selects `Classic/InterfaceOverrides.lua` through its Classic TOC and reads availability at136. Both have a real conditional LoD EngravingUI entry at PaperDollFrame.lua935–948; that full system is not implemented by this slice.

## Evidence and limits

`tests/era_engraving.rs` loads the selected original addon closure with availability supplied before XML loading. Two selectors exercise opposite initial values, transitions, independent environments, named unset errors and recovery through the XML-bound OnLoad. Executed producer-free RED `1aba02ad7` and producer GREEN `c4d9e5b2e` have byte-identical tests. All four exact original-consumer pairs are RED101/GREEN0. All four settings preservation calls (two profiles, both sides) complete `AdjustDisplaySettings` with supplied false and no collected closure diagnostics.

| Capability | Proof level |
|---|---|
| Supplied true/false and one-bool arity | Original XML load-time OnLoad plus retrieved XML-bound handler and direct result observations, both profiles |
| Unset/recovery and environment isolation | Named error at original load-time and bound callback; supplied recovery and independent environments, both profiles |
| Settings false-input attempt | Complete original closure and consumer call return normally; preservation only, no assertion of all control behavior |
| Other profiles | Seven full-feature all-target checks, zero JSON/plain warnings/errors; compilation proof, not runtime parity |

Each profile's master-equivalent stock map changes0 unique/0 occurrences to2/2. The first raw key is `[OnLoad] RuneFrameControlButton: C_Engraving.IsEngravingEnabled: unsupplied input`, reported by `src/loader/precompiled.rs:23–30` from the original PaperDollFrame.lua951 query bound by XML722. The second is its `Lua Error:` echo, formatted at `Blizzard_ScriptErrors.lua:71–80` and recorded through addframetext. Each has count1 and both are attributed to Blizzard_CharacterFrame. The removed `classic_startup_defaults.rs` false policy (predecessor lines10–18, return at16) formerly let the same original951 query hide the control without reporting an error. These are explicit accepted consequences of requiring supplied input, not stock cleanup or clean-startup claims. No key is removed or otherwise count-changed.

Do not interpret the settings preservation attempt as full-settings behavior proof. Bags and full LoD EngravingUI remain excluded, as do the other25 declared C_Engraving members on these two profiles. Compiler checks are not runtime parity proof for other profiles. The user-decided stock default supersedes the former unset error; the stock comparison for this change remains pending.

## Sources

- Era and Anniversary pinned `Blizzard_APIDocumentationGenerated/EngravingInfoDocumentation.lua:179–186` — non-null bool result, not default policy.
- Each profile's selected `Blizzard_CharacterFrame/Vanilla/PaperDollFrame.lua:935–954` and XML716–724 — real control and LoD entry.
- [Availability spec](../../specs/era-engraving-availability.md) — bounded contract and exclusions.

## See Also

- [[mists-engraving-inputs]] — shared input contract, merged Mists settings proof and stock comparison.
- [Consumer selection and RED proof](consumer-selection-and-red-proof.md) — applicability and discriminating selectors.
