# Retail producer compilation / new sealed-copy authentication

PASS — existing producer compile and two new copied executables only.

## Build
Unit exit 0; Result=success; full compile.stdout parsed: 741 JSON records (665 compiler-artifact, 75 build-script-executed, 1 build-finished); final success=true. Zero compiler diagnostics; six manifest deprecations remain (not warning-free). Cargo duration 3m 21s. Unit interval October 13, 2026 21:07:56–21:11:18 CDT (epoch name uses October 14 UTC).
MemoryPeak 8,592,105,472 bytes: 2,170,880 above 8 GiB soft MemoryHigh, below 9 GiB MemoryMax.
Epoch: `<epochs>/crafted-quality-retail-green-retry2-20261014T020756467959Z`.

## Source / metadata association
Recorded source `cd9f42756472b8f8d706fd3e592f1977b3d89868`; production delta `05665ddad`. No source tree/archive authentication; no current-HEAD binary claim.
Cargo artifacts match retained normal-artifact-association.json; actual fingerprint matches retained metadata and Retail features for both outputs. Normal wow-sim and unique `target/debug/deps/wow_sim-a69cd69216e5522c` are the same inode, nlink=2 (metadata only, no normal-binary hash). Metadata association does not establish cryptographic binary/source provenance.

## New sealed copies
Integration: `<epochs>/crafted-quality-retail-green-retry2-20261014T020756467959Z/artifacts/deps/integration-9f07e4a55f2bbbf7`
SHA-256: `23ee6683b1742006224ba40973ae199ef2518f8ef85e885aa841ccbba3e635f3`
Receipt: `<epochs>/crafted-quality-retail-green-retry2-20261014T020756467959Z/receipt-integration.json`

wow-sim: `<epochs>/crafted-quality-retail-green-retry2-20261014T020756467959Z/artifacts/wow-sim`
SHA-256: `e56ba8d112a9018648950a4a8fdf7d5cf970fc22df59b3f6d41ebef28f1246dc`
Receipt: `<epochs>/crafted-quality-retail-green-retry2-20261014T020756467959Z/receipt-wow-sim.json`

Each receipt absent before its one sequential SHA-256 read; pre/post descriptor/path stats match seal; digest matches seal. Both receipts PASS, hash_reads=1, mode 0600, fsynced, full readback matched. No old binaries/libraries/archive/source authentication.

## Runtime / native
Retail GREEN runtime remains UNEXECUTED. Native producer compilation PASS; native runtime behavior NOT PROVEN. No build/test/check/fmt rerun, runtime launch, service mutation, or delegation.

## Preserved admission denials
`<epochs>/crafted-quality-retail-green-20261014T015921391694Z`: denied; admission.json only; zero launches.
`<epochs>/crafted-quality-retail-green-retry1-20261014T020528345170Z`: denied; admission.json only; zero launches.

## Outputs
Report: `<verification>/crafted-quality-retail-producer-auth-current/report.md`
Proof: `<verification>/crafted-quality-retail-producer-auth-current/proof.json`
Both new files mode 0600, fsynced, full readback matched.
