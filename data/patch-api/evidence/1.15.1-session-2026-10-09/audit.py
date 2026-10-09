"""Literal frozen 1.15.1 accounting. SOURCE proof, not native/runtime proof."""
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
E = Path(__file__).resolve().parent
BASE = '7ad66791e64f01dd6dc61d29d3aa48faccdce3b8'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(name):
    return json.loads((E / name).read_bytes())

def validate_source(*, response=None, raw=None):
    response = (E / 'source-response.json').read_bytes() if response is None else response
    raw = (E / 'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['577687']
    revision, = page['revisions']
    for key, value in [('pageid', 577687), ('title', 'Patch 1.15.1/API changes')]:
        assert page[key] == pin[key] == value, key
    for key, value in [('revid', 5998991), ('timestamp', '2024-04-03T08:43:49Z')]:
        assert revision[key] == pin[key] == value, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '5749af9ee315150e2053b149b5127257cbadcf21eaa36c05f8729c9a47b37f67', 'raw hash'
    assert digest(response) == pin['response_sha256'] == '39fe39c08d9c1ebc267ad30bf8a56a2fc0367264303f30ad67f1f9891e9d39d7', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 835, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next(p for p in manifest['pages'] if p['version'] == '1.15.1'), 'manifest identity'
    registry_bytes = (E / 'frozen-registry.json').read_bytes()
    assert digest(registry_bytes) == manifest['registry_sha256'], 'registry hash'
    registry = json.loads(registry_bytes)['pages']
    assert len(registry) == 101 and registry[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next(p for p in registry if p['version'] == '1.15.1')
    assert all(pin[k] == v for k, v in registered.items()), 'registry identity'
    return raw.decode(), pin

def successors():
    manifest = read_json('frozen-manifest.json')
    result = []
    for n in range(2, 10):
        version = f'1.15.{n}'
        directory = E / 'successor-inputs' / version
        pin = next(p for p in manifest['pages'] if p['version'] == version)
        raw = (directory / 'source.wikitext').read_bytes()
        response = (directory / 'source-response.json').read_bytes()
        assert digest(raw) == pin['wikitext_sha256'], 'successor raw hash'
        assert digest(response) == pin['response_sha256'], 'successor response hash'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision, = page['revisions']
        assert page['pageid'] == pin['pageid'] and page['title'] == pin['title'], 'successor page'
        assert revision['revid'] == pin['revid'] and revision['timestamp'] == pin['timestamp'], 'successor revision'
        assert revision['slots']['main']['*'].encode() == raw, 'successor content'
        status = json.loads((directory / 'status.json').read_bytes())
        state = 'inflight' if n == 2 else 'queued' if n == 3 else 'integrated-canonical-not-applied'
        assert status == {'patch': version, 'state': state, 'applied': False, 'native_proof': False}, 'successor status'
        result.append(dict(status, source=pin))
    return result

def configuration():
    directory = E / 'configured-inputs'
    rust = (directory / 'src/client_profile.rs').read_text()
    interface = int(re.search(r'ClientProfile::Era \| ClientProfile::Anniversary => (\d+)', rust)[1])
    features = tomllib.loads((directory / 'Cargo.toml').read_text())['features']
    for profile in ['era', 'anniversary']:
        assert f'client-{profile}' in features, 'configured profile'
        assert (directory / f'data/blizzard-ui-files/{profile}.txt').read_bytes(), 'configured manifest'
    return {'era': interface, 'anniversary': interface}

def contract(identifier, line, kind, target, limit):
    return {'id': identifier, 'line': line, 'kind': kind, 'target': target, 'limit': limit, 'status': 'UNPROVEN', 'arguments': None, 'returns': None, 'numeric_value': None, 'state_transitions': None, 'security': None, 'native_equivalence': None}

def build():
    raw, pin = validate_source()
    lines = raw.splitlines()
    contracts = [contract('subset-004', 4, 'unspecified-retail-subset', None, 'Dragonflight subset unspecified; no imported 10.2.5 members/signatures/behavior.')]
    target, = re.findall(r'\[\[([^|]+)\|[^]]+\]\]', lines[3])
    contracts.append(contract('retail-link-004', 4, 'unexpanded-retail-page', target, 'Retail attribution only, not an Era successor or expanded contract.'))
    name, = re.findall(r'<code>([^<]+)</code>', lines[4])
    alias, = re.findall(r'<code>([^<]+)</code>', lines[5])
    contracts.append(contract('addition-005', 5, 'added-enum-member', name, 'Named finite enum member; raw page gives no numeric value.'))
    contracts.append(contract('alias-006', 6, 'deprecated-alias-continuity', alias, f'Alias must continue detecting Season of Discovery as-is; target {name}. No numeric value supplied.'))
    for i, (target, label) in enumerate(re.findall(r'\[(https://\S+) ([^]]+)\]', lines[9]), 1):
        contracts.append(contract(f'diff-010-{i}', 10, 'unexpanded-diff', target, f'{label}: linked body unexpanded; no changes/signatures/state inferred.'))
    target, label = re.findall(r'\[(https://\S+) ([^]]+)\]', lines[10])[0]
    contracts.append(contract('deprecated-link-011', 11, 'unexpanded-deprecated-api', target, f'{label}: body unexpanded; not permission to import aliases beyond literal line6.'))
    rows = [{'id': f'source-{n:03d}', 'line': n, 'literal': line, 'status': 'UNPROVEN' if n in [4, 5, 6, 10, 11] else 'metadata-only', 'capabilities': []} for n, line in enumerate(lines, 1) if line.strip()]
    headers = [{'line': n, 'label': match[1], 'numerical_count': None} for n, line in enumerate(lines, 1) if (match := re.fullmatch(r'==([^=]+)==', line))]
    inventory = [{'line': 5, 'name': name, 'role': 'added-finite-enum-member', 'numeric_value': None, 'status': 'UNPROVEN'}, {'line': 6, 'name': alias, 'role': 'deprecated-alias', 'numeric_value': None, 'status': 'UNPROVEN'}]
    return {'schema': 'patch-source-accounting/v1', 'patch': '1.15.1', 'base_revision': BASE, 'source': pin, 'source_toc': int(re.search(r'<code>(\d+)</code>', lines[8])[1]), 'literal_client_names': [], 'literal_expansion_names': ['Dragonflight'], 'client_scope': 'Same-Era task context; template and linked resource labels not native-client proof.', 'source_rows': rows, 'contracts': contracts, 'api_occurrences': inventory, 'signatures': [], 'alias': {'name': alias, 'target': name, 'literal_contract': 'continue working as-is for detecting Season of Discovery realms', 'numeric_value': None, 'status': 'UNPROVEN'}, 'headers': headers, 'templates': [{'line': 1, 'literal': lines[0], 'expansion': 'UNPROVEN'}], 'configured_interfaces': configuration(), 'successors': successors(), 'later_registers': [], 'measurements': {'runtime': 0, 'model': 0, 'native': 0}, 'model_investigation': read_json('model-investigation.json'), 'totals': {'physical_lines': len(lines), 'nonblank_rows': len(rows), 'metadata_rows': sum(x['status'] == 'metadata-only' for x in rows), 'unproven_rows': sum(x['status'] == 'UNPROVEN' for x in rows), 'api_occurrences': len(inventory), 'signature_declarations': 0, 'prose_contracts': 3, 'unexpanded_linked_contracts': 4, 'contracts': len(contracts), 'headers': len(headers), 'templates': 1}}

def validate_ledger(ledger):
    assert ledger == build(), 'serialized literal ledger'
    return ledger['totals']

def check_seals():
    seals = read_json('seals.json')
    for name, expected in seals.items():
        assert digest((E / name).read_bytes()) == expected, f'seal: {name}'
    return len(seals)

if __name__ == '__main__':
    if len(sys.argv) == 2 and sys.argv[1] == 'capture':
        assert not (E / 'ledger.json').exists(), 'refuse ledger overwrite'
        (E / 'ledger.json').write_text(json.dumps(build(), indent=2) + '\n')
    else:
        print(json.dumps({'historical_seals': check_seals(), 'totals': validate_ledger(read_json('ledger.json'))}, indent=2))
