"""Read-only stub-audit gate; fixed historical scope, no checkout path gates."""
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
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests, preserved_input_matches


def read(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_source():
    sources = ROOT / 'data/patch-api/sources'
    provenance = read(sources / '6.2.2-api-changes.provenance.json')
    page = read(HERE / 'p622-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    raw = (sources / '6.2.2-api-changes.wikitext').read_text()
    assert page['pageid'] == 122147
    assert page['title'] == provenance['title'] == 'Patch 6.2.2/API changes'
    assert revision['revid'] == provenance['revid'] == 6209268
    assert revision['timestamp'] == provenance['timestamp']
    assert raw == revision['slots']['main']['*']
    assert raw == '{{apichanges|6.2.2|prev=6.2.0|next=6.2.4}}'
    assert digest(raw.encode()) == provenance['sha256']
    assert provenance['classification'] == 'navigation-only-stub' and not provenance['redirect']
    assert provenance['generator_flags'] == provenance['extractor_flags'] == []
    register = read(sources / '6.2.2-wikitext-register.json')
    assert register['source']['revid'] == revision['revid']
    assert register['source']['sha256'] == provenance['sha256']
    assert register['entries'] == register['header_counts'] == []
    text_path = sources / '6.2.2-api-changes.txt'
    extractor = load_tool('extract_patch_non_inventory')
    assert extractor.extract_text(raw).encode() == text_path.read_bytes()
    ledger = read(sources / '6.2.2-page-coverage.json')
    assert ledger['source_sha256'] == digest((sources / '6.2.2-wikitext-register.json').read_bytes())
    assert ledger['non_inventory_source']['sha256'] == digest(text_path.read_bytes())
    assert ledger['source_rows'] == extractor.seed_rows(text_path.read_text(), '6.2.2')
    assert all(row['status'] == 'metadata-only' and not row['capabilities'] for row in ledger['source_rows'])
    assert read(HERE / 'p622-accounting-summary.json') == {
        'classification': provenance['classification'], 'inventory_rows': len(register['entries']),
        'inventory_gaps': len(read(HERE / 'p622-gap-review.json')), 'modeled_gaps': 0,
        'problematic_gaps': len(read(HERE / 'p622-problematic-contracts.json')),
        'extract_rows': len(ledger['source_rows']),
        'metadata_rows': Counter(row['status'] for row in ledger['source_rows'])['metadata-only'],
        'retirements': len(read(HERE / 'p622-retirement-decisions.json')['new_retirements']),
    }
    decisions = read(HERE / 'p622-retirement-decisions.json')
    assert decisions['removed_entries'] == decisions['new_retirements'] == decisions['scans'] == []
    assert not read(ROOT / 'tests/data/patch_6_2_2_sweep_known_gaps.json')


def check_reproduction(context):
    expected = {path.name.removesuffix('-wikitext-register.json')
                for path in historical_registers(ROOT, context['source_revision'])}
    registers = read(HERE / 'p622-register-reproduction.json')
    extracts = read(HERE / 'p622-saved-extract-reproduction.json')
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    assert len(registers) == len(extracts) == len(expected)
    inherited = read(ROOT / 'data/patch-api/evidence/7.0.3-session-2026-10-08/p703-saved-extract-reproduction.json')
    failures = {row['patch']: (row['exit'], row['error']) for row in inherited if not row['byte_identical']}
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical'], row['patch']
        path = ROOT / f"data/patch-api/sources/{row['patch']}-wikitext-register.json"
        assert digest(path.read_bytes()) == row['sha256'], row['patch']
    for row in extracts:
        assert row['byte_identical'] or failures.get(row['patch']) == (row['exit'], row['error'])
        path = ROOT / f"data/patch-api/sources/{row['patch']}-api-changes.txt"
        assert digest(path.read_bytes()) == row['sha256'], row['patch']
    preserved = read(HERE / 'p622-input-preservation.json')
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', preserved['revision'],
                                    'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
    assert set(preserved['sha256']) == set(names)
    source_suffixes = ('-api-changes.wikitext', '-api-changes.txt',
                       '-api-changes.provenance.json', '-wikitext-register.json')
    for name, sha256 in preserved['sha256'].items():
        assert digest(blob(preserved['revision'], name)) == sha256
        # Later audits may update accounting ledgers and scheduling inventories.
        # Preserve pinned source bytes, not unrelated audits' evolving outcomes.
        if name.endswith(source_suffixes):
            assert preserved_input_matches(ROOT, name, sha256), name
    return len(registers), sum(row['byte_identical'] for row in extracts)


def check_proofs(context):
    for label, policy in context['proofs'].items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt['command'] == policy['command'] and receipt['exit'] == policy['exit'], label
        log = (HERE / receipt['log']).read_bytes()
        assert digest(log) == receipt['log_sha256'], label
        runtime_delta = subprocess.check_output(
            ['git', 'diff', '--name-only', context['base_revision'], receipt['revision'],
             '--', 'src', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs'], cwd=ROOT, text=True,
        )
        assert not runtime_delta, (label, runtime_delta)
        for name in context['proof_scope']:
            assert receipt['scope'][name] == digest(blob(receipt['revision'], name)), (label, name)
        if policy['command'][:2] == ['cargo', 'test'] and policy['exit'] == 0:
            assert re.search(r'test result: ok\. [1-9]\d* passed; 0 failed;', log.decode()), label
        if label == 'p622-mists-check':
            warnings = [line for line in log.decode().splitlines() if line.startswith('warning:')]
            assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
                       for line in warnings), warnings
    for label in ['p622-discovery', 'p622-all-sweeps']:
        assert read(HERE / (label + '-results.json')) == {}
    negative_log = (HERE / 'p622-negative.log').read_text()
    assert 'register row count changed' in negative_log
    assert 'left: 1' in negative_log and 'right: 0' in negative_log
    log = (HERE / 'p622-all-sweeps.log').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))
    revision = read(HERE / 'p622-all-sweeps.proof.json')['revision']
    for path in historical_sweep_tests(ROOT, revision):
        relative = str(path.relative_to(ROOT))
        source = blob(revision, relative).decode()
        for function in re.findall(r'fn (patch_\w+_publication_sweep)\(', source):
            assert any(name.endswith('::' + function) for name in passed), function
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = json.loads(blob(revision, f'data/patch-api/sources/{patch}-wikitext-register.json'))
        result_path = 'p622-all-sweeps-results.json' if patch == '6.2.2' else path.stem + '-results.json'
        results = read(HERE / result_path)
        assert set(results) == {row['id'] for row in register['entries']}, patch
        known = json.loads(blob(revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json'))
        assert {key for key, row in results.items() if not row['ok']} == set(known), patch
    return len(passed)


def check_prior_validators(context):
    matrix = read(HERE / 'p622-prior-validator-matrix.json')
    assert matrix['scope_revision'] == context['base_revision']
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', matrix['scope_revision'],
                                    'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    expected = {name for name in names if name.endswith('/validate.py')}
    rows = matrix['validators']
    assert {row['path'] for row in rows} == expected and len(rows) == len(expected)
    for row in rows:
        assert row['exit'] == 0, row['path']
        assert digest((HERE / row['log']).read_bytes()) == row['log_sha256']
    return len(rows)


def main():
    context = read(HERE / 'p622-context.json')
    check_source()
    registers, extracts = check_reproduction(context)
    sweeps = check_proofs(context)
    validators = check_prior_validators(context)
    print(json.dumps({'patch': '6.2.2', 'classification': 'navigation-only-stub',
                      'registers': registers, 'extracts': extracts, 'sweep_cases': sweeps,
                      'prior_validators': validators, 'result': 'PASS'}))


if __name__ == '__main__':
    main()
