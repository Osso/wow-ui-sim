from pathlib import Path
import time, json, os
repo='/home/osso/Projects/wow/wow-ui-sim'
out=Path('/tmp/fixture-corrections-independent')
ledger=json.loads((out/'commands.json').read_text())
def proof(label, argv, env=None):
    start=time.time()
    cmd=cli.command(*argv).cwd(repo)
    for k,v in (env or {}).items(): cmd=cmd.env(k,v)
    result=cmd.capture().run()
    (out/(label+'.stdout')).write_text(result.stdout)
    (out/(label+'.stderr')).write_text(result.stderr)
    entry={'label':label,'argv':argv,'env':env or {},'cwd':repo,'start':start,'end':time.time(),'exit':result.exit_code}
    ledger.append(entry)
    (out/'commands.json').write_text(json.dumps(ledger,indent=2))
    print(json.dumps(entry)); print(result.stdout); print(result.stderr)
    return result
