# Crafted quality producer — bounded source/spec gate

Revision: `05665ddad6aa083406777f75887504a8e54c5725` (observed HEAD).
Scope: committed source/spec and manual Rust readability only. Both changed working files equal their exact committed text. No canonical edits, delegation, Bash, builds, checks, formatting, tests, runtime, vendor/service operations, or broad source hashing performed.

## Result

**SOURCE PASS; SPEC FAIL (stale execution-state statements); overall bounded gate FAIL.**
Runtime GREEN not run. Independent PTR gate `4189` remains pending; no closure inferred. This is not GC, native grammar, native security, rendered/native panel, or runtime preservation evidence.

## Four artifact checks

- [EXIST] PASS — `git show --name-status` identifies exactly two modified files: `src/c_api/c_trade_skill_quality.rs` and `docs/specs/crafted-item-quality-info.md`. Exact commit blobs and matching working files read; 135 and 57 lines respectively.
- [SUBSTANTIVE] PASS — getter lines 69–88 performs original-secret access, type validation, existing parser lookup, cloned explicit-record retrieval, one-nil miss, and shared projection. No placeholder implementation.
- [WIRED] PASS — `src/lua_api/globals/missing_surface.rs:217–218` calls `c_api::register_interaction_tables`; reexport `src/c_api/mod.rs:357–361`; `src/c_api/registration.rs:76–77` calls module registration under `retail-12-0-0`; module declaration `src/c_api/mod.rs:206–207` has the same gate. Producer lines 33–40 bind `GetItemCraftedQualityInfo` only when ACTIVE is Retail or Ptr.
- [ANTI-PATTERN] PASS — added TODO/FIXME/HACK/XXX counts all zero; added allow/expect suppressions zero. No extra marker or commented-out implementation introduced.

## Source coverage matrix

| Requirement | Evidence at this revision | Proof level |
|---|---|---|
| Original VM secret precedes type validation | Producer:70 calls `unwrap_secret(state, stack_val(state, 1))?`; :71 then checks Num/Str. No prior string conversion or synthetic wrapper. | Source ordering only; VM guard behavior not executed |
| Existing parser and reexport | Producer:2,:76; `item_spell/mod.rs:21–25`; `item_spell/c_item.rs:19–42`. Numeric positive cast, existing decimal-string/item-string/hyperlink path reused. | Parser/reexport files byte-for-text identical to parent; no native grammar claim |
| Explicit independent map only | Producer:77–80 reads `.crafted_item_quality.get(&item_id).cloned()`; `sim_state.rs:152–157` distinguishes recipe tuple map from `HashMap<u32, CraftingQualityInfo>`; `state.rs:310` initializes default empty map. | Source only; maps/state files unchanged |
| Clone before allocation | Clone statement completes before helper at :87. Rust-owned cloned String record remains alive throughout helper; no live state borrow spans helper. | Source lifetime reasoning only |
| Exactly one nil miss | Producer:81–85 funnels unparsed/missing records to `state.push(Val::Nil); return Ok(1)`. Invalid value kind is error, not miss. | Source only |
| Fresh complete snapshot and root ordering | :91 creates table; :92 pushes it before :93–98 numeric-key insertion and :113 string allocation. Helper uses borrowed cloned record, no host-record reference stored in Lua. | Source only, not GC-stress proof |
| Shared projection, no duplicate DTO | Existing CraftingQualityInfo at :10–25; both recipe getter :66 and crafted getter :87 use helper :90–117. | Source only |
| Existing recipe behavior preserved in source | Parent comparison: read_integer and reagent_link exactly unchanged; recipe getter pre-projection prefix identical; extracted projection statements identical, with return count adapted from Ok(1) to 1. | Runtime preservation still requires later execution |
| Profile feature gates | Cargo.toml:120–124 epoch chain; :153 Retail selects profile-retail + retail-12-1-0; :154 PTR selects retail-12-1-5; client_profile.rs:154–174 defines ACTIVE for each. Both inherit retail-12-0-0. Producer:37 excludes other ACTIVE variants. | Source wiring only, no feature builds |

## All thirteen projected fields

