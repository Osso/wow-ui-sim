# Goal

- [ ] Reproducible Patch 4.3.4 historical retail publication/source inventory, scoped meaningful modeled fixes only, committed targeted proof for coordinator integration.

# Project Plan

## Current Blocker

None. Browser-fetched response, pin and exact wikitext committed at 85b757d2d; no collector wait.

## Active TODO

- [x] Read instructions/templates; rebase clean p434-source onto integrated 5.0.1 master 3d64fedad.
- [x] Preserve verified response; existing default generator/extractor retain all 11 inventory rows and one metadata context; no new flags needed.
- [ ] Add own retail sweep with 5.0.1 and later retail successors, dynamic accounting, precise unmodeled gaps; implement only cheap source-backed real models.
- [ ] Commit coherent changes; run targeted RED/GREEN tests and formatting only, retain exact logs and revisions.
- [ ] Seal compact historical evidence under 5 MB, document capability/proof boundaries, report commits to coordinator.

## Exclusions

No delegation, push, merge, deploy, cwd switching, Bash, browser/provider changes, shims/fallbacks, vendor/Wowless edits, broad checks/all-publication/smoke/full suites or final acceptance.
