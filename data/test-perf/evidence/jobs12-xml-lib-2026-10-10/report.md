# XML GREEN compilation timing and jobs12 comparison

Date: 2026-10-10. Read-only analysis of the exact saved XML GREEN receipt and its matching Cargo timing HTML. No build, rerun, process/host operation, delegation, or source change.

## Exact XML GREEN evidence

Receipt: `/home/osso/.local/state/wow-ui-sim/verification/xml-click-green-current/20261010T163159Z/`  
Revision: `c6bc87c120e43199dadeb299691511b03a81408a`. `source-before` and `source-after` are equal; compilation exited 0. The recorded command was `/usr/bin/cargo test --no-run --lib --offline --locked --message-format=json --timings -j 12` (purpose: shared XML click-registration GREEN plus existing physical-input controls).

The matching report named in `compile.stderr` exists at `target/cargo-timings/cargo-timing-20261010T163159641Z-8cf3603a52512442.html`:

- Cargo report interval: **2026-10-10 16:31:59.641273278 UTC**, total **67.3 s**; target is `wow-ui-sim` lib, test profile.
- Cgroup `cpu.stat` deltas: usage **421,177,800 µs = 421.178 CPU-s**, user **404.422 CPU-s**, system **16.756 CPU-s**. Against the report's 67.3-second interval this is **6.26 average CPU-equivalents**. This is cgroup-level work over the aligned build interval, not a count of simultaneously busy cores and not necessarily Cargo/rustc-only work.
- Cgroup counters also increased by 674 periods, 28 throttled periods and 30,744 µs throttled time. `cpu.max=1200000 100000` is the 12-CPU quota ceiling; it does not measure achieved parallelism. Memory peak was 10,735,443,968 bytes; cap 17,179,869,184.
- Timing report says **730 Cargo units: 729 fresh, 1 dirty**. Its unit/concurrency graph shows only the single root `wow-ui-sim` unit active for 66.64 s and max Cargo unit concurrency **1 (jobs=12, ncpu=12)**. The root compilation is therefore the measured Cargo critical path in this report; no dependency chain can be attributed as the cause. This is Cargo unit overlap, not actual busy-core count. The report's sampled CPU graph is a separate aggregate CPU-use trace; neither `-j 12` nor the report label proves 12 rustc processes/cores were busy.

## Comparison with saved Mists history

The comparison source, `/home/osso/.local/state/wow-ui-sim/handoff/jobs12-utilization-resumed.md`, describes a **different, historical Mists** prefork `cargo test --no-run --test prefork_full_ui` receipt (revision `a52140e…`): walltime 116.482 s, 12 jobs configured, no before CPU counter, hence no interval-aligned CPU delta. Its cgroup-lifetime usage divided by walltime was explicitly only an approximate, contaminated estimate, and it had no process samples or retained matching timing report. Other historical rows likewise vary in profile, targets, cache and command and lack before counters. They are not like-for-like controls.

Thus this XML receipt improves the evidence for this one Retail lib test-profile compile: aligned cgroup CPU delta and exact Cargo timing report are available. It does **not** establish that jobs12 caused or fixed delay, that the Mists compile had 12 busy cores, or explain the historical Mists walltime. Do not infer a compiler-delay cause from either dataset.
