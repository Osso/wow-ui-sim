# Branch reconciliation

Reconciled 36 non-patch-equivalent commits in seven branch groups. Initial master was `34d8600fd`; master advanced during verification, so the five reconciliation commits were rebased cleanly onto observed master `5f62f6944`. Only `branch-reconcile` changed; no push, branch deletion, canonical checkout edit, vendor edit, or model delegation occurred. This records reconciliation, not a merge into master.

## Content

### Method

`git cherry -v master <branch>` identified positive patches. Every original `git show` diff was captured and compared with current code. Matching historical master commits were also compared by changed content, excluding blob IDs and hunk offsets; differences were checked for relocation, formatting, stale progress prose and later architecture changes. An empty Forever cherry-pick was skipped after verifying its document already matched master exactly. Local `classic-profile-rollout` has zero positive cherry entries, although it is not an ancestor of master; its rebased patches are present.

### Per-commit decisions

`present` means `superseded-present`; `obsolete` means `superseded-obsolete`. Branch abbreviations are defined below.

| Branch | Original commit | Decision | Evidence |
| --- | --- | --- | --- |
| forever | `3a397b363` | present | `362b65c7f`; `docs/wowforever-1.60.1-ui-api-deltas.md` identical |
| tooltip | `ab4878d10` | ported | `72cb97e8a`; nested-color API-to-sizing test; existing `tooltip.rs:measure_tooltip_text_width` strips markup |
| sleek | `7f4492eae` | ported | `4cd0718fe`; `render_textures.rs:cursor_spell_icon_id` |
| sleek | `dca908297` | ported | `b4e2542c8`; `screenshot.rs:build_screenshot_batch`, `render_textures.rs:cursor_overlay_position` |
| sunny | `7a9f28d8e` | present | `1cc042578`; `limited_listfile.rs:parse_bundled_listfile`, canonical font paths |
| sunny | `236c7832e` | present | `a8734b22b`; `font.rs:try_casc_font_bytes_by_encoding_key` |
| sunny | `254375a4e` | obsolete | `asset_resolver_config.rs:config` supplies cache-root/data explicitly instead of creating a game-engine checkout; original target-size brief is already retained and updated for the consolidated harness |
| windows | `34da5f5d5` | obsolete | `font.rs:WowFontSystem::new` loads CASC fonts without bundled fonts; pure-Rust cascette reader replaces proposed casc-lib implementation |
| windows | `ab4ea0c86` | ported | `14645a845`; removed tracked leftover `Interface/AddOns/Blizzard_FrameXML/GuildInviteFrame.xml`; runtime loads profile cache, no local TOC referenced this file |
| classic | `9bef56d03` | present | `1b6a85804`; `client_profile.rs:ClientProfile`, profile cache selection replaces old vendor symlinks |
| classic | `44f1df314` | present | `4edef81c2`; `wrath/frame_methods.rs:register_all`, event validation |
| classic | `670ac36a4` | present | `f4f996bd5`; `wrath/compat_bootstrap.lua` |
| classic | `60d7060f2` | present | `040f378bb`; `wrath/mod.rs`, bootstrap/frame-method modules |
| classic | `c40f4f75a` | present | `7df5f088f`; `wrath/compat_bootstrap.rs:init_wrath_only_proxies` |
| classic | `cbd051130` | present | `75986799d`; `toc/tests.rs:test_wrath_interface_version`, `test_mists_interface_version` |
| classic | `fe4893277` | present | `c2a3c29f1`; `toc/mod.rs`, `toc/tests.rs` |
| classic | `e4faed48d` | present | `49ca652c0`; `wrath/post_load.lua:ScrollingEdit_OnUpdate` guard |
| classic | `ff3856861` | present | `43b6000f9`; `lua_api/workarounds/mod.rs`, logging module |
| classic | `71cd05682` | present | `480ab5df4`; `loader/error.rs:LoadError::IoWithPath`, `xml/parse.rs:parse_xml_file` |
| classic | `858939819` | present | `d8e8b1ad1`; `lua_errors.rs` exec-lua stdout routing |
| classic | `56f064de7` | present | `156fadbd9`; bootstrap nil guard plus modeled `inventory_verbs.rs:drop_cursor_money` |
| classic | `83949f0b9` | present | `f0949bc44`; formatting/module order preserved, subsequent cache-path rewrites supersede old formatting hunks |
| classic | `a687ee7cd` | present | `69a2b41d8`; Gethe Wrath migration retained historically; current `data/blizzard-ui-files/wrath.txt` owns runtime source selection |
| classic | `0ca60061b` | present | `acb7d9ca6`; `tests/pandaria_installed_addons.rs`, consolidated integration harness |
| classic | `3a5b9c814` | present | `eca764ef7`; `toc/mod.rs:game_subdir`, Mists expansion helpers |
| classic | `afe91a576` | present | `70f713a94`; Mists forbidden-frame forwarding, template texture aliases, relocated source patches |
| classic | `e44fccf6a` | present | `a41679b2d`; `xml_layer_batch.rs`, quest registration, EditMode updater guard/test |
| classic | `777f85bab` | present | `9d0a4f12b`; `pvp_probes.rs`, `state_types/pvp.rs:PvpHonorState` |
| classic | `e7dfaf1a5` | present | `177babeff`; `tests/mists_world_map_opacity.rs` |
| classic | `5d8404671` | present | `dd0fe833a`; `tests/mists_nameplate_scale.rs` |
| classic | `6765febde` | present | `b60380a49`; missing-setter repro retained in `tests/classic_guild_roster_selection.rs` |
| classic | `dcfcbfe61` | present | `b35e6583a`; shared guild selection test and bootstrap state round-trip |
| classic | `bbca045d3` | present | `a007fa465`; paper-doll API, combat/unit stats, Mists post-load icons and panel regressions |
| backup | `3c9c236f7` | present | `adf20c649`; rendering, TOC/XML and loader changes present; stale ACCOMPLISHED prose not revived |
| backup | `d7a624837` | present | `3e60da01a`; addon namespace/button methods and tests retained; old progress prose obsolete |
| backup | `1d7ae08fb` | present | `e0b56b0a0`; unit events, popup globals, loaded-addon state and loader varargs retained |

