"""Untruncated whole-word grep scans; never mutate other checkouts."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True)


def main():
    register = json.loads((ROOT / 'data/patch-api/sources/6.0.2-wikitext-register.json').read_text())
    removed = {row['symbol'] for row in register['entries'] if row['direction'] == 'removed'}
    removed.update(['LE_PET_JOURNAL_FLAG_FAVORITES', 'LE_RAID_BUFF_PHYSICAL_HASTE',
                    'LE_RAID_BUFF_SPELL_HASTE'])
    scans = []
    for symbol in sorted(removed):
        for domain, paths in [('cached', [str(Path.home() / '.cache/wow-ui-sim/blizzard-ui/retail/AddOns')]),
                              ('callers', ['src', 'tests'])]:
            for kind, name in [('qualified', symbol), ('bare', re.split(r'[.:]', symbol)[-1])]:
                argv = ['/usr/bin/grep', '-R', '-n', '-w', '-E', re.escape(name)]
                if domain == 'cached':
                    argv += ['--exclude=*Documentation*', '--exclude-dir=*Documentation*']
                argv += paths
                result = subprocess.run(argv, cwd=ROOT, capture_output=True)
                assert result.returncode in (0, 1), result.stderr
                output = f"grep-{symbol.replace(':', '_').replace('.', '_')}-{domain}-{kind}.txt"
                (HERE / output).write_bytes(result.stdout)
                scans.append({'scanner': 'grep', 'symbol': symbol, 'domain': domain, 'kind': kind,
                              'argv': argv, 'exit': result.returncode, 'output': output,
                              'matches': len(result.stdout.splitlines()),
                              'sha256': hashlib.sha256(result.stdout).hexdigest()})
    later = []
    revisions = {}
    # p622-page is already integrated and its deleted branch is covered by master.
    for ref in ['master', 'p610-page', 'p620-page']:
        revision = git('rev-parse', ref).strip()
        revisions[ref] = revision
        names = git('ls-tree', '-r', '--name-only', revision, 'data/patch-api/sources').splitlines()
        if ref != 'master':
            patch = {'p610-page': '6.1.0', 'p620-page': '6.2.0', 'p622-page': '6.2.2'}[ref]
            names = [n for n in names if n.endswith('/' + patch + '-wikitext-register.json')]
        for path in names:
            if not path.endswith('-wikitext-register.json'):
                continue
            content = json.loads(git('show', revision + ':' + path))
            matches = [e for e in content['entries'] if e['symbol'] in removed]
            later.append({'ref': ref, 'revision': revision, 'path': path, 'entries': matches})
    (HERE / 'p602-retirement-scans.json').write_text(json.dumps(scans, indent=2) + '\n')
    (HERE / 'p602-later-register-scan.json').write_text(json.dumps(later, indent=2) + '\n')
    decisions = []
    for symbol in sorted(removed):
        own = [s for s in scans if s['symbol'] == symbol]
        consumers = [s['output'] for s in own if s['domain'] == 'cached' and s['matches']]
        callers = [s['output'] for s in own if s['domain'] == 'callers' and s['matches']]
        adds = [dict(ref=row['ref'], revision=row['revision'], path=row['path'], entry=e)
                for row in later for e in row['entries']
                if e['symbol'] == symbol and e['direction'] == 'added']
        decisions.append({'symbol': symbol, 'cached_consumers': consumers,
                          'caller_scans': callers, 'later_additions': adds,
                          'action': 'retain' if consumers or adds else 'review-absence',
                          'reason': 'Current consumers/later additions forbid retirement.'
                          if consumers or adds else 'Consumer-free; inspect current runtime before changing.'})
    (HERE / 'p602-retirement-decisions.json').write_text(json.dumps(
        {'revisions': revisions, 'members': decisions, 'new_retirements': []}, indent=2) + '\n')
    print(json.dumps({'members': len(removed), 'scans': len(scans),
                      'retained': sum(d['action'] == 'retain' for d in decisions), 'revisions': revisions}))


if __name__ == '__main__':
    main()
