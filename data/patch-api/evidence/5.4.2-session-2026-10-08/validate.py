"""Read-only 2013 retail audit proof; shared inputs are always historical blobs."""
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

PREFIX = 'data/patch-api/sources/5.4.2-'


def read(name):
    return json.loads((HERE / name).read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', f'{revision}:{path}')


def historical_json(revision, path):
    return json.loads(blob(revision, path))


def check_seals(context):
    for name, expected in context['session_sha256'].items():
        path = HERE / name
        assert path.is_file() and digest(path.read_bytes()) == expected, name
        git('ls-files', '--error-unmatch', path.relative_to(ROOT).as_posix())


def check_source(context):
    revision = context['source_revision']
    provenance = historical_json(revision, PREFIX + 'api-changes.provenance.json')
    page = read('p542-fetch.json')['query']['pages']['262849']
    fetched = page['revisions'][0]
    raw = blob(revision, PREFIX + 'api-changes.wikitext')
    assert page['pageid'] == provenance['pageid'] == 262849
    assert page['title'] == provenance['title'] == 'Patch 5.4.2/API changes'
    assert fetched['revid'] == provenance['revid'] == 2543251
    assert fetched['timestamp'] == provenance['timestamp']
    assert raw.decode() == fetched['slots']['main']['*']
    assert digest(raw) == provenance['sha256']
    assert provenance['generator_flags'] == provenance['extractor_flags'] == ['--mists-automated-diff']
    assert provenance['classification'] == 'retail-api-change-page'
    parent = read('p542-parent-fetch.json')['query']['pages']['151415']
    assert parent['title'] == 'Patch 5.4.2'
    assert parent['revisions'][0]['revid'] == 6441253
    parent_raw = parent['revisions'][0]['slots']['main']['*']
    for field in ('|Release = December 10, 2013', '|toc = 50400', '|Latestv = 17688'):
        assert field in parent_raw, field
    assert '5.4.1.17538 &rarr; 5.4.2.17688' in raw.decode()
    register = historical_json(revision, PREFIX + 'wikitext-register.json')
    assert register['source']['revid'] == fetched['revid']
    assert register['source']['sha256'] == digest(raw)
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    assert sum(row['parsed_count'] for row in register['header_counts']) == len(register['entries'])
    text = blob(revision, PREFIX + 'api-changes.txt')
    namespace = {'__name__': 'historical_extractor',
                 '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(blob(revision, 'tools/extract_patch_non_inventory.py'),
                 'historical_extractor', 'exec'), namespace)
    assert namespace['extract_text'](raw.decode(), mists_automated_diff=True).encode() == text
    ledger = historical_json(revision, PREFIX + 'page-coverage.json')
    assert ledger['source_sha256'] == digest(blob(revision, PREFIX + 'wikitext-register.json'))
    assert ledger['non_inventory_source']['sha256'] == digest(text)
    inventory_ids = {row['id'] for row in register['entries']}
    extract_ids = {row['source_id'] for row in namespace['seed_rows'](text.decode(), '5.4.2')}
    rows = {row['source_id']: row for row in ledger['source_rows']}
    assert len(rows) == len(ledger['source_rows'])
    assert set(rows) == inventory_ids | extract_ids and not inventory_ids & extract_ids
    gaps = set(historical_json(revision, 'tests/data/patch_5_4_2_sweep_known_gaps.json'))
    discovered = read('p542-discovery-results.json')
    assert set(discovered) == inventory_ids
    assert {key for key, row in discovered.items() if not row['ok']} == gaps
    review = read('p542-gap-review.json')
    assert review == read('p542-problematic-contracts.json')
    assert {row['source_id'] for row in review} == gaps
    for row in review:
        assert row['reason'] and row['observation'] == discovered[row['source_id']]
    for key, row in rows.items():
        assert row['note'] and row['status'] in ('bounded-coverage', 'audit-pending', 'metadata-only')
        assert bool(row['capabilities']) == (row['status'] == 'bounded-coverage')
        if key in inventory_ids:
            assert (row['status'] == 'audit-pending') == (key in gaps)
    enum_review = read('p542-enum-review.json')
    historical_values = [int(value) for value in re.findall(r'^:: _\w+ = (\d+)$', text.decode(), re.M)]
    assert enum_review['historical_values'] == historical_values
    assert enum_review['current_documented_values'] == [0, 1, 2, 3, 4]
    assert all(rows[f'prose-undated-{line:03}']['status'] == 'audit-pending'
               for line in (4, 5, 7, 16, 17, 18, 19, 20))
    return register, ledger, gaps


