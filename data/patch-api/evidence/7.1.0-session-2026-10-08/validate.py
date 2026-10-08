"""Read-only, checkout-independent validation of the sealed 7.1.0 audit."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
from collections import Counter

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests, preserved_input_matches


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def source_accounting():
    provenance = read(SOURCES / '7.1.0-api-changes.provenance.json')
    register_path = SOURCES / '7.1.0-wikitext-register.json'
    register = read(register_path)
    raw_path = SOURCES / '7.1.0-api-changes.wikitext'
    raw = raw_path.read_text()
    page = read(HERE / 'p710-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    assert revision['slots']['main']['*'] == raw
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert sha(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    assert sha(HERE / 'p710-fetch.json') == read(HERE / 'p710-fetch-receipt.json')['sha256']
    extractor = load_extractor()
    text_path = SOURCES / '7.1.0-api-changes.txt'
    text = text_path.read_text()
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    assert extractor.extract_text(raw, **options) == text
    coverage = read(SOURCES / '7.1.0-page-coverage.json')
    assert coverage['source_sha256'] == sha(register_path)
    assert coverage['non_inventory_source']['sha256'] == sha(text_path)
    references = []
    for number, line in enumerate(raw.splitlines(), 1):
        references.extend((number, match[1]) for match in re.finditer(r'\{\{api\|([^}|]+)\}\}', line))
        references.extend((number, 'Frame:' + match[1]) for match in re.finditer(
            r'\[\[API_Frame_\w+\|frame:(\w+)\([^]]*\)\]\]', line))
        references.extend((number, match[1]) for match in re.finditer(r'\[\[API_(Get\w+)\|', line))
        references.extend((number, match[1]) for match in re.finditer(r"^\*\* '''(Nameplate\w+)'''", line))
    assert Counter(references) == Counter((r['wikitext_line'], r['symbol']) for r in register['entries'])
    inventory = {r['id']: r for r in register['entries']}
    seeded = extractor.seed_rows(text, '7.1.0')
    rows = {r['source_id']: r for r in coverage['source_rows']}
    assert len(rows) == len(coverage['source_rows'])
    assert set(rows) == set(inventory) | {r['source_id'] for r in seeded}
    results = read(HERE / 'patch_7_1_0_publication_sweep-results.json')
    assert set(results) == set(inventory)
    gaps = {key for key, value in results.items() if not value['ok']}
    assert gaps == set(read(ROOT / 'tests/data/patch_7_1_0_sweep_known_gaps.json'))
    review = read(HERE / 'p710-gap-review.json')
    assert {r['source_id'] for r in review['publication_gaps']} == gaps
    for key in inventory:
        assert rows[key]['status'] == ('audit-pending' if key in gaps else 'bounded-coverage')
        assert bool(rows[key]['capabilities']) == (key not in gaps)
    scout = read(HERE / 'p710-extract-scout.json')
    assert {r['source_id'] for r in scout} == {r['source_id'] for r in seeded}
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw.splitlines()[row['wikitext_line'] - 1]
        assert rows[row['source_id']]['status'] == row['status']
        assert rows[row['source_id']]['note'] == row['reason']
    assert {r['source_id'] for r in review['unmodeled_source_contracts']} == {
        r['source_id'] for r in scout if r['status'] == 'audit-pending'}
    assert all(r['note'] for r in rows.values())
    return {'inventory': len(inventory), 'extract': len(seeded), 'ledger': len(rows),
            'gaps': len(gaps), 'statuses': dict(Counter(r['status'] for r in rows.values()))}


def reproduction(context):
    preservation = read(HERE / 'p710-input-preservation.json')
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', preservation['base_revision'],
                                     'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
    assert {r['path'] for r in preservation['rows']} == set(names)
    for row in preservation['rows']:
        original = hashlib.sha256(blob(preservation['base_revision'], row['path'])).hexdigest()
        assert original == row['before_sha256'] == row['after_sha256']
        assert preserved_input_matches(ROOT, row['path'], original), row['path']
    modes = read(HERE / 'p710-extract-preservation.json')
    assert all(r['before'] == r['after'] for r in modes)
    paths = historical_registers(ROOT, context['runtime_revision'])
    patches = {p.name.removesuffix('-wikitext-register.json') for p in paths}
    registers = read(HERE / 'p710-register-reproduction.json')
    extracts = read(HERE / 'p710-saved-extract-reproduction.json')
    assert {r['patch'] for r in registers} == patches
    assert {r['patch'] for r in extracts} == patches
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical']
        assert preserved_input_matches(ROOT, f"data/patch-api/sources/{row['patch']}-wikitext-register.json", row['sha256'])
    inherited = {r['patch']: r for r in read(ROOT / 'data/patch-api/evidence/7.2.5-session-2026-10-08/integrated/p725-saved-extract-reproduction.json')}
    for row in extracts:
        assert preserved_input_matches(ROOT, f"data/patch-api/sources/{row['patch']}-api-changes.txt", row['saved_sha256'])
        if row['patch'] in inherited:
            assert (row['byte_identical'], row['error']) == (inherited[row['patch']]['byte_identical'], inherited[row['patch']]['error'])
        else:
            assert row['byte_identical']
    return {'preserved_inputs': len(names), 'extract_modes': len(modes), 'registers': len(patches),
            'extracts_reproduced': sum(r['byte_identical'] for r in extracts)}


def retirement_scans():
    evidence = read(HERE / 'p710-retirement-scans.json')
    assert evidence['member'] == 'TitleRegion'
    assert evidence['decision'].startswith('retain:')
    for row in evidence['scans']:
        assert row['tool'] == row['command'][0] == '/usr/bin/grep'
        assert r'\bTitleRegion\b' in row['command']
        assert row['exit'] in (0, 1)
        assert sha(HERE / row['file']) == row['sha256']
        assert len((HERE / row['file']).read_text().splitlines()) == row['matches']
    assert '--exclude=*Documentation*' in evidence['scans'][0]['command']
    for branch in ('master', 'p720-page'):
        row = evidence['later_registers'][branch]
        names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', row['revision'],
                                         'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
        paths = [p for p in names if p.endswith('-wikitext-register.json')]
        assert set(paths) == set(row['registers'])
        additions = []
        for path in paths:
            data = json.loads(blob(row['revision'], path))
            additions.extend({'path': path, 'entry': entry} for entry in data['entries']
                             if entry['symbol'] == 'TitleRegion' and entry['direction'] == 'added')
        assert additions == row['readditions']
    assert evidence['later_registers']['p720-page']['p720_register_present']


def proof(context):
    for name, expected in context['receipts'].items():
        receipt = read(HERE / f'{name}.proof.json')
        assert receipt['exit'] == expected['exit'] and not receipt['invalidated']
        assert receipt['command'] == expected['command']
        assert receipt['revision'] == expected['revision']
        assert sha(HERE / receipt['log']) == receipt['log_sha256']
    positives = ['p710-discovery', 'p710-own-bounded', 'p710-all-sweeps', 'p710-screen',
                 'p710-items', 'p710-intrinsic', 'p710-generator-fixtures',
                 'p710-extractor-fixtures', 'p710-validator-fixtures',
                 'p710-source-reproduction', 'p710-final-format', 'p710-mists-check', 'p710-other-validators']
    assert all(context['receipts'][name]['exit'] == 0 for name in positives)
    for name in ['p710-own-bounded', 'p710-all-sweeps', 'p710-screen', 'p710-items', 'p710-intrinsic']:
        log = (HERE / f'{name}.txt').read_text()
        passed = re.findall(r'test result: ok\. (\d+) passed', log)
        assert passed and int(passed[-1]) > 0, name
    warnings = [line for line in (HERE / 'p710-mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    tests = {p.stem for p in historical_sweep_tests(ROOT, context['runtime_revision'])}
    summaries = read(HERE / 'p710-sweep-summary.json')
    assert {r['test'] for r in summaries} == tests
    for summary in summaries:
        observations = read(HERE / summary['file'])
        assert summary['rows'] == len(observations)
        assert summary['gaps'] == sum(not r['ok'] for r in observations.values())
        fixture = f"tests/data/{summary['test'].removesuffix('_publication_sweep')}_sweep_known_gaps.json"
        historical = json.loads(blob(context['runtime_revision'], fixture))
        assert {key for key, row in observations.items() if not row['ok']} == set(historical)
    negative = read(HERE / 'p710-negative-results.json')
    baseline = read(HERE / 'patch_7_1_0_publication_sweep-results.json')
    control = read(HERE / 'p710-negative-control.json')
    assert control['before'] == sum(not r['ok'] for r in baseline.values())
    assert control['after'] == sum(not r['ok'] for r in negative.values()) == control['before'] + 1
    assert {key for key in baseline if baseline[key]['ok'] != negative[key]['ok']} == {control['mutated_id']}
    assert context['receipts']['p710-negative']['exit'] == 1
    discovery = read(HERE / 'p710-intrinsic-discovery-context.json')
    assert sha(HERE / discovery['test_source']) == discovery['sha256']
    failed = read(HERE / 'p710-behavior-discovery.proof.json')
    assert failed['exit'] == 1
    assert sha(HERE / 'p710-behavior-discovery.txt') == failed['log_sha256']
    assert "unknown frame type 'P710ClipIntrinsic'" in (HERE / 'p710-behavior-discovery.txt').read_text()
    assert any('custom intrinsic' in row['reason'].lower() for row in read(HERE / 'p710-gap-review.json')['unmodeled_source_contracts'])
    matrix = read(HERE / 'p710-other-validator-matrix.json')
    assert matrix and all(r['exit'] == 0 for r in matrix)
    changed = subprocess.check_output(['git', 'diff', '--name-only', context['base_revision'],
                                       context['runtime_revision'], '--', 'src'], cwd=ROOT, text=True)
    assert not changed, 'unclaimed runtime change'
    wiki = read(HERE / 'p710-wiki-integrity.json')
    for row in wiki:
        before = blob(context['base_revision'], row['path']).decode()
        current = (ROOT / row['path']).read_text()
        assert row['before_lines'] == len(before.splitlines())
        assert len(current.splitlines()) >= row['after_lines'] >= row['before_lines']
        assert before in current
    return {'publication_sweeps': len(tests), 'observations': sum(r['rows'] for r in summaries),
            'negative_control': [control['before'], control['after']], 'prior_validators': len(matrix)}


def main():
    context = read(HERE / 'p710-context.json')
    result = {**source_accounting(), **reproduction(context), **proof(context)}
    retirement_scans()
    print(json.dumps({'status': 'pass', **result}, sort_keys=True))


if __name__ == '__main__':
    main()
