"""Validate pinned 8.3.0 accounting; integration-sensitive counts derive from inputs."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '8.3.0'


def read_json(path):
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_sources():
    provenance = read_json(SOURCES / f'{PATCH}-api-changes.provenance.json')
    register = read_json(SOURCES / f'{PATCH}-wikitext-register.json')
    page = next(iter(read_json(EVIDENCE / 'p830-fetch.json')['query']['pages'].values()))
    revision = page['revisions'][0]
    raw_path = SOURCES / f'{PATCH}-api-changes.wikitext'
    assert revision['slots']['main']['*'] == raw_path.read_text()
    assert (page['pageid'], page['title']) == (provenance['pageid'], provenance['title'])
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert sha256(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    lines = raw_path.read_text().splitlines()
    # Each literal API reference in consolidated columns or explicit late-build additions.
    expected = set()
    for number, line in enumerate(lines, 1):
        if line.startswith(': {{api|') or line.startswith(('* New functions:', '* New event:')):
            for body in re.findall(r'\{\{api\|([^{}]+)\}\}', line):
                symbol = [part for part in body.split('|') if '=' not in part][-1]
                expected.add((number, symbol))
    actual = {(row['wikitext_line'], row['symbol']) for row in register['entries']}
    assert actual == expected
    assert len(actual) == len(register['entries'])
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    extractor = load_extractor()
    flags = {flag.removeprefix('--').replace('-', '_'): True
             for flag in provenance['extractor_flags']}
    text = (SOURCES / f'{PATCH}-api-changes.txt').read_text()
    assert extractor.extract_text(raw_path.read_text(), **flags) == text
    return register, extractor, lines, text


def check_accounting(register, extractor, raw_lines, text):
    stem = 'patch_' + PATCH.replace('.', '_') + '_publication_sweep'
    results = read_json(EVIDENCE / f'{stem}-results.json')
    known = set(read_json(ROOT / f'tests/data/patch_{PATCH.replace(".", "_")}_sweep_known_gaps.json'))
    inventory = {row['id'] for row in register['entries']}
    assert set(results) == inventory
    assert {key for key, row in results.items() if not row['ok']} == known
    reviews = read_json(EVIDENCE / 'p830-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    for row in reviews:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['observation'] == results[row['source_id']]['observed']
        assert row['expectation'] == results[row['source_id']]['expected']
        assert row['reason']
    ledger = read_json(SOURCES / f'{PATCH}-page-coverage.json')
    indexed = {row['source_id']: row for row in ledger['source_rows']}
    extract_ids = {row['source_id'] for row in extractor.seed_rows(text, PATCH)}
    contexts = read_json(EVIDENCE / 'p830-build-context.json')
    context_ids = {row['source_id'] for row in contexts}
    assert len(indexed) == len(ledger['source_rows'])
    assert set(indexed) == inventory | extract_ids | context_ids
    assert {row['wikitext_line'] for row in contexts} == {
        number for number, line in enumerate(raw_lines, 1) if line.startswith('|+')}
    for row in contexts:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert indexed[row['source_id']]['status'] == 'metadata-only'
    assert ledger['source_sha256'] == sha256(SOURCES / f'{PATCH}-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(SOURCES / f'{PATCH}-api-changes.txt')
    for key, result in results.items():
        status = ('audit-pending' if key in known else 'bounded-coverage'
                  if result['expected']['publication'] == 'absent' else 'partial-development-green')
        assert indexed[key]['status'] == status
        assert bool(indexed[key]['capabilities']) == result['ok']
    scout = read_json(EVIDENCE / 'p830-extract-scout.json')
    assert {row['source_id'] for row in scout} == extract_ids
    for row in scout:
        source = indexed[row['source_id']]
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['reason'] == source['note'] and row['status'] == source['status']
        assert not source['capabilities']
    negative = read_json(EVIDENCE / 'p830-expanded-negative-observation.json')
    receipt = read_json(EVIDENCE / 'p830-expanded-negative-result.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == inventory
    assert sorted(failures - known) == receipt['new_gaps'] == [receipt['mutation']]
    assert sorted(known - failures) == receipt['resolved_gaps'] == []
    assert (len(known), len(failures)) == (receipt['baseline_gaps'], receipt['negative_gaps'])
    assert receipt['exit'] != 0
    initial = read_json(EVIDENCE / 'p830-discovery-results.json')
    closed = [key for key, row in initial.items() if not row['ok'] and results[key]['ok']]
    return {'inventory_rows': len(inventory), 'extract_rows': len(extract_ids),
            'context_rows': len(context_ids), 'ledger_rows': len(indexed),
            'ledger_statuses': dict(Counter(row['status'] for row in indexed.values())),
            'gaps_remaining': len(known), 'gaps_closed': len(closed)}


def check_preservation():
    hashes = read_json(EVIDENCE / 'p830-input-hashes-before.json')
    for path, digest in hashes.items():
        assert sha256(ROOT / path) == digest, path
    outcomes = read_json(EVIDENCE / 'p830-extract-preservation.json')
    prior = [row for row in outcomes if row['patch'] != PATCH]
    assert all(row['before'] == row['after'] and row['unchanged'] for row in prior)
    registers = {path.name.removesuffix('-wikitext-register.json')
                 for path in SOURCES.glob('*-wikitext-register.json')}
    reproduced = read_json(EVIDENCE / 'p830-register-reproduction.json')
    assert {row['patch'] for row in reproduced} == registers
    for row in reproduced:
        assert row['exit'] == 0 and row['byte_identical']
        assert sha256(SOURCES / f"{row['patch']}-wikitext-register.json") == row['sha256']
        provenance_path = SOURCES / f"{row['patch']}-api-changes.provenance.json"
        recorded = read_json(provenance_path).get('generator_flags') if provenance_path.exists() else None
        assert recorded == row['recorded_flags']
        if recorded is not None:
            assert recorded == row['verified_flags']
    saved = read_json(EVIDENCE / 'p830-saved-extract-reproduction.json')
    assert {row['patch'] for row in saved} == registers
    for row in saved:
        assert sha256(SOURCES / f"{row['patch']}-api-changes.txt") == row['sha256']
        if row['patch'] not in ('12.0.5', '12.0.7', '12.1.0'):
            assert row['byte_identical'], row['patch']
        if row['recorded_flags'] is not None:
            assert set(row['recorded_flags']) == set(row['verified_flags'])
    return {'preserved_inputs': len(hashes), 'preserved_extract_modes': len(prior),
            'reproduced_registers': len(registers),
            'inherited_nonreproducible_extracts': [row['patch'] for row in saved if not row['byte_identical']]}


def check_sweeps():
    summary = []
    registers = sorted(SOURCES.glob('*-wikitext-register.json'),
                       key=lambda path: tuple(map(int, path.name.split('-')[0].split('.'))))
    for register_path in registers:
        patch = register_path.name.removesuffix('-wikitext-register.json')
        stem = 'patch_' + patch.replace('.', '_') + '_publication_sweep'
        results = read_json(EVIDENCE / f'{stem}-results.json')
        rows = read_json(register_path)['entries']
        assert set(results) == {row['id'] for row in rows}
        gaps = {key for key, row in results.items() if not row['ok']}
        assert gaps == set(read_json(ROOT / f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json'))
        later = {}
        for path in registers:
            if tuple(map(int, path.name.split('-')[0].split('.'))) <= tuple(map(int, patch.split('.'))):
                continue
            for row in read_json(path)['entries']:
                if row['direction'] != 'changed':
                    later[row['symbol']] = row
        for row in rows:
            own_removed = row['direction'] == 'removed'
            newer = later.get(row['symbol'])
            changed = newer is not None and (newer['direction'] == 'removed') != own_removed
            expected = results[row['id']]['expected']
            assert expected['publication'] == ('absent' if (not own_removed if changed else own_removed) else 'published')
            assert expected['superseded_by'] == (newer['id'] if changed else None)
        summary.append({'patch': patch, 'rows': len(rows), 'ok': len(rows) - len(gaps),
                        'gaps': len(gaps), 'result': 'pass'})
    return summary


def check_proof():
    receipts = [read_json(path) for path in sorted(EVIDENCE.glob('*.proof.json'))]
    for row in receipts:
        assert row['cwd'] == str(ROOT) and row['target'] == str(ROOT / 'target')
        assert sha256(EVIDENCE / row['log']) == row['log_sha256']
        if not row.get('invalidated') and not row.get('unrelated_failure'):
            assert row['exit'] == row['expected_exit'], row['scope']
    mists = read_json(EVIDENCE / 'p830-mists-changed-surface-results.json')
    log = (EVIDENCE / mists['log']).read_text()
    for case in mists['affected_cases']:
        assert f'test {case} ... ok' in log, case
    assert mists['unrelated_failure']['case'] in log
    assert (EVIDENCE / 'p830-startup.stdout').read_text().strip() == '[]'
    warnings = [line for line in (EVIDENCE / 'p830-mists-check.txt').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)
    return {'proof_receipts': len(receipts), 'affected_mists_cases': len(mists['affected_cases']),
            'unrelated_failure': mists['unrelated_failure']}


def main():
    register, extractor, lines, text = check_sources()
    report = check_accounting(register, extractor, lines, text)
    report.update(check_preservation())
    report.update(check_proof())
    report['sweeps'] = check_sweeps()
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
