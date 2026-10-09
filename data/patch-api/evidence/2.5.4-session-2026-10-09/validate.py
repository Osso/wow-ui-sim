#!/usr/bin/env python3
"""Replay bounded historical TBC SOURCE contracts, never simulator behavior."""
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


def load_json(path):
    return json.loads(path.read_bytes())


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, EVIDENCE / f'historical-{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_seals():
    seals = load_json(EVIDENCE / 'seals.json')
    for path, expected in seals.items():
        assert digest((ROOT / path).read_bytes()) == expected, f'seal: {path}'
    return len(seals)


def inventory(raw):
    generator = load_tool('gen_patch_wikitext_register')
    sections = generator.split_sections(raw)
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        parsed, counts = generator.parse_section(section, sections.get(section, []))
        entries.extend(parsed)
        headers.extend(counts)
    assert all(h['header_count'] == h['parsed_count'] for h in headers), 'literal header reconciliation'
    assert len(entries) == len(generator.TEMPLATE.findall(raw)) + raw.count('[[CVar '), 'unaccounted literal API'
    counts = {}
    for header in headers:
        counts.setdefault(header['section'], {})[header['direction']] = header['parsed_count']
    return {'entries': entries, 'header_counts': headers, 'counts': counts,
            'enumerated_api_occurrences': len(entries), 'explicit_signatures': 0,
            'unspecified_callable_signatures': sum(e['section'] in ('global-api', 'widgets') for e in entries),
            'local_summary_contracts': 0, 'content_transclusions': 0,
            'navigation_templates': len(re.findall(r'\{\{apichanges\|', raw))}


def unknown_contract(entry, literal):
    section = entry['section']
    contract = {
        'id': entry['id'], 'wikitext_line': entry['wikitext_line'],
        'kind': {'global-api': 'callable-publication', 'widgets': 'widget-method-publication',
                 'events': 'event-publication', 'cvars': 'cvar-publication'}[section],
        'symbol': entry['symbol'], 'direction': entry['direction'],
        'status': 'UNPROVEN', 'publication': None, 'native_equivalence': None,
        'arguments': None, 'returns': None, 'event_payloads': None,
        'state_transitions': None, 'security_rules': None,
        'signature': 'not-specified' if section in ('global-api', 'widgets') else 'not-a-callable',
        'source_reference': f'2.5.4-api-changes.wikitext#L{entry["wikitext_line"]}',
    }
    if section == 'events':
        contract['limit'] = 'Name/addition only; emitter, ordering, payload and lifecycle unmeasured.'
    elif section == 'cvars':
        scope = re.search(r'Scope: <span class="apitype">([^<]+)</span>', literal)
        description = re.search(r'<small>(.*?)</small>', literal)
        contract['literal_default'] = entry.get('page_default')
        contract['literal_scope'] = scope[1] if scope else None
        contract['literal_description'] = description[1] if description else None
        contract['limit'] = 'Literal default/scope/description, if present, are source claims; storage, mutation and effect unmeasured. Removal is not a current-profile absence observation.'
    else:
        contract['limit'] = 'Symbol/addition only; argument types/defaults, return tuple, errors, state, side effects and security unspecified/unmeasured; linked API documentation not retrieved.'
    return contract


def account_source(raw, parsed):
    entries = {e['wikitext_line']: e for e in parsed['entries']}
    assert len(entries) == len(parsed['entries']), 'duplicate source line'
    rows, contracts = [], []
    section = 'resources'
    for number, literal in enumerate(raw.splitlines(), 1):
        if not literal.strip():
            continue
        heading = re.fullmatch(r'==([^=]+)==', literal)
        if heading:
            section = heading[1]
        ids = []
        kind, status, note = 'source-context', 'metadata-only', 'Literal source/layout context; no runtime credit.'
        if number in entries:
            contract = unknown_contract(entries[number], literal)
            contracts.append(contract)
            ids = [contract['id']]
            kind, status, note = 'inventory-occurrence', 'UNPROVEN', contract['limit']
        elif literal.startswith('* Diffs:'):
            for target in re.findall(r'\[(https://\S+) [^\]]+\]', literal):
                cid = f'linked-diff-{number}-{len(ids) + 1}'
                contracts.append({'id': cid, 'wikitext_line': number, 'kind': 'linked-source',
                                  'target': target, 'status': 'UNPROVEN',
                                  'member_identities': None, 'signatures': None,
                                  'state_transitions': None, 'native_equivalence': None,
                                  'limit': 'Pinned commit link is literal; linked content not retrieved or expanded.'})
                ids.append(cid)
            kind, status, note = 'linked-source-boundary', 'UNPROVEN', 'Two commit links retained without diff reconstruction.'
        elif literal.startswith('! '):
            kind = 'numerical-header'
            note = 'Literal Added/Removed count reconciled with occurrences, not publication/native proof.'
        elif literal.startswith('|+'):
            kind = 'build-caption'
            note = 'Literal 2.5.3 (41812) to 2.5.4 (44833), Jul 25 2022; not response revision date or measured client.'
        elif literal.startswith('{{apichanges|'):
            kind = 'navigation-template'
            note = 'Navigation next=3.4.0 is Wrath Classic, not same-TBC supersession; prev=2.5.3. No content transclusion.'
        rows.append({'source_id': f'p254-literal-{number:03}', 'wikitext_line': number,
                     'section': section, 'source_text': literal, 'kind': kind,
                     'status': status, 'capabilities': [], 'contract_ids': ids, 'note': note})
    return rows, contracts


def expand_features(features, feature):
    enabled = {feature}
    for child in features[feature]:
        if child in features:
            enabled.update(expand_features(features, child))
    return enabled


