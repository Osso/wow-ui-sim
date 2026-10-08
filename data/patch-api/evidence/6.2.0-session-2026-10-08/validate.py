"""Read-only, checkout-independent evidence gate for the pinned 6.2.0 parent page."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PIN = 'ddd76addc3bd451894316ab8c3575ff9e8ec0f35'
REBASE = json.loads((HERE / 'integrated/rebase-mapping.json').read_text())
MAPPING = REBASE['commits']


def mapped_row(revision):
    return next((row for row in MAPPING if row['recorded_revision'].startswith(revision)), None)


def mapped_revision(revision):
    row = mapped_row(revision)
    return row.get('rebased_revision', row.get('scope_revision', row.get('superseded_by'))) if row else revision


def historical_registers(root, revision):
    return [root / name for name in paths_at(revision, 'data/patch-api/sources')
            if name.endswith('-wikitext-register.json')]


def historical_sweep_tests(root, revision):
    return [root / name for name in paths_at(revision, 'tests')
            if name.endswith('_publication_sweep.rs')]


def preserved_input_matches(root, path, expected):
    current = sha(blob(PIN, path))
    helper = load_tool('patch_audit_validation')
    return current == expected or helper.LATER_AUDIT_REPLACEMENTS.get(path) == (expected, current)


def read(path):
    return json.loads(shared_bytes(path))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def shared_bytes(path):
    if path.is_relative_to(HERE):
        return path.read_bytes()
    return blob(PIN, path.relative_to(ROOT).as_posix())


def digest(path):
    return sha(shared_bytes(path))


def git(*args):
    arguments = list(args)
    for index, arg in enumerate(arguments):
        revision, separator, suffix = arg.partition(':')
        tail = ''
        if revision.endswith('^{commit}'):
            revision = revision.removesuffix('^{commit}')
            tail = '^{commit}'
        arguments[index] = mapped_revision(revision) + tail + separator + suffix
    return subprocess.check_output(['git', *arguments], cwd=ROOT)


def blob(revision, path):
    return git('show', revision + ':' + path)


def paths_at(revision, directory):
    inventory = REBASE.get('referenced_inventories', {}).get(revision)
    if inventory and directory == 'data/patch-api/sources':
        return inventory['paths']
    row = mapped_row(revision)
    if row and directory in ('data/patch-api/sources', 'tests'):
        key = 'recorded_registers' if directory == 'data/patch-api/sources' else 'recorded_sweeps'
        return row[key]
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def load_tool(name):
    import types
    module = types.ModuleType(name)
    module.__file__ = str(ROOT / 'tools' / (name + '.py'))
    source = blob(PIN, 'tools/' + name + '.py')
    exec(compile(source, module.__file__, 'exec'), module.__dict__)
    return module


def check_source():
    provenance = read(SOURCES / '6.2.0-api-changes.provenance.json')
    page = read(HERE / 'p620-fetch.json')['query']['pages'][0]
    revision = page['revisions'][0]
    raw = shared_bytes(SOURCES / '6.2.0-api-changes.wikitext').decode()
    assert page['pageid'] == provenance['pageid'] == 149103
    assert page['title'] == provenance['title'] == 'Patch 6.2.0/API changes'
    assert revision['revid'] == provenance['revid'] == 1460518
    assert revision['timestamp'] == provenance['timestamp']
    assert revision['slots']['main']['content'] == raw
    assert digest(HERE / 'p620-fetch.json') == provenance['sha256']
    assert sha(raw.encode()) == provenance['wikitext_sha256']
    register = read(SOURCES / '6.2.0-wikitext-register.json')
    assert register['source']['revid'] == provenance['revid']
    assert register['source']['sha256'] == sha(raw.encode())
    assert register['entries'] == register['header_counts'] == []
    extractor = load_tool('extract_patch_non_inventory')
    text = shared_bytes(SOURCES / '6.2.0-api-changes.txt').decode()
    assert provenance['extractor_flags'] == ['--retain-patch-diff-reference']
    assert extractor.extract_text(raw, retain_patch_diff_reference=True) == text
    ledger = read(SOURCES / '6.2.0-page-coverage.json')
    assert ledger['source_sha256'] == digest(SOURCES / '6.2.0-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha(text.encode())
    rows = ledger['source_rows']
    seeded = extractor.seed_rows(text, '6.2.0')
    assert {r['source_id'] for r in rows} == {r['source_id'] for r in seeded}
    assert len(rows) == len(seeded)
    for row in rows:
        number = int(row['source_id'].rsplit('-', 1)[1])
        assert row['literal'] == text.splitlines()[number - 1]
        assert row['note'] and row['status'] in ('bounded-coverage', 'audit-pending', 'metadata-only')
        assert bool(row['capabilities']) == (row['status'] == 'bounded-coverage')
    problematic = [r for r in rows if r['status'] == 'audit-pending' or 'metadata is not modeled' in r['note']]
    assert read(HERE / 'p620-problematic-contracts.json') == problematic
    assert read(ROOT / 'tests/data/patch_6_2_0_sweep_known_gaps.json') == []
    assert read(HERE / 'patch_6_2_0_publication_sweep-results.json') == {}
    return {'inventory': len(register['entries']), 'extract': len(rows),
            'statuses': dict(Counter(r['status'] for r in rows))}


def check_sources(context):
    preservation = read(HERE / 'p620-input-preservation.json')
    expected = set(paths_at(preservation['base_revision'], 'data/patch-api/sources'))
    assert {r['path'] for r in preservation['rows']} == expected
    for row in preservation['rows']:
        original = sha(blob(preservation['base_revision'], row['path']))
        assert row['before_sha256'] == row['after_sha256'] == original
        assert preserved_input_matches(ROOT, row['path'], original), row['path']
    registers = read(HERE / 'p620-register-reproduction.json')
    extracts = read(HERE / 'p620-extract-reproduction.json')
    scope = {p.name.removesuffix('-wikitext-register.json')
             for p in historical_registers(ROOT, context['source_revision'])}
    assert {r['patch'] for r in registers['rows']} == scope
    assert {r['patch'] for r in extracts['rows']} == scope
    assert len(registers['rows']) == len(extracts['rows']) == len(scope)
    for row in registers['rows']:
        assert row['exit'] == 0 and row['byte_identical'] and not row['error']
        path = 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json'
        assert preserved_input_matches(ROOT, path, row['sha256']), path
    inherited = read(ROOT / 'data/patch-api/evidence/7.0.1-session-2026-10-08/p701-saved-extract-reproduction.json')
    failures = {r['patch']: r['error'] for r in inherited if not r['byte_identical']}
    actual = {r['patch']: r['error'] for r in extracts['rows'] if not r['byte_identical']}
    assert actual == failures
    for row in extracts['rows']:
        path = 'data/patch-api/sources/' + row['patch'] + '-api-changes.txt'
        assert preserved_input_matches(ROOT, path, row['sha256']), path
    return {'protected_inputs': len(expected), 'registers': len(scope),
            'extracts_reproduced': sum(r['byte_identical'] for r in extracts['rows']),
            'inherited_extract_failures': sorted(failures)}


def check_commands(context):
    for label, expected in context['receipts'].items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt['command'] == expected['command'] and receipt['exit'] == 0
        assert receipt['revision'] == expected['revision'] and not receipt['invalidated']
        assert digest(HERE / receipt['log']) == receipt['log_sha256'] == expected['log_sha256']
        git('rev-parse', '--verify', receipt['revision'] + '^{commit}')
        if receipt['command'][:2] == ['cargo', 'test']:
            log = (HERE / receipt['log']).read_text()
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log)
            assert counts and sum(map(int, counts)) > 0, label
            assert git('rev-parse', receipt['revision'] + ':src').strip().decode() == context['runtime_src_tree']
            assert git('rev-parse', receipt['revision'] + ':tests').strip().decode() == context['runtime_tests_tree']
    discovery = read(HERE / 'p620-discovery.proof.json')
    assert discovery['exit'] == 1
    log = HERE / discovery['log']
    assert digest(log) == discovery['log_sha256']
    assert 'patch_6_2_0_spell_link_excludes_cost ... FAILED' in log.read_text()
    assert '10% of Base MANA' in log.read_text()
    assert 'patch_6_2_0_publication_sweep ... ok' in log.read_text()
    for label in ('p620-mists-check', 'p620-format'):
        receipt = read(HERE / (label + '.proof.json'))
        assert git('rev-parse', receipt['revision'] + ':src').strip().decode() == context['runtime_src_tree']
    warnings = [line for line in (HERE / 'p620-mists-check.txt').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings), warnings
    for label in ('p620-fixtures-extractor-final', 'p620-test_gen_patch_wikitext_register',
                  'p620-test_patch_audit_validation'):
        log = (HERE / (label + '.txt')).read_text()
        assert re.search(r'Ran [1-9]\d* tests', log) and '\nOK\n' in log
    for label in ('p620-branch-startup', 'p620-master-startup'):
        receipt = read(HERE / (label + '.proof.json'))
        assert '--no-addons' not in receipt['command'] and '--no-saved-vars' in receipt['command']
        log = (HERE / receipt['log']).read_text()
        assert 'Lua errors: 0 unique, 0 occurrence(s)' in log and log.rstrip().endswith('[]')
    snapshot = read(HERE / 'p620-master-snapshot.json')
    for path, expected in snapshot['scope'].items():
        assert sha(blob(snapshot['revision'], path)) == expected
    assert snapshot['revision'] == context['master_revision']
    return len(context['receipts'])


def check_sweeps(context):
    log = (HERE / 'p620-all-sweeps.txt').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))
    pages = []
    for path in historical_sweep_tests(ROOT, context['runtime_revision']):
        source = blob(context['runtime_revision'], path.relative_to(ROOT).as_posix()).decode()
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', source)
        assert functions and all(any(case.endswith('::' + fn) for case in passed) for fn in functions)
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = json.loads(blob(context['runtime_revision'], 'data/patch-api/sources/' + patch + '-wikitext-register.json'))
        known = json.loads(blob(context['runtime_revision'], 'tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        results = read(HERE / (path.stem + '-results.json'))
        assert set(results) == {r['id'] for r in register['entries']}
        assert {key for key, row in results.items() if not row['ok']} == set(known)
        pages.append({'patch': patch, 'observations': len(results), 'gaps': len(known)})
    assert pages == read(HERE / 'p620-sweep-summary.json')
    return {'pages': len(pages), 'cases': len(passed),
            'observations': sum(r['observations'] for r in pages)}


def check_scans_and_validators():
    scans = read(HERE / 'p620-retirement-scans.json')
    assert scans['members'] == scans['scans'] == [] and scans['tool'] == '/usr/bin/grep'
    for row in scans['later_registers'].values():
        # Unmerged p624 inputs are Git objects, not files in this checkout.
        expected = {p for p in paths_at(row['revision'], 'data/patch-api/sources')
                    if p.endswith('-wikitext-register.json')}
        assert set(row['registers']) == expected and row['readditions'] == []
    for row in read(HERE / 'p620-caller-scans.json'):
        assert row['scanner'] == 'grep' and row['argv'][0] == '/usr/bin/grep' and '-w' in row['argv']
        assert row['exit'] in (0, 1) and digest(HERE / row['output']) == row['sha256']
        assert len((HERE / row['output']).read_text().splitlines()) == row['lines']
    matrix = read(HERE / 'p620-prior-validators.json')
    expected = {p for p in paths_at(matrix['revision'], 'data/patch-api/evidence')
                if Path(p).name.startswith('validate') and p.endswith('.py')}
    assert {r['path'] for r in matrix['rows']} == expected and len(matrix['rows']) == len(expected)
    for row in matrix['rows']:
        assert row['exit'] == 0 and digest(HERE / row['log']) == row['log_sha256'], row['path']
    return len(expected)


def main():
    context = read(HERE / 'p620-context.json')
    row = mapped_row(context['runtime_revision'])
    assert context['runtime_src_tree'] == row['recorded_src_tree']
    assert context['runtime_tests_tree'] == row['recorded_tests_tree']
    context['runtime_src_tree'] = row['rebased_src_tree']
    context['runtime_tests_tree'] = row['rebased_tests_tree']
    result = {'status': 'PASS', **check_source(), **check_sources(context),
              **check_sweeps(context), 'commands': check_commands(context),
              'prior_validators': check_scans_and_validators()}
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
