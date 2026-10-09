#!/usr/bin/env python3
"""Replay only sealed Patch 2.5.5 SOURCE/configuration accounting."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import tomllib

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
SOURCE = ROOT / 'data/patch-api/sources'
BASE = 'acf7fbfe9b07dc342b8078cb1c54b702151a23d6'


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


def expand_features(features, feature):
    enabled = {feature}
    for child in features[feature]:
        if child in features:
            enabled.update(expand_features(features, child))
    return enabled


def configured_profiles():
    """Read the literal copied Rust arms/Cargo feature graph and manifest bytes.

    Bounded to this historical syntax, not a new runtime/profile classifier.
    Manifests contain paths, not TOC contents or measured interface versions.
    """
    config = EVIDENCE / 'configured-inputs'
    rust = (config / 'src/client_profile.rs').read_text()
    features = tomllib.loads((config / 'Cargo.toml').read_text())['features']
    arms = re.search(r'pub fn cache_subdir.*?match self \{(.*?)\n        \}',
                     rust, re.S)[1]
    names = re.findall(r'ClientProfile::(\w+) => "(\w+)"', arms)
    interface_arms = re.search(
        r'pub const fn interface_version.*?match self \{(.*?)\n        \}',
        rust.split('impl ClientProfile {', 1)[1], re.S
    )[1]
    epochs = dict(re.findall(r'RetailApiEpoch::(\w+) => (\d+)', rust))
    interfaces = {}
    for variants, value in re.findall(
        r'(ClientProfile::\w+(?: \| ClientProfile::\w+)*) => (\w+)',
        interface_arms
    ):
        for variant in re.findall(r'ClientProfile::(\w+)', variants):
            interfaces[variant] = value
    profiles = []
    for variant, name in names:
        feature = f'client-{name}'
        enabled = expand_features(features, feature)
        value = interfaces[variant]
        if value == 'RETAIL_API_INTERFACE_VERSION':
            versions = [int(version) for epoch, version in epochs.items()
                        if 'retail-' + epoch.removeprefix('Retail').replace('_', '-') in enabled]
            interface = max(versions)
        else:
            interface = int(value)
        path = config / f'data/blizzard-ui-files/{name}.txt'
        lines = path.read_text().splitlines()
        profiles.append({
            'variant': variant, 'feature': feature, 'cache_subdir': name,
            'configured_interface': interface,
            'manifest': {
                'path': path.relative_to(EVIDENCE).as_posix(),
                'sha256': digest(path.read_bytes()), 'entries': len(lines),
                'toc_entries': sum(line.lower().endswith('.toc') for line in lines),
                'mainline_toc_entries': sum(line.endswith('_Mainline.toc') for line in lines),
            },
        })
    return profiles


def validate_profile(profile):
    expected = {
        'code_revision': BASE, 'source_toc': 20505, 'source_interface_family': '205xx',
        'native_profile_correspondence': 'UNPROVEN', 'unsupported_client_diagnosis': None,
        'measurements': {key: 'not-performed' for key in
                         ['cache', 'runtime', 'model', 'native', 'full_ui']},
    }
    assert {key: profile[key] for key in expected} == expected, 'profile evidence'
    profiles = configured_profiles()
    assert profile['configured_profiles'] == profiles, 'configured profiles'
    config = EVIDENCE / 'configured-inputs'
    hashes = {path.relative_to(config).as_posix(): digest(path.read_bytes())
              for path in sorted(config.rglob('*')) if path.is_file()}
    assert profile['original_file_hashes'] == hashes, 'configuration input hashes'
    return len(profiles)


def validate_frozen(response, raw, pin, version):
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision, = page['revisions']
    for key in ['pageid', 'title']:
        assert page[key] == pin[key], key
    for key in ['revid', 'timestamp']:
        assert revision[key] == pin[key], key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'], 'source hash'
    assert digest(response) == pin['response_sha256'], 'response hash'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'
    manifest = json.loads((EVIDENCE / 'frozen-manifest.json').read_bytes())
    assert pin == next(row for row in manifest['pages'] if row['version'] == version), 'frozen source pin'
    registry_bytes = (EVIDENCE / 'frozen-registry.json').read_bytes()
    assert digest(registry_bytes) == manifest['registry_sha256'], 'frozen registry hash'
    registry = json.loads(registry_bytes)['pages']
    assert registry[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next(row for row in registry if row['version'] == version)
    assert all(registered[key] == pin[key] for key in registered), 'registry identity'


def pending_successors():
    pin = json.loads((EVIDENCE / 'successor-pin.json').read_bytes())
    raw = (EVIDENCE / 'successor-wikitext.txt').read_bytes()
    validate_frozen((EVIDENCE / 'successor-response.json').read_bytes(), raw, pin, '2.5.6')
    assert (pin['pageid'], pin['revid'], pin['timestamp']) == (
        685353, 6778086, '2026-07-22T05:42:00Z'), 'successor identity'
    assert '* TOC: <code>20506</code>' in raw.decode().splitlines(), 'successor TOC'
    assert 'prev=2.5.5' in raw.decode().splitlines()[0], 'successor navigation'
    return [{
        'patch': '2.5.6', 'client_line': 'tbc-classic', 'source_toc': 20506,
        'pin': 'successor-pin.json', 'status': 'reference-only; main integration pending',
        'supersession_credit': False,
    }]


def source_contracts(rows):
    limits = {key: None for key in [
        'member_identities', 'arguments', 'returns', 'event_triggers',
        'event_payloads', 'state_transitions', 'security_rules', 'native_equivalence',
    ]}
    summary, = [row for row in rows if row['status'] == 'UNPROVEN']
    link = re.search(r'\[\[([^]|]+)\|([^]]+)\]\]', summary['source_text'])
    assert link[2] == 'Burning Crusade Classic Anniversary Edition', 'literal client identity'
    toc_row, = [row for row in rows if row['source_text'].startswith('* TOC:')]
    toc = re.search(r'<code>(\d+)</code>', toc_row['source_text'])[1]
    contracts = [
        dict(limits, source_id=summary['source_id'], status='UNPROVEN',
             kind='linked-prepatch-state-assertion', target=link[1]),
        dict(limits, source_id=toc_row['source_id'], status='UNPROVEN',
             kind='unmeasured-native-interface-expectation', target=toc),
    ]
    for row in rows:
        for target in re.findall(r'\[(https?://\S+) [^]]+\]', row['source_text']):
            contracts.append(dict(limits, source_id=row['source_id'], status='UNPROVEN',
                                  kind='unexpanded-external-diff', target=target))
    return contracts


def inventory_counts(raw, seeds):
    generator = load_tool('gen_patch_wikitext_register')
    sections = generator.split_sections(raw)
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        parsed, counts = generator.parse_section(section, sections.get(section, []))
        entries.extend(parsed)
        headers.extend(counts)
    assert not generator.TEMPLATE.search(raw), 'unexpected explicit API reference'
    return {
        'enumerated_api_occurrences': len(entries), 'header_counts': headers,
        'register_created': False,
        'removals': sum(entry['direction'] == 'removed' for entry in entries),
        'explicit_signatures': len(re.findall(r'\w+\([^\n)]*\)', raw)),
        'local_summary_contracts': sum(seed['status'] == 'audit-pending' for seed in seeds),
        'transclusions': len(re.findall(r'\{\{:', raw)),
    }


def validate(*, response, raw, pin, ledger, text, profile):
    validate_frozen(response, raw, pin, '2.5.5')
    assert (pin['pageid'], pin['revid'], pin['timestamp']) == (
        686953, 6838475, '2026-08-20T15:48:09Z'), 'source identity'
    assert digest(raw) == '1ded7ad90515d5437907eccd15d367592d10ef876414fa6801b8dd53d15177e2', 'original source hash'
    assert digest(response) == '131f2318b95b29a9140f76c4fde87d9c3ed0564df7fbd5486cae8035075059bc', 'original response hash'
    assert ledger['source'] == pin, 'source pin'
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'],
            ledger['scope'], ledger['later_registers']) == (
        'patch-source-accounting/v1', '2.5.5', 'tbc-classic', None,
        'source-and-configured-profile-accounting', []), 'client history'
    assert '* TOC: <code>20505</code>' in raw.decode().splitlines(), 'literal source TOC'
    assert ledger['pending_successors'] == pending_successors(), 'pending successor'
    extractor = load_tool('extract_patch_non_inventory')
    expected_text = extractor.extract_text(raw.decode(), canonical_patch_navigation=True)
    assert text == expected_text.encode(), 'plaintext reproduction'
    assert ledger['non_inventory_source']['sha256'] == digest(text), 'plaintext hash'
    assert ledger['non_inventory_source']['extractor_flags'] == [
        '--text-only', '--canonical-patch-navigation'
    ], 'extractor flags'
    assert ledger['non_inventory_source']['local_transform'] is None, 'local transform'
    seeds = extractor.seed_rows(expected_text, '2.5.5')
    rows = ledger['source_rows']
    nonblank = [(i, line) for i, line in enumerate(raw.decode().splitlines(), 1) if line.strip()]
    assert [row['source_id'] for row in rows] == [row['source_id'] for row in seeds], 'row accounting'
    assert len(rows) == len(nonblank), 'row accounting'
    for row, seed, (number, literal) in zip(rows, seeds, nonblank):
        assert (row['wikitext_line'], row['text_line'], row['source_text']) == (
            number, number, literal), 'literal row accounting'
        status = 'UNPROVEN' if seed['status'] == 'audit-pending' else seed['status']
        assert row['status'] == status, 'source proof limit'
        assert row['capabilities'] == [], 'no publication/behavior credit'
        assert row['note'], 'explicit proof limit'
    inventory = inventory_counts(raw.decode(), seeds)
    assert ledger['inventory'] == inventory, 'inventory accounting'
    assert ledger['contracts'] == source_contracts(rows), 'source contracts'
    profile_count = validate_profile(profile)
    return {
        'source_rows': len(rows), 'statuses': dict(Counter(row['status'] for row in rows)),
        'enumerated_api_occurrences': inventory['enumerated_api_occurrences'],
        'header_counts': inventory['header_counts'], 'removals': inventory['removals'],
        'explicit_signatures': inventory['explicit_signatures'],
        'local_summary_contracts': inventory['local_summary_contracts'],
        'transclusions': inventory['transclusions'],
        'unproven_contracts': len(ledger['contracts']),
        'pending_successors': len(ledger['pending_successors']),
        'configured_profiles': profile_count,
        'runtime_observations': 0, 'native_observations': 0, 'model_observations': 0,
    }


def main():
    sealed_inputs = check_seals()
    summary = validate(
        response=(EVIDENCE / 'source-response.json').read_bytes(),
        raw=(SOURCE / '2.5.5-api-changes.wikitext').read_bytes(),
        pin=json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
        ledger=json.loads((SOURCE / '2.5.5-page-coverage.json').read_bytes()),
        text=(SOURCE / '2.5.5-api-changes.txt').read_bytes(),
        profile=json.loads((EVIDENCE / 'profile-observation.json').read_bytes()),
    )
    print(json.dumps(dict(summary, sealed_inputs=sealed_inputs), sort_keys=True))


if __name__ == '__main__':
    main()

