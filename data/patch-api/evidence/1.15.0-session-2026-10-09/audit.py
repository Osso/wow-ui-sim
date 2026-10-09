"""Literal Era 1.15.0 SOURCE accounting; no native or model credit."""
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
EVIDENCE = Path(__file__).resolve().parent
BASE = '0b64e636c16c59e90e3406e7ff557e94cd7732a8'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(name):
    return json.loads((EVIDENCE / name).read_bytes())

def validate_source(*, response=None, raw=None):
    response = (EVIDENCE / 'source-response.json').read_bytes() if response is None else response
    raw = (EVIDENCE / 'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['564510']
    revision, = page['revisions']
    for key, value in [('pageid', 564510), ('title', 'Patch 1.15.0/API changes')]:
        assert page[key] == pin[key] == value, key
    for key, value in [('revid', 5950848), ('timestamp', '2024-01-30T19:36:35Z')]:
        assert revision[key] == pin[key] == value, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '4c85cbb0f7815c441b276f2267054f1aabbb8571abb7db995448b689dd426b04', 'raw hash'
    assert digest(response) == pin['response_sha256'] == 'd427176850fcac49c5c66e78a8cfef6863df28efa50b42bbfe92e1e8d2a49717', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 915, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next((p for p in manifest['pages'] if p['version'] == '1.15.0')), 'manifest pin'
    registry = (EVIDENCE / 'frozen-registry.json').read_bytes()
    assert digest(registry) == manifest['registry_sha256'], 'registry hash'
    pages = json.loads(registry)['pages']
    assert len(pages) == 101 and pages[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next((p for p in pages if p['version'] == '1.15.0'))
    assert all((pin[key] == value for key, value in registered.items())), 'registry identity'
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
        profiles.append(dict(variant=variant, feature=feature, feature_dependencies=features[feature], cache_subdir=subdir, configured_interface=int(interface), manifest_sha256=digest(manifest), manifest_entries=len(manifest.splitlines()), native_correspondence='UNPROVEN'))
    return profiles

def queued_successors():
    manifest = read_json('frozen-manifest.json')
    successors = []
    for number in range(1, 10):
        version = f'1.15.{number}'
        directory = f'queued-successors/{version}'
        pin = read_json(f'{directory}/source-pin.json')
        assert pin == next((p for p in manifest['pages'] if p['version'] == version)), 'successor pin'
        raw = (EVIDENCE / directory / 'source.wikitext').read_bytes()
        response = (EVIDENCE / directory / 'source-response.json').read_bytes()
        assert digest(raw) == pin['wikitext_sha256'], 'successor raw hash'
        assert digest(response) == pin['response_sha256'], 'successor response hash'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision, = page['revisions']
        assert page['pageid'] == pin['pageid'] and page['title'] == pin['title'], 'successor page'
        assert revision['revid'] == pin['revid'] and revision['timestamp'] == pin['timestamp'], 'successor revision'
        assert revision['slots']['main']['*'].encode() == raw, 'successor content'
        status = read_json(f'{directory}/status.json')
        expected = 'in-flight-pending-main-integration' if number == 1 else 'queued-pending-main-integration' if number == 2 else 'integrated-canonical-input-not-applied'
        assert status == dict(patch=version, state=expected, history='same-Era-context-only', applied=False, native_proof=False, main_gates='unperformed-for-this-page'), 'successor boundary'
        successors.append(dict(status, source=pin))
    return successors

def contract(identifier, line, literal, kind, limit, **fields):
    return dict(id=identifier, line=line, literal=literal, kind=kind, status='UNPROVEN', arguments=None, returns=None, event_payloads=None, state_transitions=None, security_rules=None, native_equivalence=None, limit=limit, **fields)

def prose_contracts(lines):
    scopes = ['all-Wrath-3.4.3-claim', 'unspecified-Dragonflight-10.1.7-through-10.2.0-subset', 'linked-guidance', 'seasonal-rune-namespace-addition']
    limits = ['Literal all-inclusion claim retained, not downgraded to subset. Wrath 3.4.3 history unexpanded; no imported member/behavior proof.', 'Literal subset/range retained separately. Selected Dragonflight members unspecified; range not expanded or collapsed into Wrath history.', 'Guidance reference unexpanded; no concrete state/API/security contract.', 'Explicit C_Engraving addition requires seasonal rune model, but functions/arguments/returns/rune state/transitions/security unspecified. Generic namespace fallback is not model/native proof.']
    return [dict(line=n, literal=lines[n - 1], scope=scope, status='UNPROVEN', expanded_members=[], limit=limit) for n, scope, limit in zip(range(4, 8), scopes, limits)]

def linked_contracts(lines):
    links = []
    for n, literal in enumerate(lines, 1):
        for index, (target, label) in enumerate(re.findall('\\[\\[([^|]+)\\|([^]]+)\\]\\]', literal), 1):
            links.append(contract(f'wiki-link-{n:03d}-{index}', n, literal, 'unexpanded-wiki-link', 'Literal linked context only; no history expansion or semantic supersession.', target=target, label=label))
        for index, (target, label) in enumerate(re.findall('\\[(https://\\S+) ([^]]+)\\]', literal), 1):
            kind = 'unexpanded-linked-diff' if '/compare/' in target else 'unexpanded-deprecated-api-link'
            fields = dict(target=target, label=label)
            if kind == 'unexpanded-linked-diff':
                fields['compare_base'], fields['compare_head'] = target.rsplit('/compare/', 1)[1].split('..')
            links.append(contract(f'external-link-{n:03d}-{index}', n, literal, kind, 'Linked content not retained/expanded; no identities, signatures, removals, defaults or native credit imported.', **fields))
    return links

def build():
    raw, pin = validate_source()
    lines = raw.splitlines()
    navigation = re.fullmatch('\\{\\{apichanges\\|([^|]+)\\|prev=([^|]+)\\|next=([^|]+)\\}\\}', lines[0])
    assert navigation, 'navigation'
    patch, previous, following = navigation.groups()
    toc = re.fullmatch('\\* TOC: <code>(\\d+)</code>', lines[9])
    assert toc, 'TOC'
    prose = prose_contracts(lines)
    contracts = [contract(f"prose-{p['line']:03d}", p['line'], p['literal'], p['scope'], p['limit']) for p in prose] + linked_contracts(lines)
    rows = [dict(id=f'source-{n:03d}', line=n, literal=literal, status='UNPROVEN' if n in (4, 5, 6, 7, 11, 12) else 'metadata-only', capabilities=[]) for n, literal in enumerate(lines, 1) if literal.strip()]
    headers = [dict(line=n, literal=literal, label=heading[1], numerical_count=None, status='metadata-only') for n, literal in enumerate(lines, 1) if (heading := re.fullmatch('==([^=]+)==', literal))]
    namespaces = [dict(line=7, symbol='C_Engraving', kind='namespace', direction='added', literal=lines[6], status='UNPROVEN')]
    signatures = [dict(line=7, symbol='C_Engraving', kind='namespace-functions-unspecified', functions=None, arguments=None, returns=None, status='UNPROVEN')]
    return dict(schema='patch-source-accounting/v1', patch=patch, source=pin, source_toc=int(toc[1]), base_revision=BASE, client_line='Era-task-context-not-native-proof', literal_clients=['Wrath Classic', 'Dragonflight'], client_limit='Linked Wrath and Dragonflight histories are distinct from Era task context/configuration. Unexpanded apichanges does not establish native identity.', literal_navigation=dict(patch=patch, prev=previous, next=following), source_rows=rows, contracts=contracts, summary_prose=prose, headers=headers, transclusions=[dict(line=1, literal=lines[0], role='navigation-metadata-only', expansion='UNPROVEN')], api_occurrences=namespaces, signatures=signatures, configured_profiles=configured_profiles(), queued_successors=queued_successors(), later_registers=[], measurements=dict(runtime=0, model=0, native=0), preservation='Literal source retained; no link expansion, invented functions/defaults/aliases or merged histories.', totals=dict(physical_lines=len(lines), nonblank_rows=len(rows), metadata_rows=sum((r['status'] == 'metadata-only' for r in rows)), unproven_rows=sum((r['status'] == 'UNPROVEN' for r in rows)), contracts=len(contracts), headers=len(headers), numerical_inventory_headers=0, api_occurrences=len(namespaces), namespace_occurrences=len(namespaces), callable_occurrences=0, event_occurrences=0, cvar_occurrences=0, widget_method_occurrences=0, command_occurrences=0, signature_declarations=0, unspecified_signature_records=len(signatures), prose_contracts=len(prose), unexpanded_linked_contracts=len(contracts) - len(prose), unexpanded_transclusions=1))

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
        print(json.dumps(dict(historical_seals=count, totals=totals), indent=2))
if __name__ == '__main__':
    main()