| Lua key | Rust field | Lua value construction |
|---|---|---|
| quality | quality: u32 | Val::Num(f64::from(record.quality)), :96–97 |
| icon | icon: String | create_string → Val::Str, :100 |
| iconSmall | icon_small: String | create_string → Val::Str, :101 |
| iconInventory | icon_inventory: String | create_string → Val::Str, :102 |
| iconMixed | icon_mixed: String | create_string → Val::Str, :103 |
| iconAppear | icon_appear: String | create_string → Val::Str, :104 |
| iconDissolve | icon_dissolve: String | create_string → Val::Str, :105 |
| barFill | bar_fill: String | create_string → Val::Str, :106 |
| barBackground | bar_background: String | create_string → Val::Str, :107 |
| barBackgroundCap | bar_background_cap: String | create_string → Val::Str, :108 |
| barHighlight | bar_highlight: String | create_string → Val::Str, :109 |
| iconChat | icon_chat: String | create_string → Val::Str, :110 |
| iconQuestObjective | icon_quest_objective: String | create_string → Val::Str, :111 |

Each string pair uses loop :113–114. `methods.rs:367–378` constructs Val::Str; :532–534 allocates a fresh table. `methods.rs:557–590` inserts keys and temporarily roots the inserted value while interning the key. Table is already on state stack. These inspected source mechanisms establish allocation/root ordering, not executed GC survival.

## Spec findings

1. `docs/specs/crafted-item-quality-info.md:38` states five supplied-model cases and whole-consumer candidate are “authored but unexecuted”. This contradicts updated :3 and :42 and the supplied execution checkpoint: five positive-query failures plus one original-consumer failure actually executed on each Retail/PTR profile before producer implementation. Correct the stale execution statement; retain unreached downstream assertions and pending GREEN.
2. `docs/specs/crafted-item-quality-info.md:48` still requests positive-data RED “before implementing projection” and calls the explicit map proposed. Source already implements projection; :42 records later positive RED. Reconcile this stale prerequisite with the remaining post-producer GREEN gate. A completed precondition is not downstream positive acceptance.

Spec :9 labels secret-ordering policy an interpretation rather than native observation; :15 explicitly excludes a name catalog/native validation claim; :53–57 leave native classification, exact errors and accepted representations unproved. Existing parser treats unresolved name strings as misses; no item-name catalog or stricter native grammar was added.

Consumer grounding: spec :19 separates host Frame from original professions panel. `tests/crafted_quality_item_button.rs` loads whole ItemButton addon closure, uses real Button/Texture and explicitly creates `ProfessionsFrame = CreateFrame('Frame', 'CraftedQualityVisibilityInput', UIParent)` as host visibility input. This source describes the fixture; it does not prove native panel behavior or downstream runtime assertions. Unchanged test-file “unexecuted” comment is historical, not new producer proof.

## Historical execution boundaries (not rerun)

Initial tracked RED README records two distinct initial cases on both profiles: four failed profile-target outcomes, exit 101, failing first required-input rejection. Later explicit nil/context and tainted-denial/recovery remained unreached. User-supplied checkpoint and updated spec :42 record five positive-query failures + one consumer failure on BOTH profiles before producer, after closure/empty-input baselines. No fresh runtime logs were reverified here; these historical outcomes are not GREEN. Downstream complete-field/security/visibility/clear acceptance remains pending. PTR gate 4189 pending, explicitly not closed.

## Manual Rust readability violations

None identified. No automated readability command run. Reviewed changed producer lines and extracted helper manually under rust-readability skill; no suppression or new TODO marker introduced.

## Proof ledger

- `git rev-parse HEAD`: exact requested revision, successful output.
- `git show --format=fuller --no-ext-diff --no-renames REV --`: full two-file diff inspected, successful output.
- Exact commit `git show REV:path` and bounded `git grep` reads: source/spec, parser/reexport, state, registration/profile gates, allocation helpers, initial RED README, consumer fixture inspected. No execution inferred from authored assertions.
- Direct parent text comparisons: integer reader, reagent link, recipe pre-projection and projection statements unchanged; parser/reexport/state files unchanged. No hashes used.
- Local text comparison: two changed working files match REV exactly.
- Added-line source scan: zero named markers/suppressions.
- All proof scopes remain limited to observed revision; later changes invalidate affected source evidence. Runtime proof absent by design.
