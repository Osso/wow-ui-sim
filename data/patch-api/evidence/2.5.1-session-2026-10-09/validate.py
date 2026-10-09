#!/usr/bin/env python3
"""Replay only frozen 2.5.1 SOURCE/configuration contracts, never runtime parity."""
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
BASE = 'f95eed96e'
PENDING = [
    {'patch': patch, 'client_line': 'tbc-classic', 'status': 'pending-main-integration',
     'supersession_credit': False}
    for patch in ['2.5.2', '2.5.3', '2.5.4', '2.5.5', '2.5.6']
]


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


def parse_inventory(raw):
    generator = load_tool('gen_patch_wikitext_register')
    entries, headers = [], []
    for section, lines in generator.split_sections(raw).items():
        parsed, counts = generator.parse_section(section, lines, skip_plain_scripts_label=True)
        entries.extend(parsed)
        headers.extend(counts)
    assert all(h['header_count'] == h['parsed_count'] for h in headers), 'literal header reconciliation'
    assert len(entries) == len(generator.TEMPLATE.findall(raw)) + len(re.findall(r'^: \[\[UIHANDLER', raw, re.M)), 'literal inventory accounting'
    return {'entries': entries, 'headers': headers, 'explicit_signatures': 0,
            'transclusions': 0, 'source_toc': 20501}


def contract_limit(entry):
    name = entry['symbol']
    direction = entry['direction']
    if entry.get('kind') == 'command':
        return f'{name}: {direction} console-command name only; arguments, command effects, persistence, permissions and native equivalence unspecified.'
    if entry.get('kind') == 'widget-script' or entry['section'] == 'events':
        return f'{name}: {direction} notification name only; payload, trigger/order/lifecycle, dispatch security and native equivalence unspecified.'
    if entry['section'] == 'cvars':
        return f'{name}: {direction} CVar name only; default/type/range, persistence, state effects, security and native equivalence unspecified.'
    return f'{name}: {direction} API name only; arguments, returns, errors, state transitions, security rules and native equivalence unspecified.'


def linked_targets(literal):
    return re.findall(r'\[(https?://\S+)\s+[^]]+\]', literal)


def literal_accounting(raw, inventory):
    by_line = {entry['wikitext_line']: entry for entry in inventory['entries']}
    rows, contracts = [], []
    for number, literal in enumerate(raw.splitlines(), 1):
        if not literal.strip():
            continue
        source_id = f'2.5.1-raw-L{number}'
        entry = by_line.get(number)
        targets = linked_targets(literal) if literal.startswith(('* Diffs:', '* Deprecated APIs:', '* Community patch notes:')) else []
        status = 'UNPROVEN' if entry or targets else 'metadata-only'
        note = 'Literal source context/structure; no publication, model, security or native proof.'
        if entry:
            note = contract_limit(entry)
        elif targets:
            note = 'Linked content not retained or expanded; exact literal URL is context only. Member identities, signatures, migration/state/security/native contracts UNPROVEN; branch/wiki links are not frozen linked revisions.'
        rows.append({'source_id': source_id, 'wikitext_line': number,
                     'source_text': literal, 'status': status, 'capabilities': [], 'note': note})
        if not entry and not targets:
            continue
        contracts.append({
            'source_id': source_id, 'status': 'UNPROVEN',
            'kind': entry.get('kind', entry['section']) if entry else 'unexpanded-linked-summary',
            'member': entry['symbol'] if entry else None,
            'direction': entry['direction'] if entry else None,
            'linked_targets': targets, 'arguments': None, 'returns': None,
            'event_payloads': None, 'state_transitions': None, 'security_rules': None,
            'native_equivalence': None, 'limit': note,
        })
    return rows, contracts


def expand_features(features, feature):
    enabled = {feature}
    for child in features[feature]:
        if child in features:
            enabled.update(expand_features(features, child))
    return enabled


def configured_profiles():
    """Read copied historical syntax only; not a runtime/shared profile classifier."""
    config = EVIDENCE / 'configured-inputs'
    rust = (config / 'src/client_profile.rs').read_text()
    features = tomllib.loads((config / 'Cargo.toml').read_text())['features']
    arms = re.search(r'pub fn cache_subdir.*?match self \{(.*?)\n        \}', rust, re.S)[1]
    names = re.findall(r'ClientProfile::(\w+) => "(\w+)"', arms)
    interface_arms = re.search(
        r'pub const fn interface_version.*?match self \{(.*?)\n        \}',
        rust.split('impl ClientProfile {', 1)[1], re.S)[1]
    epochs = dict(re.findall(r'RetailApiEpoch::(\w+) => (\d+)', rust))
    interfaces = {}
    for variants, value in re.findall(r'(ClientProfile::\w+(?: \| ClientProfile::\w+)*) => (\w+)', interface_arms):
        for variant in re.findall(r'ClientProfile::(\w+)', variants):
            interfaces[variant] = value
    profiles = []
    for variant, name in names:
        enabled = expand_features(features, f'client-{name}')
        value = interfaces[variant]
        interface = int(value) if value != 'RETAIL_API_INTERFACE_VERSION' else max(
            int(version) for epoch, version in epochs.items()
            if 'retail-' + epoch.removeprefix('Retail').replace('_', '-') in enabled)
        path = config / f'data/blizzard-ui-files/{name}.txt'
        lines = path.read_text().splitlines()
        profiles.append({
            'variant': variant, 'feature': f'client-{name}', 'cache_subdir': name,
            'configured_interface': interface,
            'manifest': {'path': path.relative_to(EVIDENCE).as_posix(),
                         'sha256': digest(path.read_bytes()), 'entries': len(lines),
                         'toc_entries': sum(line.lower().endswith('.toc') for line in lines)},
        })
    return profiles


