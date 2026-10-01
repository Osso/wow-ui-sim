# Cooldown countdown formatter

`Cooldown:SetCountdownFormatter` attaches a live NumericFormatter; `GetCountdownFormatter` returns that handle or nil. Public methods live in `src/lua_api/frame/methods/widgets/cooldown.rs`. Cached retail `FrameAPICooldownDocumentation.lua` declares a nilable NumericFormatter and `AllowedWhenUntainted` setter arguments. See [duration core](../wiki/systems/duration-core.md) and [rendering pipeline](../rendering-pipeline.md).

## What it must do

### Configuration — inferred simulator policy

- [ ] Return exactly one nil by default and after nil clearing; attachments are per Cooldown.
- [ ] Retain the original abbreviated-number, numeric-rule, or native Seconds formatter identity and observe later configuration changes. Do not clone configuration or accept forged objects.
- [ ] Reject invalid values before changing an existing attachment. Identification must not invoke arbitrary addon formatter methods.
- [ ] Accept authenticated typed secret handles or secret nil only from untainted callers. Reject tainted secret writes atomically; plain writes remain valid without clearing caller taint. Getter returns the attached ordinary handle.
- [ ] Clearing/replacing an attachment preserves formatter configuration and existing countdown threshold settings.

### Countdown rendering — inferred simulator policy

- [ ] Consume live configured formatter output in the real library countdown text path, including numeric-rule strings, abbreviation strings, and native Seconds strings.
- [ ] Use the renderer's monotonic simulator clock and existing modRate calculation; ticks and formatter configuration changes update displayed text.
- [ ] Clearing selects the existing default countdown policy. Hide, minimum-duration, and expiry gates still suppress countdown output.
- [ ] Snapshot only attached frame IDs; reread each formatter from private `FORMATTER_ROOTS` immediately before consumption. Root the active formatter on the VM stack through identification, dispatch, and collecting callbacks; restore stack height on success or error. A cleared attachment is skipped; invalid/unavailable roots and invalid formatter handles report meaningful errors. Invoke only shared trusted typed dispatch, never addon overrides with decoded secret timing; retain secret authorization and caller taint.
- [ ] When a configured curve replaces a peer Cooldown's attachment and explicitly collects garbage, the peer consumes the replacement on that tick without stale-handle errors. This reentrant live-state policy is a simulator inference.

## How it works

- [Duration core](../wiki/systems/duration-core.md)
- [Rendering pipeline](../rendering-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/cooldown.rs` — setter/getter, validation, private per-frame GC roots.
- `src/lua_api/frame/methods/widgets/cooldown/countdown_formatter.rs` — typed tick dispatch into renderer-only strings; failed formatting reports an error and suppresses configured output, never substitutes default/stale text.
- `src/lua_api/on_update.rs` — configured-only dispatch after frame update handlers.
- `src/iced_app/quad_builders_cooldown.rs` — consumes configured text cache through the existing countdown gates, retaining default policy only when no formatter is attached.
- `src/widget/frame.rs`, `src/widget/frame_defaults.rs` — live attachment and default state.
- `src/lua_api/globals/lua_duration_object/formatting.rs` — shared authenticated formatter identification and typed dispatch.

## Tests asserting this spec

- `tests/cooldown_countdown_formatter.rs` — eight public configuration cases, RED at `e0a46d691` in `/tmp/patch-12.0.5-batch5-cooldown-formatter-red.log`.
- `src/iced_app/quad_builders_cooldown/countdown_formatter_tests.rs` — seven real engine tick → library countdown text cases. Actual RED at `5cfb08c4e`: `/tmp/patch-12.0.5-batch6-cooldown-render-red.log`; each fails on hardcoded default text instead of configured strings. Later secret-curve assertions strengthen the existing case without changing formatter implementation. These assert text selection, not GPU glyph rasterization.

## Known gaps (current cycle)

- [ ] Parent's current-revision GREEN and final integration gates remain pending. Public configuration development GREEN is 8/8 at `eac08bda3`: `/tmp/patch-12.0.5-batch6-cooldown-api-green.log`.
- [ ] Actual eight-case library execution at `c5ba89ae3` is 6 PASS / 2 FAIL: `/tmp/patch-12.0.5-batch7-lib-0.log`. The GC replacement regression expects `Some("20m 34s")`, receives `None`, and records `expected a known NumericFormatter object`: an earlier curve replaces a later attachment and collects the unrooted handle captured by the value snapshot. The frame-ID/live-root and active-stack-root correction awaits parent-owned GREEN.
- [ ] The separate secret-callback security fixture fails in post-tick `env.exec` at line 242, not setup: setup `issecure` and ticking text `8s` passed. `97a0e1270` asserts fresh host-entry security before reading the tainted callback-written `observed` global, then verifies retained read taint (`rilua runtime_ops:419`, `propagate_slot_read_taint`). This is assertion ordering, not caller-taint clearing or a runtime fix; GREEN remains pending. The earlier seven-case consumer RED was hardcoded default text, not this GC lifetime failure.
- [ ] Child FontString `GetText` consistency is not modeled: configured secret-derived strings remain trusted Rust renderer data, not plaintext Lua-readable child text.

## Out of scope

Native-client parity, arbitrary Lua formatter callback compatibility, Blizzard/vendor changes, general secret confidentiality, and final integration verification are not claimed. Identity retention, ordinary getter output after secret assignment, and clock/render policy are informed guesses, not native probes.

## Batch7 observed proof — 2026-10-01

Observed batch7 default build snapshot `c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd`, rilua `6044544b960cd68b4b0c58bb3373412757c2caee`, compiled successfully in 34m51s. Exact argv, artifact SHA256 and referenced outputs: `/tmp/patch-12.0.5-batch7-integration-runs.json` and `/tmp/patch-12.0.5-batch7-lib-runs.json`. Independent verifier 104 report `/tmp/patch-12.0.5-batch7-independent-proof.md` was not yet available when recording these logs; no independently validated final acceptance, native parity or whole-page completion is claimed.

Public configuration PASS 8/8 (`/tmp/patch-12.0.5-batch7-integration-4.log`). Library configured countdown 6 PASS / 2 FAIL (`/tmp/patch-12.0.5-batch7-lib-0.log`); unchanged countdown controls 8 PASS (`/tmp/patch-12.0.5-batch7-lib-1.log`), static defaults 2 PASS (`/tmp/patch-12.0.5-batch7-lib-2.log`). GC correction `99102f631` and post-tick security fixture `97a0e1270` were committed after this snapshot; their GREEN is pending. No rendered-current-fix acceptance.
