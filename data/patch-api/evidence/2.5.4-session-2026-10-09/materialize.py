#!/usr/bin/env python3
"""Materialize only this own SOURCE ledger from copied historical inputs."""
import json
from validate import EVIDENCE, SOURCE, expected_ledger, expected_profile, load_json, load_tool


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def main():
    raw = (SOURCE / '2.5.4-api-changes.wikitext').read_text()
    text = load_tool('extract_patch_non_inventory').extract_text(raw, canonical_patch_navigation=True).encode()
    (SOURCE / '2.5.4-api-changes.txt').write_bytes(text)
    write_json(SOURCE / '2.5.4-page-coverage.json', expected_ledger(raw, load_json(EVIDENCE / 'source-pin.json'), text))
    write_json(EVIDENCE / 'profile-observation.json', expected_profile())


if __name__ == '__main__':
    main()
