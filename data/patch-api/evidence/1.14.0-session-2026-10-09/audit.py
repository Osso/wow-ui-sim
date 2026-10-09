"""Frozen Era 1.14.0 literal SOURCE accounting, not simulator parity."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tomllib
E = Path(__file__).resolve().parent
BASE = '53bdb30cbd74ee496109f7be5e21ebf4c2ad5016'
VERSIONS = ['1.14.1', '1.14.2', '1.14.3', '1.14.4'] + [f'1.15.{n}' for n in range(10)]

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(name):
    return json.loads((E / name).read_bytes())

def validate_source(*, response=None, raw=None):
    response = (E / 'source-response.json').read_bytes() if response is None else response
    raw = (E / 'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['71995']
    revision, = page['revisions']
    for key, value in [('pageid', 71995), ('title', 'Patch 1.14.0/API changes')]:
        assert page[key] == pin[key] == value, key
    for key, value in [('revid', 710568), ('timestamp', '2022-02-06T21:21:48Z')]:
        assert revision[key] == pin[key] == value, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == 'ead23026a86829caa312eb3b9cfcb57c5bfc85099a125fce84820c94e76490ac', 'raw hash'
    assert digest(response) == pin['response_sha256'] == '9e282d2407123b5850b9c7fc62c35158246d5a93109a4eafb2f9fe7133200289', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 30381, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next((p for p in manifest['pages'] if p['version'] == '1.14.0')), 'manifest pin'
    registry = (E / 'frozen-registry.json').read_bytes()
    assert digest(registry) == manifest['registry_sha256'], 'registry hash'
    pages = json.loads(registry)['pages']
    assert len(pages) == 101 and pages[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next((p for p in pages if p['version'] == '1.14.0'))
    assert all((pin[key] == value for key, value in registered.items())), 'registry identity'
    return (raw.decode(), pin)

def configured_profiles():
    config = E / 'configured-inputs'
    rust = (config / 'src/client_profile.rs').read_text()
    features = tomllib.loads((config / 'Cargo.toml').read_text())['features']
    variants, interface = re.search('(ClientProfile::Era \\| ClientProfile::Anniversary) => (\\d+)', rust).groups()
    arms = rust.split('pub fn cache_subdir', 1)[1].split('pub const fn interface_version', 1)[0]
    profiles = []
    for variant in re.findall('ClientProfile::(\\w+)', variants):
        subdir = re.search(f'ClientProfile::{variant} => "(\\w+)"', arms)[1]
        feature = f'client-{subdir}'
        assert feature in features, 'configured feature'
        manifest = (config / f'data/blizzard-ui-files/{subdir}.txt').read_bytes()
        profiles.append(dict(feature=feature, configured_interface=int(interface), cache_subdir=subdir, manifest_sha256=digest(manifest), manifest_entries=len(manifest.splitlines()), native_correspondence='UNPROVEN'))
    return profiles

def successors():
    manifest = read_json('frozen-manifest.json')
    results = []
    for version in VERSIONS:
        directory = f'successors/{version}'
        pin = read_json(f'{directory}/source-pin.json')
        assert pin == next((p for p in manifest['pages'] if p['version'] == version)), 'successor pin'
        raw = (E / directory / 'source.wikitext').read_bytes()
        response = (E / directory / 'source-response.json').read_bytes()
        assert digest(raw) == pin['wikitext_sha256'] and len(raw) == pin['wikitext_bytes'], 'successor raw'
        assert digest(response) == pin['response_sha256'], 'successor response'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision, = page['revisions']
        assert page['pageid'] == pin['pageid'] and page['title'] == pin['title'], 'successor page'
        assert revision['revid'] == pin['revid'] and revision['timestamp'] == pin['timestamp'], 'successor revision'
        assert revision['slots']['main']['*'].encode() == raw, 'successor content'
        state = 'in-flight-pending-main-integration' if version == '1.14.1' else 'queued-pending-main-integration' if version in ('1.14.2', '1.14.3') else 'actual-input-not-applied'
        status = read_json(f'{directory}/status.json')
        assert status == dict(patch=version, state=state, history='same-Era-context-only', applied=False, native_proof=False), 'successor boundary'
        results.append(dict(status, source=pin))
    return results

def parse_inventory(lines):
    entries, headers, counts, labels = ([], [], [], [])
    section, direction, columns = (None, None, 0)
    for n, literal in enumerate(lines, 1):
        if (heading := re.fullmatch('==([^=]+)==', literal)):
            section, direction, columns = (heading[1], None, 0)
            headers.append(dict(line=n, label=section, literal=literal, status='metadata-only'))
        elif literal.startswith('! '):
            count, = re.findall('<small>\\((\\d+)\\)</small>', literal)
            counts.append(dict(line=n, literal=literal, section=section, direction='added' if len([h for h in counts if h['section'] == section]) == 0 else 'removed', header_count=int(count)))
        elif literal.startswith('| valign="top"'):
            columns += 1
            direction = 'added' if columns == 1 else 'removed'
        elif literal == '</div>':
            direction = None
        elif literal.startswith(':'):
            assert direction, f'inventory column: {n}'
            if literal == ': Scripts':
                labels.append(dict(line=n, literal=literal, section=section, direction=direction, status='metadata-only'))
                continue
            if (match := re.fullmatch(': \\{\\{api\\|t=([awec])\\|([^{}]+)\\}\\}', literal)):
                code, symbol = match.groups()
                kind = {'a': 'callable', 'w': 'widget-method', 'e': 'event', 'c': 'cvar'}[code]
            else:
                match = re.fullmatch(': \\[\\[(UIHANDLER [^|]+)\\|([^]]+)\\]\\]', literal)
                assert match and section == 'Widgets', f'unrecognized inventory: {n}'
                symbol, kind = (match[2], 'widget-script')
            entries.append(dict(id=f'inventory-{n:03d}', line=n, literal=literal, section=section, direction=direction, symbol=symbol, kind=kind, status='UNPROVEN', limit='Identity/direction only; no literal behavior, signatures, default, payload, lifecycle, state or security contract. Publication is not model/native proof.'))
    for header in counts:
        header['parsed_count'] = sum((e['section'] == header['section'] and e['direction'] == header['direction'] for e in entries))
    return (entries, headers, counts, labels)

def reference_boundaries(lines):
    links, templates = ([], [])
    for n, literal in enumerate(lines, 1):
        for m in re.finditer('\\[\\[([^|\\]]+)\\|([^]]+)\\]\\]|\\[(https://\\S+) ([^]]+)\\]|(?<![\\[\\w])(https://\\S+)(?= -->)', literal):
            target = m[1] or m[3] or m[5]
            links.append(dict(line=n, literal=m[0], target=target, label=m[2] or m[4], expanded=False, status='metadata-only' if literal.startswith('<!--') else 'UNPROVEN', limit='Unexpanded linked content, not imported behavior or same-Era supersession.'))
        for m in re.finditer('\\{\\{([^{}]+)\\}\\}', literal):
            parts = m[1].split('|')
            templates.append(dict(line=n, literal=m[0], name=parts[0], parameters=parts[1:], expanded=False, status='metadata-only' if parts[0] == 'apichanges' else 'UNPROVEN'))
    return (links, templates)

def build():
    raw, pin = validate_source()
    lines = raw.splitlines()
    inventory, headers, counts, labels = parse_inventory(lines)
    links, templates = reference_boundaries(lines)
    substantive = {e['line'] for e in inventory} | {5, 6, 7}
    rows = [dict(id=f'source-{n:03d}', line=n, literal=literal, status='UNPROVEN' if n in substantive else 'metadata-only', capabilities=[]) for n, literal in enumerate(lines, 1) if literal.strip()]
    signatures = [dict(line=e['line'], symbol=e['symbol'], kind=e['kind'], arguments=None, returns=None, payload=None, state_transitions=None, security_rules=None, status='UNPROVEN') for e in inventory if e['kind'] != 'cvar']
    defaults = [dict(line=e['line'], symbol=e['symbol'], direction=e['direction'], default=None, value_domain=None, type=None, status='UNPROVEN') for e in inventory if e['kind'] == 'cvar']
    prose = [dict(line=7, literal=lines[6], scope='foreign-2.5.2-baseline-synchronization-claim', expanded_members=[], status='UNPROVEN', limit='Literal same-baseline synchronization claim, not a selected subset. Linked Classic global guide and 2.5.2 body unexpanded; no foreign behavior/native/member inclusion or Era successor credit.')]
    return dict(schema='patch-source-accounting/v1', patch='1.14.0', source=pin, source_toc=11400, base_revision=BASE, client_line='Era-task-context-not-native-proof', navigation=dict(patch='1.14.0', prev='1.13.7', next='1.14.1', like='2.5.2', expanded=False), caption=dict(line=12, literal=lines[11], previous_patch='1.13.7', previous_build=39692, current_patch='1.14.0', current_build=39958, channel='PTR', native_proof=False), source_rows=rows, inventory=inventory, headers=headers, count_headers=counts, category_labels=labels, prose=prose, links=links, templates=templates, signature_limits=signatures, default_limits=defaults, configured_profiles=configured_profiles(), successors=successors(), later_registers=[], measurements=dict(runtime=0, model=0, native=0), totals=dict(physical_lines=len(lines), nonblank_rows=len(rows), inventory=len(inventory), api_templates=sum((t['name'] == 'api' for t in templates)), widget_scripts=sum((e['kind'] == 'widget-script' for e in inventory)), category_labels=len(labels), headers=len(headers), count_headers=len(counts), count_conflicts=sum((h['header_count'] != h['parsed_count'] for h in counts)), prose=len(prose), links=len(links), templates=len(templates), signature_limits=len(signatures), default_limits=len(defaults)))

def validate_ledger(ledger, *, expected=None):
    assert ledger == (build() if expected is None else expected), 'serialized literal ledger'
    return ledger['totals']

def check_seals():
    seals = read_json('seals.json')
    for name, expected in seals.items():
        assert digest((E / name).read_bytes()) == expected, f'seal: {name}'
    return len(seals)

def extract_default():
    spec = importlib.util.spec_from_file_location('historical_extractor', E / 'historical-tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.extract_text((E / 'source.wikitext').read_text())

def main():
    if sys.argv[1:] == ['capture']:
        assert not (E / 'ledger.json').exists(), 'refuse ledger overwrite'
        (E / 'ledger.json').write_text(json.dumps(build(), indent=2) + '\n')
    elif sys.argv[1:] == ['extract']:
        sys.stdout.write(extract_default())
    else:
        count = check_seals()
        totals = validate_ledger(read_json('ledger.json'))
        print(json.dumps(dict(historical_seals=count, totals=totals), indent=2))
if __name__ == '__main__':
    main()
