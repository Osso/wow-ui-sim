"""Validate retained audit artifacts; never rerun Cargo or query a live cache."""
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read_json(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools' / f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    register = read_json(SOURCES / '10.0.0-wikitext-register.json')
    coverage = read_json(SOURCES / '10.0.0-page-coverage.json')
    observed = read_json(EVIDENCE / 'p1000-final.json')
    final_discovery = read_json(EVIDENCE / '10.0.0-sweep.json')
    assert final_discovery == observed
    assert len(register['entries']) == len(observed) == 639
    assert register['source']['revid'] == 6789768
    assert register['source']['sha256'] == digest(SOURCES / '10.0.0-api-changes.wikitext')
    assert coverage['source_sha256'] == digest(SOURCES / '10.0.0-wikitext-register.json')
    assert set(observed) == {entry['id'] for entry in register['entries']}
    gaps = {key for key, value in observed.items() if not value['ok']}
    known = read_json(ROOT / 'tests/data/patch_10_0_0_sweep_known_gaps.json')
    assert len(gaps) == 173 and gaps == set(known)
    reviews = read_json(EVIDENCE / 'p1000-gap-review.json')
    assert len(reviews) == len(gaps)
    assert {row['source_id'] for row in reviews} == gaps
    assert all(row['reason'] and row['outcome'] == 'retained-gap' for row in reviews)
    source_scans = read_json(EVIDENCE / 'p1000-gap-source-scans.json')
    assert set(source_scans) == gaps
    consumers = read_json(EVIDENCE / 'p1000-exact-removal-consumers.json')
    assert len(consumers) == 200
    assert all(set(row['scans']) == {'qualified', 'bare'} for row in consumers.values())
    assert all(row['scans']['qualified']['exit'] in (0, 1) and
               row['scans']['bare']['exit'] in (0, 1) for row in consumers.values())
    scan_command = read_json(EVIDENCE / 'p1000-whole-caller-command.json')
    assert scan_command['command'][-2:] == ['src', 'tests']
    assert scan_command['lines'] == len((EVIDENCE / 'p1000-whole-caller-scan.txt').read_text().splitlines()) == 504

    later_paths = sorted(SOURCES.glob('*-wikitext-register.json'),
                         key=lambda path: tuple(int(value) for value in path.name.split('-')[0].split('.')))
    assert len(later_paths) == 26 and later_paths[0].name.startswith('10.0.0-')
    latest = {}
    table = []
    for path in later_paths[1:]:
        data = read_json(path)
        for entry in data['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
        result = read_json(EVIDENCE / f"{data['patch']}-sweep.json")
        assert set(result) == {entry['id'] for entry in data['entries']}
        actual = {key for key, value in result.items() if not value['ok']}
        fixture = ROOT / 'tests/data' / f"patch_{data['patch'].replace('.', '_')}_sweep_known_gaps.json"
        assert actual == set(read_json(fixture))
        table.append({'patch': data['patch'], 'rows': len(result), 'ok': len(result) - len(actual), 'gaps': len(actual)})
    for entry in register['entries']:
        expectation = observed[entry['id']]['expected']
        successor = latest.get(entry['symbol'])
        direction = successor['direction'] if successor else entry['direction']
        expected_publication = 'absent' if direction == 'removed' else 'published'
        assert expectation['publication'] == expected_publication
        flipped = successor and (successor['direction'] == 'removed') != (entry['direction'] == 'removed')
        assert expectation['superseded_by'] == (successor['id'] if flipped else None)

    extractor = load_tool('extract_patch_non_inventory')
    raw = (SOURCES / '10.0.0-api-changes.wikitext').read_text()
    text = (SOURCES / '10.0.0-api-changes.txt').read_text()
    assert text == extractor.extract_text(raw, preserve_examples=True)
    assert digest(SOURCES / '10.0.0-api-changes.txt') == coverage['non_inventory_source']['sha256']
    seeded = extractor.seed_rows(text, '10.0.0')
    scout = read_json(EVIDENCE / 'p1000-extract-scout.json')
    assert len(scout) == len(seeded) == 439
    assert [row['source_id'] for row in scout] == [row['source_id'] for row in seeded]
    assert all(row['literal'] == text.splitlines()[row['extract_line'] - 1] and not row['behavioral_credit'] for row in scout)
    context = read_json(EVIDENCE / 'p1000-inventory-context.json')
    assert len(context) == 4
    assert all(row['literal'] == raw.splitlines()[row['wikitext_line'] - 1] for row in context)
    rows = coverage['source_rows']
    assert len(rows) == len({row['source_id'] for row in rows}) == 1082
    expected_ids = set(observed) | {row['source_id'] for row in scout} | {row['source_id'] for row in context}
    assert {row['source_id'] for row in rows} == expected_ids
    counts = Counter(row['status'] for row in rows)
    assert counts == {'bounded-coverage': 466, 'audit-pending': 600, 'metadata-only': 16}
    assert all(not row['capabilities'] for row in rows if row['status'] != 'bounded-coverage')

    baseline = read_json(EVIDENCE / 'p1000-final.json')
    negative = read_json(EVIDENCE / 'p1000-negative.json')
    negative_gaps = {key for key, value in negative.items() if not value['ok']}
    assert negative_gaps - gaps == {'wt-global-api-GetUnitEmpowerHoldAtMaxTime-486'}
    assert not gaps - negative_gaps and len(negative_gaps) == 174
    own_register = read_json(SOURCES / '10.0.0-wikitext-register.json')
    control_register = read_json(EVIDENCE / 'p1000-negative-register.json')
    differences = [(first, second) for first, second in zip(own_register['entries'], control_register['entries']) if first != second]
    assert len(differences) == 1
    first, second = differences[0]
    assert first['direction'] == 'added' and second == dict(first, direction='removed')
    assert set(baseline) == set(negative)

    preserved = read_json(EVIDENCE / 'p1000-inputs-before.json')
    assert len(preserved) == 158
    assert all(digest(ROOT / name) == value for name, value in preserved.items())
    before = read_json(EVIDENCE / 'p1000-extract-before.json')
    after = read_json(EVIDENCE / 'p1000-extract-after.json')
    modes = {tuple(row['command']): row['exit'] for row in after}
    assert len(before) == 50 and len(after) == 52
    assert all(modes[tuple(row['command'])] == row['exit'] for row in before)
    reproductions = read_json(EVIDENCE / 'p1000-register-reproduction.json')
    assert len(reproductions) == 26 and all(row['reproduced'] for row in reproductions)
    proof = read_json(EVIDENCE / 'p1000-proof.json')
    assert all(row['cwd'] == str(ROOT) for row in proof)
    for row in proof:
        assert digest(EVIDENCE / row['log']) == row['log_sha256']
        if 'stdout_log' in row:
            assert digest(EVIDENCE / row['stdout_log']) == row['stdout_sha256']
        if 'expected_exit' in row:
            assert row['exit'] == row['expected_exit']
        else:
            assert row['exit'] == 0
    warnings = [line for line in (EVIDENCE / 'p1000-mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert len(warnings) == 7 and all('iced' in line for line in warnings)
    assert read_json(EVIDENCE / 'p1000-startup-stdout.json') == []
    result = {'result': 'PASS', 'inventory': 639, 'extract': 439, 'inventory_context': 4,
              'ledger': 1082, 'statuses': dict(counts), 'publication_gaps': 173,
              'prior_inputs_preserved': 158, 'prior_extract_modes_preserved': 50,
              'registers_reproduced': 26, 'proof_logs_verified': len(proof),
              'sweep_table': [{'patch': '10.0.0', 'rows': 639, 'ok': 466, 'gaps': 173}] + table}
    (EVIDENCE / 'p1000-validation-result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
