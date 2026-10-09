"""Bounded frozen 1.15.9 SOURCE accounting; no runtime or shared parser changes."""
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
EVIDENCE = Path(__file__).resolve().parent
BASE = '1174d7a8c9adb3a9933729fc256f3cc3614215f8'
PROSE = {18: ('quoted-tbc-2.5.6', 'Service support, nameplate/raid-frame changes; forward reference is not Era native proof.'), 21: ('shared-code-claim', 'Hardware, graphics, accessibility, widescreen and Battle.net benefits; no member contracts.'), 23: ('quoted-history', 'Original drag-out raid UI, 2019 BFA frames, TBC 2.5.5 TWW and 2.5.6 Midnight frames; no equivalence measurement.'), 26: ('quoted-tbc-2.5.6', 'Customization context for following commands; not a 1.15.9 signature or default declaration.'), 43: ('quoted-tbc-plan', 'Planned Options-menu settings next week; prospective, no shipped UI proof.'), 46: ('classic-era', 'Classic Era, Season of Discovery, Hardcore: weekly regional maintenance week July 19; Edit Mode/nameplates/raid frames, not widget method enumeration.'), 48: ('compatibility-advice', 'Possible addon errors and very close 2.5.6 API; advice does not establish equality, supersession or native parity.'), 50: ('future-plan', 'Future refinement/options; unspecified prospective behavior.')}

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(name):
    return json.loads((EVIDENCE / name).read_bytes())

