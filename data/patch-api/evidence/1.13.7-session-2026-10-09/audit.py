"""Frozen 1.13.7 literal SOURCE accounting. No runtime/model/native credit."""
import hashlib
import importlib.util
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


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name.removesuffix('.py'), E/'historical-tools'/name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_source(response=None, raw=None):
    response = (E/'source-response.json').read_bytes() if response is None else response
    raw = (E/'source.wikitext').read_bytes() if raw is None else raw
    pin = read_json('source-pin.json')
    page = json.loads(response)['query']['pages']['46829']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (46829, 'Patch 1.13.7/API changes', 458410, '2021-09-04T08:55:38Z'), 'identity'
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == 'b9f8be9d2db166e6b85ec868fc077bc5591071d55c05bb5f11267eb5ba11e996', 'raw hash'
    assert digest(response) == pin['response_sha256'] == 'b0d750e17ee76815e6c4917f9d98f81780ef3b5bb26466acbcc9e1a8899099c4', 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 1179, 'raw bytes'
    manifest = read_json('frozen-manifest.json')
    assert pin == next(p for p in manifest['pages'] if p['version']=='1.13.7'), 'manifest pin'
    assert digest((E/'frozen-registry.json').read_bytes()) == manifest['registry_sha256'], 'registry hash'
    pages = read_json('frozen-registry.json')['pages']
    assert len(pages) == 101 and pages[-1]['version']=='1.0.0', 'registry boundary'
    registered = next(p for p in pages if p['version']=='1.13.7')
    assert all(pin[k]==v for k,v in registered.items()), 'registry identity'
    assert all(pin[k]==v for k,v in dict(pageid=page['pageid'],title=page['title'],revid=revision['revid'],timestamp=revision['timestamp']).items()), 'pin identity'
    return raw.decode(), pin


def default_register(raw, pin):
    generator = load_tool('gen_patch_wikitext_register.py')
    buckets = generator.split_sections(raw)
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        rows, counts = generator.parse_section(section, buckets.get(section, []))
        entries.extend(rows)
        headers.extend(counts)
    return dict(schema='patch-api-wikitext-register/v1',patch='1.13.7',source=dict(path='source.wikitext',revid=pin['revid'],sha256=pin['wikitext_sha256']),header_counts=headers,entries=entries)


def profiles():
    directory = E/'state-inputs'
    rust = (directory/'src/client_profile.rs').read_text()
    interface = int(re.search(r'ClientProfile::Era \| ClientProfile::Anniversary => (\d+)',rust)[1])
    features = tomllib.loads((directory/'Cargo.toml').read_text())['features']
    return [dict(feature=f'client-{name}',configured_interface=interface,feature_dependencies=features[f'client-{name}'],manifest_sha256=digest((directory/f'data/blizzard-ui-files/{name}.txt').read_bytes()),native_correspondence='UNPROVEN') for name in ['era','anniversary']]


def successors():
    status = read_json('successors/status.json')
    versions = ['1.14.0','1.14.1','1.14.2','1.14.3','1.14.4']+[f'1.15.{n}' for n in range(10)]
    assert [s['patch'] for s in status] == versions, 'successor order'
    manifest = read_json('frozen-manifest.json')
    for item in status:
        version = item['patch']
        pin = next(p for p in manifest['pages'] if p['version']==version)
        assert item['pin']==pin, 'successor pin'
        expected = 'completed-queued' if version in ('1.14.0','1.14.1') else 'integrated-not-applied'
        assert item['state']==expected and not item['applied'] and not item['native_proof'], 'successor boundary'
        directory = E/'successors'/version
        raw = (directory/'source.wikitext').read_bytes()
        response = (directory/'response.json').read_bytes()
        assert digest(raw)==pin['wikitext_sha256'] and digest(response)==pin['response_sha256'], 'successor hashes'
        page = json.loads(response)['query']['pages'][str(pin['pageid'])]
        revision, = page['revisions']
        assert (page['pageid'],page['title'],revision['revid'],revision['timestamp'])==(pin['pageid'],pin['title'],pin['revid'],pin['timestamp']), 'successor identity'
        assert revision['slots']['main']['*'].encode()==raw, 'successor content'
    return status


def contract(kind, record):
    return dict(record,kind=kind,status='UNPROVEN',arguments=None,returns=None,state_transitions=None,security_rules=None,native_equivalence=None,limit='Literal identity only. No signature/default/description/state/security contract. Links/templates unexpanded; no inferred arguments, aliases, defaults or native/model credit.')


