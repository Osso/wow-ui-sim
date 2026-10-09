from pathlib import Path
import json, datetime, hashlib, time, os
ROOT='/home/osso/Projects/wow/wow-ui-sim'
OUT=Path('/tmp/transmog-epoch-independent')
def sha(path):
    h=hashlib.sha256()
    with open(path,'rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''): h.update(block)
    return h.hexdigest()
def snapshot():
    head=cli.git('rev-parse','HEAD').cwd(ROOT).capture().run()
    files=cli.git('ls-files','-z').cwd(ROOT).capture().run()
    paths=files.stdout.split('\0')
    hashes={p:sha(Path(ROOT)/p) for p in paths if p and (p.startswith(('src/','tests/','crates/','iced-','.cargo/')) or p in ('Cargo.toml','Cargo.lock','build.rs')) and (Path(ROOT)/p).is_file()}
    return {'head':head.stdout.strip(),'hashes':hashes,'scope':'tracked src, tests, crates, iced-* dependencies, .cargo, root Cargo.toml/Cargo.lock/build.rs'}
def record(name,argv,scope,timeout=None):
    before=snapshot(); start=datetime.datetime.now(datetime.timezone.utc).isoformat(); t=time.monotonic()
    cmd=cli.command(*argv).cwd(ROOT)
    if timeout is not None: cmd=cmd.timeout(timeout)
    result=cmd.capture().run()
    stop=datetime.datetime.now(datetime.timezone.utc).isoformat(); after=snapshot()
    for stream in ('stdout','stderr'): (OUT/(name+'.'+stream)).write_text(getattr(result,stream))
    meta={'argv':argv,'cwd':ROOT,'start':start,'end':stop,'duration_seconds':time.monotonic()-t,'exit':result.exit_code,'scope':scope,'before':before,'after':after,'changed_hashes':[p for p in set(before['hashes'])|set(after['hashes']) if before['hashes'].get(p)!=after['hashes'].get(p)],'streams':{s:{'path':str(OUT/(name+'.'+s)),'sha256':sha(OUT/(name+'.'+s)),'bytes':(OUT/(name+'.'+s)).stat().st_size} for s in ('stdout','stderr')}}
    (OUT/(name+'.json')).write_text(json.dumps(meta,indent=2))
    print(name,'exit',result.exit_code,'seconds',meta['duration_seconds'],'HEAD',before['head'],after['head'],'changed hashes',meta['changed_hashes']); print(result.stderr)
    return result
