"""Frozen Era 1.14.2 SOURCE accounting. No runtime/model/native credit."""
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
E = Path(__file__).resolve().parent
BASE = '17cc9b0f20869d34b5445adf87a89a27afd565be'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(name):
    return json.loads((E/name).read_bytes())

def validate_source(response=None, raw=None):
    response = (E/'source-response.json').read_bytes() if response is None else response
    raw = (E/'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['236101']
    revision, = page['revisions']
    assert page['pageid'] == pin['pageid'] == 236101, 'pageid'
    assert page['title'] == pin['title'] == 'Patch 1.14.2/API changes', 'title'
    assert revision['revid'] == pin['revid'] == 2290155, 'revid'
    assert revision['timestamp'] == pin['timestamp'] == '2022-06-05T00:14:08Z', 'timestamp'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '1b5752c3d35f2b39e52d7dc594e50d79fbe74104a9f8c2a2051599cb22ddaa37', 'raw hash'
    assert digest(response) == pin['response_sha256'] == 'bd0440a7195fbb690be5f33972a26d7de4f29f7d55454ce1c2b4b2ad93c6ad01', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 2460, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next(p for p in manifest['pages'] if p['version']=='1.14.2'), 'manifest pin'
    registry = (E/'frozen-registry.json').read_bytes()
    assert digest(registry) == manifest['registry_sha256'], 'registry hash'
    pages = json.loads(registry)['pages']
    assert len(pages)==101 and pages[-1]['version']=='1.0.0', 'registry boundary'
    registered = next(p for p in pages if p['version']=='1.14.2')
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
    for version in ['1.14.3','1.14.4']+[f'1.15.{n}' for n in range(10)]:
        directory=E/'successors'/version
        pin=json.loads((directory/'pin.json').read_bytes())
        assert pin==next(p for p in manifest['pages'] if p['version']==version), 'successor pin'
        raw=(directory/'source.wikitext').read_bytes()
        response=(directory/'response.json').read_bytes()
        assert digest(raw)==pin['wikitext_sha256'] and digest(response)==pin['response_sha256'], 'successor hashes'
        page=json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision,=page['revisions']
        assert page['title']==pin['title'] and revision['revid']==pin['revid'] and revision['timestamp']==pin['timestamp'], 'successor identity'
        assert revision['slots']['main']['*'].encode()==raw, 'successor content'
        status=json.loads((directory/'status.json').read_bytes())
        expected='in-flight' if version=='1.14.3' else 'queued' if version=='1.14.4' else 'integrated-not-applied'
        assert status==dict(patch=version,state=expected,applied=False,native_proof=False), 'successor boundary'
        results.append(dict(status,source=pin))
    return results

def contract(kind, record):
    return dict(record,kind=kind,status='UNPROVEN',arguments=None,returns=None,state_transitions=None,security_rules=None,native_equivalence=None,limit='Literal frozen page only. Linked content and native behavior unspecified/unexpanded; no inferred defaults, aliases, profile exposure or model credit.')

def build():
    raw,pin=validate_source()
    rows=[];headers=[];counts=[];inventory=[];cvars=[];links=[];section=None;caption=None
    for n,literal in enumerate(raw.splitlines(),1):
        if not literal.strip(): continue
        substantive = literal.startswith(': ') or literal.startswith('* Diffs:') or literal.startswith('* Deprecated API:') or literal.startswith('* Community patch notes:')
        rows.append(dict(id=f'source-{n:03d}',line=n,literal=literal,status='UNPROVEN' if substantive else 'metadata-only',capabilities=[]))
        heading=re.fullmatch(r'==([^=]+)==',literal)
        if heading:
            section=heading[1];headers.append(dict(line=n,literal=literal,label=section))
        count=re.search(r'<font color="[^"]+">(Added|Removed)</font> <small>\((\d+)\)</small>',literal)
        if count: counts.append(dict(line=n,literal=literal,section=section,direction=count[1].lower(),count=int(count[2])))
        if literal.startswith('|+ '): caption=literal[3:]
        api=re.fullmatch(r': \{\{api\|t=a\|([^}]+)\}\}',literal)
        if api: inventory.append(dict(line=n,literal=literal,symbol=api[1],section=section,direction='added',kind='callable',status='UNPROVEN'))
        cvar=re.search(r'\[\[CVar ([^|]+)\|([^]]+)\]\].*Default: <code><span class="apitype">([^<]+)</span></code><br><small>([^<]+)</small>',literal)
        if cvar:
            assert cvar[1]==cvar[2], 'CVar label'
            item=dict(line=n,literal=literal,symbol=cvar[1],section=section,direction='added',kind='cvar',status='UNPROVEN')
            inventory.append(item);cvars.append(dict(item,default=cvar[3],description=cvar[4],hidden=True))
        for index,(target,label) in enumerate(re.findall(r'\[(https://\S+) ([^]]+)\]',literal),1):
            links.append(dict(line=n,literal=literal,target=target,label=label,index=index,expanded=False))
        for index,(target,label) in enumerate(re.findall(r'\[\[([^|]+)\|([^]]+)\]\]',literal),1):
            links.append(dict(line=n,literal=literal,target=target,label=label,index=index,expanded=False))
    nav=re.fullmatch(r'\{\{apichanges\|([^|]+)\|prev=([^|]+)\|next=([^|]+)\}\}',raw.splitlines()[0])
    assert nav, 'navigation'
    toc=int(re.search(r'\* TOC: <code>(\d+)</code>',raw)[1])
    signatures=[dict(line=i['line'],symbol=i['symbol'],arguments=None,returns=None,status='UNPROVEN',declaration=None) for i in inventory if i['kind']=='callable']
    contracts=[contract('callable-unspecified',i) for i in inventory if i['kind']=='callable']+[contract('hidden-cvar-default-and-description',c) for c in cvars]+[contract('unexpanded-link',l) for l in links]
    return dict(schema='patch-source-accounting/v1',patch='1.14.2',base_revision=BASE,source=pin,source_toc=toc,caption=caption,navigation=dict(zip(['patch','prev','next'],nav.groups())),transclusions=[dict(line=1,literal=raw.splitlines()[0],expanded=False,role='navigation-only')],source_rows=rows,inventory=inventory,signatures=signatures,cvars=cvars,headers=headers,inventory_headers=counts,links=links,contracts=contracts,examples=[],configured_profiles=profiles(),successors=successors(),later_registers=[],measurements=dict(runtime=0,model=0,native=0),totals=dict(physical_lines=len(raw.splitlines()),nonblank_rows=len(rows),metadata_rows=sum(r['status']=='metadata-only' for r in rows),unproven_rows=sum(r['status']=='UNPROVEN' for r in rows),inventory=len(inventory),callables=len(signatures),cvars=len(cvars),headers=len(headers),inventory_headers=len(counts),contracts=len(contracts),signature_declarations=0,unspecified_signatures=len(signatures),links=len(links),examples=0,prose_defaults=len(cvars)))

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