def build():
    raw, pin = validate_source()
    register = default_register(raw,pin)
    extractor = load_tool('extract_patch_non_inventory.py')
    extracted = extractor.extract_text(raw)
    rows, headers, counts, inventory, cvars, links, templates = [], [], [], [], [], [], []
    context = None
    byline = {i['wikitext_line']:i for i in register['entries']}
    for n,literal in enumerate(raw.splitlines(),1):
        if not literal.strip():
            continue
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==',literal)
        if heading:
            context=heading[1]
            headers.append(dict(line=n,literal=literal,label=context))
        if literal.startswith('! '):
            count = re.search(r'\((\d+)\)',literal)
            counts.append(dict(line=n,literal=literal,section={'Global API':'global-api','CVars':'cvars'}[context],direction='added',count=int(count[1]) if count else None))
        entry = byline.get(n)
        if entry:
            item = dict(entry,line=n,literal=literal,status='UNPROVEN',capabilities=[])
            inventory.append(item)
            if entry['section']=='cvars':
                cvars.append(dict(item,default=None,description=None,scope=None))
        substantive = entry is not None or literal.startswith('* Diffs:')
        rows.append(dict(id=f'source-{n:03d}',line=n,literal=literal,status='UNPROVEN' if substantive else 'metadata-only',capabilities=[]))
        for index,(target,label) in enumerate(re.findall(r'\[(https://\S+) ([^]]+)\]',literal),1):
            links.append(dict(line=n,literal=literal,target=target,label=label,index=index,expanded=False))
        for index,template in enumerate(re.findall(r'\{\{[^{}]+\}\}',literal),1):
            templates.append(dict(line=n,literal=template,index=index,expanded=False,role='navigation' if n==1 else 'inventory-reference'))
    nav = re.fullmatch(r'\{\{apichanges\|([^|]+)\|prev=([^|]+)\|next=([^|]+)\}\}',raw.splitlines()[0])
    assert nav, 'navigation'
    signatures = [dict(line=i['line'],symbol=i['symbol'],arguments=None,returns=None,declaration=None,status='UNPROVEN') for i in inventory if i['section']=='global-api']
    contracts = [contract('callable-unspecified',i) for i in inventory if i['section']=='global-api']+[contract('cvar-unspecified',c) for c in cvars]+[contract('unexpanded-link',l) for l in links]
    extracted_rows = [dict(id=f'extract-{n:03d}',line=n,literal=line,status='SOURCE-rendering-only') for n,line in enumerate(extracted.splitlines(),1) if line.strip()]
    return dict(schema='patch-source-accounting/v1',patch='1.13.7',base_revision=BASE,audit_history='classic-era',source_client_literal=None,source=pin,source_toc=int(re.search(r'\* TOC: <code>(\d+)</code>',raw)[1]),navigation=dict(zip(['patch','prev','next'],nav.groups())),source_rows=rows,extracted_rows=extracted_rows,inventory=inventory,signatures=signatures,cvars=cvars,headers=headers,inventory_headers=counts,links=links,templates=templates,contracts=contracts,examples=[],prose=[],configured_profiles=profiles(),successors=successors(),later_registers=[],measurements=dict(runtime=0,model=0,native=0),totals=dict(physical_lines=len(raw.splitlines()),nonblank_rows=len(rows),extracted_rows=len(extracted_rows),metadata_rows=sum(r['status']=='metadata-only' for r in rows),unproven_rows=sum(r['status']=='UNPROVEN' for r in rows),inventory=len(inventory),callables=len(signatures),cvars=len(cvars),headers=len(headers),inventory_headers=len(counts),numeric_headers=sum(h['count'] is not None for h in counts),contracts=len(contracts),signature_declarations=0,unspecified_signatures=len(signatures),links=len(links),templates=len(templates),examples=0,prose=0,defaults=0))


def validate_ledger(ledger):
    assert ledger==build(), 'serialized literal ledger'
    return ledger['totals']


def check_seals():
    seals = read_json('seals.json')
    for name,expected in seals.items():
        assert digest((E/name).read_bytes())==expected, f'seal: {name}'
    return len(seals)


if __name__=='__main__':
    if sys.argv[1:]==['capture']:
        assert not (E/'ledger.json').exists(), 'refuse ledger overwrite'
        raw,pin=validate_source()
        (E/'default-extract.txt').write_text(load_tool('extract_patch_non_inventory.py').extract_text(raw))
        (E/'ledger.json').write_text(json.dumps(build(),indent=2)+'\n')
    else:
        seals=check_seals()
        raw,pin=validate_source()
        expected=json.dumps(default_register(raw,pin),indent=2,ensure_ascii=False)+'\n'
        assert (E/'default-register.json').read_bytes()==expected.encode(), 'default generator bytes'
        assert (E/'default-extract.txt').read_bytes()==load_tool('extract_patch_non_inventory.py').extract_text(raw).encode(), 'default extractor bytes'
        print(json.dumps(dict(seals=seals,totals=validate_ledger(read_json('ledger.json'))),indent=2))