Branch names: forever = local/remote `forever-ui-api-report`; tooltip = `wow-ui-sim-tooltip-fix`; sleek = `worktree-sleek-smiling-lemur`; sunny = `worktree-sunny-painting-raccoon`; windows = `origin/codex/windows-porting`; classic = `origin/classic-profile-rollout`; backup = `backup/pre-drop-spells`.

### Port inventory

- `72cb97e8a`: nested color markup regression. Original bug does not reproduce; current measurement already strips markup. Existing hidden-item-link measurement test does not cover this API-to-renderer nested-color boundary.
- `4cd0718fe`: talent cursor icon. Baseline emitted one pointer request instead of icon plus pointer.
- `b4e2542c8`: host-state cursor position and live screenshot overlay. Baseline emitted zero requests for host-only mouse input and ended screenshots with UI texture requests instead of pointer geometry.
- `1eb46d4ce`: share existing talent spell lookup between cursor and tooltip consumers, avoiding duplicated interpretation; preserves selected-node/entry/definition handling.
- `14645a845`: remove stale local FrameXML copy. No vendor/cache file was changed.

### Proof ledger

All test/build helper invocations used `python3 /home/osso/.worktrees/wow-ui-sim-branch-reconcile/scripts/build-host.py --build-host local`. Logs are retained in that worktree's ignored `target/branch-reconcile/`; this is an artifact folder, not another Cargo target root. Builds were debug, without build timeouts; runtime alone was bounded by `timeout 90`.

