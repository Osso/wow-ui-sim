# Historical API-change artifact capture

Scope: primary-source availability/delivery only. No audits or behavioral-closure claims. Separate from frozen 2020 redirect evidence; does not revise older proof epochs.

## Captured target
- URL: https://warcraft.wiki.gg/wiki/API_change_summaries/Historical
- Browser title: `API change summaries/Historical - Warcraft Wiki - Your wiki guide to the World of Warcraft`
- Page ID 209816: supplied by task context, not independently present in captured HTML.
- Revision ID 5580481: captured printfooter permalink and parser-cache metadata.
- Parser cache timestamp: 2026-10-02 15:49:34; source does not state timezone.
- Capture date: 2026-10-09; precise time unavailable.
- Method: installed browser-cli via Pyrun `cli.command`, cwd `/home/osso/Projects/wow/wow-ui-sim`; `get html #mw-content-text`. Full tool stdout retained at `/tmp/pi-tool-59eb2b8648f83c9e.log`; extracted body in `historical-body.html`.

## Index evidence
The page is an index, not inline declarations. Vanilla rows show transcriptions for 1.12, 1.11, 1.10, 1.4, 1.3; no 1.2, 1.1, or 1.0 rows. It lists archive links for 1.9 through 1.5, but those archive contents remain unverified.

Observed consolidated article links:
- 1.12: https://warcraft.wiki.gg/wiki/1.12.0_consolidated_changes_(Iriel)
- 1.11: https://warcraft.wiki.gg/wiki/1.11.0_consolidated_changes_(Iriel)
- 1.10: https://warcraft.wiki.gg/wiki/1.10.0_consolidated_changes_(Iriel)

## Limitation
Tried same-origin MediaWiki API and Special:Export through browser-cli eval. CDP failed with `Starting Chrome with remote debugging on port 9222... Error: Chrome started but failed to connect after 5 seconds`. No API response or revision wikitext captured. Retained artifact is exact article-body HTML, not revision-source JSON/wikitext. No bypass or authenticated action attempted.

## Command ledger
Commands invoked through installed browser-cli using Pyrun `cli.command(...).cwd('/home/osso/Projects/wow/wow-ui-sim').run()`:
1. `--help`: succeeded; confirmed commands.
2. `--port 9222 tabs list`: showed existing Patch 4.3.4 tab.
3. `--port 9222 snapshot --interactive --compact`: observed before navigation.
4. `--port 9222 tabs new`, followed by list: connection reset/restart; list then showed blank tab. No tab was intentionally closed.
5. Snapshot blank tab, then `--port 9222 open https://warcraft.wiki.gg/wiki/API_change_summaries/Historical`: succeeded.
6. Snapshot after navigation; `get title`, `get url`: succeeded.
7. DOM link extraction with `eval`; `get html #mw-content-text`: succeeded.
8. API and Special:Export fetches using `eval`: failed due CDP connection failure. Later list/navigation attempts also failed.

## Integrity
- `historical-body.html`: 9724 bytes; SHA-256 `59ad12d3b8cfee0670fd7bfdcd51342d9b71513910c09f0886f866e0243b9917`.
- `index-links-observed.json.txt`: 63 bytes; SHA-256 `19bcae41a3a1033ddaefc32eb692ac7c58a131efc55bedf2d906056407fcce75`.
