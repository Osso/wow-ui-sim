# Parallel GPU test lifetimes

Verified 2026-11-04: the Retail integration binary can crash or hang when test GPU probes and headless renders initialize separate wgpu instances concurrently.

## Evidence and cause

The release crash on `92a49d331` called a null address from `libvulkan.so.1` during `vkEnumerateInstanceExtensionProperties`. Another test thread was inside `vkCreateInstance` through NVIDIA 615.78.08. This fails before adapter selection; it does not establish that NVIDIA was the selected rendering adapter.

`layer5_parallel_gpu_lifetimes_complete` runs existing atlas-upload and additive-readback assertions from four synchronized workers, eight times each. Before the fix, ten process runs produced six SIGSEGVs, three 75-second timeouts and one pass. A reproduced core again shows null-call extension enumeration while another thread unloads `libvulkan_lvp.so` during extension enumeration. The evidence identifies unsafe concurrent loader/ICD initialization on this host, not a texture assertion failure or a defect proven to belong to one specific ICD.

## Lifetime boundary

Headless rendering and GPU library tests already use `headless::lock_gpu_device_lifetime`. The integration helper previously bypassed it. It now returns `TestGpuDevice`, which owns the device, queue and the same process-wide mutex guard. Atlas tests borrow its handles; the guard is released only after those handles and test resources drop. Availability-only probes release their temporary before rendering, so they do not recursively acquire the lock.

This is a host loader/driver workaround. It does not skip assertions or select a different backend. Retire the serialization only after concurrent creation, use and teardown pass on the affected loader/driver stack. A creation-only lock would leave the previously observed headless teardown/encoding deadlock boundary unprotected.

## Targeted proof

With the shared lock, the same concurrency regression passed 50 process runs without crashes or timeouts. Five further runs of the full rendering-pipeline module and all integration helper caller modules passed (25 tests per run). Ten runs of the library headless/shader subset passed (35 tests per run, including existing opt-in GPU readbacks). The integration helper selected Vulkan on NVIDIA GeForce RTX 5060 Ti, driver 615.78.08. Library color readback independently reported the same adapter.

These are targeted lane results, not full-library or release acceptance. Original CVar bytes and stat metadata were unchanged across each stress service. Compiler diagnostics contained no Rust warnings, and formatting passed.

## Sources

- [Shared GPU test helper](../../../tests/common/mod.rs)
- [Rendering pipeline regression](../../../tests/rendering_pipeline.rs)
- [Headless renderer lifetime lock](../../../src/render/headless.rs)

## See Also

- [[rendering-pipeline]] — production rendering pipeline.
