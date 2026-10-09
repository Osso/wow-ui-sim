#!/usr/bin/env python3
"""Replay sealed TBC 2.5.3 SOURCE contracts, never loaded/native parity."""
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
PATCH = '2.5.3'
SUCCESSORS = ['2.5.4', '2.5.5', '2.5.6']


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


def validate_pin(response, raw, pin, manifest, registry):
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision, = page['revisions']
    assert page['pageid'] == pin['pageid'], 'pageid'
    assert page['title'] == pin['title'], 'title'
    assert revision['revid'] == pin['revid'], 'revid'
    assert revision['timestamp'] == pin['timestamp'], 'timestamp'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'], 'source hash'
    assert digest(response) == pin['response_sha256'], 'response hash'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'
    assert pin == next(row for row in manifest['pages'] if row['version'] == pin['version']), 'manifest pin'
    registered = next(row for row in registry['pages'] if row['version'] == pin['version'])
    assert all(pin[key] == value for key, value in registered.items()), 'registry identity'


def render_register(raw):
    generator = load_tool('gen_patch_wikitext_register')
    sections = generator.split_sections(raw)
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        parsed, counts = generator.parse_section(section, sections.get(section, []))
        entries.extend(parsed)
        headers.extend(counts)
    assert all(row['parsed_count'] == row['header_count'] for row in headers), 'header reconciliation'
    return {
        'schema': 'patch-api-wikitext-register/v1', 'patch': PATCH,
        'client_line': 'tbc-classic',
        'source': {'path': 'data/patch-api/sources/2.5.3-api-changes.wikitext',
                   'revid': 523812, 'sha256': digest(raw.encode())},
        'header_counts': headers, 'entries': entries,
    }


def inventory_counts(register):
    entries = register['entries']
    return {
        'enumerated_api_occurrences': len(entries),
        'by_section': dict(Counter(row['section'] for row in entries)),
        'by_direction': dict(Counter(row['direction'] for row in entries)),
        'header_counts': register['header_counts'],
        'explicit_signatures': 0,
        'unspecified_callable_signatures': sum(row['section'] in ('global-api', 'widgets') for row in entries),
        'cvar_defaults': sum('page_default' in row for row in entries),
        'local_summary_contracts': 1, 'content_transclusions': 0,
        'register_created': True,
    }


def row_kind(line, entries):
    if line.startswith('* SharedTooltipTemplate '):
        return 'summary-contract'
    if entries:
        return 'inventory-contract'
    if line.startswith('! '):
        return 'inventory-header'
    if line.startswith('* Community patch notes:'):
        return 'linked-resource'
    if line.startswith(('* TOC:', '* Diffs:', '* Deprecated API:')):
        return 'source-resource'
    if line.startswith('{{apichanges|'):
        return 'navigation-template'
    if line.startswith('|+'):
        return 'build-caption'
    return 'editorial-markup'


def build_rows(raw, register):
    rows = []
    for number, line in enumerate(raw.splitlines(), 1):
        if not line.strip():
            continue
        entries = [entry for entry in register['entries'] if entry['wikitext_line'] == number]
        kind = row_kind(line, entries)
        status = 'UNPROVEN' if kind in ('summary-contract', 'inventory-contract') else 'metadata-only'
        note = {
            'inventory-contract': 'Literal publication/removal claim only. Linked signatures, payloads, state effects, validation, security and native equivalence UNPROVEN; retained CVar defaults/descriptions are source claims, not measured behavior.',
            'summary-contract': 'SharedTooltipTemplate and GameTooltipTemplate no longer inherit BackdropTemplate: loader inheritance, resulting methods/backdrops and native equivalence UNPROVEN.',
            'inventory-header': 'Source numerical header reconciled against own literal inventory; no runtime credit.',
            'linked-resource': 'External community changes not expanded or pinned at linked content; contracts UNPROVEN.',
            'source-resource': 'Literal TOC/diff/deprecation resource; linked content not expanded and runtime correspondence UNPROVEN.',
            'navigation-template': 'Literal 2.5.3 prev=2.5.2 next=2.5.4 navigation, not expanded template content or successor proof.',
            'build-caption': 'Literal 2.5.2 (40011) to 2.5.3 (41812), Jan 7 2022 caption; not an interface/profile inference.',
            'editorial-markup': 'Heading/table/HTML/comment syntax preserved verbatim; no runtime credit.',
        }[kind]
        rows.append({'source_id': f'p253-source-{number:03}', 'wikitext_line': number,
                     'source_text': line, 'kind': kind, 'status': status,
                     'capabilities': [], 'inventory_ids': [entry['id'] for entry in entries],
                     'note': note})
    return rows


