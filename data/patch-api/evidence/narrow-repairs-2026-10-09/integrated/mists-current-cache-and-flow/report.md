# Current Mists cache and currency flow — sanitized retention

Retained: 2026-10-10. Parent **OPEN**. Summaries of three saved independent reports, not new execution or independent revalidation of their underlying artifacts. Original report byte sizes and SHA-256 digests are in `retention-manifest.json`; originals remain private.

## Bounded results

| Report | Recorded proof | Limit |
|---|---|---|
| native-flow-current-independent | Strict currency backing/panel flow **scoped PASS**: all **13 unconditional assertions** precede the sole success marker; **19 populated rows / 3 checked rows** observed. Original/current decoded script bytes equal: 1,942 bytes. | Overall command **exit1**; not clean startup or native-client parity. Counts are observations, not exact-count assertions. |
| cache-tests-epoch-independent | Completed Mists library-test compile **exit0**; selected cache tests **6 PASS / 0 FAIL, exit0**, 1,835 filtered out. Selected executable seal matches; 3,849 captured source hashes equal before/after. | Earlier resource-blocked/incomplete checkpoints are historical; final authorized artifact update supersedes them only for this selected boundary. Not full-suite or untracked integration-test proof. |
| cache-guard-independent | Normal binary build **exit0**, recorded build-scope equality for 3,849 entries; normal executable seals match. Sync **exit0**, 2,974 extracted / 0 already present. | Build depends on **dirty Mists manifest/listfile**; bare commit is not equivalent input. Sync had an intermediate extraction diagnostic; no independently sealed per-file cache inventory. |

Startup remains **FAIL, exit1**: **20 primary records / 22 occurrences**, plus matching wrappers (raw total 40/44). Flow output has the same decoded startup error array; scoped assertion success does not erase overall failure. Six dependency-manifest deprecation warnings remain; no warning-free build claim.

Normal build revision: `15b7dac2e2914555d2ae5e393060474f57eb8b46`. Library-test revision: `ae42b23d11a56568ffc0b445afee951d11d0646d`. Both depend on refreshed uncommitted Mists manifest/listfile inputs. Equality excludes external path dependencies, uncaptured data, inherited environment, runtime cache/addon inputs and untracked index; not hermetic provenance, current-HEAD acceptance or execution-time cache closure.

## Selected hashes reported by audits

These are reported artifact/input digests, not fresh binary or input rehashes by this retention task.

| Item | SHA-256 |
|---|---|
| Strict common script | d5f8419dea4391d6976fe873ad5d4751d6b1a69cab11b19b27a18f26056cbc99 |
| Normal simulator | cfbe626d02c99bce775f63a82a9428ea0e97a31c9c994f5c696f2af9a2b45dc2 |
| Normal CLI | 5da728a439ce65f9468b6af7e846eb6f2a8238ff125ab327a26543dd715d5d41 |
| Selected library-test executable | 2ce2b5dbabfc4c3488583ffa0189cf6105af4cfd1167f7a79a8006f3945ecb1b |
| Dirty Mists manifest | 68b2f5cb89f6aff7f5eb43b248f61e0ed38f7e4a6621e035c8deb4922516b7ad |
| Dirty listfile | 8b97f9b061df98b11b343fce8f73a7de37eb4dd9cdafaed7e2ff66fb2aaf567c |

## Retention boundary

Only this sanitized summary and digest manifest retained. No original report copies, vendor/source payload, script body, binary, secrets, full stdout/stderr, raw product database, source/cache inventories or full-case inventories. No new build/test/sync, operations, delegation, staging, commit or push. No manifest, baseline or code-index modification. No broad CurrencyGREEN, clean baseline, full-profile acceptance or publication authorization; parent **OPEN**.
