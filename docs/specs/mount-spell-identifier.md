# Mount spell identifier inputs

Exact291 changes `C_MountJournal.GetMountFromSpell` argument 1 from `number` to `SpellIdentifier` for retail 12.0.5. Tests in `tests/mount_spell_identifier.rs` require the actual journal API and existing `world.mounts` records. See [Lua API architecture](../lua-api.md) for runtime context. The journal provider queries existing mount state; no new model state or production seed data is added.

## What it must do

### Public identifiers — bounded simulator policy

- [x] Resolve public numeric spell 458 to existing mount 6; return nil for unknown spell 999999.
- [x] Resolve explicit same-ID aliases without changing the mount relation.
- [x] Prefer an explicit numeric-key alias over numeric identity: alias key `"458"` to existing spell 40192 returns mount 107 for numeric 458 and string `"458"`.
- [x] Resolve explicit name aliases through lowercase registry keys. An explicitly registered numeric string or link-shaped string may resolve to spell 458. The link fixture embeds 40192 but maps to 458; its registered value wins.
- [x] Return nil for unregistered strings, including a mount name, numeric string, empty string, or link-shaped string; return nil for an alias whose spell has no mount relation.
- [x] Reflect updates to an existing mount's `mount_id` and `spell_id`, and removal of that actual record, without stale lookup results.
- [x] Reflect alias replacement/removal immediately, restoring numeric identity after numeric alias removal; keep alias registries environment-local.
- [x] Leave existing spell-to-mount relations and alias entries unchanged during successful, missing, or rejected queries.

### Inferred conservative validation — not native parity

- [x] Reject omitted/nil arguments, booleans, tables, functions, threads, and actual frame userdata with a nonempty error.
- [x] Before alias resolution, reject nonfinite, negative, fractional, or out-of-u32 public numbers and invalid UTF-8 strings. Accept numeric endpoints 0 and 4294967295, including explicit endpoint aliases.
- [x] Reject actual VM secret NUMBER and STRING values without declassification, replacement, or caller-taint changes. Preserve public recovery in secure and tainted callers after GC. **`AllowedWhenTainted` remains UNMODELED; this rejection earns no native credit.**

These bounded simulator requirements passed all 16 focused cases locally and on desktop. Independent verifier654 accepted local model/check/readability evidence. Main accepts the bounded `mount-spell-identifier` capability from producer `22ea15a23`; source291 remains **AUDIT-PENDING** because native alias grammar, `AllowedWhenTainted` and the remaining overall contract are not established. Separate desktop saved-proof acceptance below does not establish headless CASC integration.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing identifier policy](action-spell-slot-identifiers.md)

## Implementation inventory

- `tests/mount_spell_identifier.rs` — 16 focused retail-12-0-5-gated tests calling the real `C_MountJournal.GetMountFromSpell`; no API override or stub.
- `src/lua_api/state_types/collections.rs` — existing `MountData` fields `mount_id: u32` and `spell_id: u32`.
- `src/lua_api/state_defaults.rs` — existing Brown Horse 458→6 and Ashes of Al'ar 40192→107 fixtures; neither is changed.
- `src/lua_api/state.rs` — existing `spell_id_aliases` registry, explicitly populated only by these tests.
- `src/c_api/c_spell.rs` — existing alias-first identifier helper and inferred public validator; existing `C_Spell.GetMountFromSpell` companion remains unchanged. No native intent or removal claim.

B77 supplied inventory: producer `22ea15a23` adds `src/c_api/c_mount_spell_lookup.rs`, module wiring and journal registration. Alias-first validated identifiers query existing `world.mounts` read-only, returning one numeric mount ID or nil. The `C_Spell` companion remains unchanged; native `AllowedWhenTainted` is **UNMODELED**. Earlier “no provider” meant no real modeled provider, not absence of the API: compiled RED called the actual callable generic-nil API.

B77 inputs `23e38bee4`: compilation exit0 in **120.163340s**, zero diagnostics; **16 cases = 1 PASS / 15 FAIL**, run **2.210321s**, against the actual generic-nil API. Producer `22ea15a23`: compilation exit0 in **120.005424s**, zero diagnostics; **16/16 PASS**. Independent verifier654 accepted bounded model, local check and scoped formatting/readability evidence; builds were dirty-combined, not whole-tree frozen.

Desktop helper execution recorded revision `cc2b0a68d`: **16/16 PASS**, helper exit0, **212.064s**; native `--check` exit0, **359.037s**, in Ubuntu WSL `OssoBuild` as `osso-test`. Transferred-tree cleanliness/revision was not independently established. A separate build upload stalled before Cargo and main cancelled its owned process group (exit -15); this is not compile proof. At that historical checkpoint desktop `--run`, startup and Blizzard UI/CASC readiness were **UNVERIFIED**; current qualified saved acceptance below supersedes only the bounded startup/three-texture evidence gaps. All tests were asynchronous. Proofs: `/tmp/patch-12.0.5-mount-independent-proof.{md,json}` and `/tmp/patch-12.0.5-remote-helper-real-proof.{md,json}`.

