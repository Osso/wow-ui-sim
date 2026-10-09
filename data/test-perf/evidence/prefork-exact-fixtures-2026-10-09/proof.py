from pathlib import Path
import json, datetime, os, hashlib
repo='/home/osso/Projects/wow/wow-ui-sim'
out=Path('/tmp/prefork-exact-independent')
def proof(label, argv, env=None, echo=True):
    start=datetime.datetime.now().astimezone().isoformat()
    cmd=cli.command(*argv).cwd(repo)
    if env: cmd=cmd.env(env)
    result=cmd.capture().run()
    end=datetime.datetime.now().astimezone().isoformat()
    (out/(label+'.stdout')).write_text(result.stdout)
    (out/(label+'.stderr')).write_text(result.stderr)
    ledger=json.loads((out/'commands.json').read_text()) if (out/'commands.json').exists() else []
    ledger.append({'label':label,'argv':argv,'cwd':repo,'environment_overrides':env or {},'start':start,'end':end,'exit_code':result.exit_code})
    (out/'commands.json').write_text(json.dumps(ledger,indent=2))
    print(label,result.exit_code)
    if echo: print(result.stdout,result.stderr)
    return result
