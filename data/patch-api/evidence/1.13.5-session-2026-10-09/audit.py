"""Frozen Patch 1.13.5 literal accounting; SOURCE only, no native/model credit."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tomllib
from collections import Counter
E = Path(__file__).resolve().parent
BASE = '045e396b0c6f717c2d962b97cdf46b736a3328ce'
OCCURRENCE_FIELDS = ('source_rows', 'extracted_rows', 'inventory', 'signatures', 'headers', 'inventory_headers', 'captions', 'prose', 'links', 'templates', 'references', 'contracts', 'configured_profiles', 'successors', 'model_review')

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(name):
    return json.loads((E / name).read_bytes())

def load_tool(name):
    spec = importlib.util.spec_from_file_location(name.removesuffix('.py'), E / 'historical-tools' / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def validate_source(response=None, raw=None):
    response = (E / 'source-response.json').read_bytes() if response is None else response
    raw = (E / 'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['398847']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (398847, 'Patch 1.13.5/API changes', 3835002, '2021-05-06T13:20:38Z'), 'identity'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == 'f8a5356cff1fc8c2188b68aa77ab60969c102ea64767a4f4721fd07532f85ac2', 'raw hash'
    assert digest(response) == pin['response_sha256'] == 'eaa85552784310bd4cf62fab378f6863123fe1843c7826537495499f18d9f831', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 1690, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next((p for p in manifest['pages'] if p['version'] == '1.13.5')), 'manifest pin'
    assert digest((E / 'frozen-registry.json').read_bytes()) == manifest['registry_sha256'], 'registry hash'
    pages = read_json('frozen-registry.json')['pages']
    registered = next((p for p in pages if p['version'] == '1.13.5'))
    assert all((pin[k] == v for k, v in registered.items())), 'registry membership'
    assert all((pin[k] == v for k, v in dict(pageid=page['pageid'], title=page['title'], revid=revision['revid'], timestamp=revision['timestamp']).items())), 'pin identity'
    return (raw.decode(), pin)

def default_register(raw, pin):
    generator = load_tool('gen_patch_wikitext_register.py')
    buckets = generator.split_sections(raw)
    entries, headers = ([], [])
    for section in generator.SECTIONS.values():
        rows, counts = generator.parse_section(section, buckets.get(section, []))
        entries.extend(rows)
        headers.extend(counts)
    return dict(schema='patch-api-wikitext-register/v1', patch='1.13.5', source=dict(path='source.wikitext', revid=pin['revid'], sha256=pin['wikitext_sha256']), header_counts=headers, entries=entries)

def profiles():
    directory = E / 'state-inputs'
    rust = (directory / 'src/client_profile.rs').read_text()
    interface = int(re.search('ClientProfile::Era \\| ClientProfile::Anniversary => (\\d+)', rust)[1])
    features = tomllib.loads((directory / 'Cargo.toml').read_text())['features']
    return [dict(feature=f'client-{name}', configured_interface=interface, feature_dependencies=features[f'client-{name}'], manifest_sha256=digest((directory / f'data/blizzard-ui-files/{name}.txt').read_bytes()), native_correspondence='UNPROVEN') for name in ('era', 'anniversary')]

def successors():
    status = read_json('successors/status.json')
    versions = ['1.13.7', '1.14.0', '1.14.1', '1.14.2', '1.14.3', '1.14.4'] + [f'1.15.{n}' for n in range(10)]
    assert [s['patch'] for s in status] == versions, 'successor order'
    manifest = read_json('frozen-manifest.json')
    for item in status:
        version = item['patch']
        pin = next((p for p in manifest['pages'] if p['version'] == version))
        assert item['pin'] == pin, 'successor pin'
        assert item['state'] == 'canonical-input-not-applied' and (not item['applied']) and (not item['native_proof']), 'successor credit boundary'
        directory = E / 'successors' / version
        raw = (directory / 'source.wikitext').read_bytes()
        response = (directory / 'response.json').read_bytes()
        assert digest(raw) == pin['wikitext_sha256'] and digest(response) == pin['response_sha256'], 'successor hashes'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision, = page['revisions']
        assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (pin['pageid'], pin['title'], pin['revid'], pin['timestamp']), 'successor identity'
        assert revision['slots']['main']['*'].encode() == raw, 'successor content'
    return status

def reference_limits(line, literal):
    links = []
    for match in re.finditer('\\[\\[([^\\]|]+)(?:\\|([^\\]]+))?\\]\\]|https://[^\\s|}<]+', literal):
        target = match[1] if match[1] is not None else match[0]
        links.append(dict(line=line, literal=match[0], target=target, label=match[2], kind='wiki' if match[1] is not None else 'external', expanded=False, limit='Target contents unretained; no linked signatures, defaults, API members or behavior credit.'))
    templates = [dict(line=line, literal=m[0], name=m[1].split('|')[0], expanded=False, limit='Literal template text only; no transclusion or target-body credit.') for m in re.finditer('\\{\\{([^{}]+)\\}\\}', literal)]
    references = [dict(line=line, literal=m[0], name='patch notes', definition=m[0].endswith('</ref>'), expanded=False, limit='Named reference preserved, including author/title/date/url; body of target not expanded.') for m in re.finditer('<ref\\b[^>]+(?:/>|>.*?</ref>)', literal)]
    return (links, templates, references)

def prose_limit(line, literal):
    common = dict(line=line, literal=literal, status='UNPROVEN', arguments=None, returns=None, security=None, native_equivalence=None)
    if line == 10:
        return dict(common, claims=['Threat API reinstated', 'combat log range increased in dungeons and raids', 'stated purpose: reduce need for addon comms'], distance_before=None, distance_after=None, payload=None, state_transition=None, limit='No signatures/returns/calculation or numerical log ranges. Dungeon/raid context and stated purpose retained; patch-note citation unexpanded.')
    return dict(common, additional_slots=int(re.search('\\* (\\d+) additional backpack slots', literal)[1]), condition='Blizzard Authenticator secured accounts', baseline_capacity=None, final_capacity=None, entitlement_transition=None, limit='Delta 4 only, not a default or final slot count. No entitlement activation, persistence or getter contract. Authenticator reward reference unexpanded.')

def default_extract(raw):
    try:
        text = load_tool('extract_patch_non_inventory.py').extract_text(raw)
    except ValueError as error:
        return dict(result='existing-failure', text=None, error_type='ValueError', error=str(error), flags=[])
    return dict(result='success', text=text, error_type=None, error=None, flags=[])

def build():
    raw, pin = validate_source()
    register = default_register(raw, pin)
    extracted = default_extract(raw)
    byline = {i['wikitext_line']: i for i in register['entries']}
    rows, headers, counts, inventory, links, templates, references, prose, captions = ([], [], [], [], [], [], [], [], [])
    context = None
    for n, literal in enumerate(raw.splitlines(), 1):
        if not literal.strip():
            continue
        heading = re.fullmatch('==\\s*([^=]+?)\\s*==', literal)
        if heading:
            context = heading[1]
            headers.append(dict(line=n, literal=literal, label=context))
        count = re.search('\\| (\\d+) new (functions|events)$', literal)
        if count:
            section = {'Global API': 'global-api', 'Events': 'events'}[context]
            observed = sum((i['section'] == section and i['direction'] == 'added' for i in register['entries']))
            counts.append(dict(line=n, literal=literal, section=section, direction='added', count=int(count[1]), observed=observed, conflict=int(count[1]) != observed))
        if n == 4:
            match = re.fullmatch('(\\S+) \\((\\d+)\\) to (\\S+) \\((\\d+)\\)', literal)
            captions.append(dict(line=n, literal=literal, before_patch=match[1], before_build=int(match[2]), after_patch=match[3], after_build=int(match[4])))
        entry = byline.get(n)
        if entry:
            inventory.append(dict(entry, line=n, literal=literal, status='UNPROVEN', capabilities=[]))
        if n in (10, 11):
            prose.append(prose_limit(n, literal))
        new_links, new_templates, new_refs = reference_limits(n, literal)
        links.extend(new_links)
        templates.extend(new_templates)
        references.extend(new_refs)
        substantive = entry is not None or n in (5, 6, 10, 11)
        rows.append(dict(id=f'source-{n:03d}', line=n, literal=literal, status='UNPROVEN' if substantive else 'metadata-only', capabilities=[]))
    nav = re.fullmatch('\\{\\{apichanges\\|([^|]+)\\|prev=([^|]+)\\|next=([^|]+)\\}\\}', raw.splitlines()[0])
    assert nav, 'navigation'
    signatures = [dict(line=i['line'], symbol=i['symbol'], section=i['section'], arguments=None, returns=None, declaration=None, payload=None, status='UNPROVEN', limit='Literal identity only; no inferred signature, token list, event payload, optionality, default, alias or security contract.') for i in inventory]
    contracts = [dict(i, kind='identity-unspecified', limit='Reinstated identity/event only; no invocation or behavior declaration.') for i in inventory] + [dict(p, kind='prose-limit') for p in prose] + [dict(l, kind='unexpanded-link', status='UNPROVEN') for l in links]
    assert extracted['result'] == 'existing-failure', 'historical default failure changed'
    extracted_rows = []
    ledger = dict(schema='patch-source-accounting/v1', patch='1.13.5', base_revision=BASE, audit_history='classic-era', source=pin, source_client_literal='WoW Classic', client_literal_scope='reference title, not expanded linked contents or configured native identity', source_toc=int(re.search('<code>(\\d+)</code>', raw)[1]), navigation=dict(zip(('patch', 'prev', 'next'), nav.groups())), source_rows=rows, extracted_rows=extracted_rows, default_extract=extracted, inventory=inventory, signatures=signatures, defaults=[], headers=headers, inventory_headers=counts, captions=captions, prose=prose, links=links, templates=templates, references=references, contracts=contracts, examples=[], configured_profiles=profiles(), successors=successors(), queued_successor=read_json('queued-successor.json'), later_registers=[], model_review=read_json('model-review.json'), measurements=dict(runtime=0, model=0, native=0), derivation=dict(generator_flags=[], extractor_flags=[], generator_default_header_limit='Default generator omits numeric toggle headings; page-local ledger retains and compares both counts. Shared tools unchanged.'))
    ledger['totals'] = dict(physical_lines=len(raw.splitlines()), **{field: len(ledger[field]) for field in OCCURRENCE_FIELDS}, source_statuses=dict(Counter((r['status'] for r in rows))), inventory_by_section=dict(Counter((i['section'] for i in inventory))), contract_statuses=dict(Counter((c['status'] for c in contracts))), count_conflicts=sum((h['conflict'] for h in counts)), declared_signatures=sum((s['declaration'] is not None for s in signatures)), defaults=len(ledger['defaults']), examples=len(ledger['examples']), registry_pages=len(read_json('frozen-registry.json')['pages']), omission_controls=sum((len(ledger[field]) for field in OCCURRENCE_FIELDS)))
    return ledger

def validate_ledger(ledger):
    assert ledger == build(), 'serialized literal ledger'
    return ledger['totals']

def check_seals():
    seals = read_json('seals.json')
    for name, expected in seals.items():
        assert digest((E / name).read_bytes()) == expected, f'seal: {name}'
    return len(seals)

def check_defaults():
    raw, pin = validate_source()
    expected = json.dumps(default_register(raw, pin), indent=2, ensure_ascii=False) + '\n'
    assert (E / 'default-register.json').read_bytes() == expected.encode(), 'default generator bytes'
    assert not (E / 'default-extract.txt').exists(), 'default failed; no extracted output'
    assert read_json('default-extract-error.json') == default_extract(raw), 'default extractor failure equality'
if __name__ == '__main__':
    if sys.argv[1:] == ['default-extract']:
        raw, pin = validate_source()
        print(load_tool('extract_patch_non_inventory.py').extract_text(raw), end='')
    elif sys.argv[1:] == ['capture']:
        assert not (E / 'ledger.json').exists(), 'refuse ledger overwrite'
        raw, pin = validate_source()
        (E / 'default-extract-error.json').write_text(json.dumps(default_extract(raw), indent=2, ensure_ascii=False) + '\n')
        (E / 'ledger.json').write_text(json.dumps(build(), indent=2, ensure_ascii=False) + '\n')
        check_defaults()
    else:
        seals = check_seals()
        check_defaults()
        print(json.dumps(dict(seals=seals, totals=validate_ledger(read_json('ledger.json'))), indent=2))
