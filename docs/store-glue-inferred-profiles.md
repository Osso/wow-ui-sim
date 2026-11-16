# INFERRED PTR and Forever applicability

Coordinator1283 approves reusing the Retail R88q modeled contract on PTR and Forever under `client_profile::ACTIVE`, based on these selected original consumers. **Generated C_StoreGlue/C_StoreSecure declarations are absent from both pinned caches.** Native parity is not established for these profiles.

Both caches select `Blizzard_GlueXML/Blizzard_GlueXML.toc:15` for Glue. `[Family]` resolves to Mainline; lines56/59 select ListUtil/ListElements, and line74 selects Mainline CharacterSelect. Line75 additionally selects Camelot CharacterSelect, whose sole method adjusts the gamepad search box, not these store paths.

| Unconditional API call on selected branch | PTR | Forever |
| --- | --- | --- |
| `CharacterSelect/CharacterSelectListUtil.lua`, q1 tuple | 577–580 | 577–580 |
| `CharacterSelect.lua`, OnShow refresh | 193 | 193 |
| `CharacterSelect.lua`, disconnect read/set | 201–204 | 201–204 |
| `CharacterSelect.lua`, OnUpdate readiness read (no queued upgrade) | 342 | 343 |
| `CharacterSelect/CharacterSelectListElements.lua`, WaitingOnQueue request | 838–839 | 840–841 |

These are actual calls, not `if C_StoreGlue.Member then` existence guards. Behavioral branch conditions remain meaningful and are supplied in original-consumer tests. Retail native facts justify the reused bounded model; source selection justifies applicability inference, not additional service/transport semantics.

## Profile-specific test boundary and proof binding

The original `CharacterSelectListUtil.GetVASInfoForGUID` function begins at line573 in all three pinned profiles. The missing `C_Login.DisconnectFromServer` call is at Retail `Mainline/CharacterSelect.lua:211`, but PTR and Forever line204. The fixture seeds exactly the active profile's diagnostic. The Lua equality matcher and its negative control remain unchanged, so Retail does not accept line204 and no profile accepts arbitrary errors.

The verified Retail RED/GREEN common body remains its own frozen proof. This separate INFERRED commit changes profile gates and the profile-specific expected-error input only; its new formatted common-body hash is reported separately. Other consumer calls, record inputs, event-order checks and payload/state assertions are unchanged. Native Store declaration parity is not asserted.

## Applicability report

## PTR applicability ledger
Each extension reuses an existing producer unchanged and is isolated from enclosing registration blocks.

| Member | Existing evidence | Selected PTR consumer |
| --- | --- | --- |
| C_CharacterServices.GetAutomaticBoost | R54 Retail70688 VERIFIED, answers4584 | CharacterSelect.lua2153 |
| C_CharacterServices.GetAutomaticBoostCharacter | R54 Retail70688 VERIFIED | CharacterSelect.lua2160-2161 |
| C_CharacterServices.SetAutomaticBoost | R54 ordinary nil writer | CharacterSelect.lua2182 |
| C_CharacterServices.SetAutomaticBoostCharacter | R54 ordinary nil writer | CharacterSelect.lua2183 |
| SelectCharacter | R45 ANSWERS52/54 Retail/Forever, selection update before event | CharacterSelect.lua891 |
| GetSelectBackgroundModel / SetCharSelectBackground | R52 Retail/Forever comments80/83, unchanged host-row cascade | CharacterSelectUI.lua379 |
| MoveCharactersToModelFFXFrame | R58 Retail/Forever comment91, existing bounded 3D gap | CharacterSelectUI.lua403 |
| CanCreateCharacter | R43 ANSWER42 Retail/Forever, readiness and roster limit | CharacterSelect.lua819 |
| GetCharacterTimerunningSeasonID | R49 ANSWER64 Retail/Forever GUID record season | CharacterSelectListElements.lua815 |

Generated declarations are absent. R54 extension approved1296; narrow R43/R49 extensions approved1299, R45 approved1302 with a prior neutral split, R52/R58 approved1305 with separate prior neutral splits. Same registration flags and protected refusal policy; no new conversions.

## Known boundaries
- C_Login.DisconnectFromServer: no native evidence/producer. Exact original OnShow missing-call boundary approved1291; equality matcher accepts only profile-pinned diagnostic and has a different-error negative control. Retail211, PTR/Forever204. No native stub is added.
- IsCharacterListUpdatePending: no simulator producer and no native evidence on any profile, original PTR CharacterSelect.lua720. Left absent per1299 and queued by main for decomp. No Lua-error cleanliness claim for that path.
- Existing Retail R79 character_creation_submit_real_controls_wait_for_host_result remains RED at unchanged assertion1376 with identical diagnostic core SHA, explicit PRESERVATION approval1293. Other35 original Retail tests passed before work. Never label aggregate PASS.


`MoveCharactersToMapSceneFrame` at PTR CharacterSelectUI.lua392 is evidenced but unreached, not extended.
