"""Frozen Era 1.13.6 occurrence accounting; SOURCE only, offline."""
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
E = Path(__file__).resolve().parent
BASE = '5b12dac256abc0ffaf541df2bc445e8381612d2d'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(name):
    return json.loads((E/name).read_bytes())


def validate_source(response=None, raw=None):
    response = (E/'source-response.json').read_bytes() if response is None else response
    raw = (E/'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['461367']
    revision, = page['revisions']
    assert (page['pageid'],page['title'],revision['revid'],revision['timestamp']) == (461367,'Patch 1.13.6/API changes',4435603,'2021-05-06T13:20:50Z'), 'identity'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '21a9bf3248ca5db0bee46c5fec4d0332744a6e089a52e17b38dde8dbebc4e4c9', 'raw hash'
    assert digest(response) == pin['response_sha256'] == '46e81af7accc100716c0fd72c06060ec94cc9a3b4e1eae3b27778d72595c5e9d', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 827, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next(p for p in manifest['pages'] if p['version']=='1.13.6'), 'manifest pin'
    registry = (E/'frozen-registry.json').read_bytes()
    assert digest(registry) == manifest['registry_sha256'], 'registry hash'
    pages = json.loads(registry)['pages']
    assert len(pages)==101 and pages[-1]['version']=='1.0.0', 'registry boundary'
    registered = next(p for p in pages if p['version']=='1.13.6')
    assert all(pin[k]==v for k,v in registered.items()), 'registry identity'
    return raw.decode(), pin


def profiles():
    directory = E/'state-inputs'
    rust = (directory/'src/client_profile.rs').read_text()
    interface = int(re.search(r'ClientProfile::Era \| ClientProfile::Anniversary => (\d+)',rust)[1])
    features = tomllib.loads((directory/'Cargo.toml').read_text())['features']
    return [dict(feature=f'client-{name}',configured_interface=interface,feature_dependencies=features[f'client-{name}'],manifest_sha256=digest((directory/f'data/blizzard-ui-files/{name}.txt').read_bytes()),native_correspondence='UNPROVEN') for name in ['era','anniversary']]


def successors():
    results=[]
    manifest=read_json('frozen-manifest.json')
    versions=['1.13.7']+[f'1.14.{n}' for n in range(5)]+[f'1.15.{n}' for n in range(10)]
    for version in versions:
        directory=E/'successors'/version
        pin=json.loads((directory/'pin.json').read_bytes())
        assert pin==next(p for p in manifest['pages'] if p['version']==version), 'successor pin'
        raw=(directory/'source.wikitext').read_bytes()
        response=(directory/'response.json').read_bytes()
        assert digest(raw)==pin['wikitext_sha256'] and digest(response)==pin['response_sha256'], 'successor hashes'
        page=json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision,=page['revisions']
        assert page['pageid']==pin['pageid'] and page['title']==pin['title'] and revision['revid']==pin['revid'] and revision['timestamp']==pin['timestamp'], 'successor identity'
        assert revision['slots']['main']['*'].encode()==raw, 'successor content'
        status=json.loads((directory/'status.json').read_bytes())
        expected='in-flight' if version=='1.13.7' else 'queued' if version in ['1.14.0','1.14.1'] else 'integrated-not-applied'
        assert status==dict(patch=version,state=expected,applied=False,native_proof=False), 'successor boundary'
        results.append(dict(status,source=pin))
    return results


def contract(kind,record,limit):
    return dict(record,kind=kind,status='UNPROVEN',arguments=None,returns=None,state_transitions=None,security_rules=None,native_equivalence=None,limit=limit)


def build():
    raw,pin=validate_source()
    rows=[];headers=[];counts=[];inventory=[];links=[];templates=[]
    for n,literal in enumerate(raw.splitlines(),1):
        if not literal.strip(): continue
        substantive=literal.startswith(': {{api|') or literal.startswith('* Diffs:') or literal.startswith('Patch 1.13.6 ')
        rows.append(dict(id=f'source-{n:03d}',line=n,literal=literal,status='UNPROVEN' if substantive else 'metadata-only',capabilities=[]))
        heading=re.fullmatch(r'==\s*([^=]+?)\s*==',literal)
        if heading: headers.append(dict(line=n,literal=literal,label=heading[1]))
        count=re.search(r'\|\s*(\d+) new cvars$',literal)
        if count: counts.append(dict(line=n,literal=literal,count=int(count[1]),direction='added',section='cvars'))
        api=re.fullmatch(r': \{\{api\|t=c\|([^}]+)\}\}',literal)
        if api: inventory.append(dict(id=f'wt-cvars-{api[1]}-{n}',line=n,literal=literal,symbol=api[1],section='cvars',direction='added',page_default=None,status='UNPROVEN'))
        for target,label in re.findall(r'\[(https://\S+) ([^]]+)\]',literal):
            links.append(dict(line=n,literal=literal,target=target,label=label,expanded=False))
        for target,label in re.findall(r'\[\[([^|]+)\|([^]]+)\]\]',literal):
            links.append(dict(line=n,literal=literal,target=target,label=label,expanded=False))
        for match in re.finditer(r'\{\{[^{}]+\}\}',literal):
            templates.append(dict(line=n,literal=match[0],expanded=False,role='navigation-only' if n==1 else 'inventory-reference'))
    for count in counts:
        count['observed']=len(inventory)
        count['matches']=count['count']==count['observed']
    nav=re.fullmatch(r'\{\{apichanges\|([^|]+)\|prev=([^|]+)\|next=([^|]+)\}\}',raw.splitlines()[0])
    assert nav, 'navigation'
    summary=next(l for l in raw.splitlines() if l.startswith('Patch 1.13.6 '))
    prose=[dict(line=4,literal=summary,description='Naxxramas content-patch attribution; no client callable/effect contract')]
    contracts=[contract('cvar-publication-only',i,'Name/addition only. No source default, type, allowed range, mutation/effect, persistence or security contract.') for i in inventory]
    contracts += [contract('unexpanded-link',l,'Exact literal link boundary only; no linked content imported.') for l in links]
    contracts += [contract('content-patch-summary',p,'Literal Classic Naxxramas attribution only; not API behavior or native equivalence.') for p in prose]
    return dict(schema='patch-source-accounting/v1',patch='1.13.6',base_revision=BASE,source=pin,source_toc=int(re.search(r'\* TOC: <code>(\d+)</code>',raw)[1]),summary=summary,navigation=dict(zip(['patch','prev','next'],nav.groups())),source_rows=rows,inventory=inventory,signatures=[],examples=[],prose_defaults=[],prose=prose,headers=headers,inventory_headers=counts,links=links,templates=templates,contracts=contracts,configured_profiles=profiles(),successors=successors(),later_registers=[],measurements=dict(runtime=0,model=0,native=0),totals=dict(physical_lines=len(raw.splitlines()),nonblank_rows=len(rows),metadata_rows=sum(r['status']=='metadata-only' for r in rows),unproven_rows=sum(r['status']=='UNPROVEN' for r in rows),inventory=len(inventory),callables=0,cvars=len(inventory),headers=len(headers),inventory_headers=len(counts),contracts=len(contracts),signature_declarations=0,links=len(links),templates=len(templates),examples=0,prose_defaults=0,prose=len(prose)))


def validate_ledger(ledger):
    assert ledger==build(), 'serialized literal ledger'
    return ledger['totals']


def check_seals():
    seals=read_json('seals.json')
    for name,expected in seals.items():
        assert digest((E/name).read_bytes())==expected, f'seal: {name}'
    return len(seals)


if __name__=='__main__':
    if sys.argv[1:]==['capture']:
        assert not (E/'ledger.json').exists(), 'refuse ledger overwrite'
        (E/'ledger.json').write_text(json.dumps(build(),indent=2)+'\n')
    else:
        print(json.dumps(dict(seals=check_seals(),totals=validate_ledger(read_json('ledger.json'))),indent=2))
