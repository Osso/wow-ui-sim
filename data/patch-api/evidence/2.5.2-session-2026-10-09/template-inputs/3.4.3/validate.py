#!/usr/bin/env python3
"""Replay bounded historical Wrath Classic source proof, independent of later audits."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
SOURCE = ROOT / 'data/patch-api/sources'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def check_seals():
    seals = json.loads((EVIDENCE / 'seals.json').read_bytes())
    for path, expected in seals.items():
        assert digest((ROOT / path).read_bytes()) == expected, f'seal: {path}'
    return len(seals)


def load_tool(name):
    path = EVIDENCE / f'historical-{name}.py'
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def inventory_counts(raw):
    generator = load_tool('gen_patch_wikitext_register')
    sections = generator.split_sections(raw)
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        parsed, counts = generator.parse_section(section, sections.get(section, []))
        entries.extend(parsed)
        headers.extend(counts)
    # This exact source has no inventory; never silently accept a newly added
    # inline API reference outside the generator's sections either.
    assert not generator.TEMPLATE.search(raw), 'unexpected explicit API reference'
    return {
        'enumerated_api_occurrences': len(entries), 'header_counts': headers,
        'register_created': False,
        'removals': sum(entry['direction'] == 'removed' for entry in entries),
    }


def validate(*, response, raw, pin, ledger, text, profile):
    page = json.loads(response)['query']['pages']['152751']
    revision, = page['revisions']
    for key, expected in [('pageid', 152751), ('title', 'Patch 3.4.3/API changes')]:
        assert page[key] == pin[key] == ledger['source'][key] == expected, key
    for key, expected in [('revid', 5983024), ('timestamp', '2024-03-07T08:49:48Z')]:
        assert revision[key] == pin[key] == ledger['source'][key] == expected, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '1849b20140e6c83b62c4a1b5a14adba2143de8ce01d9a368b7a52d166d0f5e6e', 'source hash'
    assert digest(response) == pin['response_sha256'] == '594cacde59ac51e99109e28fea49ea153a1969994032064f83b8e3dfad5b3b05', 'response hash'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'
    assert ledger['source'] == pin, 'source pin'
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'],
            ledger['scope'], ledger['later_registers']) == (
        'patch-source-accounting/v1', '3.4.3', 'wrath-classic', 'wrath',
        'source-and-profile-accounting', []), 'client history'
    extractor = load_tool('extract_patch_non_inventory')
    expected_text = extractor.extract_text(raw.decode(), canonical_patch_navigation=True)
    assert text == expected_text.encode(), 'plaintext reproduction'
    assert ledger['non_inventory_source']['sha256'] == digest(text), 'plaintext hash'
    assert ledger['non_inventory_source']['extractor_flags'] == ['--text-only', '--canonical-patch-navigation'], 'extractor flags'
    seeds = extractor.seed_rows(expected_text, '3.4.3')
    rows = ledger['source_rows']
    assert [row['source_id'] for row in rows] == [row['source_id'] for row in seeds], 'row accounting'
    nonblank = [(i, line) for i, line in enumerate(raw.decode().splitlines(), 1) if line.strip()]
    assert len(nonblank) == len(rows), 'row accounting'
    for row, seed, (line_no, source_text) in zip(rows, seeds, nonblank):
        assert (row['wikitext_line'], row['text_line'], row['source_text']) == (line_no, line_no, source_text), 'literal row accounting'
        expected_status = 'UNPROVEN' if seed['status'] == 'audit-pending' else seed['status']
        assert row['status'] == expected_status, 'prose proof limit'
        assert row['capabilities'] == [], 'no publication/behavior credit'
        assert row['note'], 'explicit proof limit'
    inventory = inventory_counts(raw.decode())
    assert ledger['inventory'] == inventory, 'inventory accounting'
    assert '* TOC: <code>30403</code>' in raw.decode().splitlines(), 'source TOC'
    expected_profile = {
        'profile': 'wrath', 'feature': 'client-wrath', 'supported_profile': True,
        'source_toc': 30403, 'configured_interface': 38001, 'cache_subdir': 'wrath',
        'runtime_observations': 0, 'native_observations': 0,
    }
    assert {key: profile[key] for key in expected_profile} == expected_profile, 'profile evidence'
    assert profile['code_revision'] == '23c930d837530e197d5f728e07336f81847e0710', 'historical profile revision'
    assert profile['cache_files'] and all(path.startswith((
        'Blizzard_APIDocumentation/', 'Blizzard_APIDocumentationGenerated/'))
        for path in profile['cache_files']), 'historical documentation-only cache'
    # Counts are derived from this page, not from global register sets, later
    # receipts, current cache files or live shared extractor implementations.
    return {
        'source_rows': len(rows), 'statuses': dict(Counter(row['status'] for row in rows)),
        'enumerated_api_occurrences': inventory['enumerated_api_occurrences'],
        'header_counts': inventory['header_counts'], 'removals': inventory['removals'],
        'runtime_observations': profile['runtime_observations'],
    }


def main():
    sealed_inputs = check_seals()
    summary = validate(
        response=(EVIDENCE / 'source-response.json').read_bytes(),
        raw=(SOURCE / '3.4.3-api-changes.wikitext').read_bytes(),
        pin=json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
        ledger=json.loads((SOURCE / '3.4.3-page-coverage.json').read_bytes()),
        text=(SOURCE / '3.4.3-api-changes.txt').read_bytes(),
        profile=json.loads((EVIDENCE / 'profile-observation.json').read_bytes()),
    )
    print(json.dumps(dict(summary, sealed_inputs=sealed_inputs), sort_keys=True))


if __name__ == '__main__':
    main()