def configured_profiles():
    """Interpret only these copied historical Rust arms/Cargo feature graph."""
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
                                      'toc_entries': sum(line.lower().endswith('.toc') for line in lines)}})
    return profiles


def expected_profile():
    config = EVIDENCE / 'configured-inputs'
    profiles = configured_profiles()
    return {'code_revision': BASE, 'source_toc': 20504, 'source_interface_family': '205xx',
            'configured_profiles': profiles,
            'matching_configured_profiles': [p['variant'] for p in profiles if 20500 <= p['configured_interface'] < 20600],
            'original_file_hashes': {p.relative_to(config).as_posix(): digest(p.read_bytes())
                                    for p in sorted(config.rglob('*')) if p.is_file()},
            'native_profile_correspondence': 'UNPROVEN', 'unsupported_api_diagnosis': None,
            'measurements': {key: 'not-performed' for key in ['cache', 'runtime', 'model', 'native', 'full_ui']}}


def expected_ledger(raw, pin, text):
    parsed = inventory(raw)
    rows, contracts = account_source(raw, parsed)
    registry = load_json(EVIDENCE / 'frozen-registry.json')['pages']
    return {'schema': 'patch-source-accounting/v1', 'patch': '2.5.4',
            'client_line': 'tbc-classic', 'profile': None,
            'scope': 'source-and-configured-profile-accounting', 'source': pin,
            'source_toc': 20504, 'later_registers': [],
            'pending_successors': [dict(next(p for p in registry if p['version'] == version),
                                       client_line='tbc-classic', status='pending-main-integration',
                                       supersession_credit=False) for version in ['2.5.5', '2.5.6']],
            'non_inventory_source': {'path': 'data/patch-api/sources/2.5.4-api-changes.txt',
                                     'sha256': digest(text),
                                     'extractor_flags': ['--text-only', '--canonical-patch-navigation']},
            'inventory': parsed, 'source_rows': rows, 'contracts': contracts,
            'model_changes': 0, 'runtime_observations': 0, 'native_observations': 0,
            'integration_owner': 'main', 'native_gate_owner': 'main'}


def read_inputs():
    return {'response': (EVIDENCE / 'source-response.json').read_bytes(),
            'raw': (SOURCE / '2.5.4-api-changes.wikitext').read_bytes(),
            'pin': load_json(EVIDENCE / 'source-pin.json'),
            'ledger': load_json(SOURCE / '2.5.4-page-coverage.json'),
            'text': (SOURCE / '2.5.4-api-changes.txt').read_bytes(),
            'profile': load_json(EVIDENCE / 'profile-observation.json')}


def validate(*, response, raw, pin, ledger, text, profile):
    page = json.loads(response)['query']['pages']['515131']
    revision, = page['revisions']
    for key, expected in [('pageid', 515131), ('title', 'Patch 2.5.4/API changes')]:
        assert page[key] == pin[key] == expected, key
    for key, expected in [('revid', 4967736), ('timestamp', '2023-06-20T22:06:32Z')]:
        assert revision[key] == pin[key] == expected, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == 'c4e57ba61e46af02c1f0fb48d6b6048d540898f2eea08694b95b5321f2b47766', 'source hash'
    assert digest(response) == pin['response_sha256'] == 'e72b07ad6e287743b2f2d17b1d2ad200442cd893cbe6bfa83ba71203a272ffbc', 'response hash'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'
    manifest = load_json(EVIDENCE / 'frozen-manifest.json')
    assert pin == next(p for p in manifest['pages'] if p['version'] == '2.5.4'), 'frozen pin'
    registry = (EVIDENCE / 'frozen-registry.json').read_bytes()
    assert digest(registry) == manifest['registry_sha256'], 'registry hash'
    pages = json.loads(registry)['pages']
    assert len(pages) == 101 and pages[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next(p for p in pages if p['version'] == '2.5.4')
    assert all(registered[key] == pin[key] for key in registered), 'registry identity'
    assert '* TOC: <code>20504</code>' in raw.decode().splitlines(), 'literal TOC'
    assert text == load_tool('extract_patch_non_inventory').extract_text(raw.decode(), canonical_patch_navigation=True).encode(), 'plaintext reproduction'
    expected = expected_ledger(raw.decode(), pin, text)
    for key, value in expected.items():
        assert ledger.get(key) == value, f'ledger {key}'
    assert ledger.keys() == expected.keys(), 'unaccounted ledger fields'
    assert profile == expected_profile(), 'configured profile evidence'
    return {'source_rows': len(ledger['source_rows']),
            'statuses': dict(Counter(r['status'] for r in ledger['source_rows'])),
            'inventory_occurrences': ledger['inventory']['enumerated_api_occurrences'],
            'header_counts': ledger['inventory']['header_counts'],
            'contracts': len(ledger['contracts']),
            'unspecified_callable_signatures': ledger['inventory']['unspecified_callable_signatures'],
            'explicit_signatures': ledger['inventory']['explicit_signatures'],
            'summary_contracts': ledger['inventory']['local_summary_contracts'],
            'content_transclusions': ledger['inventory']['content_transclusions'],
            'linked_contracts': sum(c['kind'] == 'linked-source' for c in ledger['contracts']),
            'configured_profiles': len(profile['configured_profiles']),
            'matching_configured_profiles': profile['matching_configured_profiles'],
            'registry_pages': len(pages), 'registry_boundary': pages[-1]['version'],
            'model_changes': 0, 'runtime_observations': 0, 'native_observations': 0}


def main():
    sealed = check_seals()
    print(json.dumps(dict(validate(**read_inputs()), sealed_inputs=sealed), sort_keys=True))


if __name__ == '__main__':
    main()
