import os,json,hashlib
from datetime import datetime,timezone
from pathlib import Path
repo='/home/osso/.worktrees/wow-ui-sim-p1601-source'
root=Path('/tmp/forever-bodies-independent')
def stamp(): return datetime.now(timezone.utc).isoformat()
def gitout(*args):
    r=cli.git(*args).cwd(repo).capture().run()
    if r.exit_code: raise RuntimeError(r)
    return r.stdout
def evidence_command(name,argv):
    metadata={'argv':argv,'cwd':repo,'revision':gitout('rev-parse','HEAD').strip(),'start':stamp(),'env':dict(os.environ),'status':gitout('status','--short')}
    (root/(name+'-start.json')).write_text(json.dumps(metadata,indent=2))
    r=cli.command(*argv).cwd(repo).capture().run()
    metadata.update(end=stamp(),exit_code=r.exit_code,stdout=r.stdout,stderr=r.stderr)
    (root/(name+'-result.json')).write_text(json.dumps(metadata,indent=2))
    (root/(name+'.stdout')).write_text(r.stdout)
    (root/(name+'.stderr')).write_text(r.stderr)
    print(name,metadata['start'],metadata['end'],'EXIT',r.exit_code)
    print('STDOUT\n'+r.stdout+'\nSTDERR\n'+r.stderr)
    return r
