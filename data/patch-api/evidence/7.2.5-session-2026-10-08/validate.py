"""Read-only historical 7.2.5 proof, independent of checkout path/later audit additions."""
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
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests, preserved_input_matches


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def source_and_accounting():
    provenance = read(SOURCES / '7.2.5-api-changes.provenance.json')
    register_path = SOURCES / '7.2.5-wikitext-register.json'
    register = read(register_path)
    raw_path = SOURCES / '7.2.5-api-changes.wikitext'
    raw = raw_path.read_text()
    page = read(HERE / 'p725-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    assert revision['slots']['main']['*'] == raw
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert sha(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    assert sha(HERE / 'p725-fetch.json') == read(HERE / 'p725-fetch-receipt.json')['sha256']
    references = [(n, next(p for p in match[1].split('|') if '=' not in p))
                  for n, line in enumerate(raw.splitlines(), 1)
                  for match in re.finditer(r'\{\{api\|([^{}]+)\}\}', line)]
    assert Counter(references) == Counter((r['wikitext_line'], r['symbol']) for r in register['entries'])
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text_path = SOURCES / '7.2.5-api-changes.txt'
    text = text_path.read_text()
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    assert extractor.extract_text(raw, **options) == text
    coverage = read(SOURCES / '7.2.5-page-coverage.json')
    assert coverage['source_sha256'] == sha(register_path)
    assert coverage['non_inventory_source']['sha256'] == sha(text_path)
    inventory = {row['id']: row for row in register['entries']}
    extract = extractor.seed_rows(text, '7.2.5')
    rows = {row['source_id']: row for row in coverage['source_rows']}
    assert len(rows) == len(coverage['source_rows'])
    assert set(rows) == set(inventory) | {r['source_id'] for r in extract}
    results = read(HERE / 'patch_7_2_5_publication_sweep-results.json')
    assert set(results) == set(inventory)
    gaps = {key for key, row in results.items() if not row['ok']}
    assert gaps == set(read(ROOT / 'tests/data/patch_7_2_5_sweep_known_gaps.json'))
    review = read(HERE / 'p725-gap-review.json')
    assert {r['source_id'] for r in review['publication_gaps']} == gaps
    for key in inventory:
        assert rows[key]['status'] == ('audit-pending' if key in gaps else 'bounded-coverage')
        assert rows[key]['note']
        assert bool(rows[key]['capabilities']) == (key not in gaps)
    scout = read(HERE / 'p725-extract-scout.json')
    assert {r['source_id'] for r in scout} == {r['source_id'] for r in extract}
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw.splitlines()[row['wikitext_line'] - 1]
        assert rows[row['source_id']]['status'] == row['status']
        assert rows[row['source_id']]['note'] == row['reason']
    assert all(r['note'] for r in rows.values())
    return {'inventory': len(inventory), 'extract': len(extract), 'ledger': len(rows),
            'gaps': len(gaps), 'statuses': dict(Counter(r['status'] for r in rows.values()))}


def preservation_and_reproduction(context):
    preservation = read(HERE / 'p725-input-preservation.json')
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', preservation['base_revision'],
                                     'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
    assert {r['path'] for r in preservation['rows']} == set(paths)
    for row in preservation['rows']:
        original = hashlib.sha256(blob(preservation['base_revision'], row['path'])).hexdigest()
        assert original == row['before_sha256'] == row['after_sha256']
        assert preserved_input_matches(ROOT, row['path'], original), row['path']
    modes = read(HERE / 'p725-extract-preservation.json')
    assert all(r['before'] == r['after'] for r in modes)
    registers = historical_registers(ROOT, context['runtime_revision'])
    patches = {p.name.removesuffix('-wikitext-register.json') for p in registers}
    reproductions = read(HERE / 'p725-register-reproduction.json')
    extracts = read(HERE / 'p725-saved-extract-reproduction.json')
    assert {r['patch'] for r in reproductions} == patches
    assert {r['patch'] for r in extracts} == patches
    for row in reproductions:
        assert row['exit'] == 0 and row['byte_identical']
        path = f"data/patch-api/sources/{row['patch']}-wikitext-register.json"
        assert preserved_input_matches(ROOT, path, row['sha256'])
    inherited = {r['patch']: r for r in read(ROOT / 'data/patch-api/evidence/7.3.2-session-2026-10-08/p732-saved-extract-reproduction.json')}
    for row in extracts:
        path = f"data/patch-api/sources/{row['patch']}-api-changes.txt"
        assert preserved_input_matches(ROOT, path, row['saved_sha256'])
        if row['patch'] in inherited:
            assert (row['byte_identical'], row['error']) == (inherited[row['patch']]['byte_identical'], inherited[row['patch']]['error'])
        else:
            assert row['byte_identical']
    return {'preserved_inputs': len(paths), 'extract_modes': len(modes), 'registers': len(patches),
            'extracts_reproduced': sum(r['byte_identical'] for r in extracts)}


def scans():
    evidence = read(HERE / 'p725-retirement-scans.json')
    assert evidence['member'] == 'ReloadUI' and evidence['decision'].startswith('retain:')
    assert len(evidence['scans']) == 3
    for row in evidence['scans']:
        assert row['tool'] == '/usr/bin/grep' and row['command'][0] == '/usr/bin/grep'
        assert r'\bReloadUI\b' in row['command'] and row['exit'] in (0, 1)
        assert sha(HERE / row['file']) == row['sha256']
        assert len((HERE / row['file']).read_text().splitlines()) == row['matches']
    assert evidence['scans'][0]['matches'] > 0
    assert '--exclude=*Documentation*' in evidence['scans'][0]['command']
    for branch in ('master', 'p730-page'):
        row = evidence['later_registers'][branch]
        paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', row['revision'],
                                         'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
        registers = [p for p in paths if p.endswith('-wikitext-register.json')]
        assert set(registers) == set(row['registers'])
        additions = []
        for path in registers:
            data = json.loads(blob(row['revision'], path))
            additions.extend({'path': path, 'entry': entry} for entry in data['entries']
                             if entry['symbol'] == 'ReloadUI' and entry['direction'] == 'added')
        assert additions == row['readditions']
    assert evidence['later_registers']['p730-page']['p730_register_present']


def proof(context):
    for name, expected in context['receipts'].items():
        receipt = read(HERE / f'{name}.proof.json')
        assert receipt['exit'] == expected['exit'] and not receipt['invalidated']
        assert receipt['command'] == expected['command']
        assert receipt['revision'] == expected['revision']
        assert sha(HERE / receipt['log']) == receipt['log_sha256']
    positive = ('p725-own-green', 'p725-all-sweeps', 'p725-bare-trees',
                'p725-garrison-callers', 'p725-anima-callers', 'p725-generator-fixtures',
                'p725-extractor-fixtures', 'p725-validator-fixtures', 'p725-source-reproduction',
                'p725-format', 'p725-mists-check', 'p725-startup', 'p725-other-validators')
    for name in positive:
        assert context['receipts'][name]['exit'] == 0, name
    for name in ('p725-discovery', 'p725-garrison-red', 'p725-negative',
                 'p725-cached-garrison', 'p725-base-garrison-case'):
        assert context['receipts'][name]['exit'] == 1, name
    diagnostic = 'Blizzard_AdventuresCombatLog.lua:90'
    for name in ('p725-cached-garrison', 'p725-base-garrison-case'):
        log = (HERE / f'{name}.txt').read_text()
        assert diagnostic in log and 'table expected, got nil' in log
        assert 'blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors ... FAILED' in log
    baseline = read(HERE / 'p725-garrison-baseline-context.json')
    for row in baseline['files']:
        before = hashlib.sha256(blob(baseline['base_revision'], row['path'])).hexdigest()
        audited = hashlib.sha256(blob(context['runtime_revision'], row['path'])).hexdigest()
        assert before == audited == row['base_sha256'] == row['current_sha256']
    for name in ('p725-bare-trees', 'p725-garrison-callers', 'p725-anima-callers'):
        log = (HERE / f'{name}.txt').read_text()
        passed = re.findall(r'test result: ok\. (\d+) passed', log)
        assert passed and int(passed[-1]) > 0, name
    for path, digest in context['runtime_files'].items():
        assert hashlib.sha256(blob(context['runtime_revision'], path)).hexdigest() == digest
    warnings = [line for line in (HERE / 'p725-mists-check.txt').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    expected = {p.name.removesuffix('.rs') for p in historical_sweep_tests(ROOT, context['runtime_revision'])}
    summaries = read(HERE / 'p725-sweep-summary.json')
    assert {r['test'] for r in summaries} == expected
    for summary in summaries:
        observations = read(HERE / summary['file'])
        assert summary['rows'] == len(observations)
        assert summary['gaps'] == sum(not r['ok'] for r in observations.values())
        fixture = ROOT / f"tests/data/{summary['test'].removesuffix('_publication_sweep')}_sweep_known_gaps.json"
        assert {key for key, row in observations.items() if not row['ok']} == set(read(fixture))
    negative = read(HERE / 'p725-negative-results.json')
    baseline = read(HERE / 'patch_7_2_5_publication_sweep-results.json')
    control = read(HERE / 'p725-negative-control.json')
    assert control['before'] == sum(not r['ok'] for r in baseline.values())
    assert control['after'] == sum(not r['ok'] for r in negative.values()) == control['before'] + 1
    assert {key for key in baseline if baseline[key]['ok'] != negative[key]['ok']} == {control['mutated_id']}
    return {'publication_sweeps': len(expected), 'negative_control': [control['before'], control['after']]}


def main():
    context = read(HERE / 'p725-context.json')
    result = {**source_and_accounting(), **preservation_and_reproduction(context), **proof(context)}
    scans()
    spec = importlib.util.spec_from_file_location('integrated', HERE / 'validate_integrated.py')
    integrated = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(integrated)
    integrated.main()
    print(json.dumps({'status': 'pass', **result}, sort_keys=True))


if __name__ == '__main__':
    main()
