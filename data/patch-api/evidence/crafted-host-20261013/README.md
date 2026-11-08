# Crafted-item quality host records

`C_TradeSkillUI.GetItemCraftedQualityInfo` reads explicit per-item inputs, separate from recipe/quality pairs. Implementation: `05665ddad`; verified runtime revisions: Retail `cd9f42756`, PTR `a5397a2d6`.

| Evidence boundary | Result | Records |
|---|---|---|
| Initial required-input rejection RED | Two failures/profile; later assertions then unreached | [Initial packet](../crafted-item-quality-initial-api-red-2026-10-13/README.md) |
| Supplied-record and original-consumer RED | Six failures/profile after valid baselines | [RED](red-current/README.md) |
| Feature GREEN + preservation | Twelve unique tests/profile; 24 passing outcomes | [GREEN matrix](green-current/README.md) |
| Normal startup | Two separate passes, not extra test names | [Runtime gate](green-current/verification/crafted-quality-final-runtime-current/report.md) |
| Default check / full format | Separate PASS; six manifest deprecations | [Check gate](green-current/verification/crafted-quality-default-check-format-current/report.md) |

[RED copy gate](publication-current/red-copy/report.md) and [GREEN copy gate](publication-current/green-copy/report.md) pass at their recorded scopes. The four retained gate texts still await their separate copy check. Historical SOURCE PASS / stale-spec FAIL and additive documentation reconciliation remain retained; no historical failure is rewritten.

ItemButton diagnostics remain 19 nil observations plus one missing `C_Reveal` per profile; PlayerChoice diagnostics remain 77 nil observations plus 18 missing requirements. Zero collected Lua errors is not globally clean compatibility. Visibility uses a real host-created Frame, not the original professions panel. Native catalogs/name validation, rendered pixels, callback delivery/unregistration, earlier-profile and full-parent acceptance remain unproved. [Owning contract](../../../../docs/specs/crafted-item-quality-info.md).

Selected text is retained whole through ordered literal privacy substitutions and terminal LF only; selected JSON is not reserialized or clipped. Source/build/copy/runtime/native scopes remain distinct. Public metadata binds counts and retained gate results.
