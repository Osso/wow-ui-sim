#!/usr/bin/env python3
"""Bounded historical Wrath Classic source replay; not runtime compatibility."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
SOURCE = ROOT / 'data/patch-api/sources'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load_inputs():
    paths = {
        'raw': SOURCE / '3.4.2-api-changes.wikitext',
        'text': SOURCE / '3.4.2-api-changes.txt',
        'ledger': SOURCE / '3.4.2-page-coverage.json',
        'inventory': SOURCE / '3.4.2-source-inventory.json',
        'response': EVIDENCE / 'source-response.json',
        'pin': EVIDENCE / 'source-pin.json',
        'profile': EVIDENCE / 'profile-observation.json',
        'successor_raw': EVIDENCE / 'successor-wikitext.txt',
        'successor_response': EVIDENCE / 'successor-response.json',
        'successor_pin': EVIDENCE / 'successor-pin.json',
    }
    return {key: path.read_bytes() for key, path in paths.items()}


def load_tool(name):
    spec = importlib.util.spec_from_file_location(
        name, EVIDENCE / f'historical-{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_identity(response, raw, pin, identity, hashes):
    pageid, title, revid, timestamp = identity
    page = json.loads(response)['query']['pages'][str(pageid)]
    revision, = page['revisions']
    for key, expected in [('pageid', pageid), ('title', title)]:
        assert page[key] == pin[key] == expected, key
    for key, expected in [('revid', revid), ('timestamp', timestamp)]:
        assert revision[key] == pin[key] == expected, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == hashes[0], 'source hash'
    assert digest(response) == pin['response_sha256'] == hashes[1], 'response hash'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'


def parse_inventory(raw):
    generator = load_tool('gen_patch_wikitext_register')
    buckets = generator.split_sections(raw)
    entries, counts = [], []
    for section in generator.SECTIONS.values():
        parsed, headers = generator.parse_section(section, buckets.get(section, []))
        entries.extend(parsed)
        counts.extend(headers)
    assert all(c['parsed_count'] == c['header_count'] for c in counts), 'headers'
    assert len(list(generator.TEMPLATE.finditer(raw))) == len(entries), 'all API references'
    return entries, counts


def validate_rows(raw, ledger, entries, prose_lines):
    generator = load_tool('gen_patch_wikitext_register')
    byline = {entry['wikitext_line']: entry for entry in entries}
    assert len(byline) == len(entries), 'inventory row multiplicity'
    nonblank = [(n, line) for n, line in enumerate(raw.splitlines(), 1) if line.strip()]
    rows = ledger['source_rows']
    assert len(rows) == len(nonblank), 'literal rows'
    for row, (number, literal) in zip(rows, nonblank):
        assert (row['wikitext_line'], row['source_text']) == (number, literal), 'literal rows'
        entry = byline.get(number)
        identity = entry['id'] if entry else f'source-line-{number:03}'
        assert row['source_id'] == identity, 'row identity'
        expected = 'UNPROVEN' if entry or number in prose_lines else 'metadata-only'
        assert row['status'] == expected and row['capabilities'] == [], 'proof limit'
        assert row['note'].strip(), 'reason'
        if entry:
            assert row['inventory_id'] == entry['id'], 'inventory row identity'
        if entry and entry['section'] == 'cvars':
            _, params = generator.parse_symbol(literal)
            assert row['source_contract'] == params, 'CVar contract'
        else:
            assert 'source_contract' not in row, 'no invented signature'
    return rows


def validate_profile(profile):
    expected = {
        'code_revision': 'f0baf34d43ea6d6eb1ef0aef2a85efae9f7d55f5',
        'profile': 'wrath', 'feature': 'client-wrath', 'supported_profile': True,
        'source_toc': 30402, 'configured_interface': 38001, 'cache_subdir': 'wrath',
        'runtime_observations': 0, 'native_observations': 0,
    }
    assert {key: profile[key] for key in expected} == expected, 'profile evidence'
    assert profile['cache_files'] and all(path.startswith((
        'Blizzard_APIDocumentation/', 'Blizzard_APIDocumentationGenerated/'))
        for path in profile['cache_files']), 'historical documentation-only cache'


def validate(*, response, raw, pin, ledger, inventory, text, profile,
             successor_raw, successor_response, successor_pin):
    pin, ledger, inventory, profile, successor_pin = map(
        json.loads, (pin, ledger, inventory, profile, successor_pin))
    validate_identity(response, raw, pin,
                      (355030, 'Patch 3.4.2/API changes', 3422177,
                       '2023-08-29T19:24:02Z'),
                      ('284871b95a3150e0dd0383ae4070ad1c067c1eee874e83d5bb44ebbf12118f0e',
                       '2b81231fe3cea60671d8c61715cbef6c152f06ee2a3fb127eacb578aa9e6fd1a'))
    assert ledger['source'] == pin, 'source pin'
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'],
            ledger['scope'], ledger['later_registers']) == (
        'patch-source-accounting/v1', '3.4.2', 'wrath-classic', 'wrath',
        'source-and-profile-accounting', []), 'client history'
    decoded = raw.decode()
    entries, counts = parse_inventory(decoded)
    assert inventory == {
        'schema': 'patch-source-inventory/v1', 'patch': '3.4.2',
        'client_line': 'wrath-classic', 'source': pin, 'parser_flags': [],
        'header_counts': counts, 'entries': entries,
    }, 'inventory reproduction'
    assert ledger['inventory_path'] == 'data/patch-api/sources/3.4.2-source-inventory.json', 'inventory path'
    extractor = load_tool('extract_patch_non_inventory')
    expected_text = extractor.extract_text(decoded, canonical_patch_navigation=True)
    assert text == expected_text.encode(), 'plaintext reproduction'
    assert ledger['non_inventory_source'] == {
        'path': 'data/patch-api/sources/3.4.2-api-changes.txt',
        'sha256': digest(text),
        'extractor_flags': ['--text-only', '--canonical-patch-navigation'],
    }, 'plaintext provenance'
    # Extraction retains the pre-inventory source line numbers on this page.
    seeds = extractor.seed_rows(expected_text, '3.4.2')
    prose_lines = {int(seed['source_id'].rsplit('-', 1)[1])
                   for seed in seeds if seed['status'] == 'audit-pending'}
    rows = validate_rows(decoded, ledger, entries, prose_lines)
    assert '* TOC: <code>30402</code>' in decoded.splitlines(), 'source TOC'
    validate_profile(profile)
    validate_identity(successor_response, successor_raw, successor_pin,
                      (152751, 'Patch 3.4.3/API changes', 5983024,
                       '2024-03-07T08:49:48Z'),
                      ('1849b20140e6c83b62c4a1b5a14adba2143de8ce01d9a368b7a52d166d0f5e6e',
                       '594cacde59ac51e99109e28fea49ea153a1969994032064f83b8e3dfad5b3b05'))
    successor_entries, _ = parse_inventory(successor_raw.decode())
    assert ledger['successor'] == {
        'patch': '3.4.3', 'state': 'audited-unmerged', 'commit': '48ab8e1c3',
        'integration': 'queued-after-3.4.3',
        'enumerated_api_occurrences': len(successor_entries),
        'member_supersessions': [],
    }, 'successor'
    signatures = re.findall(r'\{\{api\|[^{}]*\}\}\s*\([^)]*\)', decoded)
    return {
        'source_rows': len(rows), 'statuses': dict(Counter(r['status'] for r in rows)),
        'inventory_occurrences': len(entries),
        'added': sum(entry['direction'] == 'added' for entry in entries),
        'removed': sum(entry['direction'] == 'removed' for entry in entries),
        'section_counts': dict(Counter(entry['section'] for entry in entries)),
        'prose_limits': len(prose_lines), 'signature_occurrences': len(signatures),
        'successor_inventory_occurrences': len(successor_entries),
        'runtime_observations': profile['runtime_observations'],
        'native_observations': profile['native_observations'],
        'cache_files': len(profile['cache_files']),
    }


def check_seals():
    seals = json.loads((EVIDENCE / 'seals.json').read_bytes())
    for path, expected in seals.items():
        assert digest((ROOT / path).read_bytes()) == expected, f'seal: {path}'
    return len(seals)


def main():
    sealed_inputs = check_seals()
    print(json.dumps(dict(validate(**load_inputs()), sealed_inputs=sealed_inputs), sort_keys=True))


if __name__ == '__main__':
    main()
