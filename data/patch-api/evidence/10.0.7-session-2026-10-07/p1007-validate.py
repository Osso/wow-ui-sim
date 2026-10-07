"""Validate retained 10.0.7 accounting and local proof artifacts; never write inputs."""
import hashlib
import importlib.util
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read_json(path):
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition, reason):
    if not condition:
        raise AssertionError(reason)


def validate_accounting():
    register_path = SOURCES / '10.0.7-wikitext-register.json'
    register = read_json(register_path)
    coverage = read_json(SOURCES / '10.0.7-page-coverage.json')
    provenance = read_json(SOURCES / '10.0.7-api-changes.provenance.json')
    require(register['source']['revid'] == provenance['wikitext']['revid'] == 1063344,
            'pinned revision drift')
    require(register['source']['sha256'] == provenance['wikitext']['sha256'] ==
            sha256(ROOT / register['source']['path']), 'wikitext hash drift')
    require(coverage['source_sha256'] == sha256(register_path), 'register hash drift')
    require(coverage['schema'] == 'patch-page-coverage/v1', 'coverage schema drift')
    inventory = {row['id'] for row in register['entries']}
    require(len(inventory) == len(register['entries']) == 70, 'inventory count/ID drift')
    for header in register['header_counts']:
        actual = sum(row['section'] == header['section'] and
                     row['direction'] == header['direction'] for row in register['entries'])
        require(actual == header['parsed_count'] == header['header_count'],
                'inventory header count drift')
    module_spec = importlib.util.spec_from_file_location(
        'extract', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(module_spec)
    module_spec.loader.exec_module(extractor)
    text_path = ROOT / coverage['non_inventory_source']['path']
    text = text_path.read_text()
    require(sha256(text_path) == coverage['non_inventory_source']['sha256'], 'text hash drift')
    require(text == extractor.extract_text((ROOT / register['source']['path']).read_text()),
            'extract reproduction drift')
    extract_rows = extractor.seed_rows(text, '10.0.7')
    require(len(extract_rows) == 2214, 'extract occurrence drift')
    scout = read_json(EVIDENCE / 'p1007-extract-scout.json')
    require({row['source_id'] for row in scout} == {row['source_id'] for row in extract_rows},
            'scout does not account for every extract occurrence')
    for row in scout:
        require(row['statement'] == text.splitlines()[row['line'] - 1], 'scout literal drift')
        require(bool(row['proof_boundary']), 'missing scout proof boundary')
    rows = coverage['source_rows']
    require(len(rows) == len({row['source_id'] for row in rows}) == 2284,
            'coverage count/duplicate ID drift')
    require({row['source_id'] for row in rows} == inventory |
            {row['source_id'] for row in scout}, 'coverage source ID omission')
    statuses = Counter(row['status'] for row in rows)
    require(statuses == {'bounded-coverage': 28, 'partial-development-green': 16,
                         'audit-pending': 2229, 'metadata-only': 11}, 'coverage credit drift')
    final = read_json(EVIDENCE / 'final.json')
    discovery = read_json(EVIDENCE / 'discovery.json')
    require(set(final) == set(discovery) == inventory, 'sweep ID omission')
    final_gaps = {key for key, value in final.items() if not value['ok']}
    discovered_gaps = {key for key, value in discovery.items() if not value['ok']}
    require(len(final_gaps) == 26 and len(discovered_gaps) == 44, 'gap count drift')
    reviews = read_json(EVIDENCE / 'p1007-gap-review.json')
    require(len(reviews) == len({row['source_id'] for row in reviews}) == 44,
            'discovery gap review duplicate/omission')
    require({row['source_id'] for row in reviews} == discovered_gaps, 'gap review omission')
    require({row['source_id'] for row in reviews if row['outcome'] == 'retained'} == final_gaps,
            'retained gap review mismatch')
    require({row['source_id'] for row in reviews if row['outcome'] == 'closed'} ==
            discovered_gaps - final_gaps, 'closed gap review mismatch')
    rows_by_id = {row['source_id']: row for row in rows}
    for source_id, result in final.items():
        row = rows_by_id[source_id]
        require((row['status'] == 'audit-pending') == (not result['ok']),
                'inventory proof credit mismatch')
    for row in scout:
        require(rows_by_id[row['source_id']]['status'] == row['outcome'], 'scout credit drift')
    return dict(statuses)


def validate_sweeps():
    files = sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs'))
    require(len(files) == 22, 'isolated sweep inventory drift')
    for file in files:
        test = file.read_text()
        patch = file.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = read_json(SOURCES / f'{patch}-wikitext-register.json')
        results = read_json(EVIDENCE / ('final.json' if patch == '10.0.7' else f'{patch}-sweep.json'))
        fixture = read_json(ROOT / 'tests/data' / f'{file.stem.removesuffix("_publication_sweep")}_sweep_known_gaps.json')
        require(set(results) == {row['id'] for row in register['entries']}, 'sweep row omission')
        require({key for key, value in results.items() if not value['ok']} == set(fixture),
                f'{patch} exact fixture drift')
        latest = {}
        for path in re.findall(r'include_str!\("../(data/patch-api/sources/[^"]+)"\)',
                               test.split('later_registers: &[', 1)[1]):
            for row in read_json(ROOT / path)['entries']:
                if row['direction'] != 'changed':
                    latest[row['symbol']] = row
        for row in register['entries']:
            newer = latest.get(row['symbol'])
            removed = row['direction'] == 'removed'
            superseded = newer if newer and (newer['direction'] == 'removed') != removed else None
            expected = results[row['id']]['expected']
            require(expected['publication'] == ('absent' if removed != bool(superseded) else 'published'),
                    'chronological publication drift')
            require(expected['superseded_by'] == (superseded['id'] if superseded else None),
                    'chronological supersession drift')
    negative = read_json(EVIDENCE / 'p1007-negative-control.json')
    require(negative == {'exit': 101, 'old_gaps': 26, 'new_gaps': 27,
                         'added': ['wt-global-api-IsAdvancedFlyableArea-47'], 'resolved': []},
            'negative control failed')
    return len(files)


def validate_preservation_and_proof():
    before = read_json(EVIDENCE / 'p1007-extract-before.json')
    after = {(row['patch'], row['preserve_examples']): row for row in
             read_json(EVIDENCE / 'p1007-extract-after.json')}
    require(all(row['exit'] == after[row['patch'], row['preserve_examples']]['exit']
                for row in before), 'prior extract success/failure boundary drift')
    require(all(after['10.0.7', mode]['exit'] == 0 for mode in (False, True)),
            'own extract is not reproducible in both modes')
    preservation = read_json(EVIDENCE / 'p1007-preservation.json')['inputs']
    require(len(preservation) == 134, 'baseline preservation scope drift')
    for row in preservation:
        require(row['unchanged'] and sha256(ROOT / row['path']) == row['sha256'],
                'preserved input drift')
    registers = read_json(EVIDENCE / 'p1007-register-reproduction.json')
    require(len(registers) == 22 and all(row['byte_identical'] for row in registers),
            'register reproduction failure')
    proof = read_json(EVIDENCE / 'p1007-proof.json')
    require(proof['cwd'] == str(ROOT) and proof['target'] == str(ROOT / 'target'),
            'proof cwd/target scope drift')
    for row in proof['entries']:
        require(row['exit'] in (0, 1, 101) and not row['invalidated'], 'invalid proof entry')
        if row.get('log'):
            require(sha256(EVIDENCE / row['log']) == row['log_sha256'], 'proof log drift')
    warnings = [line for line in (EVIDENCE / 'mists-check-cargo.log').read_text().splitlines()
                if line.startswith('warning:')]
    require(len(warnings) == 7 and all('iced' in line for line in warnings),
            'non-vendor Mists warning')
    require(read_json(EVIDENCE / 'startup.stdout.json') == [], 'startup Lua errors')
    require(not read_json(EVIDENCE / 'p1007-possible-1010-supersessions.json')['intersections'],
            'predicted 10.1.0 gap intersection drift')
    consumers = read_json(EVIDENCE / 'p1007-removal-consumers.json')
    require(len(consumers) == 36 and all(row['exit'] == 1 and not row['matches']
                                      for row in consumers), 'retired member cached consumer')
    return len(preservation)


def main():
    statuses = validate_accounting()
    sweeps = validate_sweeps()
    preserved = validate_preservation_and_proof()
    print(json.dumps({'result': 'PASS', 'source_ids': 2284, 'statuses': statuses,
                      'isolated_sweeps': sweeps, 'preserved_inputs': preserved}, indent=2))


if __name__ == '__main__':
    main()
