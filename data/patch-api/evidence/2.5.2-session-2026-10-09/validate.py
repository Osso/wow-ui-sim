#!/usr/bin/env python3
"""Replay only sealed Patch 2.5.2 SOURCE/configuration accounting."""
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
BASE = 'f95eed96e10e8044215591b216cbe45e548a6819'


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
        'code_revision': BASE, 'source_toc': 20502, 'source_interface_family': '205xx',
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
    result = []
    for version in ['2.5.3', '2.5.4', '2.5.5', '2.5.6']:
        pin = json.loads((EVIDENCE / f'{version}-pin.json').read_bytes())
        raw = (EVIDENCE / f'{version}-wikitext.txt').read_bytes()
        validate_frozen((EVIDENCE / f'{version}-response.json').read_bytes(), raw, pin, version)
        toc = int('2050' + version[-1])
        assert f'* TOC: <code>{toc}</code>' in raw.decode().splitlines(), 'successor TOC'
        result.append({'patch': version, 'client_line': 'tbc-classic', 'source_toc': toc,
                       'pin': f'{version}-pin.json',
                       'status': 'reference-only; main integration pending',
                       'supersession_credit': False})
    return result


def inventory(raw):
    generator = load_tool('gen_patch_wikitext_register')
    sections = generator.split_sections(raw)
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        parsed, counts = generator.parse_section(section, sections.get(section, []))
        entries.extend(parsed)
        headers.extend(counts)
    assert len(entries) == len(generator.TEMPLATE.findall(raw)), 'all API references'
    assert all(h['header_count'] == h['parsed_count'] for h in headers), 'header reconciliation'
    return entries, headers


def row_note(kind, entry=None):
    if kind == 'summary':
        return 'UNPROVEN synchronization assertion: 1.14.0/Classic links unexpanded; subset, signatures, state and security unspecified; no Era/TBC/native equivalence.'
    if kind == 'api':
        limits = {
            'global-api': 'Arguments, returns, state effects and security unspecified; linked API documentation unexpanded.',
            'widgets': 'Receiver behavior, arguments, returns, state effects and security unspecified; linked widget contract unexpanded.',
            'events': 'Trigger ordering, payload, state transitions and security unspecified; linked event contract unexpanded.',
            'cvars': 'Default, type, persistence, validation, state effects and security unspecified; linked CVar contract unexpanded.',
        }
        return 'UNPROVEN ' + entry['direction'] + ' publication/removal/native behavior. ' + limits[entry['section']]
    return 'Source metadata/markup only; numerical headers reconciled to literal occurrences, not publication/native proof.'


def source_rows(raw, entries):
    by_line = {entry['wikitext_line']: entry for entry in entries}
    assert len(by_line) == len(entries), 'one API per source row'
    rows = []
    for number, literal in enumerate(raw.splitlines(), 1):
        if not literal.strip():
            continue
        entry = by_line.get(number)
        kind = 'api' if entry else 'summary' if literal.startswith('The [[Patch_1.14.0/') else 'metadata'
        rows.append({'source_id': f'raw-{number:03}', 'wikitext_line': number,
                     'source_text': literal, 'kind': kind,
                     'status': 'metadata-only' if kind == 'metadata' else 'UNPROVEN',
                     'capabilities': [], 'note': row_note(kind, entry)})
    return rows


def source_contracts(rows, entries):
    limits = {key: None for key in ['arguments', 'returns', 'event_triggers', 'event_payloads',
                                  'state_transitions', 'security_rules', 'native_equivalence',
                                  'cvar_default', 'cvar_persistence']}
    result = []
    for entry in entries:
        result.append(dict(limits, source_id=f"raw-{entry['wikitext_line']:03}",
                           status='UNPROVEN', kind=entry['section'],
                           target=entry['symbol'], direction=entry['direction']))
    for row in rows:
        literal = row['source_text']
        if row['kind'] == 'summary':
            result.append(dict(limits, source_id=row['source_id'], status='UNPROVEN',
                               kind='linked-client-synchronization',
                               target='Patch_1.14.0/API_changes; Global_functions/Classic',
                               member_identities=None))
        if literal.startswith('* TOC:'):
            result.append(dict(limits, source_id=row['source_id'], status='UNPROVEN',
                               kind='unmeasured-native-interface-expectation', target='20502'))
        for target in re.findall(r'\[(https?://\S+) [^]]+\]', literal):
            result.append(dict(limits, source_id=row['source_id'], status='UNPROVEN',
                               kind='unexpanded-external-resource', target=target,
                               member_identities=None))
    return result


