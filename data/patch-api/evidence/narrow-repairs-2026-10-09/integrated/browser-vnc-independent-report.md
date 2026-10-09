# Browser / private VNC readiness — 2026-10-09

Main-thread direct observation; browser agent could not execute Pyrun or write report in its restricted role. No services changed or browser navigation performed.

`/usr/bin/ss --listening --tcp --numeric --processes` exited 0 (cwd `/home/osso/Projects/wow/wow-ui-sim`):

- Chromium CDP: `127.0.0.1:9222`, PID 691389.
- Xvnc: `127.0.0.1:5901` and `[::1]:5901`, PID 681583.
- These observed listeners are loopback-only. Authentication, SSH forwarding from user's machine, noVNC and remote end-to-end access were not tested in this observation.

Existing browser-cli ordered commands: `get url`, `get title`, `snapshot --interactive --compact` succeeded. Current public page: `https://warcraft.wiki.gg/wiki/API:GetSessionTime`; title `GetSessionTime - Warcraft Wiki - Your wiki guide to the World of Warcraft`. Snapshot exposes ordinary wiki navigation, `View source`, `View history`, API links and public article links; no visible challenge at observation. No cookies, credentials, browser profile data or diagnostic payloads read/exported.

Earlier direct-request HTTP403 reports and later successful user-cleared browser capture are distinct transport/session observations. This readiness check does not establish when the 403 began, whether direct requests remain blocked, or whether IP blocking caused it. Frozen historical captures already exist independently in source-cache; a readable current page is not a native API observation or a frozen-revision validation.