| Revision / scope | Command suffix | Result | Later-change validity |
| --- | --- | --- | --- |
| `34d8600fd` + original-boundary regression, production unchanged | `--test --test integration repro_tooltip_width -- --nocapture` | PASS 1/1 | Sizing/font/test files unchanged except formatting |
| `1982bee7b` + talent regression, production unchanged | `--test --lib -- cursor_talent_icon_renders_in_overlay --nocapture` | Expected RED 0/1: pointer only | Superseded by GREEN |
| `a42559dbc` + screenshot regressions and test visibility | `--test --lib -- cursor_overlay_ --nocapture` | Expected RED 0/2: absent host cursor; screenshot lacks overlay | Superseded by GREEN |
| `e20032a28` | `--test --lib -- cursor_ --nocapture` | PASS 11/11 | Exact port/proof source bytes retained across rebase |
| `e20032a28` | `--test --test integration tooltip_talent -- --nocapture` | PASS 2/2 | Shared lookup and tests byte-identical after rebase |
| `ac2f7b874` | `--test --lib -- loads_real_wow_fonts_from_casc --nocapture` | PASS 1/1, real font test executed | Font code unchanged by rebase |
| `ac2f7b874` | `--test --test integration limited_listfile -- --nocapture` | PASS 4/4 | Listfile/test code unchanged by rebase |
| `ac2f7b874` | `cargo fmt --check`; helper `--check` | PASS both | Repeated only after upstream Rust changes were incorporated |
| `14645a845` on master `5f62f6944` | `cargo fmt --check`; helper `--check` | PASS both | Documentation-only follow-up does not invalidate |
| `e20032a28`, then `14645a845` | helper debug build; `timeout 90 python3 ... --build-host local --no-build --run -- --no-addons --no-saved-vars lua-errors` | PASS build and exit 0; JSON `[]` on both snapshots | Final integrated runtime evidence |

Six pre-existing deprecated Clippy manifest keys in `iced-wgpu-patched/Cargo.toml` remain unsuppressed; they were emitted before any production change and the manifest is unchanged. No new Rust warning was introduced. Changed-code readability review found and removed duplicated talent lookup; new helper functions are pure and bounded. No complete suite, Windows execution, GPU pixel comparison or new Classic-profile execution is claimed.

Rebase proof compared every changed Rust/test file plus tooltip sizing, font loading, listfile and talent test sources between `ac2f7b874` and `14645a845`: zero changed bytes. Targeted behavior evidence was retained instead of rerunning identical scopes; integrated check/build/startup were rerun because upstream Rust/registration changes affected those wider scopes.

### Abandoned directions

**bevy-migration:** its Cargo manifest selects Bevy 0.18 and mlua with an Elune source patch. Master selects iced and pure-Rust rilua. The renderer/VM migration direction is superseded; wholesale merging would reintroduce rejected architecture. Individual later map/render fixes were intentionally not audited, so this assessment does not authorize deletion of all unique work.

**c-lua:** its manifest retains mlua's vendored Lua 5.1 plus `lua-src = elune-src`; master uses rilua with interpreter-native taint and the current Rust bridge. The C-runtime direction is superseded, but its later professions/layout work was outside the requested audit. Retain the branch unless a separate per-commit equivalence review establishes deletion safety.

**save/classic-profile-rollout-before-master-rebase-20260519:** this is a pre-rebase snapshot already using rilua/iced, not a competing VM migration. Master contains the rebased classic rollout and now uses profile-scoped CASC manifests/cache and the consolidated test harness. Its historical rollout direction is superseded, but the saved branch was deliberately not audited commit by commit; retain as archival until separately cleared.

### Deletion safety

Safe by reviewed patch content already on master: local/remote `forever-ui-api-report`, local `worktree-sunny-painting-raccoon`, local and remote `classic-profile-rollout`, and local `backup/pre-drop-spells`.

After these reconciliation commits are integrated into master: local `wow-ui-sim-tooltip-fix`, local `worktree-sleek-smiling-lemur`, and remote `codex/windows-porting`. Their remaining value is preserved here, not yet on master. No branches or worktrees were deleted; dirty/active-worktree cleanup safety was not granted or investigated.

## Sources

- Original commit diffs and master counterparts identified in the decision table.
- [Cursor overlay contract](../../specs/cursor-overlay.md).
- [Forever report](../../wowforever-1.60.1-ui-api-deltas.md).
- [CASC loading contract](../../specs/casc-loading.md).
- [Integration target brief](../../rust-integration-test-target-size-brief.md).

## See Also

- [Client profiles](../systems/client-profiles.md) — replacement for legacy vendor/profile layout.
- [CASC font investigation](casc-fdid-1579624-root-debug.md) — font-casing and encoding-key evidence.
