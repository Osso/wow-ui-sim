"""Literal Era 1.14.1 SOURCE accounting. Never imports linked bodies or native proof."""
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib

E = Path(__file__).resolve().parent
BASE = '53bdb30cbd74ee496109f7be5e21ebf4c2ad5016'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(name):
    return json.loads((E / name).read_bytes())

def validate_source(raw=None, response=None):
    pin = read_json('source-pin.json')
    raw = (E / 'source.wikitext').read_bytes() if raw is None else raw
    response = (E / 'source-response.json').read_bytes() if response is None else response
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision, = page['revisions']
    assert (pin['version'], page['title'], page['pageid'], revision['revid'], revision['timestamp']) == ('1.14.1', 'Patch 1.14.1/API changes', 102588, 1010974, '2022-03-02T05:21:47Z'), 'identity'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '41c615541135cabceb8ff85bb0ff633ffa8d4eafe861e724d6fdf60e739d8559', 'raw hash'
    assert digest(response) == pin['response_sha256'] == 'aac93df0da292053fee1aee1534d2de00101cf01c46dcb51ab1be858a88c30f8', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 6207, 'bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next(p for p in manifest['pages'] if p['version'] == '1.14.1'), 'manifest pin'
    registry = (E / 'frozen-registry.json').read_bytes()
    assert digest(registry) == manifest['registry_sha256'], 'registry hash'
    pages = json.loads(registry)['pages']
    assert len(pages) == 101 and pages[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next(p for p in pages if p['version'] == '1.14.1')
    assert all(pin[key] == value for key, value in registered.items()), 'registry identity'
    return raw.decode(), pin

def configured_profiles():
    folder = E / 'configured-inputs'
    rust = (folder / 'src/client_profile.rs').read_text()
    features = tomllib.loads((folder / 'Cargo.toml').read_text())['features']
    variants, interface = re.search(r'(ClientProfile::Era \| ClientProfile::Anniversary) => (\d+)', rust).groups()
    return [dict(feature=f'client-{name}', configured_interface=int(interface), manifest_sha256=digest((folder / f'data/blizzard-ui-files/{name}.txt').read_bytes()), native_proof=False) for name in ['era', 'anniversary'] if f'client-{name}' in features]

def successors():
    versions = ['1.14.2', '1.14.3', '1.14.4'] + [f'1.15.{n}' for n in range(10)]
    manifest = read_json('frozen-manifest.json')
    records = []
    for version in versions:
        folder = f'successors/{version}/'
        pin = read_json(folder + 'source-pin.json')
        assert pin == next(p for p in manifest['pages'] if p['version'] == version), 'successor pin'
        raw = (E / folder / 'source.wikitext').read_bytes()
        response = (E / folder / 'source-response.json').read_bytes()
        assert digest(raw) == pin['wikitext_sha256'] and digest(response) == pin['response_sha256'], 'successor hashes'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision, = page['revisions']
        assert (page['title'], page['pageid'], revision['revid'], revision['timestamp'], revision['slots']['main']['*'].encode()) == (pin['title'], pin['pageid'], pin['revid'], pin['timestamp'], raw), 'successor identity'
        status = read_json(folder + 'status.json')
        assert not status['applied'] and not status['native_proof'] and status['history'] == 'same-Era-only', 'successor boundary'
        expected = 'queued-pending-main-integration' if version in ['1.14.2', '1.14.3'] else 'actual-canonical-input-not-applied'
        assert status['state'] == expected, 'successor state'
        if expected.startswith('actual'):
            assert digest((E / folder / 'canonical-ledger.json').read_bytes()) == status['canonical_ledger_sha256'], 'actual canonical input'
        records.append(dict(status, source=pin))
    return records

def inventory_rows(lines):
    rows, headers = [], []
    section, direction = None, None
    column = 0
    for n, literal in enumerate(lines, 1):
        heading = re.fullmatch(r'==([^=]+)==', literal)
        if heading:
            section = heading[1]
            column = 0
        header = re.search(r'>(Added|Removed)</font> <small>\((\d+)\)</small>', literal)
        if header:
            headers.append(dict(line=n, literal=literal, section=section, direction=header[1].lower(), count=int(header[2])))
        if literal.startswith('| valign='):
            direction = 'added' if column == 0 else 'removed'
            column += 1
        api = re.search(r'\{\{api\|t=(\w)\|([^}]+)\}\}', literal)
        cvar = re.search(r'\[\[CVar ([^|]+)\|([^]]+)\]\]', literal)
        if api or cvar:
            symbol = api[2] if api else cvar[1]
            kind = {'a': 'callable', 'w': 'widget', 'e': 'event'}[api[1]] if api else 'cvar'
            rows.append(dict(id=f'inventory-{n:03d}', line=n, literal=literal, symbol=symbol, section=section, direction=direction, kind=kind, arguments=None, returns=None, payload=None, status='UNPROVEN', limit='Literal identity/direction only; signature, native behavior and semantic retirement unspecified. Do not normalize source spelling.'))
    for header in headers:
        header['observed_count'] = sum(r['section'] == header['section'] and r['direction'] == header['direction'] for r in rows)
    return rows, headers

def references(lines):
    links, templates = [], []
    for n, literal in enumerate(lines, 1):
        for match in re.finditer(r'\[\[([^]]+)\]\]|\[(https://[^\s\]]+)(?: ([^]]+))?\]', literal):
            if match[1]:
                pieces = match[1].split('|', 1)
                target, label = pieces[0], pieces[-1]
                kind = 'wiki-link'
            else:
                target, label, kind = match[2], match[3], 'external-link'
            links.append(dict(line=n, literal=match[0], target=target, label=label, kind=kind, expanded=False, status='UNPROVEN'))
        for match in re.finditer(r'\{\{[^{}]+\}\}', literal):
            templates.append(dict(line=n, literal=match[0], expanded=False, role='navigation' if n == 1 else 'inline-client-marker' if match[0] == '{{Wow-inline}}' else 'API-inventory-marker', status='UNPROVEN'))
    return links, templates

def cvar_contracts(rows):
    result = []
    for row in rows:
        if row['kind'] != 'cvar':
            continue
        literal = row['literal']
        default = re.search(r'Default: <code><span class="apitype">([^<]+)', literal)
        scope = re.search(r'Scope: <span class="apitype">([^<]+)', literal)
        description = re.search(r'<br><small>(.*?)</small>', literal)
        result.append(dict(line=row['line'], literal=literal, symbol=row['symbol'], direction=row['direction'], default=default[1] if default else None, scope=scope[1] if scope else None, description=description[1] if description else None, status='UNPROVEN', limit='Exact lexical default/scope/prose; no invented defaults for bare links. Renderer/cache/chat/notification effects not established by registry publication.'))
    return result

def model_review():
    return [
        dict(area='C_Seasons', grounded='Only namespace purpose and two member names.', reason='Active season identity, activation transitions and values absent; no season model found in retained scan. Placeholder numeric/default implementation unjustified.'),
        dict(area='Tooltip inheritance', grounded='Two exact no-longer-inherits claims.', reason='Current profile cache configured at11507, not frozen11401. No retained1.14.1 XML template bodies; do not patch vendor or infer effective legacy inheritance from headless initializer.'),
        dict(area='Hidden event trace', grounded='Logging while hidden supported; disabled by default.', reason='Blizzard event-trace UI option, not core event dispatch. No frozen tool implementation/setting identifier retained; generic frame event registration would not exercise this option.'),
        dict(area='FontString:GetTextScale', grounded='Existing registered getter reads per-frame text_scale; setter updates it.', reason='Candidate direct existing-state roundtrip, not a source-declared numeric/default/signature contract. FontStringSetTextScale remains literal typo-shaped identity; do not alias to SetTextScale.'),
        dict(area='securecallfunction/secureexecuterange', grounded='Existing rilua/bootstrap paths and tests retained.', reason='Page names only, no taint/error/iteration/security signatures. Existing tests cannot confer native1.14.1/security parity; changing secure behavior from this source is unjustified.'),
        dict(area='Club/social/commentator/messaging APIs and events', grounded='Names/add-remove directions retained.', reason='Page omits arguments, returns, event payloads, delivery/order and state transitions. Existing clubs/chat placeholders and permissive Classic event registration are not community/network/UK AADC models. No cheap new model justified.'),
        dict(area='CVar render/cache/AADC/SoM contracts', grounded='Four explicit lexical defaults, one Account scope, four descriptions; others unspecified.', reason='Registry read/write alone cannot prove GPU dynamic scaling, heap/cache strategy, regional disable-alert history or season notification effects. Do not invent missing defaults or implement no-op systems.'),
        dict(area='GetFirstBagBankSlotIndex and removals', grounded='Literal addition/removal identities.', reason='No index number/slot boundary or removal behavior specified. Do not invent number, alias renamed commentator calls, or apply modern/foreign-history retirement. Main owns same-Era successor integration.')]

def build():
    raw, pin = validate_source()
    lines = raw.splitlines()
    inventory, headers = inventory_rows(lines)
    links, templates = references(lines)
    substantive = {r['line'] for r in inventory} | {4, 5, 6, 10, 11}
    prose = [dict(line=n, literal=lines[n-1], status='UNPROVEN', default='disabled' if n == 6 else None, limit='Literal claim only; linked bodies unexpanded; no native/model credit.') for n in [4, 5, 6]]
    claims = [dict(line=5, literal=lines[4], symbol=symbol, no_longer_inherits='BackdropTemplate', status='UNPROVEN') for symbol in ['SharedTooltipTemplate', 'GameTooltipTemplate']]
    headings = [dict(line=n, literal=s, label=m[1]) for n,s in enumerate(lines,1) if (m := re.fullmatch(r'==([^=]+)==', s))]
    rows = [dict(id=f'source-{n:03d}', line=n, literal=s, status='UNPROVEN' if n in substantive else 'metadata-only', capabilities=[]) for n,s in enumerate(lines,1) if s.strip()]
    signatures = [dict(line=row['line'], symbol=row['symbol'], kind=row['kind'], arguments=None, returns=None, payload=None, declared=False, status='UNPROVEN') for row in inventory]
    return dict(schema='patch-source-accounting/v1', patch=pin['version'], source=pin, source_toc=int(re.search(r'TOC: <code>(\d+)', raw)[1]), base_revision=BASE, client_line='literal-Classic-Era-not-native-proof', source_rows=rows, inventory=inventory, signatures=signatures, headings=headings, inventory_headers=headers, caption=dict(line=16, literal=lines[15]), summary_prose=prose, cvar_contracts=cvar_contracts(inventory), template_claims=claims, navigation=dict(patch='1.14.1',prev='1.14.0',next='1.14.2',expanded=False), links=links, transclusions=templates, configured_profiles=configured_profiles(), successors=successors(), later_registers=[], measurements=dict(runtime=0,model=0,native=0), model_review=model_review(), totals=dict(physical_lines=len(lines), nonblank_rows=len(rows), inventory=len(inventory), headings=len(headings), inventory_headers=len(headers), prose=len(prose), cvars=8, template_claims=len(claims), links=len(links), transclusions=len(templates), unspecified_signatures=len(signatures), substantive_rows=sum(r['status']=='UNPROVEN' for r in rows)))

def validate_ledger(ledger):
    assert ledger == build(), 'serialized literal ledger'
    return ledger['totals']

def check_seals():
    seals = read_json('seals.json')
    for name, expected in seals.items():
        assert digest((E / name).read_bytes()) == expected, f'seal: {name}'
    return len(seals)

def main():
    if sys.argv[1:] == ['capture']:
        assert not (E / 'ledger.json').exists(), 'refuse ledger overwrite'
        (E / 'ledger.json').write_text(json.dumps(build(),indent=2)+'\n')
    else:
        count = check_seals()
        print(json.dumps(dict(historical_seals=count,totals=validate_ledger(read_json('ledger.json'))),indent=2))

if __name__ == '__main__':
    main()