def account_source(raw):
    entries, headers = inventory(raw)
    rows = source_rows(raw, entries)
    # Numeric count headers are not function signatures.
    signatures = re.findall(r'\b[A-Za-z_]\w*(?:[.:]\w+)?\([^\n)]*\)', raw)
    return {'source_rows': len(rows), 'api_occurrences': len(entries),
            'statuses': dict(Counter(row['status'] for row in rows)),
            'removals': sum(e['direction'] == 'removed' for e in entries),
            'explicit_signatures': len(signatures),
            'local_summary_contracts': sum(row['kind'] == 'summary' for row in rows),
            'transclusions': len(re.findall(r'\{\{:', raw)),
            'header_pairs': [[headers[i]['header_count'], headers[i + 1]['header_count']]
                             for i in range(0, len(headers), 2)]}


def validate(*, response, raw, pin, ledger, text, profile):
    validate_frozen(response, raw, pin, '2.5.2')
    assert (pin['pageid'], pin['revid'], pin['timestamp']) == (
        69347, 682905, '2022-03-02T05:17:18Z'), 'source identity'
    assert digest(raw) == 'b18b862196308f547c02842d4d70e40464e84a92f329f2c92c58049d036bf5b8', 'original source hash'
    assert digest(response) == 'c0494d340c89994169f373018ac202848a2caca6cbc49363c09ebdf3ceec9c04', 'original response hash'
    assert ledger['source'] == pin, 'source pin'
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'],
            ledger['scope'], ledger['later_registers']) == (
        'patch-source-accounting/v1', '2.5.2', 'tbc-classic', None,
        'source-and-configured-profile-accounting', []), 'client history'
    assert '* TOC: <code>20502</code>' in raw.decode().splitlines(), 'literal source TOC'
    assert ledger['pending_successors'] == pending_successors(), 'pending successor'
    extractor = load_tool('extract_patch_non_inventory')
    expected_text = extractor.extract_text(raw.decode(), canonical_patch_navigation=True)
    assert text == expected_text.encode(), 'plaintext reproduction'
    assert ledger['non_inventory_source'] == {
        'sha256': digest(text), 'extractor_flags': ['--text-only', '--canonical-patch-navigation'],
        'local_transform': None}, 'plaintext provenance'
    entries, headers = inventory(raw.decode())
    assert ledger['source_rows'] == source_rows(raw.decode(), entries), 'row accounting'
    assert ledger['inventory_entries'] == entries, 'API occurrence accounting'
    assert ledger['inventory_headers'] == headers, 'header accounting'
    assert ledger['accounting'] == account_source(raw.decode()), 'derived accounting'
    assert ledger['contracts'] == source_contracts(ledger['source_rows'], entries), 'source contracts'
    assert ledger['signatures'] == [], 'no explicit signatures'
    assert ledger['measurements'] == {key: 'not-performed' for key in ['runtime', 'native', 'model']}, 'no behavioral credit'
    profiles = validate_profile(profile)
    return dict(account_source(raw.decode()), unproven_contracts=len(ledger['contracts']),
                unspecified_callable_signatures=sum(e['section'] in ['global-api', 'widgets'] for e in entries),
                pending_successors=len(ledger['pending_successors']), configured_profiles=profiles,
                runtime_observations=0, native_observations=0, model_observations=0)


def inputs():
    return {'response': (EVIDENCE / 'source-response.json').read_bytes(),
            'raw': (SOURCE / '2.5.2-api-changes.wikitext').read_bytes(),
            'pin': json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
            'ledger': json.loads((SOURCE / '2.5.2-page-coverage.json').read_bytes()),
            'text': (SOURCE / '2.5.2-api-changes.txt').read_bytes(),
            'profile': json.loads((EVIDENCE / 'profile-observation.json').read_bytes())}


def main():
    sealed_inputs = check_seals()
    print(json.dumps(dict(validate(**inputs()), sealed_inputs=sealed_inputs), sort_keys=True))


if __name__ == '__main__':
    main()
