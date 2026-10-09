# Bounded exact-group controller development

Canonical repo: /home/osso/Projects/wow/wow-ui-sim.
Implementation 6e6f8cb5b; setup correction 2de36e2b36a7643c53864b1bfddb317e206b9528.

| Contract | Group coverage | Development proof |
|---|---|---|
| Exact selection, skip/unmatched without setup | chat, cast-bar, spellbook | exact_group_selection GREEN |
| Child panic captured, nonzero target, later groups continue; distinct setup process IDs | each failing group among all three | exact_group_failure GREEN |
| Timeout direct child + descendant disappear; disk pack bytes/metadata unchanged; fresh process seals and succeeds | chat, cast-bar, spellbook | exact_group_timeout GREEN |
| Original four bodies | retained prior proof only | Not rerun |
| Non-Retail wrappers; broad independent acceptance | separate open gates | Not executed |

RED: red-run.stdout/result.json, 0/3 pass, exit 1, missing seam failures.
First GREEN: green-run.stdout/result.json, 0/3 pass, exit 1, one-line assignment incorrectly used eval.
Corrected GREEN: green-fix-run.stdout/result.json, 3/3 pass, exit 0, 3.00 seconds.

Builds use shared build-lock.sh, cargo test --offline --locked --jobs 4 --test prefork_full_ui --no-run --message-format=json, no compile timeout. Runtime uses timeout 90 binary conformance::exact_group_ --test-threads=1, PREFORK_CONFORMANCE_SUITE=1. *.start.json/*.result.json preserve argv, revision, hashes, exits and timestamps; stdout/stderr files preserve complete streams. Initial queued build capture failure and cancellation are in red-build-incident.txt; successful separately recorded retry is red-build-retry.*. No command was repeated solely to recover logs.

Corrected source SHA256 d3414b70ad02c2cf5cd73e971cfdfc8053b9e786c20df645ee2e30414ef85df0.
Corrected executable SHA256 98130da61450a476eb81dcd353a2ce3839d63fa97730d25cd1cc421f65717547.

Only tests/prefork_full_ui.rs and existing prefork spec/system doc changed. Normal original constructors/bodies/gates, normal orchestration/group subprocess path and cache transitions retained. Local controlled mode substitutes fixture registries/constructor, suppresses recursive conformance, records successful setup/seal; existing generic panic/tree bodies reused. No generalized runner hooks or reset paths.

No broad suite/check/lint/readability/final verification, network, delegation, operational changes, vendor edits or push. Six existing iced-wgpu manifest deprecations remain. Independent verifier/main retains acceptance and wiki log/index reconciliation.
