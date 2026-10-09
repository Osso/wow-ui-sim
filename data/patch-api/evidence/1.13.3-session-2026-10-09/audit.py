"""Frozen Patch 1.13.3 SOURCE accounting; no runtime/native or linked credit."""
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
OCCURRENCE_FIELDS = ('source_rows', 'extracted_rows', 'inventory', 'default_inventory', 'default_omissions', 'signatures', 'headers', 'inventory_headers', 'captions', 'prose', 'links', 'templates', 'references', 'contracts', 'configured_profiles', 'successors', 'model_review')

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
    page = json.loads(response)['query']['pages']['455918']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (455918, 'Patch 1.13.3/API changes', 6472111, '2025-09-13T22:49:36Z'), 'identity'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '5aac9c7ecacf17d3d3f2961045252ac27b91884a4a2abde60f817ee2f099e85b', 'raw hash'
    assert digest(response) == pin['response_sha256'] == 'cfb09f0cb2d6daf49671b3aa4f1e400eaff90a7ba012dc55ec41191a5660f107', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 2568, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next((p for p in manifest['pages'] if p['version'] == '1.13.3')), 'manifest pin'
    assert digest((E / 'frozen-registry.json').read_bytes()) == manifest['registry_sha256'], 'registry hash'
    pages = read_json('frozen-registry.json')['pages']
    assert len(pages) == 101 and pages[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next((p for p in pages if p['version'] == '1.13.3'))
    assert all((pin[k] == v for k, v in registered.items())), 'registry identity'
    return (raw.decode(), pin)

def default_register(raw, pin):
    generator = load_tool('gen_patch_wikitext_register.py')
    buckets = generator.split_sections(raw)
    entries, headers = ([], [])
    for section in generator.SECTIONS.values():
        rows, counts = generator.parse_section(section, buckets.get(section, []))
        entries.extend(rows)
        headers.extend(counts)
    return dict(schema='patch-api-wikitext-register/v1', patch='1.13.3', source=dict(path='source.wikitext', revid=pin['revid'], sha256=pin['wikitext_sha256']), header_counts=headers, entries=entries)

def default_extract(raw):
    try:
        text = load_tool('extract_patch_non_inventory.py').extract_text(raw)
    except ValueError as error:
        return dict(result='existing-failure', text=None, error_type='ValueError', error=str(error), flags=[])
    return dict(result='success', text=text, error_type=None, error=None, flags=[])

def profiles():
    directory = E / 'state-inputs'
    rust = (directory / 'src/client_profile.rs').read_text()
    interface = int(re.search('ClientProfile::Era \\| ClientProfile::Anniversary => (\\d+)', rust)[1])
    features = tomllib.loads((directory / 'Cargo.toml').read_text())['features']
    return [dict(feature=f'client-{name}', configured_interface=interface, feature_dependencies=features[f'client-{name}'], manifest_sha256=digest((directory / f'data/blizzard-ui-files/{name}.txt').read_bytes()), native_correspondence='UNPROVEN') for name in ('era', 'anniversary')]

def check_successor(item):
    pin = item['pin']
    assert pin == next((p for p in read_json('frozen-manifest.json')['pages'] if p['version'] == item['patch'])), 'successor pin'
    directory = E / 'successors' / item['patch']
    raw, response = ((directory / 'source.wikitext').read_bytes(), (directory / 'response.json').read_bytes())
    assert digest(raw) == pin['wikitext_sha256'] and digest(response) == pin['response_sha256'], 'successor hashes'
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (pin['pageid'], pin['title'], pin['revid'], pin['timestamp']), 'successor identity'
    assert revision['slots']['main']['*'].encode() == raw, 'successor content'
    assert not item['applied'] and (not item['native_proof']), 'successor credit'

def successors():
    status = read_json('successors/status.json')
    versions = ['1.13.5', '1.13.6', '1.13.7'] + [f'1.14.{n}' for n in range(5)] + [f'1.15.{n}' for n in range(10)]
    assert [s['patch'] for s in status] == versions, 'successor order'
    for item in status:
        assert item['state'] == 'canonical-input-not-applied', 'integrated input distinct from applied register'
        check_successor(item)
    return status

def queued_successor():
    item = read_json('queued-successor.json')
    assert item['patch'] == '1.13.4' and item['state'] == 'concurrently-active-not-integrated' and (item['register'] is None), 'queued successor'
    assert item['integration_order'] == ['1.13.4', '1.13.3'], 'main integration order'
    check_successor(item)
    return item

def reference_limits(line, literal):
    links = []
    for match in re.finditer('\\[\\[([^\\]|]+)(?:\\|([^\\]]+))?\\]\\]|https://[^\\s|}<]+', literal):
        links.append(dict(line=line, literal=match[0], target=match[1] if match[1] is not None else match[0], label=match[2], kind='wiki' if match[1] is not None else 'external', expanded=False, limit='Literal target only; no linked contents, signatures, defaults or behavior credit.'))
    templates = [dict(line=line, literal=m[0], name=m[1].split('|')[0], expanded=False, limit='Literal template occurrence only; no transclusion credit.') for m in re.finditer('\\{\\{([^{}]+)\\}\\}', literal)]
    references = [dict(line=line, literal=m[0], expanded=False, limit='Exact citation author/date/title/url retained; target content unexpanded.') for m in re.finditer('<ref>.*?</ref>', literal)]
    return (links, templates, references)

def prose_limit(line, literal):
    common = dict(line=line, literal=literal, status='UNPROVEN', arguments=None, returns=None, security=None, native_equivalence=None)
    if line == 10:
        return dict(common, claim='Added Keyring feature', capacity=None, slot_mapping=None, limit='No capacity, default, item eligibility, lifecycle or query signature specified.')
    if line == 11:
        return dict(common, subject='SendChatMessage', requirement='hardware events', failure_result=None, event_lifetime=None, limit='Hardware-event restriction stated; no failure shape, event lifetime, bypass, signature or defaults inferred from empty parentheses.')
    if line == 12:
        return dict(common, subject='SendAddonMessage', removed_chat_types=['CHANNEL'], added_chat_types=['SAY', 'YELL'], failure_result=None, routing=None, limit='Exact type transition; no failure return, routing, delivery, optionality, defaults or full signature. Citation unexpanded.')
    return dict(common, subjects=['UnitHealth', 'UnitHealthMax'], scope='NPCs; hotfixed; values again instead of percentages', default=None, limit='Values-not-percentages statement only; no token/argument/default, unknown-unit, secrecy or full native parity contract. Citation unexpanded.')

def inventory_rows(raw):
    generator = load_tool('gen_patch_wikitext_register.py')
    entries, section, direction, columns = ([], None, None, 0)
    for n, literal in enumerate(raw.splitlines(), 1):
        heading = re.fullmatch('(=+)\\s*([^=]+?)\\s*\\1', literal)
        if heading:
            if len(heading[1]) == 2:
                section = generator.SECTIONS.get(heading[2])
                direction, columns = (None, 0)
            elif heading[2] == 'Added':
                direction = 'added'
        elif literal.startswith('| valign="top"'):
            columns += 1
            direction = 'added' if columns == 1 else 'removed'
        elif literal.startswith(': {{api|'):
            assert section is not None and direction is not None, 'inventory context'
            entry = generator.make_entry(section, direction, n, literal)
            entries.append(dict(entry, line=n, literal=literal, status='UNPROVEN', capabilities=[]))
    return entries

def build():
    raw, pin = validate_source()
    register = default_register(raw, pin)
    inventory = inventory_rows(raw)
    byline = {i['line']: i for i in inventory}
    rows, headers, counts, captions, prose, links, templates, references, signatures = ([], [], [], [], [], [], [], [], [])
    for n, literal in enumerate(raw.splitlines(), 1):
        if not literal.strip():
            continue
        heading = re.fullmatch('(=+)\\s*([^=]+?)\\s*\\1', literal)
        if heading:
            headers.append(dict(line=n, literal=literal, level=len(heading[1]), label=heading[2]))
        count = re.search('\\| (\\d+) (new|removed) functions$', literal)
        if count:
            direction = 'added' if count[2] == 'new' else 'removed'
            observed = sum((i['section'] == 'global-api' and i['direction'] == direction for i in inventory))
            counts.append(dict(line=n, literal=literal, section='global-api', direction=direction, count=int(count[1]), observed=observed, conflict=int(count[1]) != observed))
        if n in (4, 17):
            match = re.search('(\\d+\\.\\d+\\.\\d+) \\((\\d+)\\) (?:to|&rarr;) (\\d+\\.\\d+\\.\\d+) \\((\\d+)\\)', literal)
            captions.append(dict(line=n, literal=literal, before_patch=match[1], before_build=int(match[2]), after_patch=match[3], after_build=int(match[4])))
        if n in (10, 11, 12, 13):
            prose.append(prose_limit(n, literal))
            for m in re.finditer('\\{\\{api\\|([^{}]+)\\}\\}(\\(\\))', literal):
                signatures.append(dict(line=n, symbol=m[1], literal=m[0], fragment=m[2], kind='prose-empty-parentheses', arguments=None, returns=None, declaration=None, status='UNPROVEN', limit='Empty parentheses are citation/invocation notation, not a declared zero-argument signature.'))
        new_links, new_templates, new_refs = reference_limits(n, literal)
        links.extend(new_links)
        templates.extend(new_templates)
        references.extend(new_refs)
        substantive = n in byline or n in (5, 6, 10, 11, 12, 13)
        rows.append(dict(id=f'source-{n:03d}', line=n, literal=literal, status='UNPROVEN' if substantive else 'metadata-only', capabilities=[]))
    signatures = [dict(line=i['line'], symbol=i['symbol'], literal=i['literal'], section=i['section'], kind='inventory-unspecified', fragment=None, arguments=None, returns=None, declaration=None, status='UNPROVEN', limit='Identity only; no inferred arguments, returns, event payload, CVar default, alias, security or invocation contract.') for i in inventory] + signatures
    contracts = [dict(i, kind='inventory-identity') for i in inventory] + [dict(i, kind='prose-limit') for i in prose] + [dict(i, kind='unexpanded-link', status='UNPROVEN') for i in links]
    nav = re.fullmatch('\\{\\{apichanges\\|([^|]+)\\|prev=([^|]+)\\|next=([^|]+)\\}\\}', raw.splitlines()[0])
    assert nav, 'navigation'
    extracted = default_extract(raw)
    assert extracted['result'] == 'existing-failure', 'default extractor outcome changed'
    default_lines = {i['wikitext_line'] for i in register['entries']}
    ledger = dict(schema='patch-source-accounting/v1', patch='1.13.3', base_revision=BASE, audit_history='classic-era', source=pin, source_client_literal='Classic and WoW Classic', client_literal_scope='citation titles; not a configured native identity or linked-content expansion', source_toc=int(re.search('<code>(\\d+)</code>', raw)[1]), navigation=dict(zip(('patch', 'prev', 'next'), nav.groups())), source_rows=rows, extracted_rows=[], default_extract=extracted, inventory=inventory, default_inventory=register['entries'], default_omissions=[i for i in inventory if i['line'] not in default_lines], signatures=signatures, defaults=[], headers=headers, inventory_headers=counts, captions=captions, prose=prose, links=links, templates=templates, references=references, contracts=contracts, examples=[], configured_profiles=profiles(), successors=successors(), queued_successor=queued_successor(), later_registers=[], model_review=read_json('model-review.json'), measurements=dict(runtime=0, model=0, native=0), derivation=dict(generator_flags=[], extractor_flags=[], literal_inventory='Own opt-in SOURCE accounting; all colon-api occurrences; default tools unchanged.', generator_default_limit='22 globals only; drops event/four CVars and both numeric headers. Preserve literal boundaries and default bytes separately.'))
    ledger['totals'] = dict(physical_lines=len(raw.splitlines()), **{field: len(ledger[field]) for field in OCCURRENCE_FIELDS}, source_statuses=dict(Counter((r['status'] for r in rows))), inventory_by_section=dict(Counter((i['section'] for i in inventory))), inventory_by_direction=dict(Counter((i['direction'] for i in inventory))), contract_statuses=dict(Counter((c['status'] for c in contracts))), count_conflicts=sum((h['conflict'] for h in counts)), declared_signatures=sum((s['declaration'] is not None for s in signatures)), defaults=len(ledger['defaults']), examples=len(ledger['examples']), registry_pages=len(read_json('frozen-registry.json')['pages']), omission_controls=sum((len(ledger[field]) for field in OCCURRENCE_FIELDS)))
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
    assert read_json('default-extract-outcome.json') == default_extract(raw), 'default extractor outcome'
    assert not (E / 'default-extract.txt').exists(), 'failed extractor has no output'
if __name__ == '__main__':
    if sys.argv[1:] == ['default-extract']:
        raw, _ = validate_source()
        print(load_tool('extract_patch_non_inventory.py').extract_text(raw), end='')
    elif sys.argv[1:] == ['capture']:
        assert not (E / 'ledger.json').exists(), 'refuse ledger overwrite'
        check_defaults()
        (E / 'ledger.json').write_text(json.dumps(build(), indent=2, ensure_ascii=False) + '\n')
    else:
        seals = check_seals()
        check_defaults()
        print(json.dumps(dict(seals=seals, totals=validate_ledger(read_json('ledger.json'))), indent=2))
