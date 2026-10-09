# Sanitized provider diagnostic handoff: provenance status

Date: 2026-10-09. Read-only inspection; no rollout, restart, edit to project files, build, test, service mutation, network access, or delegation was performed. No secrets, environment values, or session filesystem were read.

## Exact blocker

Current installed/live executable provenance is inconsistent and does not establish whether the running processes contain the diagnostics implementation:

- Historical deployment receipt `/tmp/pi-diagnostics-deploy-result.json` records source revision `43e048c00af26e0bf21999d51f5e404b8ec6e31d`, successful deploy (`exit: 0`), and installed SHA-256 `0815da1d9e0c68bc3a660806661568c7c4b41b4f327eb8d5629e4d51e2a06b94`.
- Historical runtime receipts say PIDs 52211 and 59748 subsequently matched that hash. Later `/tmp/pi-diagnostics-live-artifact-check.json` reports both PIDs at `816307a52d4f42598f556032ca5415a0a86a517bf91825188711959a28aeb9a9`, matching neither the historical deployment hash nor the artifact hash noted in the diagnostics spec.
- The historical installed-artifact receipt `/tmp/pi-diagnostics-current-artifact-check.json` reports `/home/osso/.local/share/pi/pi` at `337a761af432e265368d81a05d58bf5b414d3aed9c8179d115ca0ebf50729495`, also different from historical deploy hash. During this inspection, that path hashed to `1fb3647b15577c26eea93c62d60103aec3e8a5a14920dffadfc5a6cc7637ddff`; size 97,023,176 bytes. This proves the installed target changed since the earlier receipt or observations were taken, not which source produced it.
- `/home/osso/.local/bin/pi` is a symlink to `/home/osso/.local/share/pi/pi`.

Thus the exact blocker is missing contemporaneous provenance tying the current installed binary and both live process executables to a specific source revision/build receipt. Historical evidence cannot establish current runtime applicability. The Pi checkout is `master...origin/master [ahead 14]`; this status was read-only. It does not resolve provenance. `/tmp/diagnostic-rollout-map.md` was absent (ENOENT), so its requested contents could not be considered.

## Minimal permitted read-only proof

Obtain/inspect an existing, already-produced deploy/build receipt and artifact metadata—no redeploy—to tie (1) producing commit/revision to (2) output binary SHA-256 to (3) current installed path SHA-256. Separately obtain/inspect current process executable identities using metadata/hash only for each relevant PID, without opening session files. A valid match between the current installed hash, an existing receipt's output hash and source revision, and each live process hash would establish identity provenance. If any link is absent or mismatched, provenance remains unproven; do not infer diagnostics behavior from timestamps, checkout status, deployment success, or historical receipts.

No operational change is authorized or proposed. The requested map file must be supplied if its contents are needed to resolve what specific provenance artifact was expected.