def build_contracts(raw, register):
    lines = raw.splitlines()
    contracts = []
    for entry in register['entries']:
        literal = lines[entry['wikitext_line'] - 1]
        description = re.search(r'<br><small>(.*?)</small>', literal)
        scope = re.search(r'Scope: <span class="apitype">(.*?)</span>', literal)
        contracts.append({
            'source_id': entry['id'], 'symbol': entry['symbol'], 'section': entry['section'],
            'direction': entry['direction'], 'status': 'UNPROVEN',
            'signature': None, 'arguments': None, 'returns': None, 'event_payloads': None,
            'page_default': entry.get('page_default'),
            'page_scope': scope[1] if scope else None,
            'page_description': description[1] if description else None,
            'state_transitions': None, 'validation': None, 'security_rules': None,
            'native_equivalence': None, 'capabilities': [],
            'limit': 'Literal source claim only; linked documentation not expanded; callable signatures, event payloads, operational CVar state/default enforcement, security and native contracts UNPROVEN.',
        })
    summary_line = next(i for i, line in enumerate(lines, 1) if line.startswith('* SharedTooltipTemplate '))
    contracts.append({
        'source_id': f'p253-source-{summary_line:03}', 'kind': 'template-inheritance-removal',
        'templates': ['SharedTooltipTemplate', 'GameTooltipTemplate'],
        'removed_parent': 'BackdropTemplate', 'status': 'UNPROVEN',
        'state_transitions': None, 'security_rules': None, 'native_equivalence': None,
        'capabilities': [],
        'limit': 'Exact summary claim retained. Actual template ancestry, inherited API/backdrop behavior and native equivalence UNPROVEN.',
    })
    return contracts


def linked_contracts(raw):
    links = []
    for number, line in enumerate(raw.splitlines(), 1):
        for url, label in re.findall(r'\[(https?://\S+)\s+([^]]+)\]', line):
            links.append({'wikitext_line': number, 'url': url, 'label': label,
                          'status': 'UNPROVEN', 'expanded': False,
                          'limit': 'Linked diff/deprecated/community contracts not pinned or expanded; identities, signatures, state/security/native effects UNPROVEN.'})
    return links


def expand_features(features, feature):
    enabled = {feature}
    for child in features[feature]:
        if child in features:
            enabled.update(expand_features(features, child))
    return enabled


def configured_profiles():
    """Inspect frozen literal arms/feature graph only; not a shared classifier."""
    config = EVIDENCE / 'configured-inputs'
    rust = (config / 'src/client_profile.rs').read_text()
    features = tomllib.loads((config / 'Cargo.toml').read_text())['features']
    arms = re.search(r'pub fn cache_subdir.*?match self \{(.*?)\n        \}', rust, re.S)[1]
    names = re.findall(r'ClientProfile::(\w+) => "(\w+)"', arms)
    interface_arms = re.search(r'pub const fn interface_version.*?match self \{(.*?)\n        \}',
                              rust.split('impl ClientProfile {', 1)[1], re.S)[1]
    epochs = dict(re.findall(r'RetailApiEpoch::(\w+) => (\d+)', rust))
    interfaces = {}
    for variants, value in re.findall(r'(ClientProfile::\w+(?: \| ClientProfile::\w+)*) => (\w+)', interface_arms):
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
        profiles.append({'variant': variant, 'feature': feature, 'cache_subdir': name,
                         'configured_interface': interface,
                         'manifest': {'path': path.relative_to(EVIDENCE).as_posix(),
                                      'sha256': digest(path.read_bytes()), 'entries': len(lines),
                                      'toc_entries': sum(line.lower().endswith('.toc') for line in lines),
                                      'mainline_toc_entries': sum(line.endswith('_Mainline.toc') for line in lines)}})
    return profiles


def build_profile():
    config = EVIDENCE / 'configured-inputs'
    return {
        'code_revision': BASE, 'source_toc': 20503, 'source_interface_family': '205xx',
        'native_profile_correspondence': 'UNPROVEN', 'unsupported_client_diagnosis': None,
        'measurements': {key: 'not-performed' for key in ['cache', 'runtime', 'model', 'native', 'full_ui']},
        'configured_profiles': configured_profiles(),
        'original_file_hashes': {path.relative_to(config).as_posix(): digest(path.read_bytes())
                                 for path in sorted(config.rglob('*')) if path.is_file()},
    }


