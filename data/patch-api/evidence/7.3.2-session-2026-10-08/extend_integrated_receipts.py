import json
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path('/home/osso/.worktrees/wow-ui-sim-p732-page')
HERE = ROOT / 'data/patch-api/evidence/7.3.2-session-2026-10-08'
register_path = HERE / 'p732-register-reproduction.json'
registers = json.loads(register_path.read_text())
register_path.write_text(json.dumps([r for r in registers if r['patch'] != '8.0.1'], indent=2) + '\n')
summary_path = HERE / 'p732-sweep-summary.json'
summaries = json.loads(summary_path.read_text())
for row in summaries:
    row['patch'] = row['file'].removeprefix('patch_').removesuffix('_publication_sweep-results.json').replace('_', '.')
summary_path.write_text(json.dumps(summaries, indent=2) + '\n')
command = ['python3', '-B', str(ROOT / 'tools/extend_patch_audit_receipts.py'), str(ROOT), str(HERE),
           'p732', 'Integrated 8.0.1 register and recorded historical extraction flags', '7.3.2']
result = subprocess.run(command, cwd=ROOT, env=os.environ)
# Preserve the 7.3.2 summary's original external schema.
summaries = json.loads(summary_path.read_text())
for row in summaries:
    patch = row.pop('patch')
    row['file'] = 'patch_' + patch.replace('.', '_') + '_publication_sweep-results.json'
    row.pop('result', None)
summary_path.write_text(json.dumps(sorted(summaries, key=lambda r: r['file']), indent=2) + '\n')
if result.returncode:
    sys.exit(result.returncode)
# Regenerate complete rows instead of retaining copied historical recipe fields.
sys.exit(subprocess.run(['python3', '-B', str(HERE / 'reproduce_integrated_sources.py')], cwd=ROOT).returncode)
