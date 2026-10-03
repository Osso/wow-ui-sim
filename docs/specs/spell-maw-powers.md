# Spell Maw-power identifiers

`C_Spell.GetMawPowerBorderAtlasBySpellID(spellID)` and `C_Spell.GetMawPowerLinkBySpellID(spellID)` query explicit host-declared strings using resolved spell identifiers. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 296–299 change both argument types: `# arg1.Type number -> SpellIdentifier`. Source IDs: `global api-C_Spell-GetMawPowerBorderAtlasBySpellID-297` and `global api-C_Spell-GetMawPowerLinkBySpellID-299`.

There is **no cached declaration for `GetMawPowerBorderAtlasBySpellID` in current retail generated documentation**. Its one nullable string / nil-on-miss contract is **INFERRED**, not native-verified, from the existing simulator nil shim and cached Blizzard deprecated wrapper. Search of retail cached Lua/XML found no direct consumer call to this symbol: the only occurrence is this definition in `Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua:24–27`:

```lua
function C_Spell.GetMawPowerBorderAtlasBySpellID(spellID)
    local _rarityID, rarityAtlas = C_Spell.GetMawPowerRarityInfoBySpellID(spellID);
    return rarityAtlas;
end
```

The actual current border consumer calls the replacement rarity API, not the old border API. `Blizzard_MawBuffs/Blizzard_MawBuffs.lua:253–261`:

```lua
local rarityID, rarityAtlas = C_Spell.GetMawPowerRarityInfoBySpellID(self.spellID);
if rarityID then
    local quality = mawPowerRarityIDToItemQuality[rarityID];
    local atlasData = ColorManager.GetAtlasDataForMawBuff(quality);
    if not atlasData.atlas then
        atlasData.atlas = rarityAtlas;
    end

    self.Border:SetAtlas(atlasData.atlas, TextureKitConstants.UseAtlasSize);
```

Cached retail `Blizzard_APIDocumentationGenerated/SpellDocumentation.lua:132–146` declares `GetMawPowerLinkBySpellID`, `MayReturnNothing = true`, `SecretArguments = "AllowedWhenTainted"`, nonnil `spellID: SpellIdentifier` and nonnil `link: cstring`. **INFERRED** miss arity: zero results, not one nil. This supersedes the scout's proposed one-nil test policy; neither interpretation was native-tested.

Cached caller `Blizzard_MawBuffs/Blizzard_MawBuffs.lua:293–298` uses the legacy global spelling, not the namespace function:

```lua
function MawBuffMixin:OnClick()
    if (IsModifiedClick("CHATLINK")) then
        ChatFrameUtil.InsertLink(GetMawPowerLinkBySpellID(self.spellID));
        return;
    end
end
```

These cached sources establish contract context only; no vendor/cache files are modified and no legacy global is added.

## What it must do

### Explicit host inputs and results

- [x] Start with empty, independent per-environment atlas and link maps keyed by resolved spell ID. No production catalog, generic spell-link synthesis, or automatic rarity lookup.
- [x] Return exact supplied strings for distinct numeric keys, names and full colored-link aliases, including an alias whose embedded spell ID differs from its resolved ID.
- [x] **INFERRED** border contract: exactly one public string on a hit, exactly one nil on a miss.
- [x] **INFERRED** link miss contract: exactly one public string on a hit, zero results on a miss; stored empty strings remain hits. Public outputs are a chosen simulator policy, not native output-security proof.
- [x] Reflect map replacement/removal and alias replacement immediately; maps are independent, queries are read-only, environments are isolated, caller inputs remain unchanged through GC.

### Identifier boundary

- [x] Use the existing shared public reader and lowercase seeded aliases. Numeric aliases take precedence over numeric identity; string numerals and hyperlinks require explicit aliases. Alias removal restores numeric identity, not numeric-string coercion.
- [x] **INFERRED** shared representation policy: require a public UTF-8 string or finite integral u32 number, including zero and u32 maximum. Missing/nil, wrong types, invalid UTF-8, negative/fractional/nonfinite/out-of-range numbers error before lookup or alias resolution.
- [x] Unseeded aliases and absent resolved IDs produce each API's specified miss arity; never fall back to an old alias target or another catalog.

### Caller context and conservative security

- [x] Public identifiers work from secure and tainted callers without changing caller taint.
- [x] **INFERRED** conservative policy: reject authentic VM secret numbers, strings, unknown identifiers and wrapped wrong objects in both contexts. Do not unwrap or declassify secrets; rooted identity/secrecy survive GC and subsequent public queries recover.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing public identifier boundary](cooldown-aura-spell-identifiers.md)

## Implementation inventory

- `src/c_api/c_spell_maw_powers.rs` — empty-default `MawPowers` input and two state-backed query providers.
- `src/c_api/c_spell.rs` — unchanged shared `read_public_spell_identifier_at` validation and alias resolution.
- Wired in `src/c_api/mod.rs`, `src/lua_api/globals/register.rs`, `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`.
- Retirement pending in `src/lua_api/workarounds/temporary/spell_static_defaults.rs`; remove the replaced border shim and shim-specific test expectations.

## Tests asserting this spec

`tests/spell_maw_powers.rs` — ten grouped tests: empty/unknown misses with exact arity, numeric/name/link hits, numeric alias precedence, alias replacement to absent key, independent live maps, u32 endpoints/empty strings, malformed public inputs, read-only inputs/environment isolation, tainted public calls, authentic secret rejection/GC/recovery. Fixtures do not replace either API.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs and producer landed together in `a0e23199d`. RED with the producers withheld from the working tree: 0 PASS / 10 FAIL. GREEN: 10/10; the combined run was 396 PASS / 1 FAIL, the failure being `c_system_api::test_c_console_get_all_commands_empty` on an untouched console command count. `cargo fmt --check` exit0; startup `lua-errors` `[]`.

Main accepts an independent GPT-6.1-sol review: **ACCEPT WITH QUALIFICATIONS** (report SHA256 `c82a251027147d0fa6068c968bb8769cc466fb4634ffd683a37d2c7fe111561b`, scratchpad-only), own rerun 10/10 exit0. Row 297 is partial: no cached declaration, full-load reassignment by `Deprecated_12_0_7.lua`, retained older-profile shim. Checked requirements are bounded simulator proof on the tested fixtures, not native parity. No `cargo check`, broad suite or older-profile run.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows 297, 299 under new capability `spell-maw-powers`; **107 capabilities/362 IDs; 77 pending /239 bounded /13 partial /33 metadata**.

## Known gaps (current cycle)

- [ ] The guarded nil border shim in `src/lua_api/workarounds/temporary/spell_static_defaults.rs` is retained as the older profiles' only definition, and cached `Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua` reassigns the border query on a full load. Neither path is tested, so row 297 is partial.
- [ ] Cached Blizzard sources are newer than the 12.0.5 delta; direct old-border consumer evidence is absent. Native nullable-border contract, miss arity and malformed-input behavior remain unverified.

## Out of scope

- Native secret `AllowedWhenTainted` permissions: deliberately unmodeled; conservative rejection is not parity.
- Maw-power acquisition, rarity calculation, production atlas/link catalogs, generic spell links, legacy global registration, Blizzard UI rendering or cached source changes.
- Older profiles: provider and state field are gated to `retail-12-0-5`; the pre-existing guarded border shim still serves them.
- Native error wording, extra-argument behavior and native output-security policy: unverified.
