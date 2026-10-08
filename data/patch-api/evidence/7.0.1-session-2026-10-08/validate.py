"""Read-only, checkout-independent proof of the pinned 7.0.1 redirect audit."""
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


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def paths_at(revision, directory):
    return subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision, directory],
                                   cwd=ROOT, text=True).splitlines()


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def source_accounting():
    provenance = read(SOURCES / '7.0.1-api-changes.provenance.json')
    fetch = read(HERE / 'p701-fetch.json')
    page = fetch['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    raw_path = SOURCES / '7.0.1-api-changes.wikitext'
    raw = raw_path.read_text()
    assert raw == revision['slots']['main']['*']
    assert re.fullmatch(r'#REDIRECT \[\[Patch 7\.0\.3/API changes\]\]\s*', raw)
    assert provenance['pageid'] == 336026
    assert provenance['revid'] == revision['revid'] == 3241525
    assert provenance['timestamp'] == revision['timestamp']
    assert provenance['sha256'] == digest(HERE / 'p701-fetch.json')
    assert provenance['wikitext_sha256'] == digest(raw_path)
    register_path = SOURCES / '7.0.1-wikitext-register.json'
    register = read(register_path)
    assert register['source']['revid'] == provenance['revid']
    assert register['source']['sha256'] == digest(raw_path)
    assert register['entries'] == register['header_counts'] == []
    text_path = SOURCES / '7.0.1-api-changes.txt'
    extractor = load_extractor()
    assert extractor.extract_text(raw) == text_path.read_text()
    ledger = read(SOURCES / '7.0.1-page-coverage.json')
    assert ledger['source_sha256'] == digest(register_path)
    assert ledger['non_inventory_source']['sha256'] == digest(text_path)
    seeded = extractor.seed_rows(text_path.read_text(), '7.0.1')
    assert {r['source_id'] for r in seeded} == {r['source_id'] for r in ledger['source_rows']}
    assert all(r['status'] == 'metadata-only' and not r['capabilities'] and r['note']
               for r in ledger['source_rows'])
    assert read(ROOT / 'tests/data/patch_7_0_1_sweep_known_gaps.json') == []
    assert read(HERE / 'patch_7_0_1_publication_sweep-results.json') == {}
    return {'inventory': len(register['entries']), 'extract': len(seeded),
            'statuses': dict(Counter(r['status'] for r in ledger['source_rows']))}


def preserved_sources(context):
    evidence = read(HERE / 'p701-input-preservation.json')
    expected = paths_at(evidence['base_revision'], 'data/patch-api/sources')
    assert {r['path'] for r in evidence['rows']} == set(expected)
    for row in evidence['rows']:
        original = hashlib.sha256(blob(evidence['base_revision'], row['path'])).hexdigest()
        assert row['before_sha256'] == row['after_sha256'] == original
        assert preserved_input_matches(ROOT, row['path'], original), row['path']
    modes = read(HERE / 'p701-extract-preservation.json')
    assert all(row['before'] == row['after'] for row in modes)
    registers = historical_registers(ROOT, context['runtime_revision'])
    patches = {p.name.removesuffix('-wikitext-register.json') for p in registers}
    reproduced = read(HERE / 'p701-register-reproduction.json')
    extracts = read(HERE / 'p701-saved-extract-reproduction.json')
    assert {r['patch'] for r in reproduced} == {r['patch'] for r in extracts} == patches
    for row in reproduced:
        assert row['exit'] == 0 and row['byte_identical']
        assert preserved_input_matches(ROOT, f"data/patch-api/sources/{row['patch']}-wikitext-register.json", row['sha256'])
    inherited = {r['patch']: r for r in read(ROOT / 'data/patch-api/evidence/7.1.0-session-2026-10-08/integrated/p710-saved-extract-reproduction.json')}
    for row in extracts:
        assert preserved_input_matches(ROOT, f"data/patch-api/sources/{row['patch']}-api-changes.txt", row['saved_sha256'])
        if row['patch'] in inherited:
            assert (row['byte_identical'], row['error']) == (inherited[row['patch']]['byte_identical'], inherited[row['patch']]['error'])
        else:
            assert row['byte_identical'] and row['error'] is None
    return {'inputs': len(expected), 'extract_modes': len(modes), 'registers': len(registers),
            'extracts_reproduced': sum(r['byte_identical'] for r in extracts),
            'inherited_extract_failures': [r['patch'] for r in extracts if not r['byte_identical']]}


def retirements():
    scans = read(HERE / 'p701-retirement-scans.json')
    assert scans['members'] == scans['scans'] == []
    assert scans['tool'] == '/usr/bin/grep'
    for row in scans['later_registers'].values():
        registers = [p for p in paths_at(row['revision'], 'data/patch-api/sources')
                     if p.endswith('-wikitext-register.json')]
        assert set(row['registers']) == set(registers)
        assert row['readditions'] == []
        assert row['p703_register_present'] == ('data/patch-api/sources/7.0.3-wikitext-register.json' in registers)


def command_proof(context):
    for name, expected in context['receipts'].items():
        receipt = read(HERE / f'{name}.proof.json')
        assert digest(HERE / f'{name}.proof.json') == expected['sha256']
        assert receipt['exit'] == 0 and not receipt['invalidated']
        assert receipt['revision'] == expected['revision']
        assert receipt['command'] == expected['command']
        assert digest(HERE / receipt['log']) == receipt['log_sha256']
    for name in ('p701-discovery', 'p701-all-sweeps'):
        log = (HERE / f'{name}.txt').read_text()
        passes = re.findall(r'test result: ok\. (\d+) passed', log)
        assert passes and int(passes[-1]) > 0, name
    summaries = read(HERE / 'p701-sweep-summary.json')
    tests = historical_sweep_tests(ROOT, context['runtime_revision'])
    assert {row['test'] for row in summaries} == {p.stem for p in tests}
    for row in summaries:
        observed = read(HERE / row['file'])
        assert row['rows'] == len(observed)
        assert row['gaps'] == sum(not v['ok'] for v in observed.values())
        fixture = f"tests/data/{row['test'].removesuffix('_publication_sweep')}_sweep_known_gaps.json"
        assert {k for k, v in observed.items() if not v['ok']} == set(json.loads(blob(context['runtime_revision'], fixture)))
    warnings = [line for line in (HERE / 'p701-mists-check.txt').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings), warnings
    matrix = read(HERE / 'p701-other-validator-matrix.json')
    prior = [p for p in paths_at(context['base_revision'], 'data/patch-api/evidence')
             if Path(p).name in ('validate.py', 'validate_integrated.py')]
    assert {row['path'] for row in matrix} == set(prior)
    assert all(row['exit'] == 0 for row in matrix)
    return {'sweeps': len(summaries), 'validators': len(matrix)}


def documentation(context):
    # Historical wiki proof does not freeze legitimate future wiki edits.
    for name in ('index.md', 'log.md'):
        path = f'docs/wiki/{name}'
        before = blob(context['base_revision'], path).decode()
        after = blob(context['documentation_revision'], path).decode()
        assert before in after
        assert len(after.splitlines()) >= len(before.splitlines())


def main():
    context = read(HERE / 'p701-proof-context.json')
    result = {'source': source_accounting(), 'reproduction': preserved_sources(context)}
    retirements()
    result['proof'] = command_proof(context)
    documentation(context)
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
