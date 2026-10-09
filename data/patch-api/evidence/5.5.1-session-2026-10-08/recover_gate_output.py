"""Preserve the in-flight gate output after its original Pyrun reader exited."""
from pathlib import Path
import json
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent

if '--worker' in sys.argv:
    # The resumed session orphaned this already-running gate; do not rerun it.
    with open('/proc/2785718/fd/1', 'rb', buffering=0) as pipe:
        with (HERE / 'master-gate-report.json').open('wb') as output:
            shutil.copyfileobj(pipe, output)
    report = json.loads((HERE / 'master-gate-report.json').read_text())
    print(json.dumps({'status': report['status'], 'summary': report['summary']}))
else:
    with (HERE / 'master-gate-recovery.txt').open('wb') as output:
        process = subprocess.Popen([sys.executable, '-B', __file__, '--worker'],
                                   cwd=HERE.parents[3], stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'reader_pid': process.pid, 'gate_pid': 2785718}))
