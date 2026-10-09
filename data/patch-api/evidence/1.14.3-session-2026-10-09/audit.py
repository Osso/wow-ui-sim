"""Offline occurrence accounting of frozen Era 1.14.3; no runtime policy."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
EVIDENCE = Path(__file__).resolve().parent


def digest(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()


def load_tool(evidence, name):
    path = evidence / 'historical-tools' / name
    spec = importlib.util.spec_from_file_location(name.replace('.', '_'), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def pin_source(evidence):
    original = evidence / 'original'
    raw = (original / 'source.wikitext').read_bytes()
    response = (original / 'response.json').read_bytes()
    pin = json.loads((original / 'pin.json').read_bytes())
    page = json.loads(response)['query']['pages']['480026']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
        480026, 'Patch 1.14.3/API changes', 4615755, '2023-07-10T10:27:33Z'), 'frozen identity'
    assert revision['slots']['main']['*'].encode() == raw, 'response body bytes'
    assert digest(raw) == pin['wikitext_sha256'] == '8af5d8d0d80a74eece469856cdd39ed0652f834669f7e7f759f34774dc15791e', 'raw hash'
    assert len(raw) == pin['wikitext_bytes'] == 23291, 'raw byte count'
    assert digest(response) == pin['response_sha256'] == '2abbe023bd2dd2e7a1ee2c47a8500553e4d1e4eccc33b450e06f2ebc37f5e0d6', 'response hash'
    manifest = json.loads((original / 'manifest.json').read_bytes())
    assert next(p for p in manifest['pages'] if p['version'] == '1.14.3') == pin, 'manifest membership'
    registry_bytes = (original / 'registry.json').read_bytes()
    assert digest(registry_bytes) == manifest['registry_sha256'] == 'e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c', 'registry hash'
    registry = json.loads(registry_bytes)['pages']
    assert len(registry) == 101 and registry[-1]['version'] == '1.0.0', 'registry boundary'
    indexed = {p['version']: p for p in registry}
    assert len(indexed) == 101, 'registry duplicate'
    for p in manifest['pages']:
        assert all(p[key] == value for key, value in indexed[p['version']].items()), 'manifest registry identity'
    return raw, pin


def parse_register(evidence, raw):
    generator = load_tool(evidence, 'gen_patch_wikitext_register.py')
    buckets = generator.split_sections(raw.decode())
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        rows, counts = generator.parse_section(section, buckets.get(section, []))
        entries.extend(rows)
        headers.extend(counts)
    return dict(schema='patch-api-wikitext-register/v1', patch='1.14.3',
                source=dict(path='original/source.wikitext', revid=4615755, sha256=digest(raw)),
                header_counts=headers, entries=entries)


def default_extract(evidence):
    extractor = load_tool(evidence, 'extract_patch_non_inventory.py')
    raw, _ = pin_source(evidence)
    return extractor.extract_text(raw.decode())


def literal_cvar_fields(line):
    fields = {}
    for name, pattern in (
        ('default', r'Default: <code><span class="apitype">([^<]*)</span></code>'),
        ('scope', r'Scope: <span class="apitype">([^<]*)</span>'),
        ('description', r'<br><small>(.*?)</small>')):
        match = re.search(pattern, line)
        if match:
            fields[name] = match[1]
    return fields


def references(number, line):
    patterns = [('wiki-link', r'\[\[[^\]]+\]\]'),
                ('external-link', r'\[https?://[^\]]+\]')]
    matches = sorted((m.start(), kind, m[0]) for kind, pattern in patterns
                     for m in re.finditer(pattern, line))
    return [dict(id=f'reference-{number}-{index}', line=number, literal=value,
                 kind=kind, expanded=False, status='UNPROVEN')
            for index, (_, kind, value) in enumerate(matches, 1)]


def contract_limit(section):
    limits = {
        'events': 'Identity only; producer, payload, order and native event firing unspecified.',
        'widgets': 'Method identity only; arguments, returns, attribute transition and handler suppression contract unspecified.',
        'cvars': 'Literal default/scope/description retained; persistence, setting transitions, rendering/input/voice effects and native defaults UNPROVEN.',
        'global-api': 'Identity only; arguments, returns, coercion, state transitions and security unspecified.',
    }
    return limits[section]


def successors(evidence, manifest):
    status = json.loads((evidence / 'successor-inputs/status.json').read_bytes())
    versions = ['1.14.4'] + [f'1.15.{n}' for n in range(10)]
    assert [p['patch'] for p in status] == versions, 'successor sequence'
    for item in status:
        pin = next(p for p in manifest['pages'] if p['version'] == item['patch'])
        assert item['pin'] == pin and not item['applied'] and not item['native_proof'], 'successor boundary'
        raw = (evidence / f"successor-inputs/{item['patch']}-wikitext.txt").read_bytes()
        response = (evidence / f"successor-inputs/{item['patch']}-response.json").read_bytes()
        assert digest(raw) == pin['wikitext_sha256'] and digest(response) == pin['response_sha256'], 'successor hashes'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision, = page['revisions']
        assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
            pin['pageid'], pin['title'], pin['revid'], pin['timestamp']), 'successor identity'
        assert revision['slots']['main']['*'].encode() == raw, 'successor body'
    return status


def build(evidence):
    raw, pin = pin_source(evidence)
    text = raw.decode()
    lines = text.splitlines()
    register = parse_register(evidence, raw)
    inventory, byline = [], {}
    for entry in register['entries']:
        literal = lines[entry['wikitext_line'] - 1]
        row = dict(entry, literal=literal, literal_fields=literal_cvar_fields(literal),
                   status='UNPROVEN', capabilities=[], limit=contract_limit(entry['section']))
        inventory.append(row)
        byline.setdefault(entry['wikitext_line'], []).append(row)
    rows, headers, signatures, prose, refs = [], [], [], [], []
    context = ''
    for number, literal in enumerate(lines, 1):
        if not literal.strip():
            continue
        heading = re.fullmatch(r'==([^=]+)==', literal)
        count = re.search(r'<small>\((\d+)\)</small>', literal)
        if heading:
            context = heading[1]
            headers.append(dict(id=f'header-{number}', line=number, literal=literal,
                                title=context, literal_count=None, status='metadata-only'))
        elif count:
            headers.append(dict(id=f'header-{number}', line=number, literal=literal,
                                title=context, literal_count=int(count[1]), status='metadata-only'))
        line_inventory = byline.get(number, [])
        line_refs = references(number, literal)
        refs.extend(line_refs)
        for entry in line_inventory:
            if entry['section'] in ('global-api', 'widgets'):
                signatures.append(dict(id='signature-' + entry['id'], inventory_id=entry['id'],
                                       line=number, symbol=entry['symbol'], literal=literal,
                                       arguments=None, returns=None, status='UNPROVEN',
                                       limit='No literal signature declaration; do not infer from the name or linked page.'))
            if 'description' in entry['literal_fields']:
                prose.append(dict(id=f'prose-{number}', line=number, literal=literal,
                                  description=entry['literal_fields']['description'],
                                  status='UNPROVEN', limit=entry['limit']))
        if number == 4:
            prose.append(dict(id='prose-4', line=4, literal=literal, status='UNPROVEN',
                              limit='Qualified appears-to-be-in-sync TBC 2.5.4 comparison, not inclusion/equality or foreign supersession; link unexpanded.'))
        substantive = bool(line_inventory) or number == 4
        rows.append(dict(id=f'source-{number}', line=number, literal=literal, context=context,
                         inventory_ids=[r['id'] for r in line_inventory],
                         reference_ids=[r['id'] for r in line_refs],
                         status='UNPROVEN' if substantive else 'metadata-only', capabilities=[]))
    configured = (evidence / 'configured-inputs/src/client_profile.rs').read_text()
    interface = int(re.search(r'ClientProfile::Era \| ClientProfile::Anniversary => (\d+)', configured)[1])
    caption = lines[11]
    assert caption == '|+ 1.14.2 (42214) &rarr; 1.14.3 (43639) May 10 2022', 'literal caption'
    manifest = json.loads((evidence / 'original/manifest.json').read_bytes())
    return dict(schema='patch-full-literal-source/v1', patch='1.14.3', client_line='classic-era',
                source=pin, source_interface=11403, configured_interface=interface,
                source_caption=dict(literal=caption, base_version='1.14.2', base_build=42214,
                                    source_version='1.14.3', source_build=43639, date='May 10 2022'),
                navigation=dict(literal=lines[0], previous='1.14.2', next='1.14.4', expanded=False),
                source_rows=rows, inventory_rows=inventory, header_ledger=headers,
                signature_ledger=signatures, prose_ledger=prose, reference_ledger=refs,
                extracted_rows=[dict(id=f'extract-{n}', line=n, literal=line,
                                     status='UNPROVEN', limit='Default rendering only; original raw occurrences remain authoritative.')
                                for n, line in enumerate(default_extract(evidence).splitlines(), 1) if line.strip()],
                successors=successors(evidence, manifest), later_registers=[],
                measurements=dict(runtime=0, model=0, native=0),
                proof_policy='SOURCE only. Configured11507 is not source11403/build43639. Publication is not model/native proof.')


def validate_ledger(ledger, expected):
    assert ledger == expected, 'complete literal accounting/client/proof ledger'
    counts = Counter((r['section'], r['direction']) for r in ledger['inventory_rows'])
    return dict(inventory=len(ledger['inventory_rows']),
                inventory_counts={f'{s}/{d}': n for (s, d), n in sorted(counts.items())},
                **{key: len(ledger[key]) for key in ('source_rows', 'signature_ledger', 'prose_ledger',
                                                    'header_ledger', 'reference_ledger', 'extracted_rows')},
                measurements=ledger['measurements'])


def replay(evidence):
    seals = json.loads((evidence / 'seals.json').read_bytes())
    for name, expected in seals.items():
        data = (evidence / name).read_bytes()
        assert len(data) < 5_000_000 and digest(data) == expected, 'seal: ' + name
    ledger = json.loads((evidence / 'original/ledger.json').read_bytes())
    result = validate_ledger(ledger, build(evidence))
    raw, _ = pin_source(evidence)
    assert (evidence / 'original/default-register.json').read_bytes() == json_bytes(parse_register(evidence, raw)), 'default register bytes'
    assert (evidence / 'original/default-extract.txt').read_text() == default_extract(evidence), 'default extract bytes'
    result['original_seals'] = len(seals)
    return result


if __name__ == '__main__':
    print(json.dumps(replay(EVIDENCE), sort_keys=True))
