# PTR 12.1.5 CDN Content Index

`tools/gen_blizzard_ui_content_index.py` rebuilds the pinned PTR 12.1.5 file-to-CDN-range index from already downloaded Blizzard metadata. It does not download content and never uses Gethe files as runtime bytes.

## Inputs

- Gethe checkout at `49b69918fcdc77e109813281e4f537d45ec7dcbf`, version `12.1.5.69594`.
- Blizzard `wowxptr` build config, CDN config, decoded neutral/enUS TVFS files, download manifest, and verified archive indexes.
- `data/wow-ui-sim-listfile.csv` for FileDataID-to-path mapping. New IDs fall back only when one source path has the verified content key.

The generator verifies the exact Gethe revision/version, `WOW-69594patch12.1.5_XPTR`, `wowxptr`, TVFS content keys, download encoding keys, archive-index footer hashes, and complete source coverage.

## Regenerate

```text
python3 tools/gen_blizzard_ui_content_index.py \
  --metadata-prefix /tmp/pi-ptr125 \
  --source-dir ~/.cache/wow-ui-sim/wow-ui-source-git/ptr2/Interface/AddOns \
  --output data/blizzard-ui-content-index/ptr-12.1.5.69594.json
```

The output schema is versioned and has top-level `product`, `version`, `build_key`, `cdn_key`, `install_key`, `gethe_revision`, and per-file path, FileDataID, content/encoding keys, archive, range offset, and size.

## Tests

```text
python3 -m unittest tools.tests.test_gen_blizzard_ui_content_index
```
