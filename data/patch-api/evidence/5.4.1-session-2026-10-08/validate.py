"""Read-only retail 5.4.1 proof: sealed own files and fixed Git scopes."""
import ast
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


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((HERE / name).read_text())


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', revision + ':' + path)


def source_json(revision, path):
    return json.loads(blob(revision, path))


def check_seal():
    seals = read('seal.json')
    for name, expected in seals.items():
        path = HERE / name
        assert digest(path.read_bytes()) == expected, 'own evidence drift: ' + name
    assert 'validate.py' in seals and 'context.json' in seals


def check_source(context):
    revision = context['source_revision']
    prefix = 'data/patch-api/sources/5.4.1-'
    page = read('source-response.json')['query']['pages']['221650']
    source = page['revisions'][0]
    assert page['title'] == 'Patch 5.4.1/API changes'
    assert source['revid'] == 2150406
    raw = blob(revision, prefix + 'api-changes.wikitext')
    assert raw.decode() == source['slots']['main']['*'] == (HERE / 'source.wiki').read_text()
    parent = next(iter(read('parent-response.json')['query']['pages'].values()))
    parent_raw = parent['revisions'][0]['slots']['main']['*']
    assert parent['revisions'][0]['revid'] == 6428967
    assert parent_raw == (HERE / 'parent.wiki').read_text()
    assert '|Version = 17538' in parent_raw and '|toc = 50400' in parent_raw
    assert '|Release = October 29, 2013' in parent_raw
    provenance = source_json(revision, prefix + 'api-changes.provenance.json')
    assert provenance['revid'] == source['revid'] and provenance['sha256'] == digest(raw)
    assert provenance['generator_flags'] == ['--mists-automated-diff']
    assert provenance['extractor_flags'] == ['--mists-automated-diff', '--lowercase-reflist']
    register = source_json(revision, prefix + 'wikitext-register.json')
    assert register['source']['revid'] == source['revid']
    assert register['source']['sha256'] == digest(raw)
    for header in register['header_counts']:
        count = sum(e['section'] == header['section'] and e['direction'] == header['direction']
                    for e in register['entries'])
        assert header['header_count'] == header['parsed_count'] == count
    assert all(e['direction'] == 'added' for e in register['entries'])
    ledger = source_json(revision, prefix + 'page-coverage.json')
    assert ledger['source_sha256'] == digest(blob(revision, prefix + 'wikitext-register.json'))
    text = blob(revision, prefix + 'api-changes.txt')
    assert ledger['non_inventory_source']['sha256'] == digest(text)
    namespace = {'__name__': 'pinned_extractor', '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(blob(revision, 'tools/extract_patch_non_inventory.py'), 'pinned_extractor', 'exec'), namespace)
    supplemental = namespace['seed_rows'](text.decode(), '5.4.1')
    ids = [e['id'] for e in register['entries']] + [e['source_id'] for e in supplemental]
    rows = ledger['source_rows']
    assert len(rows) == len(ids) == len(set(ids))
    assert {r['source_id'] for r in rows} == set(ids)
    gaps = source_json(revision, 'tests/data/patch_5_4_1_sweep_known_gaps.json')
    assert set(gaps) == {r['source_id'] for r in read('gap-review.json')}
    assert all(r['reason'] for r in read('gap-review.json'))
    results = read('patch_5_4_1_publication_sweep-results.json')
    assert set(results) == {e['id'] for e in register['entries']}
    assert {key for key, row in results.items() if not row['ok']} == set(gaps)
    bounded = {r['source_id'] for r in rows if r['status'] == 'bounded-coverage'}
    assert bounded == set(results) - set(gaps)
    decisions = read('retirement-decisions.json')
    assert decisions['removed_entries'] == decisions['new_retirements'] == []
    assert not git('diff', '--name-only', context['base_revision'], revision, '--', 'src')
    sweep = blob(revision, 'tests/patch_5_4_1_publication_sweep.rs').decode()
    later = sweep.split('later_registers: &[', 1)[1]
    ordered = ['// Queued retail 5.4.2', '// Queued retail 5.4.7', '// Queued retail 5.4.8',
               '6.0.1-wikitext-register', '6.0.2-wikitext-register']
    assert [later.index(s) for s in ordered] == sorted(later.index(s) for s in ordered)
    assert '5.5.' not in later
    return len(register['entries']), len(supplemental), len(gaps)


def check_scans():
    scans = read('caller-scans.json')
    caller_blobs = {}
    for row in scans:
        assert row['scanner'] == '/usr/bin/grep' and '-w' in row['argv']
        data = (HERE / row['output']).read_bytes()
        assert digest(data) == row['sha256'] and len(data.splitlines()) == row['matches']
        assert row['exit'] == (0 if row['matches'] else 1)
        if row['domain'] == 'cached':
            assert '--exclude=*Documentation*' in row['argv']
            assert '--exclude-dir=*Documentation*' in row['argv']
        else:
            assert row['argv'][-2:] == ['src', 'tests']
            for line in data.decode().splitlines():
                path, number, contents = line.split(':', 2)
                key = (row['revision'], path)
                if key not in caller_blobs:
                    caller_blobs[key] = blob(*key).decode().splitlines()
                assert caller_blobs[key][int(number) - 1] == contents, (path, number)
    for snapshot in read('later-register-snapshots.json'):
        names = git('ls-tree', '-r', '--name-only', snapshot['revision'], 'data/patch-api/sources').decode()
        assert (HERE / snapshot['tree_log']).read_text() == names
        assert snapshot['registers'] == [p for p in names.splitlines() if p.endswith('-wikitext-register.json')]
        for match in snapshot['matches']:
            entries = source_json(snapshot['revision'], match['path'])['entries']
            assert all(entry in entries for entry in match['entries'])
    return len(scans)


def check_reproduction(context):
    revision = context['source_revision']
    expected = {p.name.removesuffix('-wikitext-register.json') for p in historical_registers(ROOT, revision)}
    registers = read('p541-register-reproduction.json')
    extracts = read('p541-saved-extract-reproduction.json')
    assert {r['patch'] for r in registers} == {r['patch'] for r in extracts} == expected
    for row in registers:
        assert row['revision'] == revision and row['exit'] == 0 and row['byte_identical'], row['patch']
        assert row['sha256'] == digest(blob(revision, 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json'))
    inherited = source_json(context['base_revision'],
        'data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/p602-saved-extract-reproduction.json')
    failures = {r['patch']: r['error'] for r in inherited if not r['byte_identical']}
    for row in extracts:
        assert row['revision'] == revision
        assert row['saved_sha256'] == digest(blob(revision, 'data/patch-api/sources/' + row['patch'] + '-api-changes.txt'))
        assert row['byte_identical'] or row['error'] == failures[row['patch']]
    preservation = read('p541-input-preservation.json')
    names = git('ls-tree', '-r', '--name-only', context['base_revision'], 'data/patch-api/sources').decode().splitlines()
    assert [r['path'] for r in preservation['rows']] == names
    for row in preservation['rows']:
        assert row['before_sha256'] == digest(blob(context['base_revision'], row['path']))
        assert row['before_sha256'] == row['after_sha256'] == digest(blob(revision, row['path']))
    return len(registers), sum(r['byte_identical'] for r in extracts)


def check_proofs(context):
    revision = context['source_revision']
    results = read('targeted-results.json')
    assert set(results) == set(context['proofs'])
    for name, command in context['proofs'].items():
        receipt = read(name + '.proof.json')
        assert receipt['revision'] == revision and receipt['command'] == command
        assert receipt['exit'] == results[name] == (1 if name == 'negative' else 0)
        assert not receipt['invalidated']
        log = (HERE / receipt['log']).read_bytes()
        assert digest(log) == receipt['log_sha256']
    sweeps_log = (HERE / 'all-sweeps.txt').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', sweeps_log, re.M))
    assert passed
    for path in historical_sweep_tests(ROOT, revision):
        source = blob(revision, path.relative_to(ROOT).as_posix()).decode()
        for function in re.findall(r'fn (patch_\w+_publication_sweep)\(', source):
            assert any(name.endswith('::' + function) for name in passed), function
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = source_json(revision, f'data/patch-api/sources/{patch}-wikitext-register.json')
        observations = read(path.stem + '-results.json')
        assert set(observations) == {r['id'] for r in register['entries']}, patch
        gaps = source_json(revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        assert {key for key, row in observations.items() if not row['ok']} == set(gaps), patch
    own_log = (HERE / 'own-green.txt').read_text()
    assert 'patch_5_4_1_default_realm_cvar_is_unavailable ... ok' in own_log
    original = read('patch_5_4_1_publication_sweep-results.json')
    negative = read('negative-results.json')
    assert set(negative) == set(original)
    assert sum(not r['ok'] for r in negative.values()) == sum(not r['ok'] for r in original.values()) + 1
    warnings = [line for line in (HERE / 'mists.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    return len(passed)


def check_parser_origin(context):
    for row in read('parser-origin.json'):
        functions = []
        for revision in (row['origin_revision'], context['source_revision']):
            source = blob(revision, row['path']).decode()
            node = next(n for n in ast.parse(source).body
                        if isinstance(n, ast.FunctionDef) and n.name == row['function'])
            functions.append('\n'.join(source.splitlines()[node.lineno - 1:node.end_lineno]) + '\n')
        assert functions[0] == functions[1]
        assert digest(functions[0].encode()) == row['sha256']
        assert row['origin_patch'] == '5.4.2' and row['byte_identical']


def check_wiki(context):
    for path, lines in read('wiki-baseline.json').items():
        before = blob(context['base_revision'], path)
        after = blob(context['documentation_revision'], path)
        assert len(before.decode().splitlines()) == lines
        assert len(after.decode().splitlines()) >= lines and before in after


def main():
    check_seal()
    context = read('context.json')
    inventory, extract, gaps = check_source(context)
    scans = check_scans()
    registers, extracts = check_reproduction(context)
    sweeps = check_proofs(context)
    check_wiki(context)
    check_parser_origin(context)
    print(json.dumps({'status': 'PASS', 'inventory': inventory, 'extract_rows': extract,
                      'publication_gaps': gaps, 'new_retirements': 0, 'scans': scans,
                      'registers_reproduced': registers, 'extracts_reproduced': extracts,
                      'sweep_cases': sweeps}, sort_keys=True))


if __name__ == '__main__':
    main()
