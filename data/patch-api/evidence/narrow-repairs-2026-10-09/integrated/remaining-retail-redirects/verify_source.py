from pathlib import Path
import ast, datetime, hashlib, json, os, re, tarfile
ROOT = Path('/home/osso/Projects/wow/wow-ui-sim')
OUT = Path('/tmp/remaining-redirects-integrated-independent')
VERSIONS = ['1.10.1','1.10.0','1.9.0','1.8.0','1.7.0','1.6.0','1.5.0']
PATTERNS = [rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----', rb'\b(?:sk-(?:proj-|ant-)?)[A-Za-z0-9_-]{32,}', rb'\bgh[pousr]_[A-Za-z0-9]{30,}', rb'\bBearer\s+[A-Za-z0-9._-]{25,}', rb'\bAKIA[A-Z0-9]{16}\b']
def now(): return datetime.datetime.now(datetime.timezone.utc).isoformat()
def sha(data): return hashlib.sha256(data).hexdigest()
def private(data): return sum(len(re.findall(p,data)) for p in PATTERNS)
def save(path, data):
    data = data if isinstance(data,bytes) else data.encode()
    assert not private(data), 'privacy pattern matched; refusing retention'
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(data)
def js(path, obj): save(path,json.dumps(obj,indent=2,sort_keys=True)+'\n')
def scope(directory):
    return {str(p.relative_to(directory)):sha(p.read_bytes()) for p in sorted(directory.rglob('*')) if p.is_file()}
def scope_hash(mapping): return sha(json.dumps(mapping,sort_keys=True,separators=(',',':')).encode())
def maps(d):
    original=json.loads((d/'seals.json').read_bytes())
    receipt=json.loads((d/'receipt-seals.json').read_bytes())
    if 'files' in receipt: receipt=receipt['files']
    assert all(isinstance(value,str) for value in receipt.values()), 'unknown receipt schema'
    failures=[]
    for kind,mapping in [('original',original),('receipt',receipt)]:
        for name, expected in mapping.items():
            p=Path(name)
            assert not p.is_absolute() and '..' not in p.parts
            if not (d/name).is_file() or sha((d/name).read_bytes()) != expected:
                failures.append({'kind':kind,'file':name,'expected':expected,'actual':sha((d/name).read_bytes()) if (d/name).is_file() else None})
    assert not set(original)&set(receipt), 'receipt map overlaps originals'
    return original,receipt,failures
commands=json.loads((OUT/'commands.json').read_bytes()) if (OUT/'commands.json').exists() else []
def git(args,label):
    argv=['git']+args; start=now()
    result=cli.git(*args).cwd(str(ROOT)).capture().run()
    stop=now(); prefix=OUT/'epochs'/label
    save(prefix.with_suffix('.stdout'),result.stdout)
    save(prefix.with_suffix('.stderr'),result.stderr)
    rec={'argv':argv,'cwd':str(ROOT),'start':start,'end':stop,'exit':result.exit_code,'stdout':str(prefix.with_suffix('.stdout')),'stderr':str(prefix.with_suffix('.stderr'))}
    commands.append(rec); js(OUT/'commands.json',commands)
    assert result.exit_code==0
    return result.stdout.strip()

def run_script(v, copied, script, index):
    d=ROOT/'data/patch-api/evidence'/f'{v}-session-2026-10-09'
    folder=OUT/v; stem=folder/f'{index}-{script}'
    pre=scope(d); copy_pre=scope(copied)
    rev=git(['rev-parse','HEAD'],f'{v}-{index}-pre')
    argv=['/usr/bin/python3','-I','-B',str(copied/script)]
    if script=='test_portable.py': argv.append(str(folder/'fresh-portable-proof.json'))
    env={'PATH':'/nonexistent','PYTHONNOUSERSITE':'1','PYTHONDONTWRITEBYTECODE':'1'}
    start=now(); rec={'argv':argv,'cwd':str(ROOT),'revision_pre':rev,'start':start,'environment_assignments':env,'pre_scope_sha256':scope_hash(pre),'pre_copy_scope_sha256':scope_hash(copy_pre),'exit':None}
    js(stem.with_suffix('.pending.json'),rec)
    try:
        result=cli.command(*argv).cwd(str(ROOT)).env(env).capture().run()
    except Exception as error:
        rec.update(end=now(),capture_loss=True,error_type=type(error).__name__,error=str(error))
        js(stem.with_suffix('.receipt.json'),rec)
        return rec
    rec.update(end=now(),exit=result.exit_code,capture_loss=False)
    save(stem.with_suffix('.stdout'),result.stdout);save(stem.with_suffix('.stderr'),result.stderr)
    rec['stdout']=str(stem.with_suffix('.stdout'));rec['stderr']=str(stem.with_suffix('.stderr'))
    post=scope(d);copy_post=scope(copied)
    rec.update(revision_post=git(['rev-parse','HEAD'],f'{v}-{index}-post'),post_scope_sha256=scope_hash(post),post_copy_scope_sha256=scope_hash(copy_post),integrated_inputs_unchanged=pre==post,copied_inputs_unchanged=copy_pre==copy_post)
    js(stem.with_suffix('.pre-scope.json'),pre);js(stem.with_suffix('.post-scope.json'),post)
    js(stem.with_suffix('.copy-pre-scope.json'),copy_pre);js(stem.with_suffix('.copy-post-scope.json'),copy_post)
    js(stem.with_suffix('.receipt.json'),rec)
    return rec

OUT.mkdir(parents=True,exist_ok=True)
key_matches=sum(bool(re.search(r'token|secret|password|credential|api.?key|private.?key',key,re.I)) for key in os.environ.keys())
js(OUT/'privacy.json',{'inspected_at':now(),'environment_key_count':len(os.environ),'sensitive_environment_key_count':key_matches,'environment_values_inspected':False,'retained_environment_values':'explicit replay assignments only','patterns':'private-key headers, provider/GitHub token shapes, long Bearer tokens, AWS access-key shape','limits':'Heuristic, not exhaustive; no secret values requested or retained.'})
if (OUT/'summary.json').exists():
    summary=json.loads((OUT/'summary.json').read_bytes())
else:
    initial=git(['rev-parse','HEAD'],'initial-head'); status=git(['status','--short'],'initial-status')
    summary={'start':now(),'initial_revision':initial,'initial_status':status,'pages':[],'limits':'Independent integrated SOURCE only; no target expansion, native/model/runtime/historical/parent closure credit.'}
    js(OUT/'summary.json',summary)
for v in VERSIONS:
    d=ROOT/'data/patch-api/evidence'/f'{v}-session-2026-10-09';folder=OUT/v;folder.mkdir(exist_ok=True)
    page={'version':v,'start':now(),'scope_pre':scope(d),'runs':[]}
    # Inspect every bounded evidence file before copying/retaining; archives inspected member-by-member below.
    scan={str(p.relative_to(d)):private(p.read_bytes()) for p in d.rglob('*') if p.is_file() and p.name!='replay-archive.tar.gz'}
    assert not any(scan.values()), f'privacy pattern in evidence {v}'
    js(folder/'privacy.json',{'scan_at':now(),'file_counts':scan,'archive_scanned_as_members':True})
    original,receipt,failures=maps(d)
    page.update(original_seals=len(original),receipt_seals=len(receipt),seal_failures=failures)
    assert not failures, failures
    context=json.loads((d/'portable-context.json').read_bytes());identity=context.get('identity',context)
    if (d/'archive-identity.json').exists(): identity.update(json.loads((d/'archive-identity.json').read_bytes()))
    archive=d/'replay-archive.tar.gz';archive_hash=sha(archive.read_bytes());map_hash=sha((d/'seals.json').read_bytes())
    assert archive_hash==identity['archive_sha256']
    map_pin=identity.get('original_seals_sha256',identity.get('original_map_sha256',identity.get('seals_sha256')))
    assert map_hash==map_pin
    copied=folder/'archive-copy';assert not copied.exists();copied.mkdir()
    members=[]
    with tarfile.open(archive) as tf:
        all_members=tf.getmembers(); names=[m.name for m in all_members]
        assert len(names)==len(set(names))
        assert set(names)==set(original)|{'seals.json'}
        for member in all_members:
            p=Path(member.name)
            assert member.isfile() and not p.is_absolute() and not set(p.parts)&{'..','.git','target','Interface','__pycache__'}
            data=tf.extractfile(member).read()
            assert not private(data)
            assert data==(d/member.name).read_bytes(), f'archive/integrated mismatch: {member.name}'
            assert sha(data)==(map_hash if member.name=='seals.json' else original[member.name])
            members.append({'name':member.name,'bytes':len(data),'sha256':sha(data)})
        tf.extractall(copied,filter='data')
    assert len(members)==len(original)+1
    page.update(archive_sha256=archive_hash,original_map_sha256=map_hash,archive_members=len(members),archive_bytes=archive.stat().st_size,archive_equals_integrated_originals=True,identity_pins=identity)
    js(folder/'archive-validation.json',{'validated_at':now(),'members':members,'archive_sha256':archive_hash,'original_map_sha256':map_hash,'original_seals':original,'separate_receipt_seals':receipt,'receipt_overlap':[]})
    # Confirm copied active programs only use stdlib and no command/network APIs. Historical extractor.main is not invoked.
    inspected=[]
    for name in ['audit.py','test_source_accounting.py','test_portable.py','historical-tools/gen_patch_wikitext_register.py','historical-tools/extract_patch_non_inventory.py']:
        p=copied/name;t=ast.parse(p.read_text())
        imports=[ast.unparse(n) for n in ast.walk(t) if isinstance(n,(ast.Import,ast.ImportFrom))]
        forbidden=[ast.unparse(n) for n in ast.walk(t) if isinstance(n,ast.Call) and any(s in ast.unparse(n.func) for s in ['subprocess','os.system','popen','urlopen','socket','requests.','eval','exec('])]
        assert not forbidden,forbidden
        inspected.append({'file':name,'sha256':sha(p.read_bytes()),'imports':imports,'forbidden_calls':forbidden})
    js(folder/'script-inspection.json',inspected)
    for index,script in enumerate(['audit.py','test_source_accounting.py','test_portable.py'],1):
        rec=run_script(v,copied,script,index);page['runs'].append(rec)
        js(folder/'page.json',page)
    portable=json.loads((folder/'fresh-portable-proof.json').read_bytes())
    assert portable['success'] and portable['tests']==3
    controls=portable['receipts']; assert len(controls)==5
    for name in ['ledger.json','green.log']:
        rejected=next(c for c in controls if c['control']=='serialized-tamper-rejected' and c['file']==name)
        restored=next(c for c in controls if c['control']=='exact-restoration' and c['file']==name)
        assert rejected['original_sha256']==restored['restored_sha256']==original[name]
        assert rejected['tampered_sha256']!=original[name]
        assert restored['original_seals_sha256']==map_hash
    page.update(end=now(),scope_post=scope(d),fresh_portable_receipt_sha256=sha((folder/'fresh-portable-proof.json').read_bytes()))
    page['integrated_scope_unchanged']=page['scope_pre']==page['scope_post']
    page['pass']=not failures and page['integrated_scope_unchanged'] and all(r.get('exit')==0 and r.get('integrated_inputs_unchanged') and r.get('copied_inputs_unchanged') for r in page['runs'])
    js(folder/'page.json',page);summary['pages'].append(page);js(OUT/'summary.json',summary)
    print(v,'PASS' if page['pass'] else 'FAIL','seals',len(original),'+',len(receipt),'archive',len(members))
summary.update(end=now(),final_revision=git(['rev-parse','HEAD'],'final-head'),final_status=git(['status','--short'],'final-status'))
summary['pass']=all(p['pass'] for p in summary['pages']) and len(summary['pages'])==7
js(OUT/'summary.json',summary)
print('Complete:',summary['pass'])
