# Party map position — bounded default Retail receipt PASS

verified: 2026-10-10

Producer `40f3801705e2d930dc34c2716e8011234dd67295`; receipt submission `d25c1fdb444c2dd1c8b0c45dd4f8032b1ca75289`. Independent final report privately retained at `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/runtime-report.md`. No test/build rerun during retention.

| Scope | Actual result | Evidence |
|---|---|---|
| C_Map probes | 30 named PASS, exit0 | `map-probes.stdout` |
| C_Map API controls | 51 named PASS, exit0 | `map-api.stdout` |
| Compile | exit0, source equality | `compile-result.json` |
| Missing-only formatting/default check | both exit0, source equality | `checks/` |
| Original party positive | unchanged RED body becomes PASS | original RED retained separately |

Independent inspection matched all81 named successes to declarations, including two explicitly included `c_map_api::texture` child-module tests. The original controller incorrectly expected49, aborted after successful51 and skipped checks. Original receipt/correction preserved; `count-calibration-erratum.json` corrects the original sidecar's imprecise “expanded” explanation. Later checks fill only missing proof; they do not turn the original controller into a successful run.

Artifact SHA256 `6bf26583ca0c623df5a0cdc66044b21ea8553c9bfd2e38358d79b56b8d9d66df` independently recomputed and matched; execution records unchanged artifact. Captured3853 source hashes equal before/after and inspected current relevant source. Actual default/client-retail profile; no alternate profile credit. Six preexisting iced manifest deprecations remain: not warning-free.

Explicit normalized, map-associated active party-member inputs are simulator state. Player behavior unchanged; absence, mismatch, inactive group and roster shrink/regrow controls pass. No native coordinate acquisition, projection, raid/invalid-token parity, all-profile, full-suite or whole-HEAD acceptance. External dependencies/source-to-artifact provenance, inherited environment, untracked files and runtime assets outside fixture inputs remain excluded. Binary, raw stderr, source maps and vendor payload omitted from tracked evidence.