## Independent qualified saved acceptance — 2026-10-03

Main accepts independent674 **QUALIFIED SAVED PASS** in `/tmp/patch-12.0.5-desktop-casc-independent-proof.{md,json}`. Earlier B77 local16/desktop16/check acceptance by654/653 is retained without reruns. These are separate proof scopes:

| Scope | Saved evidence | Credit |
|---|---|---|
| Desktop headless startup | `ea7b6cc41`; authorized `python3 scripts/build-host.py --build-host desktop --run -- --no-saved-vars --no-addons lua-errors`; exit0 in107.869642s, runtime marker observed7.545953s; full drained streams,290 Blizzard addons, `[]`, `casc=false` | Headless startup only; no headless CASC integration |
| Existing native `casc_smoke` | `e40569d65`; build0/43.405748s, runtime0/34.404647s; actual fresh CASC extraction FDIDs130828/131071/134400, decoded128x32/64x64/64x64 | Three asserted TextureManager texture loads/decodes only |
| Prerequisites |4041 manifest files,0 missing | Existence/layout only, not content/version provenance |

Original `04586` wrapper exit1 remains historical: missing `rustup` before Cargo, not compiler/CASC failure. Three successful texture probes do not mean all advisory direct lookups succeeded:11 probes yielded8 Some/3 None. Audio/ALSA and absent WTF diagnostics remain disclosed.

Captured inner stdin/build/run arguments, native executable hashes, whole-source frozen identity and install/cache content-version provenance are missing. Revision labels and matching fixture hash do not bind a complete clean native workspace. Later worker source may corroborate arguments, not supply immutable captured input. Saved full streams were inspected by independent674; this docs update executes nothing and claims no new runs.

Main adds one bounded capability, `mount-spell-identifier`: **83 capabilities** (prior82), **167 pending / 174 bounded / 14 partial / 7 metadata,362 source IDs** unchanged. Source291 remains AUDIT-PENDING; source245/B78 unchanged. No GUI/frame rendering, font decode, all-asset completeness, native API, protected-path, global-format, profile or full-suite readiness credit. Original broad suite **12,022 PASS / 60 FAIL / 18 ignored**, plus **11 custom FAIL**, remains uncleared.

## Tests asserting this spec

- `tests/mount_spell_identifier.rs` — all requirements above, in an ordinary source file with no added Cargo target.
- `tests/action_spell_slot_identifiers.rs` — source reference for explicit lowercase alias keys, alias precedence, and rooted actual VM secrets; not mount-specific proof.
- `tests/c_spell_probes.rs` — unchanged numeric 458→6 control on the distinct `C_Spell` companion; not proof for the journal namespace.

### Fixture types and assumptions

`world.mounts` is `Vec<MountData>`; tests use real records, not a separate spell-to-mount test map. `spell_id_aliases` is the existing `HashMap<String, u32>`. The test snapshot contains `Vec<(u32, u32)>` spell/mount relations and the alias map; readonly assertions cover those inputs, not every world field. Relation lifecycle tests mutate an existing record to mount 6006 / spell 999998 or remove mount 6; they add no records. Host secrets are actual rilua NUMBER/STRING wrappers, rooted in globals and a Lua table before GC, not public values relabeled by a test stub.

Names, numeric strings, and colored links are **explicit registry aliases only**. No registered spelling, special link parser, native grammar, or native alias parity is assumed. Wrong-type/domain errors and secret rejection are deliberately inferred from the existing public helper's simulator policy, not established by the cached declaration.

Primary source: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/MountJournalDocumentation.lua`, lines 249–261 declares `C_MountJournal.GetMountFromSpell(spellID: SpellIdentifier) -> mountID?` and `SecretArguments = "AllowedWhenTainted"`. It establishes the namespace and argument delta, not alias grammar or native secret rejection. Scoped investigation: `/tmp/patch-12.0.5-mount-namespace-exact-map.md`.

## Known gaps (current cycle)

- [x] Committed inputs, compiled behavioral RED against the actual journal API, implemented provider, and obtained focused GREEN plus independent bounded verification.
- [x] Main accepts separate qualified saved desktop headless startup and three-texture CASC decode evidence; no reruns or integration credit.
- [ ] GUI/frame/font/all-asset readiness, immutable native input/binary/source identity and content-version provenance remain unverified; source291 overall contract remains AUDIT-PENDING.
- [ ] Native `AllowedWhenTainted` secret-input permissions remain unmodeled and unverified.

## Out of scope

- New model state, new production fixtures, and changes to the `C_Spell` companion.
- Native alias grammar/parity, special link parsing, native intent claims, duplicate-spell mount selection policy, and other mount APIs.
- Protected aura paths, additional Cargo targets, broader audit accounting, legacy profile parity, and native probes.
- Whole-suite acceptance and desktop readiness inferred solely from focused tests/checks.
