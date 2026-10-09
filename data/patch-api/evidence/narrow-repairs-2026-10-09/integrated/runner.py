from pathlib import Path
import json, datetime, os
root=Path('/tmp/forever-cfg-independent')
def receipt(name,argv,cwd):
    meta={'argv':argv,'cwd':cwd,'revision':cli.git('rev-parse','HEAD').cwd(cwd).capture().run().stdout.strip(),'environment':dict(os.environ),'start_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
    (root/(name+'-start.json')).write_text(json.dumps(meta,indent=2))
    result=cli.command(*argv).cwd(cwd).capture().run()
    meta.update(end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),exit_code=result.exit_code,stdout=result.stdout,stderr=result.stderr)
    (root/(name+'-result.json')).write_text(json.dumps(meta,indent=2))
    (root/(name+'.stdout')).write_text(result.stdout)
    (root/(name+'.stderr')).write_text(result.stderr)
    print(name,'exit',result.exit_code,'stdout bytes',len(result.stdout.encode()),'stderr bytes',len(result.stderr.encode()))
    print(result.stdout); print(result.stderr)
    return meta
