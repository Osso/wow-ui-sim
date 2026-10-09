"""Frozen 1.13.4 lossless SOURCE accounting; observations/successors separate."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tomllib
E = Path(__file__).resolve().parent
BASE = '3de87465828db7cc7f6f900d4b63e24f1b399825'
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
    page = json.loads(response)['query']['pages']['333398']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (333398, 'Patch 1.13.4/API changes', 3216451, '2021-05-06T13:21:20Z'), 'identity'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == 'a8cbcc1f62bb90e33a40334dece4f943b3f44648c34d50fbbb0a55160b8e00f5', 'raw hash'
    assert digest(response) == pin['response_sha256'] == 'a49c62f6c4663ab12a8eac66083676c1284ab5f5ac870023e542b892924ffc55', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 2146, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next((p for p in manifest['pages'] if p['version'] == '1.13.4')), 'manifest pin'
    assert digest((E / 'frozen-registry.json').read_bytes()) == manifest['registry_sha256'], 'registry hash'
    registered = next((p for p in read_json('frozen-registry.json')['pages'] if p['version'] == '1.13.4'))
    assert all((pin[k] == v for k, v in registered.items())), 'registry membership'
    return (raw.decode(), pin)

def default_register(raw, pin, patch='1.13.4'):
    generator = load_tool('gen_patch_wikitext_register.py')
    buckets = generator.split_sections(raw)
    entries, headers = ([], [])
    for section in generator.SECTIONS.values():
        rows, counts = generator.parse_section(section, buckets.get(section, []))
        entries.extend(rows)
        headers.extend(counts)
    return dict(schema='patch-api-wikitext-register/v1', patch=patch, source=dict(path='source.wikitext', revid=pin['revid'], sha256=pin['wikitext_sha256']), header_counts=headers, entries=entries)

def default_extract(raw):
    text = load_tool('extract_patch_non_inventory.py').extract_text(raw)
    return dict(result='success', text=text, flags=[])

def profiles():
    directory = E / 'state-inputs'
    rust = (directory / 'src/client_profile.rs').read_text()
    interface = int(re.search('ClientProfile::Era \\| ClientProfile::Anniversary => (\\d+)', rust)[1])
    features = tomllib.loads((directory / 'Cargo.toml').read_text())['features']
    return [dict(feature=f'client-{name}', configured_interface=interface, feature_dependencies=features[f'client-{name}'], manifest_sha256=digest((directory / f'data/blizzard-ui-files/{name}.txt').read_bytes()), native_correspondence='UNPROVEN') for name in ('era', 'anniversary')]

def successors():
    status = read_json('successors/status.json')
    versions = ['1.13.5', '1.13.6', '1.13.7', '1.14.0', '1.14.1', '1.14.2', '1.14.3', '1.14.4'] + [f'1.15.{n}' for n in range(10)]
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

def build():
    raw, pin = validate_source()
    register = default_register(raw, pin)
    byline = {i['wikitext_line']: i for i in register['entries']}
    rows, headers, counts, inventory, links, templates, prose, captions = ([], [], [], [], [], [], [], [])
    context = None
    for n, literal in enumerate(raw.splitlines(), 1):
        if not literal.strip():
            continue
        heading = re.fullmatch('==\\s*([^=]+?)\\s*==', literal)
        if heading:
            context = heading[1]
            headers.append(dict(line=n, literal=literal, label=context))
        count = re.search('\\| (\\d+) new (functions|events|CVar)$', literal)
        if count:
            section = {'Global API': 'global-api', 'Events': 'events', 'CVars': 'cvars'}[context]
            observed = sum((i['section'] == section and i['direction'] == 'added' for i in register['entries']))
            counts.append(dict(line=n, literal=literal, section=section, direction='added', count=int(count[1]), observed=observed, conflict=int(count[1]) != observed))
        if n == 4:
            match = re.fullmatch('(\\S+) \\((\\d+)\\) to (\\S+) \\((\\d+)\\)', literal)
            captions.append(dict(line=n, literal=literal, before_patch=match[1], before_build=int(match[2]), after_patch=match[3], after_build=int(match[4])))
        entry = byline.get(n)
        if entry:
            inventory.append(dict(entry, line=n, literal=literal, status='UNPROVEN', capabilities=[]))
        if n in (10, 11):
            claim = 'Totem API reinstated' if n == 10 else 'WoW Token support in China'
            limit = 'No slot count, arguments, returns, durations, targeting transition, security or invocation contract.' if n == 10 else 'China context only; no commerce policy, regional activation, item/price/payment/default/event or member declaration.'
            prose.append(dict(line=n, literal=literal, claim=claim, arguments=None, returns=None, security=None, state_transition=None, status='UNPROVEN', limit=limit))
        for match in re.finditer('\\[\\[([^\\]|]+)(?:\\|([^\\]]+))?\\]\\]|https://[^\\s|}<]+', literal):
            links.append(dict(line=n, literal=match[0], target=match[1] if match[1] is not None else match[0], label=match[2], kind='wiki' if match[1] is not None else 'external', expanded=False, limit='Target unexpanded; no linked signature/default/prose/member/native credit.'))
        templates.extend((dict(line=n, literal=m[0], name=m[1].split('|')[0], expanded=False, limit='Literal boundary only; no transclusion.') for m in re.finditer('\\{\\{([^{}]+)\\}\\}', literal)))
        rows.append(dict(id=f'source-{n:03d}', line=n, literal=literal, status='UNPROVEN' if entry is not None or n in (5, 6, 10, 11) else 'metadata-only', capabilities=[]))
    nav = re.fullmatch('\\{\\{apichanges\\|([^|]+)\\|prev=([^|]+)\\|next=([^|]+)\\}\\}', raw.splitlines()[0])
    assert nav, 'navigation'
    signatures = [dict(line=i['line'], symbol=i['symbol'], section=i['section'], arguments=None, returns=None, declaration=None, payload=None, status='UNPROVEN', limit='Literal name only; no inferred invocation, optionality, default, alias or security.') for i in inventory]
    contracts = [dict(i, kind='identity-unspecified', limit='Name/direction only; no native or semantic model proof.') for i in inventory] + [dict(p, kind='prose-limit') for p in prose] + [dict(l, kind='unexpanded-link', status='UNPROVEN') for l in links]
    extracted = default_extract(raw)
    extracted_rows = [dict(line=n, literal=s) for n, s in enumerate(extracted['text'].splitlines(), 1) if s.strip()]
    ledger = dict(schema='patch-source-accounting/v1', patch='1.13.4', base_revision=BASE, audit_history='classic-era', source=pin, source_client_literal=None, source_toc=int(re.search('<code>(\\d+)</code>', raw)[1]), navigation=dict(zip(('patch', 'prev', 'next'), nav.groups())), source_rows=rows, extracted_rows=extracted_rows, default_extract=extracted, inventory=inventory, signatures=signatures, defaults=[], headers=headers, inventory_headers=counts, captions=captions, prose=prose, links=links, templates=templates, references=[], contracts=contracts, examples=[], configured_profiles=profiles(), successors=successors(), later_registers=[], model_review=read_json('model-review.json'), measurements=dict(runtime=0, model=0, native=0), derivation=dict(generator_flags=[], extractor_flags=[], generator_default_header_limit='Numeric toggle headings omitted by unchanged default tool; all three retained here.'))
    ledger['totals'] = dict(physical_lines=len(raw.splitlines()), **{field: len(ledger[field]) for field in OCCURRENCE_FIELDS}, source_statuses=dict(Counter((r['status'] for r in rows))), inventory_by_section=dict(Counter((i['section'] for i in inventory))), contract_statuses=dict(Counter((c['status'] for c in contracts))), count_conflicts=sum((h['conflict'] for h in counts)), declared_signatures=0, defaults=len(ledger['defaults']), examples=len(ledger['examples']), registry_pages=len(read_json('frozen-registry.json')['pages']), omission_controls=sum((len(ledger[field]) for field in OCCURRENCE_FIELDS)))
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
    assert (E / 'default-register.json').read_bytes() == (json.dumps(default_register(raw, pin), indent=2, ensure_ascii=False) + '\n').encode(), 'default generator bytes'
    assert (E / 'default-extract.txt').read_text() == default_extract(raw)['text'], 'default extract bytes'
if __name__ == '__main__':
    if sys.argv[1:] == ['default-extract']:
        raw, pin = validate_source()
        print(default_extract(raw)['text'], end='')
    elif sys.argv[1:] == ['capture']:
        assert not (E / 'ledger.json').exists(), 'refuse ledger overwrite'
        raw, pin = validate_source()
        (E / 'default-register.json').write_text(json.dumps(default_register(raw, pin), indent=2, ensure_ascii=False) + '\n')
        (E / 'default-extract.txt').write_text(default_extract(raw)['text'])
        (E / 'ledger.json').write_text(json.dumps(build(), indent=2, ensure_ascii=False) + '\n')
        check_defaults()
    else:
        seals = check_seals()
        check_defaults()
        print(json.dumps(dict(seals=seals, totals=validate_ledger(read_json('ledger.json'))), indent=2))