def check_retirements(context, register):
    decisions = read('p542-retirement-decisions.json')
    assert decisions['new_retirements'] == []
    assert decisions['removed_entries'] == [row for row in register['entries'] if row['direction'] == 'removed']
    removed = {row['symbol'] for row in decisions['removed_entries']}
    scans = read('p542-scans.json')
    for symbol in removed:
        matching = [row for row in scans if row['symbol'] == symbol]
        assert {row['scope'] for row in matching} == {'retail', 'callers'}
        for row in matching:
            assert row['command'][0] == '/usr/bin/grep' and '-w' in row['command']
            assert symbol in row['command'] and row['exit'] == 1 and row['matches'] == 0
            assert (HERE / row['log']).read_bytes() == b''
            if row['scope'] == 'retail':
                assert '--exclude-dir=*Documentation*' in row['command']
                assert '--exclude=*Documentation*' in row['command']
    cache = read('p542-retail-cache.json')
    assert cache['files'] > 0 and cache['bytes'] > 0 and cache['provenance']
    for row in scans:
        assert row['matches'] == len((HERE / row['log']).read_text().splitlines())
    assert any(row['symbol'] == 'IsOnGlueScreen' and row['scope'] == 'retail' and row['matches'] > 0
               for row in scans)
    readditions = []
    for snapshot in decisions['register_snapshots']:
        revision = snapshot['revision']
        tree = git('ls-tree', '-r', '--name-only', revision, 'data/patch-api/sources').decode()
        assert (HERE / snapshot['tree_log']).read_text() == tree
        paths = [path for path in tree.splitlines() if path.endswith('-wikitext-register.json')]
        assert snapshot['registers'] == paths
        for path in paths:
            rows = historical_json(revision, path)['entries']
            readditions.extend({'revision': revision, 'register': path, 'entry': row}
                               for row in rows if row['symbol'] in removed and row['direction'] == 'added')
    assert readditions == decisions['later_readditions'] == []
    source = blob(context['source_revision'], 'tests/patch_5_4_2_publication_sweep.rs').decode()
    ordering = source.split('later_registers: &[', 1)[1]
    assert ordering.index('// 5.4.7') < ordering.index('// 5.4.8') < ordering.index('6.0.1') < ordering.index('6.0.2')
    assert '5.5.' not in ordering


def check_reproduction(context):
    revision = context['source_revision']
    paths = historical_registers(ROOT, revision)
    expected = {path.name.removesuffix('-wikitext-register.json') for path in paths}
    registers = read('p542-register-reproduction.json')
    extracts = read('p542-saved-extract-reproduction.json')
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    assert len(registers) == len(extracts) == len(expected)
    inherited = historical_json(context['base_revision'],
        'data/patch-api/evidence/6.0.2-session-2026-10-08/p602-saved-extract-reproduction.json')
    failures = {row['patch']: (row['byte_identical'], row['error']) for row in inherited if not row['byte_identical']}
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical'], row['patch']
        path = f"data/patch-api/sources/{row['patch']}-wikitext-register.json"
        assert digest(blob(revision, path)) == row['sha256'], path
    for row in extracts:
        assert row['byte_identical'] or failures.get(row['patch']) == (row['byte_identical'], row['error'])
        path = f"data/patch-api/sources/{row['patch']}-api-changes.txt"
        assert digest(blob(revision, path)) == row['saved_sha256'], path
    preserved = read('p542-input-preservation.json')
    base_paths = git('ls-tree', '-r', '--name-only', context['base_revision'], 'data/patch-api/sources').decode().splitlines()
    assert {row['path'] for row in preserved['rows']} == set(base_paths)
    for row in preserved['rows']:
        before = blob(context['base_revision'], row['path'])
        after = blob(revision, row['path'])
        assert before == after and digest(before) == row['before_sha256'] == row['after_sha256']
    preserved_extracts = read('p542-extract-preservation.json')
    assert {row['path'] for row in preserved_extracts} == {path for path in base_paths if path.endswith('-api-changes.wikitext')}
    assert all(row['before'] == row['after'] for row in preserved_extracts)
    return registers, extracts


def check_receipt(receipt, policy, context):
    assert receipt['command'] == policy['command'] and receipt['exit'] == policy['exit']
    log = (HERE / receipt['log']).read_bytes()
    assert digest(log) == receipt['log_sha256'], receipt['log']
    assert not git('diff', '--name-only', context['source_revision'], receipt['revision'], '--', *policy['paths'])
    return log.decode()


