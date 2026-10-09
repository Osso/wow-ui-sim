"""Frozen Patch 1.13.2 literal SOURCE accounting; no expanded/runtime/native credit."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tomllib
E = Path(__file__).resolve().parent
BASE = '9252c6cc93c087518f25e64987e095ed76312a51'
OCCURRENCE_FIELDS = ('source_rows', 'extracted_rows', 'inventory', 'default_inventory', 'default_omissions', 'signatures', 'defaults', 'headers', 'inventory_headers', 'captions', 'prose', 'links', 'templates', 'references', 'contracts', 'examples', 'configured_profiles', 'successors', 'model_review')
BEHAVIOR_LINES = (14, 15, 16, 19, 20, 21, 24, 25, 28, 29, 30, 33, 34)
PROSE_LIMITS = {2: 'Literal Classic 1.13.2/8.1.0 and demo1.13.0/7.3.5 provenance, not imported APIs or semantic supersession.', 14: 'focus UnitId removal only; token failure shape and other token behavior unspecified.', 15: 'Quest/Spell links become plain chat text and server rejects manual posting; routing, parsing, return/failure/security shapes unspecified.', 16: '50-yard player-centered combat-log restriction, Build32600 Nov20 2019; range metric, boundary equality, event payloads and failure shape unspecified. Citation unexpanded.', 19: 'Crafting UI re-added for Enchanting and Beast Training; recipes, reagents, costs, lifecycle, queries and signatures unspecified.', 20: 'Deprecated APIs/addon removed; old C_FriendList aliases emphasized but no alias inventory or signature declared. Do not infer linked/current aliases.', 21: 'SendWho requires hardware event; event lifetime and rejection/security/return shapes unspecified. Empty parentheses do not declare zero arguments.', 24: 'UnitCastingInfo downgraded to CastingInfo, described by player invocation; tuple/optional arguments/defaults and other unit behavior unspecified.', 25: 'Idem UnitChannelInfo and ChannelInfo; analogy retained literally, not an invented signature or implementation alias.', 28: 'nameplateMaxDistance effectively locked to20 yards; not a stated CVar default or generic setter/getter failure shape.', 29: 'chatClassColorOverride default1 never color versus prior0 always color; only stated default transition, no setter/event/full signature contract.', 30: 'Classic default UI scale0.9 versus retail0.64, minimumAutomaticUiScale referenced; not a full scaling/CVar algorithm or foreign model credit.', 33: 'Combat-log Spell IDs always0; patch2.4 provenance is not TBC payload/signature import. Payload positions unspecified.', 34: 'UNIT_SPELLCAST_* only player; wildcard preserved without expanding event set or inventing payloads/order.'}

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
    page = json.loads(response)['query']['pages']['115161']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (115161, 'Patch 1.13.2/API changes', 6471351, '2025-09-13T09:41:57Z'), 'identity'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == 'ea1badd4412eb6d50257ae97b77d4155c96c5043469c099957a8d85744332526', 'raw hash'
    assert digest(response) == pin['response_sha256'] == '81bb1f86038a137f16c0c4b80d1a89ac5e5b8a3eea234500ed3b5ca423eaf4d3', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 171914, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next((p for p in manifest['pages'] if p['version'] == '1.13.2')), 'manifest pin'
    assert digest((E / 'frozen-registry.json').read_bytes()) == manifest['registry_sha256'], 'registry hash'
    pages = read_json('frozen-registry.json')['pages']
    assert len(pages) == 101 and pages[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next((p for p in pages if p['version'] == '1.13.2'))
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
    return dict(schema='patch-api-wikitext-register/v1', patch='1.13.2', source=dict(path='source.wikitext', revid=pin['revid'], sha256=pin['wikitext_sha256']), header_counts=headers, entries=entries)

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
    return [dict(feature=f'client-{n}', configured_interface=interface, feature_dependencies=features[f'client-{n}'], manifest_sha256=digest((directory / f'data/blizzard-ui-files/{n}.txt').read_bytes()), native_correspondence='UNPROVEN') for n in ('era', 'anniversary')]

def successors():
    items = read_json('successors/status.json')
    versions = [f'1.13.{n}' for n in range(3, 8)] + [f'1.14.{n}' for n in range(5)] + [f'1.15.{n}' for n in range(10)]
    assert [i['patch'] for i in items] == versions, 'same-Era order'
    manifest = read_json('frozen-manifest.json')
    for item in items:
        assert item['state'] == 'integrated-input-not-applied' and (not item['applied']) and (not item['native_proof']), 'successor credit'
        pin = item['pin']
        assert pin == next((p for p in manifest['pages'] if p['version'] == item['patch'])), 'successor pin'
        directory = E / 'successors' / item['patch']
        raw, response = ((directory / 'source.wikitext').read_bytes(), (directory / 'response.json').read_bytes())
        assert digest(raw) == pin['wikitext_sha256'] and digest(response) == pin['response_sha256'], 'successor hash'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        rev, = page['revisions']
        assert (page['title'], rev['revid'], rev['timestamp']) == (pin['title'], pin['revid'], pin['timestamp']), 'successor identity'
        assert rev['slots']['main']['*'].encode() == raw, 'successor content'
    return items

def references(line, literal):
    links = []
    pattern = '\\[\\[([^\\]|]+)(?:\\|([^\\]]+))?\\]\\]|\\[(https?://[^\\s\\]]+)(?:\\s+([^\\]]+))?\\]|(?<![\\w\\[])https?://[^\\s|}<\\]]+'
    for m in re.finditer(pattern, literal):
        target = m[1] or m[3] or m[0]
        links.append(dict(line=line, literal=m[0], target=target, label=m[2] or m[4], kind='wiki' if m[1] else 'external', expanded=False, limit='Literal target only; no linked signatures/defaults/behavior imported.'))
    templates = [dict(line=line, literal=m[0], name=m[1].split('|')[0], expanded=False, limit='Literal occurrence only; no transcluded contents.') for m in re.finditer('\\{\\{([^{}]+)\\}\\}', literal)]
    refs = [dict(line=line, literal=m[0], expanded=False, limit='Citation fields retained; reference contents unexpanded.') for m in re.finditer('<ref>.*?</ref>', literal)]
    return (links, templates, refs)

def inventory_rows(raw):
    generator = load_tool('gen_patch_wikitext_register.py')
    rows, section, columns = ([], None, 0)
    for n, literal in enumerate(raw.splitlines(), 1):
        heading = re.fullmatch('(=+)\\s*([^=]+?)\\s*\\1', literal)
        if heading and len(heading[1]) == 2:
            section = generator.SECTIONS.get(heading[2])
            columns = 0
        elif literal.startswith('| valign="top"'):
            columns += 1
        elif section and literal.startswith(': '):
            assert columns in (1, 2), 'inventory context'
            entry = generator.make_entry(section, 'added' if columns == 1 else 'removed', n, literal)
            wiki = re.search('\\[\\[([^|\\]]+)\\|([^\\]]+)\\]\\]', literal)
            kind = 'widget-type' if literal.startswith(': [[UIOBJECT ') else 'widget-method' if section == 'widgets' else section
            gap = {'global-api': 'Identity-only API occurrence: historical arguments/returns/security/lifecycle unspecified; current registration or stub is not parity.', 'widgets': 'Historical widget type/method availability and behavior untested; link target/display preserved separately, no inferred aliases.', 'events': 'Historical production/payload/order untested; permissive Era registration is not event behavior.', 'cvars': 'Historical inventory-only CVar default/effects unprovided except separate prose limits; current store defaults are not source defaults.'}[section]
            rows.append(dict(entry, line=n, literal=literal, target=wiki[1] if wiki else None, label=wiki[2] if wiki else None, kind=kind, status='UNPROVEN', capabilities=[], gap=gap))
    return rows

def count_headers(raw, inventory):
    result, section, direction = ([], None, None)
    for n, literal in enumerate(raw.splitlines(), 1):
        h = re.fullmatch('==([^=]+)==', literal)
        if h:
            section = load_tool('gen_patch_wikitext_register.py').SECTIONS.get(h[1])
        if not literal.startswith('! '):
            continue
        m = re.search('\\| (\\d+) (new|removed) (\\w+)', literal)
        assert m, 'numeric inventory header'
        direction = 'added' if m[2] == 'new' else 'removed'
        terms = [(int(m[1]), m[3])] + [(int(a), b) for a, b in re.findall(', (\\d+) (\\w+)', literal)]
        for count, kind in terms:
            observed = sum((i['section'] == section and i['direction'] == direction and (section != 'widgets' or i['kind'] == ('widget-type' if kind == 'widgets' else 'widget-method')) for i in inventory))
            result.append(dict(line=n, literal=literal, section=section, direction=direction, kind=kind, count=count, observed=observed, conflict=count != observed))
    return result

def build():
    raw, pin = validate_source()
    register = default_register(raw, pin)
    inventory = inventory_rows(raw)
    byline = {i['line']: i for i in inventory}
    rows, headers, captions, prose, links, templates, refs, signatures, examples = ([], [], [], [], [], [], [], [], [])
    for n, literal in enumerate(raw.splitlines(), 1):
        if not literal.strip():
            continue
        heading = re.fullmatch('(=+)\\s*([^=]+?)\\s*\\1', literal)
        if heading:
            headers.append(dict(line=n, literal=literal, level=len(heading[1]), label=heading[2]))
        if literal.startswith('|+'):
            m = re.search('(\\d+\\.\\d+\\.\\d+) \\((\\d+)\\) &rarr; (\\d+\\.\\d+\\.\\d+) \\((\\d+)\\)', literal)
            assert m, 'caption'
            captions.append(dict(line=n, literal=literal, before_patch=m[1], before_build=int(m[2]), after_patch=m[3], after_build=int(m[4])))
        is_prose = n not in byline and (not heading) and (not literal.startswith(('{{', '{|', '|', '!', '</div>')))
        if is_prose:
            prose.append(dict(line=n, literal=literal, role='behavior' if n in BEHAVIOR_LINES else 'provenance-or-reference', status='UNPROVEN', arguments=None, returns=None, security=None, native_equivalence=None, limit=PROSE_LIMITS.get(n, 'Literal prose/reference boundary; linked pages/diffs and current/native contracts unexpanded.')))
            for m in re.finditer('\\{\\{api\\|([^{}]+)\\}\\}(\\(\\))', literal):
                signatures.append(dict(line=n, symbol=m[1], literal=m[0], fragment=m[2], kind='prose-empty-parentheses', arguments=None, returns=None, declaration=None, status='UNPROVEN', limit='Invocation/citation notation, not declared zero-argument signature.'))
            for m in re.finditer('<code>([^<]*\\([^<]*\\))</code>', literal):
                examples.append(dict(line=n, literal=m[0], fragment=m[1], status='UNPROVEN', limit='Literal invocation only; no full signature/alias/default inferred.'))
                signatures.append(dict(line=n, symbol=m[1].split('(')[0], literal=m[0], fragment=m[1], kind='prose-invocation', arguments=None, returns=None, declaration=None, status='UNPROVEN', limit='Player example retained; return tuple and full historical signature unspecified.'))
        a, b, c = references(n, literal)
        links.extend(a)
        templates.extend(b)
        refs.extend(c)
        substantive = n in byline or is_prose
        rows.append(dict(id=f'source-{n:04d}', line=n, literal=literal, status='UNPROVEN' if substantive else 'metadata-only', capabilities=[]))
    signatures = [dict(line=i['line'], symbol=i['symbol'], literal=i['literal'], section=i['section'], kind='inventory-unspecified', fragment=None, arguments=None, returns=None, declaration=None, status='UNPROVEN', limit='Identity only; no full signature, payload, default, security or alias declaration.') for i in inventory] + signatures
    defaults = [dict(line=29, symbol='chatClassColorOverride', default_literal='1', prior_literal='0', scope='CVar default', status='UNPROVEN'), dict(line=30, symbol='minimumAutomaticUiScale', default_literal='0.9', prior_literal='0.64', scope='default UI scale statement; full CVar contract unspecified', status='UNPROVEN')]
    contracts = [dict(i, contract_kind='inventory-identity') for i in inventory] + [dict(i, contract_kind='prose-limit') for i in prose] + [dict(i, contract_kind='unexpanded-link', status='UNPROVEN') for i in links]
    nav = re.fullmatch('\\{\\{apichanges\\|([^|]+)\\|next=([^|]+)\\}\\}', raw.splitlines()[0])
    assert nav, 'navigation'
    extracted = default_extract(raw)
    assert extracted['result'] == 'existing-failure', 'historical extraction boundary'
    default_lines = {i['wikitext_line'] for i in register['entries']}
    d = dict(schema='patch-source-accounting/v1', patch='1.13.2', base_revision=BASE, audit_history='classic-era', source=pin, source_client_literal='Classic 1.13.2; BlizzCon2018 demo1.13.0; Vanilla API', source_toc=int(re.search('<code>(\\d+)</code>', raw)[1]), navigation=dict(zip(('patch', 'next'), nav.groups())), source_rows=rows, extracted_rows=[], default_extract=extracted, inventory=inventory, default_inventory=register['entries'], default_omissions=[i for i in inventory if i['line'] not in default_lines], signatures=signatures, defaults=defaults, headers=headers, inventory_headers=count_headers(raw, inventory), captions=captions, prose=prose, links=links, templates=templates, references=refs, contracts=contracts, examples=examples, configured_profiles=profiles(), successors=successors(), later_registers=[], integration_order=['1.13.2', '1.12.0'], foreign_history_policy='Retail1.12.0 is a separate parallel audit; no Retail/TBC/Wrath/Forever semantic supersession.', model_review=read_json('model-review.json'), measurements=dict(runtime=0, model=0, native=0), derivation=dict(generator_flags=[], extractor_flags=[], literal_inventory='Own page-local opt-in accounting; shared defaults unchanged.', default_limit='All2758 identities retained; default generator loses nine numeric count terms; extractor fails on ref web.'))
    d['totals'] = dict(physical_lines=len(raw.splitlines()), **{f: len(d[f]) for f in OCCURRENCE_FIELDS}, source_statuses=dict(Counter((i['status'] for i in rows))), inventory_by_section=dict(Counter((i['section'] for i in inventory))), inventory_by_direction=dict(Counter((i['direction'] for i in inventory))), contract_statuses=dict(Counter((i['status'] for i in contracts))), count_conflicts=sum((i['conflict'] for i in d['inventory_headers'])), declared_signatures=sum((i['declaration'] is not None for i in signatures)), registry_pages=len(read_json('frozen-registry.json')['pages']), omission_controls=sum((len(d[f]) for f in OCCURRENCE_FIELDS)))
    return d

def validate_ledger(ledger, expected=None):
    assert ledger == (build() if expected is None else expected), 'serialized literal ledger'
    return ledger['totals']

def check_defaults():
    raw, pin = validate_source()
    expected = json.dumps(default_register(raw, pin), indent=2, ensure_ascii=False) + '\n'
    assert (E / 'default-register.json').read_bytes() == expected.encode(), 'default generator bytes'
    assert read_json('default-extract-outcome.json') == default_extract(raw), 'default extractor outcome'
    assert not (E / 'default-extract.txt').exists(), 'failed extractor produces no text'

def check_seals():
    seals = read_json('seals.json')
    for name, expected in seals.items():
        assert digest((E / name).read_bytes()) == expected, f'seal: {name}'
    return len(seals)
if __name__ == '__main__':
    if sys.argv[1:] == ['default-extract']:
        raw, _ = validate_source()
        print(load_tool('extract_patch_non_inventory.py').extract_text(raw), end='')
    elif sys.argv[1:] == ['capture']:
        assert not (E / 'ledger.json').exists(), 'refuse original ledger overwrite'
        check_defaults()
        (E / 'ledger.json').write_text(json.dumps(build(), indent=2, ensure_ascii=False) + '\n')
    else:
        seals = check_seals()
        check_defaults()
        print(json.dumps(dict(seals=seals, totals=validate_ledger(read_json('ledger.json'))), indent=2))
