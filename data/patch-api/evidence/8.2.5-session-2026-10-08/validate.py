"""Dynamic 8.2.5 acceptance: counts derive from pinned sources and proof artifacts."""
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

import sys
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_json, historical_registers, preserved_input_matches

AUDIT_REVISION = 'a63653163221085dae86fb8a977d5cf5db5ff985'
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '8.2.5'
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p825-page'


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def patch_key(patch):
    return tuple(map(int, patch.split('.')))


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_sources():
    register = read(SOURCES / f'{PATCH}-wikitext-register.json')
    provenance = read(SOURCES / f'{PATCH}-api-changes.provenance.json')
    page = next(iter(read(EVIDENCE / 'p825-fetch.json')['query']['pages'].values()))
    revision = page['revisions'][0]
    raw_path = SOURCES / f'{PATCH}-api-changes.wikitext'
    raw = raw_path.read_text()
    assert revision['slots']['main']['*'] == raw
    assert page['pageid'] == provenance['pageid'] and page['title'] == provenance['title']
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert sha(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    expected = set()
    for number, line in enumerate(raw.splitlines(), 1):
        if not line.startswith(': '):
            continue
        for symbol in re.findall(r'\[\[API [^|]+\|([^]]+)\]\]', line):
            expected.add((number, symbol))
        for body in re.findall(r'\{\{api\|([^{}]+)\}\}', line):
            symbol = [part for part in body.split('|') if '=' not in part][-1]
            expected.add((number, symbol))
    actual = {(row['wikitext_line'], row['symbol']) for row in register['entries']}
    assert actual == expected and len(actual) == len(register['entries'])
    assert register['header_counts']
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    assert {row['symbol'] for row in register['entries'] if row.get('kind') == 'command'} == {'DefragmentGPU'}
    extractor = load_extractor()
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    text = (SOURCES / f'{PATCH}-api-changes.txt').read_text()
    assert extractor.extract_text(raw, **options) == text
    return register, extractor, raw.splitlines(), text


def check_page_list():
    batches = read(EVIDENCE / 'p825-allpages-responses.json')
    assert batches and all('continue' in row for row in batches[:-1])
    assert 'continue' not in batches[-1]
    found = {}
    for batch in batches:
        for page in batch['query']['pages'].values():
            match = re.fullmatch(r'Patch (\d+(?:\.\d+)+)/API changes', page['title'])
            if match and patch_key(match[1]) < patch_key('8.3.0'):
                revision = page['revisions'][0]
                found[page['title']] = {'title': page['title'], 'version': match[1],
                                        'pageid': page['pageid'], 'revid': revision['revid'],
                                        'timestamp': revision['timestamp']}
    retained = read(SOURCES / 'api-change-pages-remaining.json')['pages']
    assert {row['title']: row for row in retained} == found
    assert retained == sorted(retained, key=lambda row: patch_key(row['version']), reverse=True)
    return {'enumerated_older_pages_including_current': len(retained),
            'oldest_page': retained[-1]['title'], 'pagination_batches': len(batches)}


def check_accounting(register, extractor, raw_lines, text):
    results = read(EVIDENCE / 'patch_8_2_5_publication_sweep-results.json')
    known = set(read(ROOT / 'tests/data/patch_8_2_5_sweep_known_gaps.json'))
    inventory = {row['id'] for row in register['entries']}
    assert set(results) == inventory
    assert {key for key, row in results.items() if not row['ok']} == known
    reviews = read(EVIDENCE / 'p825-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    for row in reviews:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['observation'] == results[row['source_id']]['observed']
        assert row['expectation'] == results[row['source_id']]['expected']
        assert row['reason']
    ledger = read(SOURCES / f'{PATCH}-page-coverage.json')
    indexed = {row['source_id']: row for row in ledger['source_rows']}
    extract_ids = {row['source_id'] for row in extractor.seed_rows(text, PATCH)}
    contexts = read(EVIDENCE / 'p825-build-context.json')
    context_ids = {row['source_id'] for row in contexts}
    assert len(indexed) == len(ledger['source_rows'])
    assert set(indexed) == inventory | extract_ids | context_ids
    assert {row['wikitext_line'] for row in contexts} == {
        i for i, line in enumerate(raw_lines, 1) if line.startswith('|+')}
    for row in contexts:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert indexed[row['source_id']]['status'] == 'metadata-only'
    assert ledger['source_sha256'] == sha(SOURCES / f'{PATCH}-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha(SOURCES / f'{PATCH}-api-changes.txt')
    for key, result in results.items():
        expected_status = ('audit-pending' if key in known else 'bounded-coverage'
                           if result['expected']['publication'] == 'absent' else 'partial-development-green')
        assert indexed[key]['status'] == expected_status
        assert bool(indexed[key]['capabilities']) == result['ok']
    scout = read(EVIDENCE / 'p825-extract-scout.json')
    assert {row['source_id'] for row in scout} == extract_ids
    for row in scout:
        source = indexed[row['source_id']]
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['reason'] == source['note'] and row['status'] == source['status']
        assert not source['capabilities']
    return {'inventory_rows': len(inventory), 'extract_rows': len(extract_ids),
            'context_rows': len(context_ids), 'ledger_rows': len(indexed),
            'ledger_statuses': dict(Counter(row['status'] for row in indexed.values())),
            'gaps_remaining': len(known),
            'pending_substantive_extract_rows': sum(row['status'] == 'audit-pending' for row in scout)}


def check_negative(register):
    results = read(EVIDENCE / 'patch_8_2_5_publication_sweep-results.json')
    negative = read(EVIDENCE / 'p825-negative-observation.json')
    receipt = read(EVIDENCE / 'p825-negative-result.json')
    mutated = read(EVIDENCE / 'p825-negative-register.json')
    differences = [(before, after) for before, after in zip(register['entries'], mutated['entries']) if before != after]
    assert len(differences) == 1
    before, after = differences[0]
    assert before['id'] == receipt['mutation']
    assert after == dict(before, direction='removed') and before['direction'] == 'added'
    assert set(negative) == set(results)
    known = {key for key, row in results.items() if not row['ok']}
    failures = {key for key, row in negative.items() if not row['ok']}
    assert sorted(failures - known) == receipt['new_gaps'] == [receipt['mutation']]
    assert sorted(known - failures) == receipt['resolved_gaps'] == []
    assert (len(known), len(failures)) == (receipt['baseline_gaps'], receipt['negative_gaps'])
    assert receipt['exit'] == read(EVIDENCE / 'p825-negative.proof.json')['exit'] != 0
    return {'negative_control_baseline': len(known), 'negative_control_gaps': len(failures)}


def check_retirements():
    before = read(EVIDENCE / 'p825-removal-consumers.json')
    after = read(EVIDENCE / 'p825-whole-callers-after.json')
    assert before['cache_exists']
    indexed = {row['symbol']: row for row in before['rows']}
    final = {row['symbol']: row for row in after['rows']}
    assert set(indexed) == set(final)
    retired = {symbol for symbol, row in indexed.items() if row['decision'] == 'retire'}
    source = (ROOT / 'src/c_api/patch_retired_members.rs').read_text()
    block = source.split('const RETIRED_8_2_5_MEMBERS:', 1)[1].split('\n];', 1)[0]
    actual = set()
    for namespace, members in re.findall(r'"(C_[^"]+)"\s*,\s*&\[(.*?)\]', block, re.S):
        actual.update(namespace + '.' + member for member in re.findall(r'"([^"]+)"', members))
    assert retired == actual
    for symbol in retired:
        for stage in (indexed, final):
            for kind in ('qualified', 'bare'):
                scan = stage[symbol]['scans'][kind]
                assert scan['exit'] == 1 and scan['stdout'] == scan['stderr'] == ''
        for scan in final[symbol]['scans'].values():
            assert scan['exit'] in (0, 1) and not scan['stderr']
            assert '\\b' in ' '.join(scan['command'])
            assert '--exclude-dir=*Documentation*' in scan['command']
    initial = read(EVIDENCE / 'p825-discovery-results.json')
    current = read(EVIDENCE / 'patch_8_2_5_publication_sweep-results.json')
    closed = {value['expected']['symbol'] for key, value in initial.items() if not value['ok'] and current[key]['ok']}
    assert closed == retired
    for row in read(EVIDENCE / 'p825-later-gap-closures.json'):
        patch = row['patch']
        known = read(ROOT / ('tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        ledger = {r['source_id']: r for r in read(SOURCES / f'{patch}-page-coverage.json')['source_rows']}
        results = read(EVIDENCE / ('patch_' + patch.replace('.', '_') + '_publication_sweep-results.json'))
        for key in row['resolved']:
            assert key not in known and ledger[key]['status'] == 'bounded-coverage' and results[key]['ok']
    return {'retired_members': len(retired), 'retained_removal_candidates': sorted(set(indexed) - retired)}


def check_preservation():
    preservation = read(EVIDENCE / 'p825-input-preservation.json')
    for row in preservation['rows']:
        assert sha(ROOT / row['path']) == row['current']
        if not row['unchanged_after_rebase']:
            assert row['allowed_change'] and row['path'] == 'data/patch-api/sources/9.2.5-page-coverage.json'
            old = subprocess.check_output(['git', 'show', preservation['base_revision'] + ':' + row['path']], cwd=ROOT, text=True)
            old = json.loads(old)
            current = read(ROOT / row['path'])
            old_rows, current_rows = old.pop('source_rows'), current.pop('source_rows')
            assert old == current
            changed = [new for before, new in zip(old_rows, current_rows) if before != new]
            resolved = {key for group in read(EVIDENCE / 'p825-later-gap-closures.json') for key in group['resolved']}
            assert {row['source_id'] for row in changed} == resolved
            assert len(old_rows) == len(current_rows)
    outcomes = read(EVIDENCE / 'p825-extract-preservation.json')
    assert all(row['before'] == row['after'] and row['unchanged'] for row in outcomes)
    patches = {path.name.removesuffix('-wikitext-register.json') for path in SOURCES.glob('*-wikitext-register.json')}
    registers = read(EVIDENCE / 'p825-register-reproduction.json')
    saved = read(EVIDENCE / 'p825-saved-extract-reproduction.json')
    assert {row['patch'] for row in registers} == {row['patch'] for row in saved} == patches
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical']
        assert sha(SOURCES / f"{row['patch']}-wikitext-register.json") == row['sha256']
        recorded = read(SOURCES / f"{row['patch']}-api-changes.provenance.json").get('generator_flags')
        assert recorded == row['recorded_flags']
        assert recorded is None or recorded == row['verified_flags']
    for row in saved:
        assert sha(SOURCES / f"{row['patch']}-api-changes.txt") == row['sha256']
        if row['patch'] not in ('12.0.5', '12.0.7', '12.1.0'):
            assert row['byte_identical'], row['patch']
        recorded = read(SOURCES / f"{row['patch']}-api-changes.provenance.json").get('extractor_flags')
        assert recorded == row['recorded_flags']
        assert recorded is None or set(recorded) == set(row['verified_flags'])
    assert {(row['patch'], tuple(row['flags'])) for row in outcomes} == {
        (patch, flags) for patch in patches - {PATCH} for flags in ((), ('--preserve-examples',))}
    return {'preservation_inputs': len(preservation['rows']), 'preserved_extract_modes': len(outcomes),
            'reproduced_registers': len(registers),
            'inherited_nonreproducible_extracts': [row['patch'] for row in saved if not row['byte_identical']]}


def check_sweeps():
    registers = sorted(SOURCES.glob('*-wikitext-register.json'), key=lambda p: patch_key(p.name.split('-')[0]))
    summary = []
    for path in registers:
        patch = path.name.removesuffix('-wikitext-register.json')
        results = read(EVIDENCE / ('patch_' + patch.replace('.', '_') + '_publication_sweep-results.json'))
        rows = read(path)['entries']
        assert set(results) == {row['id'] for row in rows}
        gaps = {key for key, row in results.items() if not row['ok']}
        assert gaps == set(read(ROOT / ('tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json')))
        later = {}
        for newer in registers:
            if patch_key(newer.name.split('-')[0]) > patch_key(patch):
                for row in read(newer)['entries']:
                    if row['direction'] != 'changed':
                        later[row['symbol']] = row
        for row in rows:
            removed = row['direction'] == 'removed'
            newer = later.get(row['symbol'])
            superseded = newer is not None and (newer['direction'] == 'removed') != removed
            expected = results[row['id']]['expected']
            assert expected['publication'] == ('absent' if (not removed if superseded else removed) else 'published')
            assert expected['superseded_by'] == (newer['id'] if superseded else None)
        summary.append({'patch': patch, 'rows': len(rows), 'ok': len(rows) - len(gaps),
                        'gaps': len(gaps), 'result': 'pass'})
    assert summary == read(EVIDENCE / 'p825-sweep-summary.json')
    return {'sweeps': summary, 'sweep_inventory_rows': sum(row['rows'] for row in summary)}


def check_proof():
    receipts = [read(path) for path in sorted(EVIDENCE.glob('*.proof.json'))]
    for row in receipts:
        assert sha(EVIDENCE / row['log']) == row['log_sha256']
        assert row['exit'] == row['expected_exit'], row['scope']
        if row['exit'] == 0 and 'test' in row['command'] and not row.get('calibration_only'):
            log = (EVIDENCE / row['log']).read_text()
            assert re.search(r'[1-9]\d* passed; 0 failed', log), row['scope']
    assert (EVIDENCE / 'p825-startup.stdout').read_text().strip() == '[]'
    for name in ('p825-mists-check', 'p825-default-check'):
        warnings = [line for line in (EVIDENCE / f'{name}.txt').read_text().splitlines() if line.startswith('warning:')]
        assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings)
    required = ('p825-bare-green', 'p825-cached-green', 'p825-all-sweeps', 'p825-negative',
                'p825-extractor-fixtures', 'p825-generator-fixtures', 'p825-mists-behavior',
                'p825-mists-check', 'p825-format', 'p825-default-check', 'p825-retail-build', 'p825-startup')
    by_scope = {row['scope']: row for row in receipts}
    assert all(name in by_scope for name in required)
    assert by_scope['p825-negative']['exit'] != 0
    assert all(by_scope[name]['exit'] == 0 for name in required if name != 'p825-negative')
    return {'proof_receipts': len(receipts), 'calibration_only': [row['scope'] for row in receipts if row.get('calibration_only')]}


def main():
    register, extractor, lines, text = check_sources()
    report = check_accounting(register, extractor, lines, text)
    for check in (check_page_list, check_retirements, check_preservation, check_sweeps, check_proof):
        report.update(check())
    report.update(check_negative(register))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
