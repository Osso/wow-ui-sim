# Base spell aura secrecy

`C_Secrets.GetSpellAuraSecrecy` classifies a spell from client `SpellMisc.Attributes_15` data. It reports base secrecy, not whether auras are currently secret in combat. The implementation is in `src/c_api/c_secrets.rs`, published for Retail 12.1+ and Forever through `aura-containers`. This availability change reuses the existing attribute dataset and classifier; it does not establish new Forever-specific data parity.

## What it must do

- [x] Return `NeverSecret` (0) for `AURA_NEVER_SECRET`, `AlwaysSecret` (1) for `AURA_ALWAYS_SECRET`, and `ContextuallySecret` (2) when neither attribute is present.
- [x] Reuse `C_Spell`'s existing numeric, numeric-string, and case-insensitive modeled-name resolver. Numeric identifiers retain that resolver's pass-through behavior; unresolved identifiers retain its nil result. Native unknown-identifier behavior has not been separately verified.
- [x] Make the namespace available before secure-environment copying; retain the existing native-valued `Enum.SecrecyLevel` and metadata in both environments.
- [x] Reject conflicting attributes explicitly rather than choosing undocumented precedence or affecting unrelated spells.
- [x] Drive Blizzard's `CanApplyIdentityCandidateFilters` never-secret exemption using actual spell data.

## How it works

The generated sorted table stores only the 76 base-difficulty spells carrying either relevant attribute. An absent entry means no aura secrecy override, not never-secret. Generation retains both bits for ambiguous data.

[Generated provenance](../../data/spell_aura_secrecy.provenance.json) is the source of truth for the export hash, build, definition-tree/blob pins, counts, and ambiguous IDs. `tools/gen_spell_aura_secrecy.py` validates the flag-definition blob against the supplied tree before writing output. It uses Python's standard library only.

Regenerate with the pinned local inputs:

```text
python3 tools/gen_spell_aura_secrecy.py --input /tmp/pi-SpellMisc-12.1.0.69497.csv --flags /tmp/pi-SpellAttributes15.dbdf --definitions-tree /tmp/pi-wowdbdefs-tree.json --build 12.1.0.69497
```

## Implementation inventory

- `src/c_api/c_secrets.rs` — namespace and classification.
- `src/c_api/c_spell.rs` — existing identifier resolver, shared without changing its behavior.
- `src/c_api/mod.rs` — shared `aura-containers` registration before secure copying.
- `data/spell_aura_secrecy.rs` — generated sparse attributes and lookup.
- `tools/gen_spell_aura_secrecy.py` — reproducible extraction and provenance validation.

## Tests asserting this spec

- `tests/spell_aura_secrecy.rs` — concrete classifications, identifier handling, enum metadata, ambiguity error, and actual Blizzard aura filtering.
- `tools/tests/test_gen_spell_aura_secrecy.py` — base difficulty selection, signed flags, dual-flag preservation, duplicate conflicts, required columns, and pinned definition identity.

## Known gaps

- [ ] Shared Forever publication GREEN is pending; `tests/spell_aura_secrecy.rs` now runs under `aura-containers`. No Cargo was run for this availability change. Full combat secrecy/access enforcement remains excluded.

Focused proof at `721836971`: four Rust tests passed, including the actual secure Blizzard filter. Four generator tests passed; regeneration produced byte-identical Rust and provenance files. Actual addon/SavedVariables startup later returned `[]`, exit 0 (`/tmp/pi-accepted-final-startup.*`), but this remains neither a broad check nor another-profile proof.
- Spell `1317008`, base row `863018`, carries both aura flags. Neither the supplied flag definitions nor the API documentation establishes precedence. Querying it raises an explicit ambiguity error. It is absent from the current compact spell metadata and local name export, but that absence is not used to discard its native attributes.
- Invalid/unknown-identifier native semantics remain unverified; this slice introduces no separate resolver or fallback classification policy.

## Out of scope

`ShouldAurasBeSecret`, combat/reaction-dependent secrecy, cast/cooldown secrecy queries, and guessed unconditional `NeverSecret` results.
