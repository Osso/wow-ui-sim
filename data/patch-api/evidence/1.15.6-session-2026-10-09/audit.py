"""Bounded literal 1.15.6 SOURCE accounting, never runtime/native proof."""
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
EVIDENCE = Path(__file__).resolve().parent
BASE = '2d83b9b8088e110e6e170473ed59650d1821612e'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(name):
    return json.loads((EVIDENCE / name).read_bytes())

def validate_source(*, response=None, raw=None):
    response = (EVIDENCE / 'source-response.json').read_bytes() if response is None else response
    raw = (EVIDENCE / 'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['619994']
    revision, = page['revisions']
    for key, value in [('pageid', 619994), ('title', 'Patch 1.15.6/API changes')]:
        assert page[key] == pin[key] == value, key
    for key, value in [('revid', 6281951), ('timestamp', '2025-04-04T10:14:39Z')]:
        assert revision[key] == pin[key] == value, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == 'c43468b42190bbb8410f666b6f71bfa67e8c9582ec4ef12df2c227aa8e6810d0', 'raw hash'
    assert digest(response) == pin['response_sha256'] == '7e6837d03c7e3f942624866c85339685870e35d05c89649e66d94060635690aa', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 278, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next((p for p in manifest['pages'] if p['version'] == '1.15.6')), 'manifest pin'
    registry_bytes = (EVIDENCE / 'frozen-registry.json').read_bytes()
    assert digest(registry_bytes) == manifest['registry_sha256'], 'registry hash'
    registry = json.loads(registry_bytes)['pages']
    assert len(registry) == 101 and registry[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next((p for p in registry if p['version'] == '1.15.6'))
    assert all((pin[k] == v for k, v in registered.items())), 'registry identity'
    return (raw.decode(), pin)

def configured_profiles():
    config = EVIDENCE / 'configured-inputs'
    rust = (config / 'src/client_profile.rs').read_text()
    features = tomllib.loads((config / 'Cargo.toml').read_text())['features']
    variants, interface = re.search('(ClientProfile::Era \\| ClientProfile::Anniversary) => (\\d+)', rust).groups()
    cache_arms = rust.split('pub fn cache_subdir', 1)[1].split('pub const fn interface_version', 1)[0]
    profiles = []
    for variant in re.findall('ClientProfile::(\\w+)', variants):
        subdir = re.search(f'ClientProfile::{variant} => "(\\w+)"', cache_arms)[1]
        feature = f'client-{subdir}'
        assert feature in features, 'configured feature'
        manifest = (config / f'data/blizzard-ui-files/{subdir}.txt').read_bytes()
        profiles.append({'variant': variant, 'feature': feature, 'feature_dependencies': features[feature], 'cache_subdir': subdir, 'configured_interface': int(interface), 'manifest_sha256': digest(manifest), 'manifest_entries': len(manifest.splitlines()), 'native_correspondence': 'UNPROVEN'})
    return profiles

def queued_successors():
    manifest = read_json('frozen-manifest.json')
    successors = []
    for version in ['1.15.7', '1.15.8', '1.15.9']:
        directory = f'queued-successors/{version}'
        pin = read_json(f'{directory}/source-pin.json')
        assert pin == next((p for p in manifest['pages'] if p['version'] == version)), 'queued source pin'
        raw = (EVIDENCE / directory / 'source.wikitext').read_bytes()
        response = (EVIDENCE / directory / 'source-response.json').read_bytes()
        assert digest(raw) == pin['wikitext_sha256'], 'queued source hash'
        assert digest(response) == pin['response_sha256'], 'queued response hash'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision, = page['revisions']
        assert page['title'] == pin['title'] and page['pageid'] == pin['pageid'], 'queued page identity'
        assert revision['revid'] == pin['revid'] and revision['timestamp'] == pin['timestamp'], 'queued revision'
        assert revision['slots']['main']['*'].encode() == raw, 'queued literal source'
        status = read_json(f'{directory}/status.json')
        assert status['patch'] == version and status['applied'] is False, 'queued only'
        expected = 'inflight-pending-main-integration' if version == '1.15.7' else 'pending-main-integration'
        assert status['state'] == expected and status['history'] == 'same-Era-context-only', 'queued status'
        assert status['native_proof'] is False and status['main_gates'] == 'unperformed', 'no successor credit'
        successors.append(dict(status, source=pin))
    return successors

def build():
    raw, pin = validate_source()
    lines = raw.splitlines()
    navigation = re.fullmatch('\\{\\{apichanges\\|([^|]+)\\|prev=([^|]+)\\|next=([^|]+)\\}\\}', lines[0])
    assert navigation, 'literal navigation'
    patch, prev, following = navigation.groups()
    toc = re.fullmatch('\\* TOC: <code>(\\d+)</code>', lines[3])
    assert toc, 'literal TOC'
    links = re.findall('\\[(https://\\S+) ([^]]+)\\]', lines[4])
    contracts = []
    for index, (target, label) in enumerate(links, 1):
        contracts.append({'id': f'linked-diff-005-{index}', 'line': 5, 'kind': 'unexpanded-linked-diff', 'label': label, 'target': target, 'status': 'UNPROVEN', 'arguments': None, 'returns': None, 'event_payloads': None, 'state_transitions': None, 'security_rules': None, 'native_equivalence': None, 'limit': 'Unexpanded link; identities, changes, signatures, defaults and behavior unspecified.'})
    rows = [{'id': f'source-{n:03d}', 'line': n, 'literal': literal, 'status': 'UNPROVEN' if n == 5 else 'metadata-only', 'capabilities': []} for n, literal in enumerate(lines, 1) if literal.strip()]
    headers = [{'line': n, 'literal': literal, 'label': heading[1], 'numerical_count': None, 'status': 'metadata-only'} for n, literal in enumerate(lines, 1) if (heading := re.fullmatch('==([^=]+)==', literal))]
    return {'schema': 'patch-source-accounting/v1', 'patch': patch, 'source': pin, 'source_toc': int(toc[1]), 'client_line': 'UNPROVEN-unexpanded-apichanges', 'literal_clients': [], 'literal_navigation': {'patch': patch, 'prev': prev, 'next': following}, 'client_limit': 'No named client in literal page. Same-Era context/successor attribution remains separate; queued 1.15.7/1.15.8/1.15.9 pending main integration; do not guess from version/TOC or expand template.', 'base_revision': BASE, 'later_registers': [], 'queued_successors': queued_successors(), 'measurements': {'runtime': 0, 'model': 0, 'native': 0}, 'source_rows': rows, 'contracts': contracts, 'headers': headers, 'transclusions': [{'line': 1, 'literal': lines[0], 'expansion': 'UNPROVEN', 'role': 'navigation-metadata-only'}], 'api_occurrences': [], 'signatures': [], 'configured_profiles': configured_profiles(), 'preservation': 'Raw bytes/all occurrences retained; no normalization, aliases, defaults, shims or link/transclusion expansion.', 'totals': {'physical_lines': len(lines), 'nonblank_rows': len(rows), 'metadata_rows': sum((r['status'] == 'metadata-only' for r in rows)), 'unproven_rows': sum((r['status'] == 'UNPROVEN' for r in rows)), 'contracts': len(contracts), 'headers': len(headers), 'numerical_inventory_headers': 0, 'api_occurrences': 0, 'event_occurrences': 0, 'cvar_occurrences': 0, 'widget_method_occurrences': 0, 'command_occurrences': 0, 'signature_declarations': 0, 'prose_contracts': 0, 'unexpanded_linked_contracts': len(contracts), 'unexpanded_transclusions': 1}}

def validate_ledger(ledger):
    assert ledger == build(), 'serialized literal ledger'
    return ledger['totals']

def check_seals():
    seals = read_json('seals.json')
    for name, expected in seals.items():
        assert digest((EVIDENCE / name).read_bytes()) == expected, f'seal: {name}'
    return len(seals)

def main():
    if len(sys.argv) == 2 and sys.argv[1] == 'capture':
        assert not (EVIDENCE / 'ledger.json').exists(), 'refuse ledger overwrite'
        (EVIDENCE / 'ledger.json').write_text(json.dumps(build(), indent=2) + '\n')
    else:
        count = check_seals()
        totals = validate_ledger(read_json('ledger.json'))
        print(json.dumps({'historical_seals': count, 'totals': totals}, indent=2))
if __name__ == '__main__':
    main()
