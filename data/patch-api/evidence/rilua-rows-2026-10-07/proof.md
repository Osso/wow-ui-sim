# Rilua capability row proof

Branch `rilua-rows`, base `13283ab24`. Behavioral build `83dfb7f16`; final verification build `5f2500dd7` differs only by removal of two known-gap fixture IDs. Later ledger/docs edits do not invalidate source/test proof. [Machine ledger](proof.json) contains commands, revisions, counts, target publication observations, exact remaining gaps and non-acceptance history. No independent review, full-suite, historical-epoch runtime or native-client parity claim.

## Row outcomes

| Contract | Outcome | Exact evidence and limits |
|---|---|---|
| 12.0.0 `dropsecretaccess`, `wt-global-api-dropsecretaccess-470` | Bounded closure | VM immediate-caller revocation and exact unwrap guard; secure/addon calls, descendant/securecall/coroutine-entry denial, unchanged taint, normal/error recovery. Inherited lifetime remains inferred VM policy. |
| 12.0.0 `issecrettable`, `wt-global-api-issecrettable-472` | Bounded closure | Wrapped table and SecretWrapContents metadata, ordinary table containing wrappers false, secure/addon/revoked query and GC. Addon availability is local policy despite cached AllowedWhenUntainted annotation; not full native permission parity. |
| 12.0.5 `Ambiguate`, exact416/prose038/wt538 | Bounded closure | Opaque host transformation accepts secret names from tainted/revoked callers and produces secret short/none outputs. Existing exact415 original-context rejection and public mapping controls pass. Native context enumeration/output propagation are inferred. |
| 12.0.5 `C_ChatInfo.ReplaceIconAndGroupExpressions`, exact253/wt376 | Bounded closure | Real typed icon/group vocabularies, independent flags, live updates/isolation, secret input/output, revoked/addon calls, wrong payload/flag denial, arbitrary byte preservation. Secret flags reject before malformed text. Cached English raid icon paths seeded; automatic roster/localized vocabulary/native full grammar not claimed. |
| 12.0.5 `prose-2026-03-12-050` | Still pending | VM blocker gone; normal owned event handler exhausts, PLAYER_LOGOUT/ADDONS_UNLOADING complete without refilling quota, reset recovers. Missing event-wide non-frame ownership/exemption and native time thresholds/periods. |
| 12.0.5 `prose-2026-03-25-095` | Still pending | Same two frame-dispatch-path proofs; addon-load/OnUpdate wiring inspected, not behaviorally proven. Cumulative instructions and inferred 10-million/default-frame reset are not native elapsed-time throttle parity. Chronology retained. |

## Verification

Each test filter ran as its own `cargo test --test integration FILTER` command, with combined output retained once and inspected from its saved log.

| Filter/check | Result |
|---|---|
| `rilua_rows` | 8 PASS |
| `ambiguate_context` | 22 PASS |
| `p1200_global_security` | 4 PASS |
| `patch_12_0_0` | 21 PASS; 1,010 publication observations, 989 OK / 21 exact known gaps |
| `patch_12_0_5` | 57 PASS; 363 publication observations, 352 OK / 11 exact known gaps |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; zero non-vendor warnings |
| `cargo fmt --check`; direct rustfmt of changed integration modules | Exit 0 |
| `timeout 90 target/debug/wow-sim --no-addons --no-saved-vars lua-errors` | Exit 0, `[]` |

Both patch filters initially failed before probing: other tests initialized the shared bytecode cache before `preload_full_game_ui` entered parent bypass. Same scopes reran with **`WOW_SIM_ENABLE_BYTECODE_CACHE=0`**, source loading unchanged; no harness edit. This execution configuration is required for these grouped-filter proofs. Startup used the same configuration. Only the two helper publication IDs were removed from 12.0.0's fixture; 12.0.5 fixture unchanged.

All Cargo runs retain six pre-existing vendored `iced-wgpu-patched/Cargo.toml` deprecated hyphenated lint-key warnings (plus the summary line). Vendor edits forbidden; no warning suppressions added. Changed Rust was manually checked for readable transform/dispatch boundaries, bounded nesting, ownership wiring and no new placeholders/suppressions.

## Contract grounding

- Cached retail `Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua:133–136`: immediate calling function loses secret access; `:246–260`: wrapped table or table flags producing secret contents. The latter declares `AllowedWhenUntainted`; metadata availability in addon callers is explicitly bounded policy, not permission-parity evidence.
- Cached retail `PlayerScriptDocumentation.lua:22–35`: `AllowedWhenTainted`, original context `NeverSecret`, non-nil string return. Existing shortening is simulator policy.
- Cached retail `ChatInfoDocumentation.lua:485–498`: `AllowedWhenTainted`, two optional `NeverSecret` booleans, non-nil string return. `Blizzard_ChatFrameBase/Shared/ChatFrameConstants.lua:30–105` gives icon/group dictionaries, not native complete expansion. Host maps are explicit input, not invented native roster state.
- [Retained 12.0.5 source](../../sources/12.0.5-api-changes.txt), lines 50 and 95, exempts PLAYER_LOGOUT/ADDONS_UNLOADING processing; no native instruction threshold is declared.
- [Contract](../../../../docs/specs/rilua-capability-rows.md), [12.0.0 ledger](../../sources/12.0.0-page-coverage.json), [12.0.5 ledger](../../sources/12.0.5-page-coverage.json).
