import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/home/osso/.worktrees/wow-ui-sim-p732-page')
HERE = ROOT / 'data/patch-api/evidence/7.3.2-session-2026-10-08'
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p732-page'
for job in json.loads(Path(sys.argv[1]).read_text()):
    name = job['name']
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET, PYTHONDONTWRITEBYTECODE='1', **job.get('env', {}))
    log = HERE / (name + '.log')
    start = datetime.datetime.now(datetime.timezone.utc).isoformat()
    with log.open('w') as output:
        result = subprocess.run(job['command'], cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
    receipt = dict(command=job['command'], revision=revision, cwd=str(ROOT), target=TARGET,
                   environment=job.get('env', {}), exit=result.returncode, log=log.name,
                   log_sha256=hashlib.sha256(log.read_bytes()).hexdigest(), invalidated=False,
                   started=start, finished=datetime.datetime.now(datetime.timezone.utc).isoformat())
    (HERE / (name + '.proof.json')).write_text(json.dumps(receipt, indent=2) + '\n')
    if result.returncode != job.get('expected_exit', 0):
        sys.exit(result.returncode or 2)
