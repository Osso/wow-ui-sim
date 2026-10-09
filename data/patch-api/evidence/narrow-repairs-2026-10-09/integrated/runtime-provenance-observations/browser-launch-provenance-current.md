# Browser launch provenance — current observation

Observed at **2026-10-09T20:22:37.906918+00:00 UTC** (exact timestamp captured before inspection).

## Current state

- `/proc/net/tcp` and `/proc/net/tcp6`: loopback listeners on port `5901` (`127.0.0.1` and `::1`) were present. No listener on port `9222` was present.
- Current process scan found PID `3441151`, executable `Xvnc`, sleeping. Its full command line was not retained or printed. Historical PIDs `691389` (Chromium) and `681583` (Xvnc) no longer exist.
- User systemd unit directories contained no browser/VNC-named service files. `default.target.wants` contained only `syncthing.service`; no matching browser/VNC desktop autostart file was found in the narrowly inspected locations.
- `/proc` and systemd checks were read-only. No service, process, browser, or configuration was changed; no authentication or navigation occurred.

## Launch provenance result

**No safe existing browser launch recipe is verifiable.** Current Chromium is absent, so its launch flags/profile directory cannot be read from live process metadata. The inspected user-systemd/autostart locations yielded no matching startup unit. The historical browser-cli skill documents a generic default launch (`google-chrome-stable --remote-debugging-port=9222 --user-data-dir=/home/osso/.config/chromium`), but that does not prove it was the original launch or that it is safe to reuse after the reported OOM. Do not infer or restart with it.

**Blocker:** original browser command/profile provenance remains unknown. No browser profile files, cookies, credentials, environment values, secrets, or session/transcript stores were read.