def validate_source(*, response=None, raw=None):
    response = (EVIDENCE / 'source-response.json').read_bytes() if response is None else response
    raw = (EVIDENCE / 'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['685352']
    revision, = page['revisions']
    for key, value in [('pageid', 685352), ('title', 'Patch 1.15.9/API changes')]:
        assert page[key] == pin[key] == value, key
    for key, value in [('revid', 6780591), ('timestamp', '2026-07-24T15:29:35Z')]:
        assert revision[key] == pin[key] == value, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == 'beea5ee3ad4a1a10c031cd02777a3d5bc1b5198e4637c713a088c4c9f94423db', 'raw hash'
    assert digest(response) == pin['response_sha256'] == '2691663c5ffc7efbd0b862a8a94dd8632141c55eb7393f2aca8003d3448e6baf', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 4085, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next((p for p in manifest['pages'] if p['version'] == '1.15.9')), 'manifest pin'
    registry_bytes = (EVIDENCE / 'frozen-registry.json').read_bytes()
    assert digest(registry_bytes) == manifest['registry_sha256'], 'registry hash'
    registry = json.loads(registry_bytes)['pages']
    assert len(registry) == 101 and registry[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next((p for p in registry if p['version'] == '1.15.9'))
    assert all((pin[k] == v for k, v in registered.items())), 'registry identity'
    return (raw.decode(), pin)

def configured_profiles():
    config = EVIDENCE / 'configured-inputs'
    rust = (config / 'src/client_profile.rs').read_text()
    features = tomllib.loads((config / 'Cargo.toml').read_text())['features']
    variants, interface = re.search('(ClientProfile::Era \\| ClientProfile::Anniversary) => (\\d+)', rust).groups()
    profiles = []
    for variant in re.findall('ClientProfile::(\\w+)', variants):
        cache_arms = rust.split('pub fn cache_subdir', 1)[1].split('pub const fn interface_version', 1)[0]
        subdir = re.search(f'ClientProfile::{variant} => "(\\w+)"', cache_arms)[1]
        feature = f'client-{subdir}'
        assert feature in features, 'configured feature'
        manifest = (config / f'data/blizzard-ui-files/{subdir}.txt').read_bytes()
        profiles.append({'variant': variant, 'feature': feature, 'feature_dependencies': features[feature], 'cache_subdir': subdir, 'configured_interface': int(interface), 'manifest_sha256': digest(manifest), 'manifest_entries': len(manifest.splitlines()), 'native_correspondence': 'UNPROVEN'})
    return profiles

def contract(number, kind, context, limit, **fields):
    suffix = '-' + digest(fields['target'].encode())[:8] if 'target' in fields else ''
    return dict(id=f'contract-{number:03d}-{kind}{suffix}', line=number, kind=kind, context=context, status='UNPROVEN', limit=limit, arguments=None, returns=None, event_payloads=None, state_transitions=None, security_rules=None, native_equivalence=None, **fields)

def build():
    raw, pin = validate_source()
    lines = raw.splitlines()
    assert '* TOC: <code>11509</code>' in lines, 'literal TOC'
    clients = re.search('WoW Classic games \\(([^)]+)\\)', lines[45])[1].split(', ')
    assert clients == ['Classic Era', 'Season of Discovery', 'Hardcore'], 'literal clients'
    contracts = [contract(8, 'partial-return-removal', 'classic-era', 'Removal only: no return index, replacement, arity or remaining signature stated.', symbol='UnitAura', removed_return='shouldConsolidate', removed_position=None, remaining_returns=None)]
    for target in re.findall('\\[(https://\\S+) ([^]]+)\\]', lines[4]):
        contracts.append(contract(5, 'unexpanded-linked-diff', 'classic-era', 'Unexpanded link: member identities/signatures/behavior remain UNPROVEN.', target=target[0], label=target[1]))
    contracts.extend((contract(n, 'prose', context, limit) for n, (context, limit) in PROSE.items()))
    occurrences = [{'line': 8, 'symbol': 'UnitAura', 'kind': 'partial-return-note'}]
    for number, literal in enumerate(lines, 1):
        call = re.fullmatch('/script (C_CVar.SetCVar)\\("([^"]+)", (\\d+)\\);', literal)
        if call:
            symbol, cvar, value = call.groups()
            contracts.append(contract(number, 'partial-chat-call', 'quoted-tbc-2.5.6', 'Example argument values, not defaults/full signatures; no Era behavior/native proof.', command='/script', symbol=symbol, cvar=cvar, value=int(value), default=None))
            occurrences.append({'line': number, 'symbol': symbol, 'kind': 'call-example'})
    contracts.sort(key=lambda c: (c['line'], c['id'], c.get('target', '')))
    headers = []
    for number, literal in enumerate(lines, 1):
        heading = re.fullmatch('==([^=]+)==', literal) or re.fullmatch('\\{\\{apisummary.heading\\|([^{}]+)\\}\\}', literal)
        if heading:
            headers.append({'line': number, 'literal': literal, 'label': heading[1], 'numerical_count': None, 'status': 'metadata-only'})
    substantive = {c['line'] for c in contracts} | {28, 33, 38}
    rows = [{'id': f'source-{n:03d}', 'line': n, 'literal': s, 'status': 'UNPROVEN' if n in substantive else 'metadata-only', 'capabilities': [], 'note': 'Literal contract/context; no runtime/model/native credit.' if n in substantive else 'Literal metadata, label or markup; no behavior credit.'} for n, s in enumerate(lines, 1) if s.strip()]
    return {'schema': 'patch-source-accounting/v1', 'patch': '1.15.9', 'source': pin, 'client_line': 'classic-era', 'source_toc': 11509, 'literal_clients': clients, 'base_revision': BASE, 'later_registers': [], 'measurements': {'runtime': 0, 'model': 0, 'native': 0}, 'source_rows': rows, 'contracts': contracts, 'headers': headers, 'api_occurrences': occurrences, 'configured_profiles': configured_profiles(), 'signature_limits': 'No complete declaration; 3 partial call examples and one partial return-removal note.', 'preservation': 'Raw bytes and every occurrence retained; no normalization, aliases, defaults, shims or transclusion expansion.', 'totals': {'nonblank_rows': len(rows), 'contracts': len(contracts), 'explicit_api_reference_occurrences': len(occurrences), 'distinct_api_names': len({r['symbol'] for r in occurrences}), 'cvar_argument_occurrences': 3, 'chat_command_occurrences': 3, 'event_occurrences': 0, 'widget_method_occurrences': 0, 'enumerated_inventory_entries': 0, 'numerical_inventory_headers': 0, 'explicit_signature_declarations': 0, 'partial_call_examples': 3, 'partial_return_removal_notes': 1, 'prose_contracts': len(PROSE), 'unexpanded_linked_contracts': 2, 'runtime_observations': 0, 'model_observations': 0, 'native_observations': 0}}

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
