# TimerCallbackProbe

Captures **real-client ground truth** for three `C_Timer` callback behaviors
that require live-client confirmation beyond wow-ui-sim's focused runtime tests.
Results can be compared against the simulator and Wowless's model.

## Why

A test surfaced ("C_Timer state not shared") that feeds the object returned by
`C_Timer.NewTicker` straight back into a second `NewTicker` as its callback:

```lua
local count = 8
local cb = C_FunctionContainers.CreateCallback(function()
    count = count - 1
    if count == 0 then done(function() end) end
end)
local obj1 = C_Timer.NewTicker(0.05, cb, 5)
C_Timer.NewTicker(0.08, obj1, 3)   -- the returned ticker, reused as a callback
```

For this to pass (8 = 5 + 3), three things must hold. The probe was added when
wow-ui-sim did not satisfy these assumptions; the current simulator models the
container path, while this addon remains the live-client check for timer
integration and scheduling behavior:

1. `C_Timer.NewTicker` accepts a `C_FunctionContainers` callback object (not a raw
   function) as its callback.
2. `NewTicker` returns the **same** callback object you passed in, so it can be
   fed back into another `NewTicker`.
3. With one callback object backing two tickers, each ticker's iteration count is
   independent ("state not shared").

This test does **not** exist in Wowless's own suite, and Wowless only *models*
the behavior — neither is live-client proof. This probe exercises `NewTicker`,
not `After`; its capture cannot establish `After` callback acceptance, arguments,
or lifecycle. The historical simulator divergence is retained below; the only way
to establish retail behavior is to probe the live client.

## What wow-ui-sim does (current model)

The current simulator models the container path:

- `C_FunctionContainers.CreateCallback(fn)` returns **userdata**, is not callable
  via `()`, and exposes `:Invoke`, `:Cancel`, and `:IsCancelled`.
- `C_Timer.NewTimer` and `C_Timer.NewTicker` accept a function or container and
  return the callback container; a plain function is wrapped in a fresh container.
- The simulator's `C_Timer.After` wrapper accepts its existing ordinary callback
  forms and dispatches no callback arguments. This is simulator behavior, not a
  result captured by this probe.
- Timer registrations keep independent iteration state, and container
  cancellation cancels the registrations it backs.

The focused Rust proof is limited to userdata type, method exposure,
cancellation/invoke suppression, per-instance fields, and read-only keys. This
probe remains useful for live scheduling, timer integration, and edge semantics.

## What it probes

Written to `TimerCallbackProbeDB` (SavedVariables); a summary prints at
`PLAYER_LOGIN` (~1s later, after the tickers run) and on `/timerprobe report`.

Synchronous facts (`static`):
- `CreateCallback` return type, whether it is callable via `()`, whether it has `:Invoke`.
- Whether `NewTicker` accepts a container callback (and the error text if not).
- What `NewTicker` returns, and whether it `== ` the callback object passed in.
- Same for a plain-function callback (baseline).

Async scenarios (counters tally as the tickers fire; `expected = 8`):
- `sharedContainer` — the exact Feronn case: `cb = CreateCallback(...)`,
  `obj1 = NewTicker(0.05, cb, 5)`, `NewTicker(0.08, obj1, 3)`. Records whether each
  ticker started, what `obj1` is, and the total invocation count.
- `plainFunction` — the same plain function passed to two tickers
  (`NewTicker(0.05, fn, 5)` + `NewTicker(0.08, fn, 3)`). Definitely-valid WoW; the
  control that isolates the exotic container path.

## Run it

