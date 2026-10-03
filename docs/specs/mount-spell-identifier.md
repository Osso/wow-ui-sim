# Mount spell identifier inputs

Exact291 changes `C_MountJournal.GetMountFromSpell` argument 1 from `number` to `SpellIdentifier` for retail 12.0.5. Tests in `tests/mount_spell_identifier.rs` require the actual journal API and existing `world.mounts` records. See [Lua API architecture](../lua-api.md) for runtime context. This inputs-only slice adds no provider, model state, or production seed data.

## What it must do

### Public identifiers — bounded simulator policy

- [ ] Resolve public numeric spell 458 to existing mount 6; return nil for unknown spell 999999.
- [ ] Resolve explicit same-ID aliases without changing the mount relation.
- [ ] Prefer an explicit numeric-key alias over numeric identity: alias key `"458"` to existing spell 40192 returns mount 107 for numeric 458 and string `"458"`.
- [ ] Resolve explicit name aliases through lowercase registry keys. An explicitly registered numeric string or link-shaped string may resolve to spell 458. The link fixture embeds 40192 but maps to 458; its registered value wins.
- [ ] Return nil for unregistered strings, including a mount name, numeric string, empty string, or link-shaped string; return nil for an alias whose spell has no mount relation.
- [ ] Reflect updates to an existing mount's `mount_id` and `spell_id`, and removal of that actual record, without stale lookup results.
- [ ] Reflect alias replacement/removal immediately, restoring numeric identity after numeric alias removal; keep alias registries environment-local.
- [ ] Leave existing spell-to-mount relations and alias entries unchanged during successful, missing, or rejected queries.

### Inferred conservative validation — not native parity

- [ ] Reject omitted/nil arguments, booleans, tables, functions, threads, and actual frame userdata with a nonempty error.
- [ ] Before alias resolution, reject nonfinite, negative, fractional, or out-of-u32 public numbers and invalid UTF-8 strings. Accept numeric endpoints 0 and 4294967295, including explicit endpoint aliases.
- [ ] Reject actual VM secret NUMBER and STRING values without declassification, replacement, or caller-taint changes. Preserve public recovery in secure and tainted callers after GC. **`AllowedWhenTainted` remains UNMODELED; this rejection earns no native credit.**

All requirements remain unchecked: tests are authored but neither compiled nor executed in this inputs-only task.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing identifier policy](action-spell-slot-identifiers.md)

## Implementation inventory

- `tests/mount_spell_identifier.rs` — 16 focused retail-12-0-5-gated tests calling the real `C_MountJournal.GetMountFromSpell`; no API override or stub.
- `src/lua_api/state_types/collections.rs` — existing `MountData` fields `mount_id: u32` and `spell_id: u32`.
- `src/lua_api/state_defaults.rs` — existing Brown Horse 458→6 and Ashes of Al'ar 40192→107 fixtures; neither is changed.
- `src/lua_api/state.rs` — existing `spell_id_aliases` registry, explicitly populated only by these tests.
- `src/c_api/c_spell.rs` — existing alias-first identifier helper and inferred public validator; existing `C_Spell.GetMountFromSpell` companion remains unchanged. No native intent or removal claim.

The scoped namespace map found no journal provider. Registering/implementing the journal API is a separate producer step, only after main-owned compiled behavioral RED.

## Tests asserting this spec

- `tests/mount_spell_identifier.rs` — all requirements above, in an ordinary source file with no added Cargo target.
- `tests/action_spell_slot_identifiers.rs` — source reference for explicit lowercase alias keys, alias precedence, and rooted actual VM secrets; not mount-specific proof.
- `tests/c_spell_probes.rs` — unchanged numeric 458→6 control on the distinct `C_Spell` companion; not proof for the journal namespace.

### Fixture types and assumptions

`world.mounts` is `Vec<MountData>`; tests use real records, not a separate spell-to-mount test map. `spell_id_aliases` is the existing `HashMap<String, u32>`. The test snapshot contains `Vec<(u32, u32)>` spell/mount relations and the alias map; readonly assertions cover those inputs, not every world field. Relation lifecycle tests mutate an existing record to mount 6006 / spell 999998 or remove mount 6; they add no records. Host secrets are actual rilua NUMBER/STRING wrappers, rooted in globals and a Lua table before GC, not public values relabeled by a test stub.

Names, numeric strings, and colored links are **explicit registry aliases only**. No registered spelling, special link parser, native grammar, or native alias parity is assumed. Wrong-type/domain errors and secret rejection are deliberately inferred from the existing public helper's simulator policy, not established by the cached declaration.

Primary source: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/MountJournalDocumentation.lua`, lines 249–261 declares `C_MountJournal.GetMountFromSpell(spellID: SpellIdentifier) -> mountID?` and `SecretArguments = "AllowedWhenTainted"`. It establishes the namespace and argument delta, not alias grammar or native secret rejection. Scoped investigation: `/tmp/patch-12.0.5-mount-namespace-exact-map.md`.

## Known gaps (current cycle)

- [ ] Main must commit this tests/spec slice, then obtain asynchronous compiled RED against the actual journal API before producer work. A compile error is not behavioral RED; a missing provider is a separate surface gap, not proof of alias semantics.
- [ ] Journal provider implementation and subsequent focused GREEN/independent acceptance remain pending.
- [ ] Native `AllowedWhenTainted` secret-input permissions remain unmodeled and unverified.

## Out of scope

- Production edits, new model state, new production fixtures, and changes to the `C_Spell` companion.
- Native alias grammar/parity, special link parsing, native intent claims, duplicate-spell mount selection policy, and other mount APIs.
- Protected aura paths, additional Cargo targets, broader audit accounting, legacy profile parity, and native probes.
- Builds, test/check execution, delegation, and commits in this inputs-only task; main owns compiled RED and subsequent producer work.
