#!/usr/bin/env python3
"""Read-only historical 8.0.1 proof; checkout paths and later audits are not gates."""
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCES = ROOT / 'data/patch-api/sources'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests


def read(path):
    return json.loads(path.read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git_blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def historical_json(revision, path):
    return json.loads(git_blob(revision, path))


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools' / f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_seal():
    seal = read(HERE / 'p801-artifact-hashes.json')
    for path, digest in seal.items():
        assert sha((ROOT / path).read_bytes()) == digest, f'changed retained artifact: {path}'
    return len(seal)


def check_source():
    provenance = read(SOURCES / '8.0.1-api-changes.provenance.json')
    fetched = read(HERE / 'p801-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = fetched['revisions'][0]
    raw = (SOURCES / '8.0.1-api-changes.wikitext').read_text()
    register = read(SOURCES / '8.0.1-wikitext-register.json')
    assert fetched['title'] == provenance['title'] == 'Patch 8.0.1/API changes'
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert revision['slots']['main']['*'] == raw
    assert sha(raw.encode()) == provenance['wikitext_sha256'] == register['source']['sha256']
    attempts = read(HERE / 'p801-fetch-attempts.json')
    selected = attempts['attempts'][attempts['selected_attempt']]
    assert selected['status'] == 200 and json.loads(selected['body']) == read(HERE / 'p801-fetch.json')
    assert attempts['source_pinned'] and attempts['pinned_revid'] == revision['revid']
    generator = load_tool('gen_patch_wikitext_register')
    assert generator.parse_bfa_prepatch(raw) == register['entries']
    # Independently cover every event and nested New reference, including repeats.
    inventory = set()
    section, direction = None, None
    for number, line in enumerate(raw.splitlines(), 1):
        heading = re.fullmatch(r'(=+)\s*(.*?)\s*\1', line)
        if heading:
            if len(heading[1]) == 2:
                section = heading[2]
                direction = 'added' if section == 'New' else None
            elif section == 'Events':
                direction = {'Added': 'added', 'Removed': 'removed'}.get(heading[2])
        elif section in ('New', 'Events') and direction and line.startswith('*'):
            for match in re.finditer(r'\{\{api\|([^{}]+)\}\}', line):
                symbol = [part for part in match[1].split('|') if '=' not in part][-1]
                inventory.add((number, symbol, direction))
    actual = {(row['wikitext_line'], row['symbol'], row['direction']) for row in register['entries']}
    assert inventory <= actual
    extractor = load_tool('extract_patch_non_inventory')
    flags = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    text = (SOURCES / '8.0.1-api-changes.txt').read_text()
    assert extractor.extract_text(raw, **flags) == text
    return register, extractor, raw, text


def check_accounting(context, register, extractor, raw, text):
    result = read(HERE / 'patch_8_0_1_publication_sweep-results.json')
    known = set(historical_json(context['integrated_runtime_revision'], 'tests/data/patch_8_0_1_sweep_known_gaps.json'))
    ids = {row['id'] for row in register['entries']}
    assert len(ids) == len(register['entries']) and set(result) == ids
    assert {key for key, row in result.items() if not row['ok']} == known
    ledger_path = 'data/patch-api/sources/8.0.1-page-coverage.json'
    ledger_blob = git_blob(context['integrated_accounting_revision'], ledger_path)
    assert sha(ledger_blob) == read(HERE / 'p801-accounting-hashes.json')[ledger_path]
    ledger = json.loads(ledger_blob)
    rows = {row['source_id']: row for row in ledger['source_rows']}
    assert len(rows) == len(ledger['source_rows'])
    assert ledger['source_sha256'] == sha((SOURCES / '8.0.1-wikitext-register.json').read_bytes())
    assert ledger['non_inventory_source']['sha256'] == sha(text.encode())
    review = read(HERE / 'p801-gap-review.json')
    assert {row['source_id'] for row in review} == known
    lines = raw.splitlines()
    for row in review:
        observed = result[row['source_id']]
        assert row['literal'] == lines[row['wikitext_line'] - 1]
        assert row['expectation'] == observed['expected']
        assert row['observation'] == observed['observed']
        assert row['reason'] == rows[row['source_id']]['note'] and row['reason']
    for identifier, observed in result.items():
        assert bool(rows[identifier]['capabilities']) == observed['ok']
        assert (rows[identifier]['status'] == 'audit-pending') == (identifier in known)
    scout = read(HERE / 'p801-extract-scout.json')
    seeded = {row['source_id'] for row in extractor.seed_rows(text, '8.0.1')}
    assert {row['source_id'] for row in scout} == seeded
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == lines[row['wikitext_line'] - 1]
        assert row['status'] == rows[row['source_id']]['status']
        assert row['reason'] == rows[row['source_id']]['note']
    assert set(rows) == ids | seeded
    summary = {'inventory_rows': len(ids), 'inventory_ok': len(ids) - len(known),
               'inventory_gaps': len(known), 'extract_rows': len(seeded), 'ledger_rows': len(rows),
               'ledger_statuses': dict(Counter(row['status'] for row in rows.values())),
               'pending_prose': sum(row['status'] == 'audit-pending' for row in scout)}
    assert summary == read(HERE / 'p801-accounting-summary.json')
    initial = read(HERE / 'p801-discovery-results.json')
    closures = [key for key in ids if not initial[key]['ok'] and result[key]['ok']]
    summary['discovery_closures'] = sorted(closures)
    negative = read(HERE / 'p801-negative-observation.json')
    modified = read(HERE / 'p801-negative-register.json')
    changes = [(before, after) for before, after in zip(register['entries'], modified['entries'])
               if before != after]
    assert len(register['entries']) == len(modified['entries']) and len(changes) == 1
    before, after = changes[0]
    assert after == dict(before, symbol='P801_NEGATIVE_NONEXISTENT_API')
    receipt = read(HERE / 'p801-integration-negative.proof.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == ids and receipt['exit'] != 0
    assert before['id'] == receipt['mutation']
    assert sorted(failures - known) == receipt['new_gaps'] == [receipt['mutation']]
    assert sorted(known - failures) == receipt['resolved_gaps'] == []
    assert (len(known), len(failures)) == (receipt['baseline_gaps'], receipt['negative_gaps'])
    return summary


def check_sweeps_and_reproduction(context):
    revision = context['integrated_runtime_revision']
    paths = historical_registers(ROOT, revision)
    assert {path.stem.removesuffix('-wikitext-register') for path in paths} == {
        path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        for path in historical_sweep_tests(ROOT, revision)
    }
    summary = []
    for path in paths:
        relative = path.relative_to(ROOT).as_posix()
        patch = path.name.split('-')[0]
        rows = historical_json(revision, relative)['entries']
        result = read(HERE / f"patch_{patch.replace('.', '_')}_publication_sweep-results.json")
        assert set(result) == {row['id'] for row in rows}
        known = historical_json(revision, f"tests/data/patch_{patch.replace('.', '_')}_sweep_known_gaps.json")
        assert {key for key, row in result.items() if not row['ok']} == set(known)
        summary.append({'patch': patch, 'rows': len(rows),
                        'ok': sum(row['ok'] for row in result.values()),
                        'gaps': sum(not row['ok'] for row in result.values())})
    assert sorted(summary, key=lambda row: row['patch']) == read(HERE / 'p801-sweep-summary.json')
    patches = {path.name.split('-')[0] for path in paths}
    inherited = read(ROOT / 'data/patch-api/evidence/8.2.0-session-2026-10-08/p820-saved-extract-reproduction.json')
    inherited_by_patch = {row['patch']: row for row in read(
        ROOT / 'data/patch-api/evidence/8.1.0-session-2026-10-08/p810-saved-extract-reproduction.json')}
    prior_failures = {row['patch'] for row in inherited if not row['byte_identical']}
    generated = read(HERE / 'p801-register-reproduction.json')
    extracts = read(HERE / 'p801-saved-extract-reproduction.json')
    assert {row['patch'] for row in generated} == {row['patch'] for row in extracts} == patches
    assert {row['patch'] for row in extracts if not row['byte_identical']} == prior_failures
    for row in extracts:
        if row['patch'] in prior_failures:
            assert row['error'] == inherited_by_patch[row['patch']]['error']
    for proof, kind, suffix in [(generated, 'generator_flags', 'wikitext-register.json'),
                               (extracts, 'extractor_flags', 'api-changes.txt')]:
        for row in proof:
            path = f"data/patch-api/sources/{row['patch']}-api-changes.provenance.json"
            provenance = historical_json(revision, path)
            assert row['recorded_flags'] == provenance.get(kind)
            if row['recorded_flags'] is not None:
                assert row['verified_flags'] == row['recorded_flags']
            content = git_blob(revision, f"data/patch-api/sources/{row['patch']}-{suffix}")
            assert row['sha256'] == sha(content)
            if kind == 'generator_flags' or row['patch'] not in prior_failures:
                assert row['exit'] == 0 and row['byte_identical']
    return {'sweeps': len(paths), 'sweep_inventory_rows': sum(row['rows'] for row in summary),
            'reproduced_registers': len(generated),
            'reproduced_extracts': sum(row['byte_identical'] for row in extracts),
            'inherited_extract_failures': sorted(prior_failures)}


def check_scans(register):
    scans = read(HERE / 'p801-removal-scans.json')
    indexed = {row['symbol']: row for row in scans['members']}
    extra = {'SendAddonMessage', 'RegisterAddonMessagePrefix', 'IsAddonMessagePrefixRegistered',
             'GetRegisteredAddonMessagePrefixes', 'UNIT_POWER', 'C_Vignettes'}
    assert set(indexed) == {row['symbol'] for row in register['entries'] if row['direction'] == 'removed'} | extra
    decisions = read(HERE / 'p801-retirement-decisions.json')
    assert not decisions['new_retirements']
    assert {row['symbol'] for row in decisions['decisions']} == set(indexed)
    snapshots = [read(HERE / f'p801-later-{branch}.json') for branch in ('master', 'p810-page', 'p815-page')]
    for snapshot in snapshots:
        names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', snapshot['revision'],
                                         'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
        paths = sorted(name for name in names if name.endswith('-wikitext-register.json'))
        assert paths == snapshot['register_paths']
        matches, count = [], 0
        for path in paths:
            entries = historical_json(snapshot['revision'], path)['entries']
            count += len(entries)
            matches.extend({'path': path, **entry} for entry in entries if entry['symbol'] in indexed)
        assert count == snapshot['source_rows_scanned'] and matches == snapshot['entries']
    for symbol, row in indexed.items():
        assert len(row['scans']) == 4
        for scan in row['scans']:
            assert scan['command'][0] == '/usr/bin/grep'
            assert '-w' in scan['command'] and '-F' in scan['command']
            assert scan['untruncated'] and not scan['stderr'] and scan['exit'] in (0, 1)
            assert scan['lines'] == len(scan['stdout'].splitlines())
            assert bool(scan['stdout']) == (scan['exit'] == 0)
            if scan['kind'].startswith('cached'):
                assert '--exclude=*Documentation*' in scan['command']
                assert '--exclude-dir=*Documentation*' in scan['command']
        readds = [{'branch': snapshot['branch'], 'revision': snapshot['revision'],
                   'path': entry['path'], 'id': entry['id'], 'direction': entry['direction']}
                  for snapshot in snapshots for entry in snapshot['entries']
                  if entry['symbol'] == symbol and entry['direction'] == 'added']
        assert row['later_readds'] == readds
    return len(indexed)


def check_proofs(context):
    proofs = read(HERE / 'p801-proof.json')['receipts']
    required = {'all-sweeps', 'cached-semantics', 'model-acceptance', 'map-probes', 'map-api',
                'mists-check', 'format', 'default-check', 'retail-build', 'startup',
                'parser-fixtures', 'generator-fixtures', 'negative'}
    for label in required:
        path = HERE / f'p801-{label}.proof.json'
        receipt = read(path)
        assert receipt in proofs
        assert receipt['revision'] == context['runtime_revision']
        assert receipt['exit'] == (1 if label == 'negative' else 0)
        assert receipt['command'] and receipt['scope']
        assert sha((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
        assert not receipt.get('non_vendor_warnings', [])
    assert read(HERE / 'p801-startup.proof.json')['stdout'].strip() == '[]'
    assert read(HERE / 'p801-mists-check.proof.json')['command'] == [
        'cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']
    for receipt in proofs:
        assert sha((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
    return len(proofs)


def check_preservation(context):
    before = read(HERE / 'p801-input-hashes-before.json')
    for path, digest in before.items():
        assert (ROOT / path).is_file(), path
        assert sha(git_blob(context['prebase_input_revision'], path)) == digest, path
    # The original complete wiki is a suffix at this audit's accounting commit.
    # Later wiki changes do not retroactively invalidate this historical proof.
    for name in ('index.md', 'log.md'):
        path = 'docs/wiki/' + name
        base = git_blob(context['base_revision'], path)
        audited = git_blob(context['accounting_revision'], path)
        assert base and audited.endswith(base) and len(audited.splitlines()) > len(base.splitlines())
    return len(before)


def check_integrated_proofs(context):
    proof = read(HERE / 'p801-integration-proof.json')
    for row in proof['required']:
        receipt = read(HERE / row['receipt'])
        assert receipt['exit'] == row['expected_exit'] and not receipt['invalidated']
        assert sha((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
        for path in row.get('source_scope', []):
            original = subprocess.check_output(['git', 'rev-parse', f"{receipt['revision']}:{path}"], cwd=ROOT)
            integrated = subprocess.check_output(['git', 'rev-parse', f"{context['integrated_runtime_revision']}:{path}"], cwd=ROOT)
            assert original == integrated, (row['receipt'], path)
    check = read(HERE / 'p801-integration-mists-check.proof.json')
    warnings = [line for line in (HERE / check['log']).read_text().splitlines()
                if line.startswith('warning:') and not line.startswith(
                    ('warning: iced-wgpu-patched/', 'warning: `iced_wgpu`'))]
    assert not warnings, warnings
    receipt = read(HERE / 'p801-integration-all-sweeps.proof.json')
    log = (HERE / receipt['log']).read_text()
    passing = set(re.findall(r'test patch_([\d_]+)_publication_sweep::patch_[\d_]+_publication_sweep \.\.\. ok', log))
    patches = {path.name.removesuffix('-wikitext-register.json').replace('.', '_')
               for path in historical_registers(ROOT, context['integrated_runtime_revision'])}
    assert passing == patches and '0 failed' in log
    return {'integrated_required_receipts': len(proof['required']),
            'non_vendor_mists_warnings': len(warnings)}


def check_supersessions(context):
    closures = read(HERE / 'p801-later-gap-closures.json')
    current = set(historical_json(context['integrated_runtime_revision'], 'tests/data/patch_8_0_1_sweep_known_gaps.json'))
    assert len(closures) == 1 and closures[0]['patch'] == '8.0.1'
    previous = set(historical_json(closures[0]['prior_revision'], 'tests/data/patch_8_0_1_sweep_known_gaps.json'))
    assert set(closures[0]['resolved']) == previous - current and not current - previous
    results = read(HERE / 'patch_8_0_1_publication_sweep-results.json')
    ledger = {row['source_id']: row for row in historical_json(
        context['integrated_accounting_revision'], 'data/patch-api/sources/8.0.1-page-coverage.json')['source_rows']}
    for closure in closures[0]['closures']:
        identifier = closure['source_id']
        assert results[identifier]['ok']
        assert results[identifier]['expected']['superseded_by'] == closure['superseded_by']
        later = read(SOURCES / (closure['superseded_by_patch'] + '-wikitext-register.json'))
        assert any(row['id'] == closure['superseded_by'] and row['symbol'] == closure['symbol']
                   and row['direction'] == 'removed' for row in later['entries'])
        assert ledger[identifier]['status'] == 'bounded-coverage'
        assert ledger[identifier]['note'].startswith('Superseded by ' + closure['superseded_by_patch'] + ' removal')
    assert {row['source_id'] for row in closures[0]['closures']} == previous - current
    impact = read(HERE / 'p801-later-sweep-impact.json')
    assert not impact['later_resolved_gaps'] and not impact['later_new_gaps']
    assert not impact['later_audit_replacements_required']
    return {'integrated_gap_closures': len(previous - current)}


def main():
    context = read(HERE / 'p801-context.json')
    summary = {'sealed_artifacts': check_seal()}
    register, extractor, raw, text = check_source()
    summary.update(check_accounting(context, register, extractor, raw, text))
    summary.update(check_sweeps_and_reproduction(context))
    summary['scanned_removal_identities'] = check_scans(register)
    summary['proof_receipts'] = check_proofs(context)
    summary['historical_preserved_inputs'] = check_preservation(context)
    summary.update(check_integrated_proofs(context), check_supersessions(context))
    print(json.dumps({'status': 'PASS', **summary}, indent=2))


if __name__ == '__main__':
    main()
