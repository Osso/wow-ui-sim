from pathlib import Path
import hashlib, json, datetime, re
REPO = '/home/osso/Projects/wow/wow-ui-sim'
OUT = Path('/tmp/headless-gui-independent')
PATHS = [p for p in (OUT/'tracked-files.stdout').read_text().split('\0') if p and (p.startswith(('src/','tests/','crates/','iced-','.cargo/')) or p in ('Cargo.toml','Cargo.lock','build.rs'))]
def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for chunk in iter(lambda: f.read(1024*1024), b''): h.update(chunk)
    return h.hexdigest()
def snapshot():
    r = cli.git('rev-parse','HEAD').cwd(REPO).capture().run()
    hashes = {p:sha(Path(REPO)/p) for p in PATHS if (Path(REPO)/p).is_file()}
    return {'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':r.stdout.strip(),'head_exit':r.returncode,'scope_hashes':hashes,'aggregate':hashlib.sha256(json.dumps(hashes,sort_keys=True).encode()).hexdigest()}
def execute(label, argv, timeout=None):
    before = snapshot()
    command = {'argv':argv,'cwd':REPO,'timeout_seconds':timeout,'before':before}
    (OUT/(label+'.json')).write_text(json.dumps(command,indent=2))
    builder = cli.command(*argv).cwd(REPO)
    if timeout is not None: builder = builder.timeout(timeout)
    r = builder.capture().run()
    (OUT/(label+'.stdout')).write_text(r.stdout)
    (OUT/(label+'.stderr')).write_text(r.stderr)
    (OUT/(label+'.exit')).write_text(str(r.returncode))
    command.update({'exit':r.returncode,'after':snapshot()})
    (OUT/(label+'.json')).write_text(json.dumps(command,indent=2))
    print(label,'exit',r.returncode,'scope unchanged',command['before']['aggregate']==command['after']['aggregate'])
    return r

def inspect_compile(label):
    records=[]; malformed=[]
    for line in (OUT/(label+'.stdout')).read_text().splitlines():
        try: records.append(json.loads(line))
        except json.JSONDecodeError: malformed.append(line)
    diagnostics=[r for r in records if r.get('reason')=='compiler-message']
    rendered='\n'.join(r['message'].get('rendered','') for r in diagnostics)
    (OUT/(label+'-diagnostics.json')).write_text(json.dumps(diagnostics,indent=2))
    (OUT/(label+'-diagnostics.txt')).write_text(rendered)
    artifacts=[r for r in records if r.get('reason')=='compiler-artifact' and r.get('executable') and r.get('target',{}).get('name')=='wow_ui_sim' and r.get('profile',{}).get('test')]
    evidence={'artifacts':artifacts,'diagnostic_count':len(diagnostics),'unparsed_lines':malformed,'build_finished':[r for r in records if r.get('reason')=='build-finished']}
    for r in artifacts: r['binary_sha256']=sha(r['executable'])
    (OUT/(label+'-inspection.json')).write_text(json.dumps(evidence,indent=2))
    print(json.dumps(evidence,indent=2))
    print('FULL STDERR:\n'+(OUT/(label+'.stderr')).read_text())
    print('FULL RENDERED DIAGNOSTICS:\n'+rendered)
    return artifacts
