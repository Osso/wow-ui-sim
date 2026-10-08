"""Read-only portable proof of the pinned 6.0.2 audit, not current runtime execution."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests


def read(path):
    return json.loads(path.read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', revision + ':' + path)


def historical_tool(name, revision):
    path = 'tools/' + name + '.py'
    namespace = {'__name__': name, '__file__': str(ROOT / path)}
    exec(compile(blob(revision, path), path, 'exec'), namespace)
    return namespace


def check_sources(revision):
    sources = 'data/patch-api/sources/'
    provenance = json.loads(blob(revision, sources + '6.0.2-api-changes.provenance.json'))
    register = json.loads(blob(revision, sources + '6.0.2-wikitext-register.json'))
    raw = blob(revision, sources + '6.0.2-api-changes.wikitext').decode()
    page = read(HERE / 'p602-fetch.json')['query']['pages'][str(provenance['pageid'])]
    rev = page['revisions'][0]
    assert page['title'] == provenance['title'] == 'Patch 6.0.2/API changes'
    assert rev['revid'] == provenance['revid'] == register['source']['revid']
    assert rev['timestamp'] == provenance['timestamp']
    assert rev['slots']['main']['*'] == raw
    assert sha(raw.encode()) == provenance['sha256'] == register['source']['sha256']
    diff_prov = provenance['transcluded_diff']
    diff_page = read(HERE / 'p602-diff-fetch.json')['query']['pages'][str(diff_prov['pageid'])]
    diff = blob(revision, diff_prov['path']).decode()
    assert diff_page['title'] == diff_prov['title']
    assert diff_page['revisions'][0]['revid'] == diff_prov['revid']
    assert diff_page['revisions'][0]['slots']['main']['*'] == diff
    assert sha(diff.encode()) == diff_prov['sha256']
    generator = historical_tool('gen_patch_wikitext_register', revision)
    entries, counts = generator['parse_warlords_diff'](diff)
    assert generator['parse_warlords_prepatch'](raw) + entries == register['entries']
    assert counts == register['header_counts']
    assert all(c['header_count'] == c['parsed_count'] for c in counts)
    extractor = historical_tool('extract_patch_non_inventory', revision)
    text = blob(revision, sources + '6.0.2-api-changes.txt').decode()
    assert extractor['extract_text'](raw, retain_patch_diff_reference=True) == text
    return register, extractor, text, diff


def check_accounting(revision, register, extractor, text, diff):
    rows = json.loads(blob(revision, 'data/patch-api/sources/6.0.2-page-coverage.json'))['source_rows']
    # Accounting is committed after the runtime revision and pinned separately.
    by_id = {row['source_id']: row for row in rows}
    assert len(by_id) == len(rows)
    results = read(HERE / 'patch_6_0_2_publication_sweep-results.json')
    inventory = {e['id'] for e in register['entries']}
    known = set(json.loads(blob(revision, 'tests/data/patch_6_0_2_sweep_known_gaps.json')))
    assert set(results) == inventory
    assert {key for key, value in results.items() if not value['ok']} == known
    gaps = read(HERE / 'p602-gap-review.json')
    assert {g['source_id'] for g in gaps} == known
    for gap in gaps:
        assert gap['reason'] == by_id[gap['source_id']]['note']
        assert gap['observed'] == results[gap['source_id']]['observed']
    for key in inventory:
        assert bool(by_id[key]['capabilities']) == results[key]['ok']
    scout = read(HERE / 'p602-extract-scout.json')
    prose = {row['source_id'] for row in extractor['seed_rows'](text, '6.0.2')}
    assert {row['source_id'] for row in scout} == prose
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['note'] == by_id[row['source_id']]['note']
    enums = read(HERE / 'p602-enum-register.json')
    for row in enums:
        assert row['literal'] == diff.splitlines()[row['wikitext_line'] - 1]
        assert by_id[row['source_id']]['status'] == 'audit-pending'
    enum_ids = {row['source_id'] for row in enums}
    groups = {f'diff-enum-group-{n:03}' for n, line in enumerate(diff.splitlines(), 1)
              if n >= 679 and line.startswith(': [[')}
    assert set(by_id) == inventory | prose | enum_ids | groups
    summary = {'inventory_rows': len(inventory), 'inventory_gaps': len(known),
               'inventory_ok': len(inventory) - len(known), 'prose_rows': len(scout),
               'enum_members': len(enums), 'ledger_rows': len(rows),
               'statuses': dict(Counter(row['status'] for row in rows))}
    assert summary == read(HERE / 'p602-accounting-summary.json')
    negative = read(HERE / 'p602-negative-results.json')
    negative_register = read(HERE / 'p602-negative-register.json')
    changed = [entry for entry in negative_register['entries'] if entry['symbol'] == 'P602NegativeMissingAPI']
    assert len(changed) == 1
    assert {key for key, value in negative.items() if not value['ok']} == known | {changed[0]['id']}
    return summary


def check_reproduction(context):
    revision = context['reproduction_revision']
    expected = {p.name.removesuffix('-wikitext-register.json') for p in historical_registers(ROOT, revision)}
    registers = read(HERE / 'p602-register-reproduction.json')
    extracts = read(HERE / 'p602-saved-extract-reproduction.json')
    assert {r['patch'] for r in registers} == {r['patch'] for r in extracts} == expected
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical']
        assert sha(blob(revision, 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json')) == row['sha256']
    inherited = json.loads(blob(context['base_revision'], 'data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/p624-saved-extract-reproduction.json'))
    failures = {r['patch']: r['error'] for r in inherited if not r['byte_identical']}
    assert {r['patch']: r['error'] for r in extracts if not r['byte_identical']} == failures
    for row in read(HERE / 'p602-input-preservation.json')['rows']:
        assert blob(context['base_revision'], row['path']) == blob(revision, row['path'])
    return len(expected), failures


def check_scans():
    scans = read(HERE / 'p602-retirement-scans.json')
    decisions = read(HERE / 'p602-retirement-decisions.json')
    for row in scans:
        assert row['scanner'] == 'grep' and row['argv'][0] == '/usr/bin/grep'
        assert '-w' in row['argv'] and row['exit'] in (0, 1)
        assert sha((HERE / row['output']).read_bytes()) == row['sha256']
        assert len((HERE / row['output']).read_bytes().splitlines()) == row['matches']
        if row['domain'] == 'cached':
            assert '--exclude=*Documentation*' in row['argv'] and '--exclude-dir=*Documentation*' in row['argv']
    assert {s['symbol'] for s in scans} == {d['symbol'] for d in decisions['members']}
    for symbol in decisions['new_retirements']:
        evidence = [s for s in scans if s['symbol'] == symbol]
        assert len(evidence) == 4 and all(s['matches'] == 0 and s['exit'] == 1 for s in evidence)
        decision = next(d for d in decisions['members'] if d['symbol'] == symbol)
        assert not decision['later_additions']
    symbols = {d['symbol'] for d in decisions['members']}
    for row in read(HERE / 'p602-later-register-scan.json'):
        later = json.loads(blob(row['revision'], row['path']))
        assert row['entries'] == [e for e in later['entries'] if e['symbol'] in symbols]
        assert not any(e['symbol'] in decisions['new_retirements'] and e['direction'] == 'added' for e in row['entries'])
    return len(scans)


def check_receipts(context):
    for label, expected_exit in context['proofs'].items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt['exit'] == expected_exit
        assert sha((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
        git('rev-parse', '--verify', receipt['revision'] + '^{commit}')
        if label in ('p602-all-sweeps', 'p602-own-green', 'p602-own-bare', 'p602-scenario-regressions'):
            log = (HERE / receipt['log']).read_text()
            assert re.search(r'test result: ok\. [1-9]\d* passed; 0 failed;', log)
    log = (HERE / 'p602-all-sweeps.txt').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))
    for path in historical_sweep_tests(ROOT, context['runtime_revision']):
        source = blob(context['runtime_revision'], str(path.relative_to(ROOT))).decode()
        for name in re.findall(r'fn (patch_\w+_publication_sweep)\(', source):
            assert any(case.endswith('::' + name) for case in passed), name
    warnings = [line for line in (HERE / 'p602-mists.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    for label in ['p602-master-startup', 'p602-branch-startup']:
        assert (HERE / (label + '.txt')).read_text().rstrip().endswith('[]')
    assert git('diff', context['base_revision'], context['startup_master_revision'], '--', 'src', 'Cargo.toml', 'Cargo.lock', 'Interface') == b''
    # Prior validators are historical files, not today's expanding set.
    paths = git('ls-tree', '-r', '--name-only', context['base_revision'], 'data/patch-api/evidence').decode().splitlines()
    prior = [p for p in paths if p.endswith('/validate.py')]
    assert set(prior) == set(context['prior_validators'])
    for path in prior:
        assert blob(context['base_revision'], path) == blob(context['accounting_revision'], path)
    return len(passed), len(prior)


def main():
    context = read(HERE / 'p602-context.json')
    seal = read(HERE / 'p602-artifact-hashes.json')
    for name, digest in seal['sha256'].items():
        assert sha((HERE / name).read_bytes()) == digest, name
    register, extractor, text, diff = check_sources(context['runtime_revision'])
    summary = check_accounting(context['accounting_revision'], register, extractor, text, diff)
    registers, failures = check_reproduction(context)
    scans = check_scans()
    cases, prior = check_receipts(context)
    print(json.dumps({'status': 'PASS', **summary, 'registers': registers,
                      'inherited_extract_failures': sorted(failures), 'grep_scans': scans,
                      'sweep_cases': cases, 'historical_prior_validators': prior}, sort_keys=True))


if __name__ == '__main__':
    main()
