# Tooltip clamp bounded diagnostic attempt

Diagnostic-only commit: 9f91aca971a8d66c02e81f602923aea6f6f23f68.
Only tests/tooltip_text_layout.rs changed: four assertion messages now include rect and screen dimensions; every predicate and fixture text unchanged. rustfmt completed before commit.

Compile request argv/cwd/revision/start timestamp: compile.json. The tool returned no result across context continuation. Observed wrapper PID 4157074 (PPID 1) and waiter 4157079 blocked on shared build lock, with stdout/stderr pipes and no saved capture. Subsequent /proc inspection confirmed both exited. No compile completion, exit code, Cargo JSON, or current executable provenance recovered. No second compile attempted.

Preexisting executable hash recorded in preexisting-artifact.json; artifact predates this request and was not executed. Exact RED and top-left control NOT RUN. Actual rect/screen dimensions remain unobserved. Historical fixture screen is 400x300, but this is input, not measured output.

Source-supported conditional only: src/iced_app/tooltip.rs measure_tooltip_content_width takes maximum measured line width and measure_tooltip adds horizontal insets without a viewport cap; update_tooltip_sizes writes frame width/height and marks rect dirty. src/layout.rs clamp_rect_to_screen changes x/y only; clamp_axis_to_viewport returns zero for size >= viewport. IF measured width exceeds 400, moving x to zero cannot satisfy right containment. Without numeric reproduction, oversize tooltip versus stale state is unresolved; neither clamp nor fixture is established faulty. The top-left control checks only nonnegative x/y, not right/bottom containment, so its historical pass does not prove dimensions fit.

No runtime/fixture/vendor changes, remediation, delegation, network, push, merge, deploy, final gates, broad tests, or second compile. Stopped pending main review; build wrapper no longer holds lock.
