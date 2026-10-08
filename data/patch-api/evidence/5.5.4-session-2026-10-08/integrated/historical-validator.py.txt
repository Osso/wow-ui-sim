"""Read-only historical Mists audit gate; shared inputs are Git-pinned."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((HERE / name).read_text())


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def pinned_json(revision, path):
    return json.loads(blob(revision, path))


def check_seals(context):
    for name, expected in context['session_sha256'].items():
        assert digest((HERE / name).read_bytes()) == expected, name
    for path, expected in context['shared_sha256'].items():
        assert digest(blob(context['code_revision'], path)) == expected, path
    assert digest(Path(__file__).read_bytes()) == context['validator_sha256']


def check_source(context):
    prefix = 'data/patch-api/sources/5.5.4-'
    revision = context['source_revision']
    provenance = pinned_json(revision, prefix + 'api-changes.provenance.json')
    raw = blob(revision, prefix + 'api-changes.wikitext')
    response = (HERE / 'p554-source-response.json').read_bytes()
    page = json.loads(response)['query']['pages'][0]
    record = page['revisions'][0]
    assert page['pageid'] == provenance['pageid'] == 686958
    assert page['title'] == provenance['title'] == 'Patch 5.5.4/API changes'
    assert record['revid'] == provenance['revid'] == 6778083
    assert record['timestamp'] == provenance['timestamp']
    assert record['slots']['main']['content'].encode() == raw
    assert digest(response) == provenance['sha256']
    assert digest(raw) == provenance['wikitext_sha256']
    assert provenance['classification'] == 'resources-only-stub'
    assert provenance['client_line'] == 'mists-classic' and not provenance['redirect']
    register = pinned_json(revision, prefix + 'wikitext-register.json')
    assert register['client_line'] == 'mists-classic'
    assert register['source']['revid'] == record['revid']
    assert register['source']['sha256'] == digest(raw)
    assert register['entries'] == register['header_counts'] == []
    ledger = pinned_json(revision, prefix + 'page-coverage.json')
    assert ledger['source_sha256'] == digest(blob(revision, prefix + 'wikitext-register.json'))
    assert ledger['non_inventory_source']['sha256'] == digest(blob(revision, prefix + 'api-changes.txt'))
    rows = ledger['source_rows']
    extractor_path = 'tools/extract_patch_non_inventory.py'
    extractor = {'__file__': str(ROOT / extractor_path), '__name__': 'pinned_extractor'}
    exec(compile(blob(revision, extractor_path), extractor_path, 'exec'), extractor)
    text = extractor['extract_text'](raw.decode(), canonical_patch_navigation=True)
    assert text.encode() == blob(revision, prefix + 'api-changes.txt')
    assert rows == extractor['seed_rows'](text, '5.5.4')
    assert len(rows) == len({row['source_id'] for row in rows})
    assert all(row['status'] == 'metadata-only' and not row['capabilities'] for row in rows)
    assert not read('p554-gap-review.json') and not read('p554-problematic-contracts.json')
    retirements = read('p554-retirement-decisions.json')
    assert retirements['removed_entries'] == retirements['new_retirements'] == []
    expected = {'inventory_rows': len(register['entries']), 'extract_rows': len(rows),
                'metadata_rows': Counter(row['status'] for row in rows)['metadata-only'],
                'modeled_gaps': 0, 'problematic_gaps': len(read('p554-problematic-contracts.json')),
                'publication_gaps': len(read('p554-gap-review.json')),
                'retirements': len(retirements['new_retirements'])}
    assert read('p554-accounting-summary.json') == expected
    return expected


def check_reproduction(context):
    report = read('p554-reproduction.json')
    rows = report['records']
    expected = {path.name.removesuffix('-wikitext-register.json') for path in
                historical_registers(ROOT, report['revision'])}
    assert {row['patch'] for row in rows} == expected and len(rows) == len(expected)
    historical_prefix = 'data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/'
    inherited = pinned_json(context['base_revision'], historical_prefix + 'p620-saved-extract-reproduction.json')
    failures = {row['patch'] for row in inherited if not row['byte_identical']}
    generator_records = pinned_json(context['base_revision'], historical_prefix + 'p620-register-reproduction.json')
    generator_flags = {row['patch']: row['verified_flags'] for row in generator_records}
    extractor_flags = {row['patch']: row['verified_flags'] for row in inherited}
    own_provenance = pinned_json(report['revision'], 'data/patch-api/sources/5.5.4-api-changes.provenance.json')
    generator_flags['5.5.4'] = own_provenance['generator_flags']
    extractor_flags['5.5.4'] = own_provenance['extractor_flags']
    assert {row['patch'] for row in rows if not row['extract_byte_identical']} == failures
    for row in rows:
        assert row['generator_flags'] == generator_flags[row['patch']]
        assert row['extractor_flags'] == extractor_flags[row['patch']]
        assert row['register_exit'] == 0 and row['register_byte_identical'], row['patch']
        prefix = f"data/patch-api/sources/{row['patch']}-"
        assert row['register_sha256'] == digest(blob(report['revision'], prefix + 'wikitext-register.json'))
        assert row['extract_sha256'] == digest(blob(report['revision'], prefix + 'api-changes.txt'))
        assert row['extract_exit'] == 0 or row['patch'] in failures
    assert report['historical_flags_revision'] == context['base_revision']
    return {'registers': len(rows), 'extracts_passed': sum(row['extract_byte_identical'] for row in rows),
            'inherited_extract_failures': sorted(failures)}


def check_proofs(context):
    for label, policy in context['proofs'].items():
        proof = read(label + '.proof.json')
        assert proof['command'] == policy['command'], label
        assert proof['exit'] == policy['exit'] and not proof['invalidated'], label
        log = (HERE / proof['log']).read_bytes()
        assert digest(log) == proof['log_sha256'], label
        if policy.get('scope'):
            delta = subprocess.check_output(['git', 'diff', '--name-only', proof['revision'],
                                             context['code_revision'], '--', *policy['scope']],
                                            cwd=ROOT, text=True)
            assert not delta, (label, delta)
        if policy.get('passed_tests'):
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.decode())
            assert int(counts[-1]) == policy['passed_tests'], (label, counts)
        if label == 'p554-tools-tests':
            assert re.search(r'Ran [1-9]\d* tests', log.decode()) and log.decode().rstrip().endswith('OK')
        if policy.get('warning_clean'):
            warnings = [line for line in log.decode().splitlines() if line.startswith('warning:')]
            assert all('iced-wgpu-patched/Cargo.toml:' in line or
                       '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    mists = read('p554-mists-evidence/patch_5_5_4_publication_sweep-results.json')
    assert mists == {}
    assert pinned_json(context['code_revision'], 'tests/data/patch_5_5_4_sweep_known_gaps.json') == []
    assert read('p554-mists-evidence.proof.json')['profile'] == 'mists'
    control = read('p554-mists-evidence/patch_5_5_4_publication_sweep-MISTS_LINE_CONTROL_OUT-results.json')
    assert control['own']['expected']['publication'] == 'absent' and not control['own']['ok']
    negative = (HERE / 'p554-mists-negative.log').read_text()
    assert 'register row count changed' in negative
    assert 'left: 1' in negative and 'right: 0' in negative
    red = (HERE / 'p554-client-lines-red.log').read_text()
    assert 'new gaps:' in red and 'own' in red
    assert 'test did not panic as expected' in red
    assert read('p554-client-lines-red.proof.json')['exit'] != 0


def check_sweeps(context):
    summaries = []
    for source in historical_sweep_tests(ROOT, context['base_revision']):
        patch = source.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        prefix = f'data/patch-api/sources/{patch}-'
        register = pinned_json(context['base_revision'], prefix + 'wikitext-register.json')
        assert register.get('client_line', 'retail') == 'retail'
        name = source.stem + '-results.json'
        before = read('p554-retail-before/' + name)
        after = read('p554-retail-final/' + name)
        assert before == after, patch
        assert set(before) == {entry['id'] for entry in register['entries']}, patch
        gaps = pinned_json(context['base_revision'], f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        assert {name for name, row in after.items() if not row['ok']} == set(gaps), patch
        summaries.append({'patch': patch, 'rows': len(after), 'gaps': len(gaps), 'unchanged': True})
    assert read('p554-retail-comparison.json') == summaries
    return {'pages': len(summaries), 'observations': sum(row['rows'] for row in summaries)}


def check_scans(context):
    rows = read('p554-scan-receipts.json')
    for row in rows:
        assert row['tool'] == '/usr/bin/grep' and row['untruncated']
        assert '-RInw' in row['command']
        data = (HERE / row['log']).read_bytes()
        assert digest(data) == row['log_sha256']
        assert len(data.decode().splitlines()) == row['line_count']
    cache = read('p554-mists-cache-snapshot.json')
    assert cache['profile'] == 'mists' and cache['path'].endswith('/mists/AddOns')
    assert cache['file_count'] == len(cache['files']) and cache['file_count'] > 0
    assert cache['manifest_sha256'] == digest(blob(cache['manifest_revision'], cache['manifest_path']))
    assert all('Documentation' not in path for path in cache['files'])
    # No external cache reads or live source hashes: this proves the retained scan scope.


def main():
    context = read('p554-context.json')
    check_seals(context)
    accounting = check_source(context)
    reproduction = check_reproduction(context)
    check_proofs(context)
    sweeps = check_sweeps(context)
    check_scans(context)
    print(json.dumps({'status': 'PASS', 'accounting': accounting,
                      'reproduction': reproduction, 'retail': sweeps}, sort_keys=True))


if __name__ == '__main__':
    main()
