# Patch 7.3.2 publication and session protection

## Source and scope

Audit Warcraft Wiki page 230850, revision 6200179, refetched 2026-10-08. Current retail is the target, not a reconstructed Legion client. Source: `data/patch-api/sources/7.3.2-api-changes.wikitext` and its provenance.

The page has one Changes statement: Logout and Quit became protected. Retain both changed API-link occurrences in the register with their complete literal annotation, plus every retained extract identity in the coverage ledger. No source additions or removals exist. Publication alone does not prove protection.

## Required behavior

- Both globals remain published after unmodified full-UI startup.
- Insecure addon-tainted calls to Logout and Quit fail before changing login/exit state.
- Secure calls retain existing simulator transitions: Logout clears login state; Quit requests GUI-owned exit.
- Bare and cached full-UI probes distinguish publication from protection. Neither should exit the test process.
- Older-profile behavior remains unchanged. No Blizzard Lua patch, vendor mutation, namespace shim, or fabricated output may close a gap.

## Evidence

Pin raw fetch, revision, hashes and opt-in extraction/generation flags. Preserve and reproduce prior audit artifacts; pin historical register scope to a Git revision so future earlier-page audits do not expand this audit's proof. Retain command revisions, complete logs, exit codes, negative control and per-identity accounting. Counts derive from artifacts; receipt cwd/target locations are descriptive, never path-equality gates.

## Boundaries

No logout countdown, cancellation lifecycle, native error text, notification event parity, hardware-event policy, or historical 7.3.2 signature/security parity claim. ForceLogout/ForceQuit/QuitGame are not named by this page. Reference list and Reddit citation are retained without expanding linked discussion. No full-suite/CASC texture acceptance is required.
