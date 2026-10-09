from pathlib import Path
import hashlib, json, shutil, re, time
REPO=Path('/home/osso/Projects/wow/wow-ui-sim')
OUT=Path('/tmp/older-source-independent-fna_vx3l')
VERSIONS=['2.3.0','2.2.0','2.1.0','2.0.1','1.60.1','1.15.9','1.15.8']
records=[]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def run(label, script, cwd, expect=0, args=(), nogit=False):
    env={'PATH':str(OUT/'empty-bin') if nogit else '/usr/bin:/bin','HOME':str(OUT/'home'),'TMPDIR':str(OUT/'tmp'),'PYTHONDONTWRITEBYTECODE':'1','PYTHONPATH':'','LC_ALL':'C.UTF-8'}
    argv=['/usr/bin/env','-i',*[f'{k}={v}' for k,v in env.items()],'/usr/bin/python3','-B',str(script),*args]
    started=time.time()
    r=cli.command(*argv).cwd(str(cwd)).capture().run()
    (OUT/(label+'.stdout')).write_text(r.stdout)
    (OUT/(label+'.stderr')).write_text(r.stderr)
    m=re.search(r'Ran (\d+) tests',r.stderr)
    row={'label':label,'argv':argv,'environment':env,'cwd':str(cwd),'revision':revision,'script_sha256':sha(Path(script)),'exit_code':r.exit_code,'expected_exit':expect,'passed':r.exit_code==expect if expect is not None else r.exit_code!=0,'tests':int(m[1]) if m else None,'stdout':r.stdout,'stderr':r.stderr,'seconds':time.time()-started}
    records.append(row)
    (OUT/'commands.json').write_text(json.dumps(records,indent=2))
    print(label,'exit',r.exit_code,'tests',row['tests'],'PASS' if row['passed'] else 'FAIL')
    return row
for d in ['tmp','empty-bin','home']: (OUT/d).mkdir(exist_ok=True)
r=cli.git('rev-parse','HEAD').cwd(str(REPO)).capture().run(); revision=r.stdout.strip()
(OUT/'revision.txt').write_text(revision+'\n')
scopes={}
for v in VERSIONS:
    directory=REPO/'data/patch-api/evidence'/f'{v}-session-2026-10-09'
    for p in directory.rglob('*'):
        if p.is_file() and '__pycache__' not in p.parts: scopes[str(p.relative_to(REPO))]=sha(p)
    for folder in ['data/patch-api/sources','data/patch-api/source-cache/legacy-2026-10-09']:
        for p in (REPO/folder).glob(v+'-*'):
            if p.is_file(): scopes[str(p.relative_to(REPO))]=sha(p)
    slug=v.replace('.','-')
    for relative in [f'docs/specs/patch-{slug}-source-accounting.md',f'docs/wiki/investigations/patch-{slug}-api-audit.md']:
        if (REPO/relative).exists(): scopes[relative]=sha(REPO/relative)
for p in (REPO/'tools').glob('*.py'): scopes[str(p.relative_to(REPO))]=sha(p)
(OUT/'scope-before.json').write_text(json.dumps(scopes,indent=2))
for v in VERSIONS:
    rel=Path('data/patch-api/evidence')/f'{v}-session-2026-10-09'
    src=REPO/rel
    test=REPO/'tools'/f'test_patch_{v.replace(".","_")}_source.py' if v in ['2.1.0','2.0.1'] else src/('test_source.py' if v=='1.60.1' else 'test_source_accounting.py')
    run(v+'-current-source',test,REPO)
    root=OUT/('copy-'+v); here=root/rel
    here.parent.mkdir(parents=True,exist_ok=True)
    shutil.copytree(src,here,ignore=shutil.ignore_patterns('__pycache__'))
    if v=='2.3.0':
        for name in json.loads((src/'seals.json').read_text()):
            p=root/name; p.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(REPO/name,p)
    assert not (root/'.git').exists() and not (root/'target').exists() and not (root/'tools').exists()
    validator=here/('audit.py' if v in ['1.15.9','1.15.8'] else 'validate.py')
    clean=run(v+'-copied-validator',validator,root,nogit=True)
    (OUT/(v+'-counts.json')).write_text(clean['stdout'])
    copied_test=here/test.name
    if copied_test.exists(): run(v+'-copied-source',copied_test,root,nogit=True)
    if v=='2.3.0': targets=[root/'data/patch-api/sources/2.3.0-page-coverage.json',here/'green.log']
    elif v in ['2.2.0','2.1.0','2.0.1']: targets=[here/'historical-page-coverage.json',here/('source-green.log' if v!='2.2.0' else 'source-green.log')]
    elif v=='1.60.1': targets=[here/'original/ledger.json',here/'original/red.log']
    else: targets=[here/'ledger.json',here/'green.log']
    for n,p in enumerate(targets):
        original=p.read_bytes()
        try:
            if p.suffix=='.json':
                ledger=json.loads(original)
                key=next(k for k in ['contracts','source_rows','inventory_rows','occurrences'] if isinstance(ledger.get(k),list) and ledger[k])
                ledger[key].pop(0)
                p.write_text(json.dumps(ledger))
            else: p.write_bytes(original+b'\nFABRICATED SUCCESS: all native behavior complete\n')
            failed=run(v+f'-tamper-{n}',validator,root,expect=None,nogit=True)
            failed['tamper_path']=str(p.relative_to(root)); failed['tampered_sha256']=sha(p)
            failed['seal_rejection']='seal' in failed['stderr'].lower()
        finally: p.write_bytes(original)
        assert sha(p)==hashlib.sha256(original).hexdigest()
        restored=run(v+f'-restored-{n}',validator,root,nogit=True)
        restored['same_stdout']=restored['stdout']==clean['stdout']
    if v=='2.0.1': run(v+'-copied-current-validator',here/'current/validate.py',root,nogit=True)
(OUT/'commands.json').write_text(json.dumps(records,indent=2))
after={name:sha(REPO/name) for name in scopes}
(OUT/'scope-after.json').write_text(json.dumps(after,indent=2))
drift=[name for name in scopes if scopes[name]!=after[name]]
(OUT/'scope-drift.json').write_text(json.dumps(drift,indent=2))
print('SCOPE DRIFT',drift)
print('COMMANDS',len(records),'FAILURES',[r['label'] for r in records if not r['passed']])
