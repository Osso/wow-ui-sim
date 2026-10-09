# Historical input isolation

`validate.py` always maps these mutable logical paths to frozen sidecars:

| Logical path | Sidecar |
| --- | --- |
| `data/patch-api/sources/4.1.0-page-coverage.json` | `historical-page-coverage.json` |
| `tests/data/patch_4_1_0_sweep_known_gaps.json` | `historical-known-gaps.json` |

Sidecars contain exact `git show 216568b5e:<logical-path>` bytes. The original 23-file manifest checks their original logical-path SHA-256 and byte length; missing sidecars fail without consulting current inputs. Seal checking and accounting use the same fixed mapping. Source files and own logs still use their pinned repository paths.

Original manifest, archive, logs and observations remain unchanged. Replay retains 81 publication rows, 50 matches, 31 gaps and 32 negative-control gaps. Current successor closures are independent of these receipts. This is neither historical compilation nor native parity nor current-head acceptance proof.