def check_proofs(context):
    logs = {}
    for label, policy in context['proofs'].items():
        logs[label] = check_receipt(read(label + '.proof.json'), policy, context)
        if label in ('p542-own', 'p542-all-sweeps', 'p542-guild', 'p542-slider'):
            assert re.search(r'test result: ok\. [1-9]\d* passed; 0 failed;', logs[label]), label
    warnings = [line for line in logs['p542-mists'].splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    assert 'Finished `dev`' in logs['p542-mists']
    fixtures = read('p542-python-fixtures.json')
    tree = git('ls-tree', '-r', '--name-only', context['source_revision'], 'tools').decode().splitlines()
    scripts = {path for path in tree if re.fullmatch(r'tools/test_[^/]+\.py', path)}
    assert {row['command'][-1] for row in fixtures} == scripts
    for row in fixtures:
        assert row['exit'] == 0 and row['tests'] > 0
        log = (HERE / row['log']).read_bytes()
        assert digest(log) == row['log_sha256']
        assert re.search(rf'Ran {row["tests"]} tests?', log.decode()) and '\nOK\n' in log.decode()
        assert not git('diff', '--name-only', context['source_revision'], row['revision'], '--', 'tools')
    sweep_revision = read('p542-all-sweeps.proof.json')['revision']
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', logs['p542-all-sweeps'], re.M))
    for path in historical_sweep_tests(ROOT, sweep_revision):
        relative = path.relative_to(ROOT).as_posix()
        source = blob(sweep_revision, relative).decode()
        for function in re.findall(r'fn (patch_\w+_publication_sweep)\(', source):
            assert any(name.endswith('::' + function) for name in passed), function
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = historical_json(sweep_revision, f'data/patch-api/sources/{patch}-wikitext-register.json')
        results = read(path.stem + '-results.json')
        assert set(results) == {row['id'] for row in register['entries']}, patch
        known = historical_json(sweep_revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        assert {key for key, row in results.items() if not row['ok']} == set(known), patch
    assert read('p542-own-results.json') == read('patch_5_4_2_publication_sweep-results.json')
    original = read('p542-own-results.json')
    negative = read('p542-negative-results.json')
    control = read('p542-negative-control.json')
    changed = control['modified_entry']['id']
    assert set(negative) == set(original)
    assert all(negative[key] == row for key, row in original.items() if key != changed)
    assert original[changed]['ok'] and not negative[changed]['ok']
    assert negative[changed]['expected']['symbol'] == control['modified_entry']['symbol']
    assert read('p542-negative.proof.json')['exit'] == control['expected_exit'] != 0
    assert f'new gaps: ["{changed}"]' in logs['p542-negative']
    return len(passed), fixtures


def check_preservation(context):
    baseline = read('p542-wiki-baseline.json')
    for path, entry in baseline.items():
        original = blob(context['base_revision'], path)
        updated = blob(context['documentation_revision'], path)
        assert digest(original) == entry['sha256']
        assert len(original.decode().splitlines()) == entry['lines']
        assert len(updated.decode().splitlines()) >= entry['lines'] and original in updated
    names = git('ls-tree', '-r', '--name-only', context['base_revision'], 'data/patch-api/evidence').decode().splitlines()
    validators = [name for name in names if name.endswith('/validate.py')]
    assert validators == context['prior_validators']
    own_path = HERE.relative_to(ROOT).as_posix()
    assert not git('diff', '--name-only', context['base_revision'], context['source_revision'], '--',
                   'data/patch-api/evidence', ':!' + own_path)
    assert not git('diff', '--name-only', context['base_revision'], context['source_revision'], '--',
                   'src', 'Cargo.toml', 'Cargo.lock', 'build.rs', 'build')


def main():
    context = read('p542-context.json')
    check_seals(context)
    register, ledger, gaps = check_source(context)
    check_retirements(context, register)
    registers, extracts = check_reproduction(context)
    sweeps, fixtures = check_proofs(context)
    check_preservation(context)
    summary = read('p542-accounting-summary.json')
    assert summary == {
        'patch': '5.4.2', 'inventory_rows': len(register['entries']),
        'extract_rows': len(ledger['source_rows']) - len(register['entries']),
        'total_rows': len(ledger['source_rows']),
        'statuses': dict(Counter(row['status'] for row in ledger['source_rows'])),
        'inventory_gaps': len(gaps), 'newly_modeled_gaps': 0,
        'problematic_publication_gaps': len(gaps), 'new_retirements': 0,
        'registers_reproduced': len(registers),
        'extracts_reproduced': sum(row['byte_identical'] for row in extracts),
        'inherited_extract_failures': [row['patch'] for row in extracts if not row['byte_identical']],
        'publication_sweep_cases': sweeps,
        'own_cases': int(re.search(r'test result: ok\. (\d+) passed', (HERE / 'p542-own.log').read_text())[1]),
        'python_fixtures': sum(row['tests'] for row in fixtures),
        'negative_gap_count': sum(not row['ok'] for row in read('p542-negative-results.json').values()),
    }
    print(json.dumps(dict(summary, result='PASS'), sort_keys=True))


if __name__ == '__main__':
    main()