1. Install + enable (see `../create-and-install-wow-addon.md`).
2. Log in, or run `/timerprobe`.
3. Wait ~1s, then read the printed summary, or `/timerprobe report`.
4. `/reload` or log out so SavedVariables flush.
5. Pull `TimerCallbackProbeDB` back and compare (see the "Read SavedVariables
   Back" section of the create-and-install doc).

## Reading the verdict

For each async scenario:

- `PASS (8/8 — state not shared)` — both tickers ran their full counts on the
  shared callback object with independent iteration state.
- `ticker A rejected: <err>` / `ticker B rejected: <err>` — the client refused a
  container (or reused-ticker) as a callback; the assumption is false on the live
  client too.
- `DIVERGES (n/8)` — both started but the total is wrong; iteration state is
  shared or the reused callback no-ops.

A live `PASS` for `sharedContainer` confirms the retail timer/container path
that the simulator now models. If the client rejects the container, the Feronn
test does not encode supported retail behavior.

## Captured result (client 12.0.7.68182, interface 120007, 2026-06-16)

The simulator values in this capture are historical, from before the container
model was fixed.

```
static:
  createCallbackType        = userdata     (sim: table)
  containerCallableViaParen = false        (sim: false)
  containerHasInvoke        = false*        (*probe artifact — see note below)
  newTickerAcceptsContainer = true          (sim: false ← "function expected, got table")
  newTickerReturnType       = userdata
  tickerEqualsContainer     = true          (sim: returns a fresh {__id,Cancel} handle)
  tickerCallableViaParen    = false
  plainReturnType           = userdata
  plainTickerEqualsFn       = false

sharedContainer (the Feronn test):
  tickerAStarted = true, tickerBStarted = true
  obj1Type = userdata, obj1EqualsCb = true, firstArgType = userdata
  total = 8 / 8  → PASS (state not shared)

plainFunction (control):
  total = 8 / 8  → PASS
```

### Conclusion

**The Feronn test encodes genuine retail behavior; the simulator gap identified by this capture was fixed.**

> **Probe-bug correction:** `containerHasInvoke = false` above is a **false
> negative**. The probe checked `type(cb)=="table" and type(cb.Invoke)=="function"`,
> but retail `cb` is *userdata*, so the `and` short-circuited regardless of whether
> `:Invoke` exists. Retail containers **do** have `:Invoke` — confirmed by the
> retail-ground-truth test `Interface/AddOns/FcTest`. Don't read this row as
> "retail lacks Invoke".

On retail, for the `NewTicker` path this probe exercised:

- `C_FunctionContainers.CreateCallback(fn)` returns a **userdata** cancelable
  object with `:Cancel`, `:IsCancelled`, and `:Invoke` methods, a protected
  metatable (`getmetatable() == false`), and per-instance field storage. It is
  not callable via `()`.
- `C_Timer.NewTicker` **accepts that container as its callback** and **returns the
  very same object** (`tickerEqualsContainer = true`; in the async run
  `obj1EqualsCb = true`). A ticker *is* a FunctionContainer.
- Because the returned ticker is the same callback container, feeding it back into
  a second `NewTicker` re-registers the same wrapped function. Each registration
  keeps its **own** iteration count, so the shared container fires 5 + 3 = 8 times.
  That is exactly what "state not shared" verifies.
- A plain-function callback is wrapped into a container too
  (`plainTickerEqualsFn = false`).

The capture does **not** establish `After` behavior. Cached timer API documentation
labels `After`'s callback as `TimerCallback` (no arguments), while `NewTimer` and
`NewTicker` use `TickerCallback` (one callback argument).

The historical simulator capture diverged on every container-related point:

1. `CreateCallback` returned a Lua **table** (retail: userdata).
2. `NewTicker` rejected a container callback; retail accepts it.
3. `NewTicker` returned a fresh opaque handle, not the callback container.

The plain-function path matched retail (both reached 8/8). The gap was the
FunctionContainer model, now fixed.

### Fix (landed)

Containers are now **real userdata**, modeled on wowless's `luaobjects` using
rilua's `newproxy` (one shared metatable per kind; instances via `newproxy(proto)`;
a weak map from each userdata to its backing state table; `__eq` compares backing
identity). This matches `FcTest` and the retail capture:

- `CreateCallback` returns userdata: `type()=="userdata"`, `getmetatable()==false`,
  `:Invoke`/`:Cancel`/`:IsCancelled`, read-only methods, per-instance fields,
  rejects C functions (via `debug.getinfo(...).what=="C"`). `:Invoke` calls the
  wrapped function and **returns nothing**.
- `C_Timer.NewTicker` accepts a function **or** a container; a plain function is
  wrapped in a fresh container.
- `NewTicker` **returns the callback container** (a returned ticker can
  be fed back in); per-registration iteration count is independent.
- A fired callback receives a **proxy** of the container: `proxy == handle` (via
  `__eq`) yet a distinct raw table key, sharing the handle's fields.
- `container:Cancel()` cancels every timer the container backs.

Notes:

- Iteration state was already per-registration — the plain-function control
  passed before the fix too.
- Fixing this surfaced a latent GC bug: the timer-callback registry table was
  created without a write barrier, so it could be collected under the extra
  allocation, dropping all timer callbacks. Fixed alongside (separate commit).
- An inline self-test in `proxy_object_factories.rs` wrongly expected `:Invoke`
  to return a value; corrected to match retail (returns nothing).
