# Historical Retail 2.4.0 source audit

## Scope

Audit frozen page 73272, revision 6471380, timestamp 2025-09-13T09:55:32Z as historical 2008 Retail, not TBC Classic 2.5.x. Pin response and wikitext against the committed legacy manifest before accounting. Preserve the complete registry through 1.0.0; this bounded task audits only 2.4.0.

## Requirements

- Account for every nonblank literal line, header, API reference occurrence, event, CVar, command example, widget method, explicit signature and unspecified signature boundary. Retain source text and occurrence-specific unproven contracts; linked pages and forum posts remain unexpanded.
- Parser/extractor changes are opt-in. Existing default bytes and previously recorded outputs must not change. The full mixed prose/inventory extract deliberately preserves raw markup, including inline texture syntax and citations.
- Later historical Retail registers alone can supersede publication: actual 3.2.0/3.3.0/3.3.3/3.3.5/4.0.1 and later, with queued 2.4.2/3.0.2/3.0.3/3.0.8/3.1.0 separately pending main integration. Classic 2.5.x, Wrath 3.4.x and Era cannot supersede this page. Contextual references are not new additions or inferred removals.
- Never infer signature, return, behavior, native or security parity from publication or source accounting. Only established literal contracts backed by real state justify a meaningful closure. Otherwise retain precise gaps; no aliases, shims, fallbacks or vendor/cache/Wowless edits.
- Historical originals and any closures have separate ledgers, gaps, code/parser/log archives and seals. A fresh copied replay must work without Git, target or current mutable source files. Serialized tampering must reject and restored bytes must reproduce.

## Proof boundary

Only focused RED/GREEN development tests and source replay. No broad, check, lint, readability, profile, startup, full-suite or final gates. Main owns integration, current-runtime publication measurement and native acceptance. No push, merge, deployment or delegation.