def validate_profile(profile):
    expected = {
        'code_revision': BASE, 'source_toc': 20501, 'source_interface_family': '205xx',
        'literal_client_line_label': None,
        'history_label_basis': 'TBC Classic audit label; literal TOC 20501 and navigation 1.13.7 -> 2.5.1 -> 2.5.2; source does not spell out a client-line label.',
        'native_profile_correspondence': 'UNPROVEN', 'unsupported_client_diagnosis': None,
        'measurements': {key: 'not-performed' for key in ['cache', 'runtime', 'model', 'native', 'full_ui']},
    }
    assert {key: profile[key] for key in expected} == expected, 'profile evidence'
    profiles = configured_profiles()
    assert profile['configured_profiles'] == profiles, 'configured profiles'
    config = EVIDENCE / 'configured-inputs'
    hashes = {p.relative_to(config).as_posix(): digest(p.read_bytes())
              for p in sorted(config.rglob('*')) if p.is_file()}
    assert profile['original_file_hashes'] == hashes, 'configuration hashes'
    return len(profiles)


def validate(*, response, raw, pin, ledger, text, profile):
    page = json.loads(response)['query']['pages']['71215']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
        71215, 'Patch 2.5.1/API changes', 701879, '2022-02-06T21:22:09Z'), 'source identity'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '994581e0b655316f1038f136ee4a9dd49fa4eabffc3ddcbeb92b0b13e7939daf', 'source hash'
    assert digest(response) == pin['response_sha256'] == 'de095f3c391a05dbeef9d43d32e10bbda74fed3570d4c2c2817bc1c42b60a230', 'response hash'
    manifest = json.loads((EVIDENCE / 'frozen-manifest.json').read_bytes())
    assert pin == ledger['source'] == next(p for p in manifest['pages'] if p['version'] == '2.5.1'), 'frozen pin'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'
    registry_bytes = (EVIDENCE / 'frozen-registry.json').read_bytes()
    assert digest(registry_bytes) == manifest['registry_sha256'], 'registry hash'
    registry = json.loads(registry_bytes)['pages']
    assert registry[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next(p for p in registry if p['version'] == '2.5.1')
    assert all(registered[k] == pin[k] for k in registered), 'registry identity'
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'],
            ledger['scope'], ledger['later_registers'], ledger['pending_successors']) == (
        'patch-source-accounting/v1', '2.5.1', 'tbc-classic', None,
        'source-and-configured-profile-accounting', [], PENDING), 'client history'
    literal = raw.decode()
    assert '* TOC: <code>20501</code>' in literal.splitlines(), 'literal TOC'
    assert literal.splitlines()[0] == '{{apichanges|2.5.1|prev=1.13.7|next=2.5.2}}', 'literal navigation'
    inventory = parse_inventory(literal)
    assert ledger['inventory'] == inventory, 'inventory accounting'
    rows, contracts = literal_accounting(literal, inventory)
    assert ledger['source_rows'] == rows, 'literal rows accounting'
    assert ledger['contracts'] == contracts, 'contracts accounting'
    extractor = load_tool('extract_patch_non_inventory')
    expected_text = extractor.extract_text(literal, canonical_patch_navigation=True)
    assert text == expected_text.encode(), 'plaintext reproduction'
    assert ledger['non_inventory_source'] == {
        'sha256': digest(text), 'extractor_flags': ['--text-only', '--canonical-patch-navigation'],
        'limit': 'Supplemental plaintext omits inventory; literal rows above retain every nonblank source row.'}, 'plaintext provenance'
    profile_count = validate_profile(profile)
    return {'source_rows': len(rows), 'statuses': dict(Counter(row['status'] for row in rows)),
            'enumerated_api_occurrences': len(inventory['entries']), 'contracts': len(contracts),
            'header_counts': inventory['headers'], 'explicit_signatures': inventory['explicit_signatures'],
            'transclusions': inventory['transclusions'], 'configured_profiles': profile_count,
            'registry_pages': len(registry), 'runtime_observations': 0, 'native_observations': 0}


def main():
    sealed_inputs = check_seals()
    summary = validate(
        response=(EVIDENCE / 'source-response.json').read_bytes(),
        raw=(SOURCE / '2.5.1-api-changes.wikitext').read_bytes(),
        pin=json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
        ledger=json.loads((SOURCE / '2.5.1-page-coverage.json').read_bytes()),
        text=(SOURCE / '2.5.1-api-changes.txt').read_bytes(),
        profile=json.loads((EVIDENCE / 'profile-observation.json').read_bytes()))
    print(json.dumps(dict(summary, sealed_inputs=sealed_inputs), sort_keys=True))


if __name__ == '__main__':
    main()
