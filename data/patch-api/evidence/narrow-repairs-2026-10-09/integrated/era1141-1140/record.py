from pathlib import Path
import datetime,json,hashlib,time
root=Path('/tmp/era1141-1140-independent')
repo='/home/osso/Projects/wow/wow-ui-sim'
revision='ce40cfb899bcc342457fc1cd03c2b23b26969406'
def record(label,args,cwd=repo,env=None):
    start=datetime.datetime.now(datetime.timezone.utc).isoformat()
    cmd=cli.command(*args).cwd(cwd)
    if env: cmd=cmd.env(env)
    result=cmd.capture().run()
    (root/(label+'.stdout')).write_text(result.stdout)
    (root/(label+'.stderr')).write_text(result.stderr)
    meta={'argv':args,'cwd':cwd,'env_overrides':env or {},'revision':revision,'start':start,'end':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exit_code':result.exit_code}
    (root/(label+'.json')).write_text(json.dumps(meta,indent=2)+'\n')
    print(label,result)
    return result