def validate(*, response, raw, pin, ledger, text, profile, register):
    manifest = json.loads((EVIDENCE / 'frozen-manifest.json').read_bytes())
    registry_bytes = (EVIDENCE / 'frozen-registry.json').read_bytes()
    assert digest(registry_bytes) == manifest['registry_sha256'], 'registry hash'
    registry = json.loads(registry_bytes)
    assert registry['pages'][-1]['version'] == '1.0.0', 'registry boundary'
    validate_pin(response, raw, pin, manifest, registry)
    assert (pin['pageid'], pin['revid'], pin['timestamp']) == (53568, 523812, '2022-06-04T23:59:56Z'), 'source identity'
    assert pin['wikitext_sha256'] == 'f922ba1f9ee5e22f6a6c5a70bf8b5fe614b76a2a993ff5ee68149250198a2ca1', 'fixed source hash'
    assert pin['response_sha256'] == '0feb1ca97798ce6b2169e97207cc44f7e4847165d28e95fa9fbd8dbb6d71283e', 'fixed response hash'
    assert ledger['source'] == pin, 'ledger pin'
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'], ledger['scope'], ledger['later_registers']) == (
        'patch-source-accounting/v1', PATCH, 'tbc-classic', None,
        'source-and-configured-profile-accounting', SUCCESSORS), 'client history'
    assert ledger['successor_policy'] == {
        'status': 'pending-main-integration', 'client_line': 'tbc-classic',
        'positive_supersession_credit': False, 'versions': SUCCESSORS,
    }, 'successor policy'
    for version in SUCCESSORS:
        path = EVIDENCE / 'pending-successors' / version
        successor_pin = json.loads((path / 'source-pin.json').read_bytes())
        validate_pin((path / 'source-response.json').read_bytes(), (path / 'source.wikitext').read_bytes(),
                     successor_pin, manifest, registry)
    literal = raw.decode()
    assert '* TOC: <code>20503</code>' in literal.splitlines(), 'literal TOC'
    assert register == render_register(literal), 'inventory register'
    expected_text = load_tool('extract_patch_non_inventory').extract_text(literal, canonical_patch_navigation=True)
    assert text == expected_text.encode(), 'plaintext reproduction'
    assert ledger['non_inventory_source'] == {
        'path': 'data/patch-api/sources/2.5.3-api-changes.txt', 'sha256': digest(text),
        'extractor_flags': ['--text-only', '--canonical-patch-navigation'],
        'limit': 'Stock plaintext intentionally omits inventories; complete raw-row ledger and register retain them.',
    }, 'plaintext provenance'
    assert ledger['source_rows'] == build_rows(literal, register), 'literal row accounting'
    assert ledger['inventory'] == inventory_counts(register), 'inventory accounting'
    assert ledger['contracts'] == build_contracts(literal, register), 'contract accounting'
    assert ledger['linked_contracts'] == linked_contracts(literal), 'linked contract accounting'
    assert profile == build_profile(), 'profile evidence'
    inventory = ledger['inventory']
    return {
        'source_rows': len(ledger['source_rows']),
        'statuses': dict(Counter(row['status'] for row in ledger['source_rows'])),
        'inventory_occurrences': len(register['entries']), 'headers': len(register['header_counts']),
        'summary_contracts': 1, 'explicit_signatures': 0,
        'unspecified_callable_signatures': inventory['unspecified_callable_signatures'],
        'cvar_defaults': inventory['cvar_defaults'], 'transclusions': 0,
        'contracts': len(ledger['contracts']), 'linked_contracts': len(ledger['linked_contracts']),
        'configured_profiles': len(profile['configured_profiles']),
        'runtime_observations': 0, 'native_observations': 0, 'model_observations': 0,
    }


def main():
    sealed_inputs = check_seals()
    summary = validate(
        response=(EVIDENCE / 'source-response.json').read_bytes(),
        raw=(SOURCE / '2.5.3-api-changes.wikitext').read_bytes(),
        pin=json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
        ledger=json.loads((SOURCE / '2.5.3-page-coverage.json').read_bytes()),
        text=(SOURCE / '2.5.3-api-changes.txt').read_bytes(),
        profile=json.loads((EVIDENCE / 'profile-observation.json').read_bytes()),
        register=json.loads((SOURCE / '2.5.3-wikitext-register.json').read_bytes()),
    )
    print(json.dumps(dict(summary, sealed_inputs=sealed_inputs), sort_keys=True))


if __name__ == '__main__':
    main()
