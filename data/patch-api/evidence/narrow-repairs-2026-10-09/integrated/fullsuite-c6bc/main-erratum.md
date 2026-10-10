# Main erratum: exact retirement case

Verified 2026-10-10 by main independently reading saved c6bc log lines 13799–13810.

The literal record at line **13804** is:

`test patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors ... ok`

This establishes PASS for that exact saved prefork case. The original independent report's Minimap/cached21 lookup answers a different question and is **incorrect as the requested retirement-case lookup**. Do not retain that mapping as correct or substitute Minimap, cached secure delegation, or absence of a cached21 label for the literal case above. The original audit is retained alongside this correction solely for traceability.

Suite accounting remains 14,999 PASS / 3 unchanged FAIL / 19 integration skips; +3 XML library cases versus bc58. Garrison's JSON `new_failures` metadata is runner-relative, not a newly failing identity versus bc58. No current-HEAD, native, all-profile or clean-startup acceptance follows.
