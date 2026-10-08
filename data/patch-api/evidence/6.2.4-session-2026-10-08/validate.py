"""Read-only, checkout-independent validation of the sealed 6.2.4 audit scope."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests, preserved_input_matches


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*arguments):
    return subprocess.check_output(['git', *arguments], cwd=ROOT)


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_source():
    provenance = read(SOURCES / '6.2.4-api-changes.provenance.json')
    raw_path = SOURCES / '6.2.4-api-changes.wikitext'
    raw = raw_path.read_text()
    response = read(HERE / 'p624-fetch.json')
    page = response['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    assert page['pageid'] == 212964 and page['title'] == provenance['title']
    assert revision['revid'] == provenance['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert revision['slots']['main']['*'] == raw
    assert digest(raw_path) == provenance['wikitext_sha256']
    assert digest(HERE / 'p624-fetch.json') == read(HERE / 'p624-fetch-receipt.json')['sha256']
    register_path = SOURCES / '6.2.4-wikitext-register.json'
    register = read(register_path)
    assert register['source']['revid'] == revision['revid']
    assert register['source']['sha256'] == digest(raw_path)
    with tempfile.TemporaryDirectory(prefix='p624-validator-') as temporary:
        output = Path(temporary) / 'register.json'
        command = [sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                   '6.2.4', str(raw_path), str(revision['revid']), str(output),
                   *provenance['generator_flags']]
        subprocess.run(command, cwd=ROOT, check=True, capture_output=True)
        assert output.read_bytes() == register_path.read_bytes()
    extractor = load_extractor()
    options = {flag.removeprefix('--').replace('-', '_'): True
               for flag in provenance['extractor_flags']}
    text_path = SOURCES / '6.2.4-api-changes.txt'
    text = text_path.read_text()
    assert extractor.extract_text(raw, **options) == text
    # The pinned page is neither a redirect nor a stub. Preserve every explicit list
    # item and both identities of each rename independently of generated row counts.
    expected = []
    section = None
    for number, line in enumerate(raw.splitlines(), 1):
        if line.startswith('=='):
            section = line.strip('= ')
        elif section == 'New':
            expected.extend(('global-api', name, 'added', number)
                            for name in re.findall(r'\{\{api\|([^}|]+)\}\}', line))
        elif section == 'Changes' and ' -> ' in line:
            names = re.findall(r'\{\{api\|([^}|]+)\}\}', line)
            assert len(names) == 2
            expected.extend(('global-api', name, direction, number)
                            for name, direction in zip(names, ('removed', 'added')))
        elif section == 'Removals':
            if re.fullmatch(r'  BN\w+', line):
                expected.append(('global-api', line.strip(), 'removed', number))
            elif '“realmName” [[CVar]] no longer exists' in line:
                expected.append(('cvars', 'realmName', 'removed', number))
    assert Counter(expected) == Counter((row['section'], row['symbol'], row['direction'], row['wikitext_line'])
                                        for row in register['entries'])
    coverage = read(SOURCES / '6.2.4-page-coverage.json')
    assert coverage['source_sha256'] == digest(register_path)
    assert coverage['non_inventory_source']['sha256'] == digest(text_path)
    inventory = {row['id']: row for row in register['entries']}
    seeded = extractor.seed_rows(text, '6.2.4')
    rows = {row['source_id']: row for row in coverage['source_rows']}
    assert len(rows) == len(coverage['source_rows'])
    assert set(rows) == set(inventory) | {row['source_id'] for row in seeded}
    results = read(HERE / 'patch_6_2_4_publication_sweep-results.json')
    assert set(results) == set(inventory)
    gaps = {key for key, value in results.items() if not value['ok']}
    assert gaps == set(read(ROOT / 'tests/data/patch_6_2_4_sweep_known_gaps.json'))
    review = read(HERE / 'p624-gap-review.json')
    assert {row['source_id'] for row in review['publication_gaps']} == gaps
    for key in inventory:
        assert rows[key]['status'] == ('audit-pending' if key in gaps else 'bounded-coverage')
        assert bool(rows[key]['capabilities']) == (key not in gaps)
    scout = read(HERE / 'p624-extract-scout.json')
    assert {row['source_id'] for row in scout} == {row['source_id'] for row in seeded}
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw.splitlines()[row['wikitext_line'] - 1]
        assert rows[row['source_id']]['status'] == row['status']
        assert rows[row['source_id']]['note'] == row['reason']
    assert {row['source_id'] for row in review['unmodeled_source_contracts']} == {
        row['source_id'] for row in scout if row['status'] == 'audit-pending'}
    assert all(row['note'] for row in rows.values())
    return {'inventory': len(inventory), 'extract': len(seeded), 'ledger': len(rows),
            'publication_gaps': len(gaps), 'statuses': dict(Counter(row['status'] for row in rows.values()))}


def validate_reproduction(context):
    preservation = read(HERE / 'p624-input-preservation.json')
    names = git('ls-tree', '-r', '--name-only', preservation['base_revision'],
                'data/patch-api/sources').decode().splitlines()
    assert {row['path'] for row in preservation['rows']} == set(names)
    for row in preservation['rows']:
        original = hashlib.sha256(git('show', f"{preservation['base_revision']}:{row['path']}")).hexdigest()
        assert original == row['before_sha256'] == row['after_sha256']
        assert preserved_input_matches(ROOT, row['path'], original), row['path']
    modes = read(HERE / 'p624-extract-preservation.json')
    assert all(row['before'] == row['after'] for row in modes)
    paths = historical_registers(ROOT, context['runtime_revision'])
    patches = {path.name.removesuffix('-wikitext-register.json') for path in paths}
    registers = read(HERE / 'p624-register-reproduction.json')
    extracts = read(HERE / 'p624-saved-extract-reproduction.json')
    assert {row['patch'] for row in registers} == patches
    assert {row['patch'] for row in extracts} == patches
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical']
        assert preserved_input_matches(ROOT, f"data/patch-api/sources/{row['patch']}-wikitext-register.json", row['sha256'])
    inherited = {row['patch']: row for row in read(
        ROOT / 'data/patch-api/evidence/7.1.0-session-2026-10-08/integrated/p710-saved-extract-reproduction.json')}
    for row in extracts:
        assert preserved_input_matches(ROOT, f"data/patch-api/sources/{row['patch']}-api-changes.txt", row['saved_sha256'])
        if row['patch'] in inherited:
            assert (row['byte_identical'], row['error']) == (inherited[row['patch']]['byte_identical'], inherited[row['patch']]['error'])
        else:
            assert row['byte_identical'], row['patch']
    return {'preserved_inputs': len(names), 'extract_modes': len(modes),
            'registers': len(patches), 'extracts_reproduced': sum(row['byte_identical'] for row in extracts)}


def validate_retirement_scans(context):
    raw = (SOURCES / '6.2.4-api-changes.wikitext').read_text()
    members = set(re.findall(r'^  (BN\w+)$', raw, re.M)) | {'realmName'}
    scans = read(HERE / 'p624-retirement-scans.json')
    assert Counter((row['member'], row['scope']) for row in scans) == Counter(
        (member, scope) for member in members for scope in ('retail', 'callers'))
    for row in scans:
        assert row['tool'] == row['command'][0] == '/usr/bin/grep'
        assert rf"\b{row['member']}\b" in row['command']
        assert row['exit'] in (0, 1)
        assert digest(HERE / row['file']) == row['sha256']
        assert len((HERE / row['file']).read_text().splitlines()) == row['matches']
        if row['scope'] == 'retail':
            assert '--exclude=*Documentation*' in row['command']
    for branch, snapshot in read(HERE / 'p624-later-retirement-check.json').items():
        paths = git('ls-tree', '-r', '--name-only', snapshot['revision'],
                    'data/patch-api/sources').decode().splitlines()
        paths = [path for path in paths if path.endswith('-wikitext-register.json')]
        assert set(paths) == set(snapshot['registers'])
        if branch == 'p703-page':
            assert 'data/patch-api/sources/7.0.3-wikitext-register.json' in paths
        additions = []
        for path in paths:
            register = json.loads(git('show', f"{snapshot['revision']}:{path}"))
            additions.extend({'path': path, 'entry': entry} for entry in register['entries']
                             if entry['symbol'] in members and entry['direction'] == 'added')
        assert additions == snapshot['readditions'] == []
    assert not git('diff', '--name-only', context['base_revision'], context['runtime_revision'], '--', 'src')
    return {'members_scanned': len(members), 'scans': len(scans), 'runtime_retirements': 0}


def validate_proofs(context):
    required = {'p624-all-sweeps', 'p624-cached-identity', 'p624-bare-identity',
                'p624-bnet-model', 'p624-deprecated-bnet', 'p624-final-format',
                'p624-mists-check', 'p624-generator-fixtures', 'p624-extractor-fixtures-final',
                'p624-validator-fixtures', 'p624-source-reproduction', 'p624-other-validators',
                'p624-negative'}
    assert required <= set(context['receipts'])
    for name in required:
        assert context['receipts'][name]['exit'] == (1 if name == 'p624-negative' else 0)
    for name, expected in context['receipts'].items():
        receipt = read(HERE / f'{name}.proof.json')
        assert receipt['exit'] == expected['exit']
        assert receipt['command'] == expected['command']
        assert receipt['revision'] == expected['revision'] and not receipt['invalidated']
        assert digest(HERE / receipt['log']) == receipt['log_sha256']
    for name in ('p624-all-sweeps', 'p624-cached-identity', 'p624-bare-identity',
                 'p624-bnet-model', 'p624-deprecated-bnet'):
        log = (HERE / f'{name}.txt').read_text()
        matches = re.findall(r'test result: ok\. (\d+) passed', log)
        assert matches and int(matches[-1]) > 0, name
    warnings = [line for line in (HERE / 'p624-mists-check.txt').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings), warnings
    tests = {path.stem for path in historical_sweep_tests(ROOT, context['runtime_revision'])}
    summaries = read(HERE / 'p624-sweep-summary.json')
    assert {row['test'] for row in summaries} == tests
    for row in summaries:
        observations = read(HERE / row['file'])
        assert row['rows'] == len(observations)
        assert row['gaps'] == sum(not value['ok'] for value in observations.values())
        fixture = f"tests/data/{row['test'].removesuffix('_publication_sweep')}_sweep_known_gaps.json"
        known = json.loads(git('show', f"{context['runtime_revision']}:{fixture}"))
        assert {key for key, value in observations.items() if not value['ok']} == set(known)
    baseline = read(HERE / 'patch_6_2_4_publication_sweep-results.json')
    negative = read(HERE / 'p624-negative-results.json')
    control = read(HERE / 'p624-negative-control.json')
    assert {key for key, row in negative.items() if not row['ok']} == {
        key for key, row in baseline.items() if not row['ok']} | {control['source_id']}
    assert control['before'] == sum(not row['ok'] for row in baseline.values())
    assert control['after'] == sum(not row['ok'] for row in negative.values()) == control['before'] + 1
    previous_paths = git('ls-tree', '-r', '--name-only', context['base_revision'],
                         'data/patch-api/evidence').decode().splitlines()
    validators = {path for path in previous_paths if Path(path).name in ('validate.py', 'validate_integrated.py')}
    matrix = read(HERE / 'p624-other-validator-matrix.json')
    assert {row['path'] for row in matrix} == validators
    assert all(row['exit'] == 0 for row in matrix)
    return {'sweeps': len(summaries), 'observations': sum(row['rows'] for row in summaries),
            'prior_validators': len(matrix)}


def main():
    context = read(HERE / 'p624-context.json')
    for artifact in context['artifacts']:
        assert preserved_input_matches(ROOT, artifact['path'], artifact['sha256']), artifact['path']
    # Sealed wiki proof uses a recorded revision, never today's mutable index/log.
    wiki = read(HERE / 'p624-wiki-integrity.json')
    for row in wiki['files']:
        before = git('show', f"{context['base_revision']}:{row['path']}")
        after = git('show', f"{wiki['revision']}:{row['path']}")
        assert before.decode() in after.decode()
        assert len(after.splitlines()) >= len(before.splitlines())
        assert hashlib.sha256(after).hexdigest() == row['sha256']
    result = {**validate_source(), **validate_reproduction(context),
              **validate_retirement_scans(context), **validate_proofs(context)}
    print(json.dumps(result, sort_keys=True))


if __name__ == '__main__':
    main()
