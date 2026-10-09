from pathlib import Path
import hashlib, json, datetime, time, os
REPO = Path('/home/osso/Projects/wow/wow-ui-sim')
OUT = Path('/tmp/era1132-retail-redirect-independent')
VERSIONS = ['1.13.2', '1.12.0', '1.11.0', '1.10.2']
def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()
def scope_hashes():
    files = set()
    for v in VERSIONS:
        files.update(p for p in (REPO/'data/patch-api/evidence'/f'{v}-session-2026-10-09').rglob('*') if p.is_file() and '__pycache__' not in p.parts)
        files.add(REPO/'docs/specs'/f'patch-{v.replace(".", "-")}-source-accounting.md')
        files.add(REPO/'docs/wiki/investigations'/f'patch-{v.replace(".", "-")}-api-audit.md')
    for root in ['src', 'patch-tests', '.cargo']:
        files.update(p for p in (REPO/root).rglob('*') if p.is_file())
    for name in ['Cargo.toml', 'Cargo.lock', 'build.rs', 'docs/wiki/index.md', 'data/patch-api/source-cache/legacy-2026-10-09/manifest.json', 'data/patch-api/sources/api-change-pages-remaining.json']:
        p = REPO/name
        if p.is_file(): files.add(p)
    return {str(p.relative_to(REPO)): sha(p) for p in sorted(files)}
def head():
    r = cli.git('rev-parse', 'HEAD').cwd(str(REPO)).capture().run()
    assert r.exit_code == 0, r.stderr
    return r.stdout.strip()
def proof(label, argv, cwd=REPO, env=None):
    d = OUT/label
    d.mkdir(parents=True, exist_ok=False)
    before = scope_hashes()
    record = {'label':label, 'argv':list(map(str,argv)), 'cwd':str(cwd), 'head_before':head(), 'started':datetime.datetime.now(datetime.timezone.utc).isoformat(), 'environment_keys_only':sorted(os.environ), 'explicit_environment':env or {}, 'scope_before':before}
    (d/'receipt-start.json').write_text(json.dumps(record,indent=2))
    command = cli.command(*record['argv']).cwd(str(cwd))
    if env: command = command.env(env)
    t = time.monotonic()
    result = command.capture().run()
    (d/'stdout.txt').write_text(result.stdout)
    (d/'stderr.txt').write_text(result.stderr)
    record.update(exit_code=result.exit_code, seconds=time.monotonic()-t, ended=datetime.datetime.now(datetime.timezone.utc).isoformat(), head_after=head(), scope_after=scope_hashes(), stdout_sha256=sha(d/'stdout.txt'), stderr_sha256=sha(d/'stderr.txt'))
    record['changed_scope'] = sorted(k for k in set(before)|set(record['scope_after']) if before.get(k)!=record['scope_after'].get(k))
    (d/'receipt.json').write_text(json.dumps(record,indent=2))
    print(label, 'exit', result.exit_code, 'seconds', round(record['seconds'],2), 'HEAD',record['head_before'],record['head_after'],'changed',record['changed_scope'])
    print(result.stdout); print(result.stderr)
    return record
