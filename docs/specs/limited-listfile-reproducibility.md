# Limited listfile reproducibility

The bundled `data/wow-ui-sim-listfile.csv` must reproduce from reviewed, pinned inputs. It is generated data and must not be hand-edited.

## Pinned inputs

`data/wow-ui-sim-listfile.provenance.json` records:

- The immutable `wowdev/wow-listfile` release/asset URL, the recovery fetch timestamp, and the source byte count and SHA-256. Release `202611141741` is the exact source already recorded by the Retail 70783 metadata refresh, not a newer community refresh.
- The authoritative extra row source `data/listfile-overrides.csv`, including its byte count and SHA-256. Overrides retain their canonical display casing; lookup normalization remains case insensitive.
- Counts and SHA-256 identities for the effective path candidates, requested FileDataIDs (including override IDs), and the union of all committed Blizzard UI profile manifests. Each identity hashes sorted string entries joined with LF and one trailing LF; even an empty selection hashes one LF. FileDataIDs are converted to decimal strings before sorting.
- The output byte count, row count and SHA-256, with UTF-8/LF serialization and normalized-path sorting.

The generator validates the source, extra row source, effective requests and regenerated output **before writing**. `--check` additionally compares the existing output and never rewrites it. Source, override, request or output drift is an error; it is not silently accepted as a new pin.

## Reproduce

Download only the URL in the committed provenance file and verify its SHA-256 before using it. Do not use `releases/latest` as a replacement. The source is a local development prerequisite (about 154 MB), not a committed duplicate of the upstream listfile.

```sh
mkdir -p "${XDG_CACHE_HOME:-$HOME/.cache}/asset-resolver/data"
curl -fLsS \
  https://github.com/wowdev/wow-listfile/releases/download/202611141741/community-listfile.csv \
  -o "${XDG_CACHE_HOME:-$HOME/.cache}/asset-resolver/data/community-listfile.csv"
# Compare this file's SHA-256 with provenance.source.sha256.
python3 tools/gen_limited_listfile.py --check
python3 -m unittest tests.test_gen_limited_listfile tests.test_limited_listfile_reproducibility
```

`ASSET_RESOLVER_DATA_DIR` overrides the data directory; otherwise the generator honors `XDG_CACHE_HOME` and falls back to `~/.cache`. `--source`, `--output`, and `--provenance` support explicit paths. For the full regeneration test, `WOW_UI_SIM_LISTFILE_SOURCE` can identify the installed pinned source. A missing source fails that full test with a prerequisite message; it does not claim regeneration passed or silently skip it. The small fixture tests have no network/cache dependency.

## Updating a pin

Input changes require a reviewed provenance update along with regeneration. Verify the approved immutable source identity first. The generator exposes `file_identity`, `request_identity`, `load_source`, `load_listfile_overrides`, `collect_requests`, `collect_blizzard_ui_files`, `resolve_rows`, `serialize_rows`, and `write_rows` for computing the proposed metadata and artifact; do not edit CSV rows to satisfy the old digest. Preserve the exact source release/hash and fetch timestamp unless the source actually changes. Include extra row sources and effective request changes in the review. Then run normal generation and `--check` against the updated provenance.

## Evidence and historical boundary

At task base `892fd3e83df8439123fd41383623c06a18a27309`, the recovered source is 153,950,356 bytes, SHA-256 `c95a29658b1274d8c7cb33f1d629ffe8be8b6dcb2b7bb9177414dcd5a48e0e21`. Regeneration yields 147,015 rows and output SHA-256 `338b4f7894ba061c5cc007e4e13dc30cb88c930ea598eb9763018c3973cd651e`, byte-identical to the committed CSV. There is no current six-row deficit with those inputs.

Task lt-58c4 refers to an earlier PR33 verification where six committed rows were absent from the local regeneration. That private report and its original source identity are not present in this VM. The historical six row identities/root cause remain **UNKNOWN** pending that evidence; current reproducibility is not proof of what caused the earlier discrepancy. Do not invent an explanation or silently fetch a newer source to make the result pass.

The pre-existing Python override tests expected lowercase display values even though commit `2f88b2cabb81fd542cd5fea86498ecee8f51a007` explicitly preserved authoritative casing. Their exact expectations now enforce the existing canonical display contract while keeping normalized lookup-key and FDID assertions. The generated CSV is unchanged.

## Tests

- `tests/test_limited_listfile_reproducibility.py`: unchanged-input non-writing checks; source, extra-row-source, request and output drift; committed digest; full pinned-source regeneration.
- `tests/test_gen_limited_listfile.py`: source/override lookup, profile manifest collection and canonical override paths.
- `tests/limited_listfile.rs`: Rust consumer lookups retain the bundled canonical asset paths and case-insensitive resolution.
