#!/usr/bin/env python3
"""Validate pinned source, all occurrence IDs, runtime evidence and preservation.

Counts are derived exclusively from source/register/ledger/result/receipt files.
Inherited extract failures must match the prior audit, not a hard-coded allowlist.
"""
import hashlib
import importlib.util
import json
import re
import subprocess
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '8.2.0'
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p820-page'


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_source():
    provenance = read(SOURCES / f'{PATCH}-api-changes.provenance.json')
    register = read(SOURCES / f'{PATCH}-wikitext-register.json')
    fetched = read(HERE / 'p820-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = fetched['revisions'][0]
    raw_path = SOURCES / f'{PATCH}-api-changes.wikitext'
    assert revision['slots']['main']['*'] == raw_path.read_text()
    assert fetched['title'] == provenance['title']
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert digest(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    attempts = read(HERE / 'p820-fetch-attempts.json')
    assert attempts['source_pinned'] and attempts['pinned_revid'] == revision['revid']
    selected = attempts['attempts'][attempts['selected_attempt']]
    assert selected['status'] == 200 and json.loads(selected['body']) == read(HERE / 'p820-fetch.json')
    lines = raw_path.read_text().splitlines()
    # Every colon inventory reference, including repeated handler/command tables.
    expected = set()
    for number, line in enumerate(lines, 1):
        if not line.startswith(': '):
            continue
        api = re.search(r'\{\{api\|([^{}]+)\}\}', line)
        link = re.search(r'\[\[[^|\]]*\|([^\]]+)\]\]', line)
        assert api or link, line
        symbol = [part for part in api[1].split('|') if '=' not in part][-1] if api else link[1]
        expected.add((number, symbol))
    actual = {(row['wikitext_line'], row['symbol']) for row in register['entries']}
    assert expected == actual and len(actual) == len(register['entries'])
    assert all(row['parsed_count'] == row['header_count'] for row in register['header_counts'])
    extractor = load_extractor()
    flags = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    text = (SOURCES / f'{PATCH}-api-changes.txt').read_text()
    assert extractor.extract_text(raw_path.read_text(), **flags) == text
    return register, extractor, lines, text


def check_accounting(register, extractor, lines, text):
    results = read(HERE / 'patch_8_2_0_publication_sweep-results.json')
    inventory = {row['id'] for row in register['entries']}
    known = set(read(ROOT / 'tests/data/patch_8_2_0_sweep_known_gaps.json'))
    assert set(results) == inventory
    assert {key for key, row in results.items() if not row['ok']} == known
    ledger = read(SOURCES / f'{PATCH}-page-coverage.json')
    indexed = {row['source_id']: row for row in ledger['source_rows']}
    assert len(indexed) == len(ledger['source_rows'])
    assert ledger['source_sha256'] == digest(SOURCES / f'{PATCH}-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == digest(SOURCES / f'{PATCH}-api-changes.txt')
    review = read(HERE / 'p820-gap-review.json')
    assert {row['source_id'] for row in review} == known
    for row in review:
        assert row['literal'] == lines[row['wikitext_line'] - 1]
        assert row['expectation'] == results[row['source_id']]['expected']
        assert row['observation'] == results[row['source_id']]['observed']
        assert row['reason'] == indexed[row['source_id']]['note'] and row['reason']
    for key, row in results.items():
        absent = row['expected']['publication'] == 'absent'
        status = 'audit-pending' if key in known else 'bounded-coverage' if absent else 'partial-development-green'
        assert indexed[key]['status'] == status
        assert bool(indexed[key]['capabilities']) == row['ok']
    extract_ids = {row['source_id'] for row in extractor.seed_rows(text, PATCH)}
    scout = read(HERE / 'p820-extract-scout.json')
    assert {row['source_id'] for row in scout} == extract_ids
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == lines[row['wikitext_line'] - 1]
        assert row['reason'] == indexed[row['source_id']]['note']
        assert row['status'] == indexed[row['source_id']]['status']
        assert not indexed[row['source_id']]['capabilities']
    contexts = read(HERE / 'p820-build-context.json')
    context_ids = {row['source_id'] for row in contexts}
    assert {row['wikitext_line'] for row in contexts} == {i for i, line in enumerate(lines, 1) if line.startswith('|+')}
    for row in contexts:
        assert row['literal'] == lines[row['wikitext_line'] - 1]
        assert indexed[row['source_id']]['status'] == 'metadata-only'
    assert set(indexed) == inventory | extract_ids | context_ids
    negative = read(HERE / 'p820-negative-observation.json')
    receipt = read(HERE / 'p820-negative-result.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == inventory
    assert sorted(failures - known) == receipt['new_gaps'] == [receipt['mutation']]
    assert sorted(known - failures) == receipt['resolved_gaps'] == []
    assert (len(known), len(failures)) == (receipt['baseline_gaps'], receipt['negative_gaps'])
    assert receipt['exit'] != 0
    initial = read(HERE / 'p820-discovery-results.json')
    return {'inventory_rows':len(inventory), 'extract_rows':len(extract_ids), 'context_rows':len(context_ids),
            'ledger_rows':len(indexed), 'ledger_statuses':dict(Counter(row['status'] for row in indexed.values())),
            'inventory_gaps':len(known), 'prose_pending':sum(row['status']=='audit-pending' for row in scout),
            'discovery_closures':[key for key in inventory if not initial[key]['ok'] and results[key]['ok']]}


def check_preservation():
    hashes = read(HERE / 'p820-input-hashes-before.json')
    for path, value in hashes.items():
        if digest(ROOT / path) != value:
            # Only the merged 8.2.5 audit's attributable ledger update is allowed.
            assert path == 'data/patch-api/sources/9.2.5-page-coverage.json', path
            allowed_change = subprocess.check_output(
                ['git', 'show', '127aa3724:' + path], cwd=ROOT)
            assert (ROOT / path).read_bytes() == allowed_change, path
    preserved = read(HERE / 'p820-extract-preservation.json')
    assert all(row['unchanged'] and row['before'] == row['after'] for row in preserved)
    patches = {p.name.removesuffix('-wikitext-register.json') for p in SOURCES.glob('*-wikitext-register.json')}
    regenerated = read(HERE / 'p820-register-reproduction.json')
    saved = read(HERE / 'p820-saved-extract-reproduction.json')
    prior_saved = read(ROOT / 'data/patch-api/evidence/8.3.0-session-2026-10-08/p830-saved-extract-reproduction.json')
    inherited = {row['patch'] for row in prior_saved if not row['byte_identical']}
    assert {row['patch'] for row in regenerated} == {row['patch'] for row in saved} == patches
    assert {row['patch'] for row in saved if not row['byte_identical']} == inherited
    for rows, kind in [(regenerated, 'generator_flags'), (saved, 'extractor_flags')]:
        for row in rows:
            provenance = read(SOURCES / f"{row['patch']}-api-changes.provenance.json")
            assert provenance.get(kind) == row['recorded_flags']
            if row['recorded_flags'] is not None:
                assert row['verified_flags'] == row['recorded_flags']
            suffix = 'wikitext-register.json' if kind == 'generator_flags' else 'api-changes.txt'
            assert row['sha256'] == digest(SOURCES / f"{row['patch']}-{suffix}")
            if kind == 'generator_flags' or row['patch'] not in inherited:
                assert row['exit'] == 0 and row['byte_identical'], row['patch']
    return {'preserved_inputs':len(hashes), 'preserved_extract_modes':len(preserved),
            'reproduced_registers':len(regenerated), 'inherited_nonreproducible_extracts':sorted(inherited)}


def check_sweeps():
    key = lambda patch: tuple(map(int, patch.split('.')))
    paths = sorted(SOURCES.glob('*-wikitext-register.json'), key=lambda p:key(p.name.split('-')[0]))
    summary = []
    for path in paths:
        patch = path.name.split('-')[0]
        entries = read(path)['entries']
        results = read(HERE / f"patch_{patch.replace('.', '_')}_publication_sweep-results.json")
        assert set(results) == {row['id'] for row in entries}
        gaps = {identifier for identifier,row in results.items() if not row['ok']}
        assert gaps == set(read(ROOT / f"tests/data/patch_{patch.replace('.', '_')}_sweep_known_gaps.json"))
        later = {}
        for newer in paths:
            if key(newer.name.split('-')[0]) > key(patch):
                later.update({row['symbol']:row for row in read(newer)['entries'] if row['direction'] != 'changed'})
        for row in entries:
            removed = row['direction'] == 'removed'
            newer = later.get(row['symbol'])
            changed = newer is not None and (newer['direction'] == 'removed') != removed
            expected = results[row['id']]['expected']
            assert expected['publication'] == ('absent' if (not removed if changed else removed) else 'published')
            assert expected['superseded_by'] == (newer['id'] if changed else None)
        summary.append({'patch':patch,'rows':len(entries),'ok':len(entries)-len(gaps),'gaps':len(gaps),'result':'pass'})
    assert summary == read(HERE / 'p820-sweep-summary.json')
    return summary


def check_scans_and_proof(register):
    scans = read(HERE / 'p820-removal-consumers.json')
    assert {row['source_id'] for row in scans} == {row['id'] for row in register['entries'] if row['direction']=='removed'}
    for row in scans:
        assert row['scanner'] == '/usr/bin/grep' and row['whole_word'] and row['untruncated']
        assert {scan['label'] for scan in row['scans']} == {'cached-qualified','cached-bare','callers'}
        for scan in row['scans']:
            assert scan['argv'][0] == '/usr/bin/grep' and '-w' in scan['argv']
            assert scan['exit'] in (0,1) and not scan['stderr']
    decisions = read(HERE / 'p820-retirement-decisions.json')
    retired = [row for row in decisions if row['decision']=='retired']
    indexed = {row['symbol']:row for row in scans}
    for row in retired:
        assert all(not scan['stdout'] for scan in indexed[row['symbol']]['scans'])
    receipts = [read(p) for p in HERE.glob('*.proof.json')]
    for row in receipts:
        assert row['cwd'] == str(ROOT) and row['target'] == TARGET
        assert row['log_sha256'] == digest(HERE / row['log'])
        assert row['exit'] == row['expected_exit'], row['scope']
    for label in ('mists-check','default-check'):
        warnings = [line for line in (HERE/f'p820-{label}.log').read_text().splitlines() if line.startswith('warning:')]
        assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings)
    assert (HERE/'p820-startup.log').read_text().splitlines()[-1] == '[]'
    for label in ('cached-green', 'bare-green', 'mists-behavior', 'voice-regressions', 'all-sweeps'):
        log = (HERE/f'p820-{label}.log').read_text()
        assert 'test result: ok.' in log and re.search(r'\b0 failed\b', log)
        assert not re.search(r'^test .* \.\.\. FAILED$', log, re.M)
    return {'scanned_removals':len(scans),'retirements':[row['symbol'] for row in retired], 'proof_receipts':len(receipts)}


def main():
    register, extractor, lines, text = check_source()
    report = check_accounting(register, extractor, lines, text)
    report.update(check_preservation())
    report.update(check_scans_and_proof(register))
    report['sweeps'] = check_sweeps()
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
