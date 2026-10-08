# Integrated 6.2.0 proof ledger

Runtime scope: `ddd76addc3bd451894316ab8c3575ff9e8ec0f35`. Master: `8dd11c1b90fb1f5ca925de62054a671179652f15`.
All logs retained; docs/evidence-only commits do not invalidate src/tests/tools proof.

| Command | Revision | Exit | Log |
|---|---|---|---|
| `cargo test --test prefork_full_ui -- publication_sweep` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [all-sweeps.txt](all-sweeps.txt) |
| `cargo build --bin wow-sim` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [build-startup.txt](build-startup.txt) |
| `cargo fmt --check` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [format.txt](format.txt) |
| `cargo test --test integration patch_6_2_0` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [integration-patch.txt](integration-patch.txt) |
| `cargo test --manifest-path /tmp/p620-master-c6eamwb2/Cargo.toml --test prefork_full_ui -- publication_sweep` | `8dd11c1b90fb1f5ca925de62054a671179652f15` | 0 | [master-all-sweeps.txt](master-all-sweeps.txt) |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [mists-check.txt](mists-check.txt) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p620-page/data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/negative_source.py` | `a91019f6eedbf54df533efdb7c57b39ea9e72b46` | 1 | [negative-source.txt](negative-source.txt) |
| `cargo test --test prefork_full_ui -- patch_6_2_0_publication_sweep` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 1 | [negative.txt](negative.txt) |
| `cargo test --test prefork_full_ui -- patch_6_2_0_publication_sweep` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [own-sweep.txt](own-sweep.txt) |
| `cargo test --test prefork_full_ui -- patch_6_2_0` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [prefork-patch.txt](prefork-patch.txt) |
| `cargo test --test prefork_full_ui -- tooltip` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [prefork-tooltip.txt](prefork-tooltip.txt) |
| `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p620-page/debug/wow-sim --no-saved-vars lua-errors` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 0 | [startup.txt](startup.txt) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p620-page/tools/test_check_patch_validators.py` | `73c8261c268af2e477e2e9ee46bbee591de00d4f` | 0 | [test_check_patch_validators.txt](test_check_patch_validators.txt) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p620-page/tools/test_extract_patch_non_inventory.py` | `73c8261c268af2e477e2e9ee46bbee591de00d4f` | 0 | [test_extract_patch_non_inventory.txt](test_extract_patch_non_inventory.txt) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p620-page/tools/test_gen_patch_wikitext_register.py` | `73c8261c268af2e477e2e9ee46bbee591de00d4f` | 0 | [test_gen_patch_wikitext_register.txt](test_gen_patch_wikitext_register.txt) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p620-page/tools/test_patch_audit_validation.py` | `73c8261c268af2e477e2e9ee46bbee591de00d4f` | 0 | [test_patch_audit_validation.txt](test_patch_audit_validation.txt) |
| `cargo test --test integration tooltip_basic::` | `a91019f6eedbf54df533efdb7c57b39ea9e72b46` | 0 | [tooltip_basic.txt](tooltip_basic.txt) |
| `cargo test --test integration tooltip_gc_rooting::` | `a91019f6eedbf54df533efdb7c57b39ea9e72b46` | 0 | [tooltip_gc_rooting.txt](tooltip_gc_rooting.txt) |
| `cargo test --test integration tooltip_item_sources::` | `a91019f6eedbf54df533efdb7c57b39ea9e72b46` | 0 | [tooltip_item_sources.txt](tooltip_item_sources.txt) |
| `cargo test --test integration tooltip_item_spell::` | `a91019f6eedbf54df533efdb7c57b39ea9e72b46` | 0 | [tooltip_item_spell.txt](tooltip_item_spell.txt) |
| `cargo test --test integration tooltip_spell_mount_identifiers::` | `a91019f6eedbf54df533efdb7c57b39ea9e72b46` | 0 | [tooltip_spell_mount_identifiers.txt](tooltip_spell_mount_identifiers.txt) |
| `cargo test --test integration tooltip_talent::` | `a91019f6eedbf54df533efdb7c57b39ea9e72b46` | 0 | [tooltip_talent.txt](tooltip_talent.txt) |
| `cargo test --test integration tooltip_` | `ddd76addc3bd451894316ab8c3575ff9e8ec0f35` | 101 | [integration-tooltip.txt](integration-tooltip.txt) |
| `cargo test --manifest-path /tmp/p620-master-tooltip-hz1etng_/Cargo.toml --test integration tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges -- --exact` | `8dd11c1b90fb1f5ca925de62054a671179652f15` | 101 | [master-tooltip-clamp.txt](master-tooltip-clamp.txt) |

51 saved registers / 48 extracts reproduced; three inherited failures unchanged.
Negative controls: zero-row injection rejected before probing; one-field source tamper rejected by provenance equality.
All other publication observations match immutable master byte-for-byte as JSON.
