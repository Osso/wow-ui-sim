"""Read-only historical retail 5.4.7 proof; shared inputs are pinned Git blobs."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
PREFIX = 'data/patch-api/sources/'


def read(path):
    return json.loads(path.read_text())


def digest(value):
    return hashlib.sha256(value).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', f'{revision}:{path}')


def pinned(revision, path):
    return json.loads(blob(revision, path))


def names(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def historical_helpers(revision):
    namespace = {'__name__': 'pinned_helpers'}
    exec(compile(blob(revision, 'tools/patch_audit_validation.py'), 'pinned_helpers', 'exec'), namespace)
    return namespace


def check_own_seals(context):
    for name, expected in read(HERE / 'p547-artifact-hashes.json').items():
        path = HERE / name
        assert path.is_relative_to(HERE) and path.is_file(), name
        assert digest(path.read_bytes()) == expected, name
    # Every proof input is committed. No checkout-local scratch dependency.
    tracked = set(names(context['seal_revision'], 'data/patch-api/evidence/5.4.7-session-2026-10-08'))
    for name in read(HERE / 'p547-artifact-hashes.json'):
        assert (HERE / name).relative_to(ROOT).as_posix() in tracked, name


def check_source(context):
    revision = context['accounting_revision']
    provenance = pinned(revision, PREFIX + '5.4.7-api-changes.provenance.json')
    register = pinned(revision, PREFIX + '5.4.7-wikitext-register.json')
    raw = blob(revision, PREFIX + '5.4.7-api-changes.wikitext')
    fetched = read(HERE / 'p547-fetch.json')['query']['pages']['549368']
    current = fetched['revisions'][0]
    assert current['revid'] == provenance['revid'] == register['source']['revid']
    assert current['slots']['main']['*'].encode() == raw
    assert digest(raw) == provenance['sha256'] == register['source']['sha256']
    assert provenance['client_line'] == 'retail' and provenance['toc'] == 50400
    patch = read(HERE / 'p547-patch.json')['query']['pages']['491086']['revisions'][0]
    assert patch['revid'] == provenance['toc_source']['revid']
    assert '|toc = 50400' in patch['slots']['main']['*']
    assert '|Release = February 18, 2014' in patch['slots']['main']['*']
    assert '5.4.7.17956' in raw.decode()
    assert not raw.decode().lstrip().lower().startswith('#redirect')
    for header in register['header_counts']:
        assert header['header_count'] == header['parsed_count']
    return register


def check_accounting(context, register):
    revision = context['accounting_revision']
    ledger = pinned(revision, PREFIX + '5.4.7-page-coverage.json')['source_rows']
    rows = {row['source_id']: row for row in ledger}
    assert len(rows) == len(ledger)
    extract = {'__name__': 'pinned_extractor', '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(blob(revision, 'tools/extract_patch_non_inventory.py'), 'pinned_extractor', 'exec'), extract)
    text = blob(revision, PREFIX + '5.4.7-api-changes.txt').decode()
    expected = {row['source_id'] for row in extract['seed_rows'](text, '5.4.7')}
    expected.update(entry['id'] for entry in register['entries'])
    raw = blob(revision, PREFIX + '5.4.7-api-changes.wikitext').decode()
    expected.update(f'source-caption-{number:03}' for number, line in enumerate(raw.splitlines(), 1) if line.startswith('|+'))
    assert set(rows) == expected
    results = read(HERE / 'patch_5_4_7_publication_sweep-results.json')
    assert set(results) == {entry['id'] for entry in register['entries']}
    known = pinned(revision, 'tests/data/patch_5_4_7_sweep_known_gaps.json')
    gaps = {name for name, result in results.items() if not result['ok']}
    assert gaps == set(known) == {row['source_id'] for row in read(HERE / 'p547-gap-review.json')}
    for entry in register['entries']:
        row = rows[entry['id']]
        assert row['status'] == ('audit-pending' if entry['id'] in gaps else 'bounded-coverage')
        assert row['note']
    summary = read(HERE / 'p547-accounting-summary.json')
    assert summary['total_ids'] == len(ledger) and summary['inventory'] == len(register['entries'])
    assert summary['publication_gaps'] == len(gaps)
    assert summary['statuses'] == {status: sum(row['status'] == status for row in ledger)
                                  for status in {row['status'] for row in ledger}}
    negative = read(HERE / 'p547-negative-results.json')
    assert set(negative) == set(results)
    delta = {name for name in results if results[name] != negative[name]}
    assert len(delta) == 1
    added = delta.pop()
    assert results[added]['ok'] and not negative[added]['ok']
    assert {name for name, result in negative.items() if not result['ok']} == gaps | {added}
    assert read(HERE / 'p547-retirement-decisions.json')['runtime_retirements'] == []
    assert not [entry for entry in register['entries'] if entry['direction'] == 'removed']
    return summary


def check_historical_scope(context):
    revision = context['reproduction_revision']
    helpers = historical_helpers(revision)
    registers = helpers['historical_registers'](ROOT, revision)
    patches = {path.name.removesuffix('-wikitext-register.json') for path in registers}
    reproduced = read(HERE / 'p547-register-reproduction.json')
    extracts = read(HERE / 'p547-saved-extract-reproduction.json')
    assert {row['patch'] for row in reproduced} == patches == {row['patch'] for row in extracts}
    for row in reproduced:
        path = PREFIX + row['patch'] + '-wikitext-register.json'
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == digest(blob(revision, path))
    base = context['base_revision']
    prior_path = 'data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/p602-saved-extract-reproduction.json'
    previous = {row['patch']: row for row in pinned(base, prior_path)}
    for row in extracts:
        path = PREFIX + row['patch'] + '-api-changes.txt'
        assert row['saved_sha256'] == digest(blob(revision, path))
        if row['patch'] in previous:
            before = previous[row['patch']]
            assert (row['byte_identical'], row['error']) == (before['byte_identical'], before['error'])
        else:
            assert row['byte_identical']
    preserved = read(HERE / 'p547-input-preservation.json')
    assert {row['path'] for row in preserved['rows']} == set(names(base, 'data/patch-api/sources'))
    for row in preserved['rows']:
        assert row['before_sha256'] == digest(blob(base, row['path']))
        assert row['after_sha256'] == digest(blob(revision, row['path'])) == row['before_sha256']
    assert all(row['before'] == row['after'] for row in read(HERE / 'p547-extract-preservation.json'))
    for scope in read(HERE / 'p547-later-register-scan.json'):
        paths = [path for path in names(scope['revision'], 'data/patch-api/sources') if path.endswith('-wikitext-register.json')]
        assert paths == scope['register_paths']
        symbols = {'BNGetFriendInviteInfoByAddon', 'BNSendGameData', 'GetSpecializationNameForSpecID'}
        matches = [{'register': path, 'entry': entry} for path in paths
                   for entry in pinned(scope['revision'], path)['entries'] if entry['symbol'] in symbols]
        assert matches == scope['matches']
    assert not git('diff', '--name-only', base, context['accounting_revision'], '--', 'src', 'tools', 'Cargo.toml', 'Cargo.lock', 'Interface')
    return len(registers), sum(row['byte_identical'] for row in extracts)


def check_proofs(context):
    for name, expected_exit in context['proofs'].items():
        receipt = read(HERE / (name + '.proof.json'))
        assert receipt['exit'] in expected_exit and not receipt['invalidated'], name
        assert receipt['log_sha256'] == digest((HERE / receipt['log']).read_bytes()), name
        assert git('rev-parse', receipt['revision'] + '^{commit}').strip()
    source_revision = context['acceptance_revision']
    helpers = historical_helpers(source_revision)
    sweeps = helpers['historical_sweep_tests'](ROOT, source_revision)
    for path in sweeps:
        register_path = PREFIX + path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.') + '-wikitext-register.json'
        register = pinned(source_revision, register_path)
        result = read(HERE / (path.stem + '-results.json'))
        assert set(result) == {entry['id'] for entry in register['entries']}
    source = blob(source_revision, 'tests/patch_5_4_7_publication_sweep.rs').decode()
    later = source.split('later_registers: &[', 1)[1]
    assert later.index('// 5.4.8') < later.index('// 6.0.1') < later.index('6.0.2-wikitext-register.json')
    assert '5.5.' not in later
    for name in ['p547-own-behavior', 'p547-cached-bnet', 'p547-specialization', 'p547-bnet-namespace']:
        log = (HERE / (name + '.txt')).read_text()
        assert ('passed' in log or '[PASS]' in log) and not re.search(r'\b0 passed\b', log), name
    warnings = [line for line in (HERE / 'p547-mists.txt').read_text().splitlines() if line.startswith('warning:')]
    historical_log = blob(context['base_revision'], 'data/patch-api/evidence/6.0.2-session-2026-10-08/p602-mists.txt').decode()
    allowed = {line for line in historical_log.splitlines() if line.startswith('warning:')
               and ('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line)}
    assert set(warnings) <= allowed, warnings
    return len(sweeps)


def main():
    context = read(HERE / 'p547-context.json')
    check_own_seals(context)
    register = check_source(context)
    accounting = check_accounting(context, register)
    registers, extracts = check_historical_scope(context)
    sweeps = check_proofs(context)
    print(json.dumps({'status': 'PASS', 'accounting': accounting,
                      'registers': registers, 'reproduced_extracts': extracts,
                      'publication_sweeps': sweeps}, sort_keys=True))


if __name__ == '__main__':
    main()
