"""Read-only 6.1.0 proof. Historical code and scope are resolved at pinned Git revisions."""
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
from patch_audit_validation import historical_registers, historical_sweep_tests


def read(path):
    return json.loads(path.read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def paths_at(revision, directory):
    return subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision, directory],
                                   cwd=ROOT, text=True).splitlines()


def historical_extractor(revision):
    namespace = {'__name__': 'historical_extractor',
                 '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(blob(revision, 'tools/extract_patch_non_inventory.py'),
                 'historical_extractor', 'exec'), namespace)
    return namespace


def source_accounting(context):
    revision_scope = context['accounting_revision']
    def source_bytes(name):
        return blob(revision_scope, 'data/patch-api/sources/' + name)

    provenance = json.loads(source_bytes('6.1.0-api-changes.provenance.json'))
    fetch = read(HERE / 'p610-fetch.json')
    page = fetch['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    raw = source_bytes('6.1.0-api-changes.wikitext').decode()
    assert provenance['pageid'] == 123523
    assert provenance['revid'] == revision['revid'] == 1216027
    assert provenance['timestamp'] == revision['timestamp']
    assert provenance['sha256'] == sha((HERE / 'p610-fetch.json').read_bytes())
    assert raw == revision['slots']['main']['*']
    assert provenance['wikitext_sha256'] == sha(raw.encode())
    register_bytes = source_bytes('6.1.0-wikitext-register.json')
    register = json.loads(register_bytes)
    assert register['source']['sha256'] == sha(raw.encode())
    assert register['source']['revid'] == provenance['revid']
    # Exact occurrence inventory, including changed prose, never a transclusion expansion.
    assert {(r['symbol'], r['direction'], r['wikitext_line']) for r in register['entries']} == {
        ('DeathRecap_HasEvents', 'added', 7), ('DeathRecap_GetEvents', 'added', 8),
        ('GetDeathRecapLink', 'added', 9), ('SendChatMessage', 'changed', 14)}
    extractor = historical_extractor(context['source_revision'])
    text_bytes = source_bytes('6.1.0-api-changes.txt')
    flags = {f.removeprefix('--').replace('-', '_'): True for f in provenance['extractor_flags']}
    text = extractor['extract_text'](raw, **flags)
    assert text.encode() == text_bytes
    seeded = extractor['seed_rows'](text, '6.1.0')
    ledger = json.loads(source_bytes('6.1.0-page-coverage.json'))
    assert ledger['source_sha256'] == sha(register_bytes)
    assert ledger['non_inventory_source']['sha256'] == sha(text_bytes)
    expected_ids = {r['id'] for r in register['entries']} | {r['source_id'] for r in seeded}
    rows = ledger['source_rows']
    assert len(rows) == len(expected_ids)
    assert {r['source_id'] for r in rows} == expected_ids
    assert all(r['note'] and r['status'] in ('bounded-coverage', 'audit-pending', 'metadata-only') for r in rows)
    assert all(not r['capabilities'] for r in rows if r['status'] != 'bounded-coverage')
    observed = read(HERE / 'patch_6_1_0_publication_sweep-results.json')
    assert set(observed) == {r['id'] for r in register['entries']}
    gaps = {key for key, value in observed.items() if not value['ok']}
    fixture = json.loads(blob(revision_scope, 'tests/data/patch_6_1_0_sweep_known_gaps.json'))
    assert gaps == set(fixture) == {'wt-global-api-SendChatMessage-14'}
    for row in rows:
        if row['source_id'] in observed:
            expected_status = 'bounded-coverage' if observed[row['source_id']]['ok'] else 'audit-pending'
            assert row['status'] == expected_status
    pending = read(HERE / 'p610-problematic-contracts.json')
    assert {r['source_id'] for r in pending} == {r['source_id'] for r in rows if r['status'] == 'audit-pending'}
    assert all(r['reason'] for r in pending)
    return {'inventory': len(register['entries']), 'extract': len(seeded),
            'statuses': dict(Counter(r['status'] for r in rows)), 'publication_gaps': len(gaps)}


def historical_preservation(context):
    preservation = read(HERE / 'p610-input-preservation.json')
    base = preservation['base_revision']
    expected = paths_at(base, 'data/patch-api/sources')
    assert {r['path'] for r in preservation['rows']} == set(expected)
    # Later audits may change their ledgers. Do not compare live other-audit inputs.
    for row in preservation['rows']:
        assert row['before_sha256'] == sha(blob(base, row['path']))
        assert row['after_sha256'] == sha(blob(preservation['audit_revision'], row['path']))
        assert row['before_sha256'] == row['after_sha256']
    modes = read(HERE / 'p610-extract-preservation.json')
    assert all(row['before'] == row['after'] for row in modes)
    revision = context['source_revision']
    patches = {p.name.removesuffix('-wikitext-register.json') for p in historical_registers(ROOT, revision)}
    reproduced = read(HERE / 'p610-register-reproduction.json')
    extracts = read(HERE / 'p610-saved-extract-reproduction.json')
    assert {r['patch'] for r in reproduced} == {r['patch'] for r in extracts} == patches
    for row in reproduced:
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == sha(blob(revision, f"data/patch-api/sources/{row['patch']}-wikitext-register.json"))
    for row in extracts:
        assert row['saved_sha256'] == sha(blob(revision, f"data/patch-api/sources/{row['patch']}-api-changes.txt"))
    inherited_path = 'data/patch-api/evidence/7.1.0-session-2026-10-08/integrated/p710-saved-extract-reproduction.json'
    inherited = {r['patch']: r for r in json.loads(blob(context['base_revision'], inherited_path))}
    for row in extracts:
        prior = inherited.get(row['patch'])
        if prior:
            assert (row['byte_identical'], row['error']) == (prior['byte_identical'], prior['error'])
        else:
            assert row['byte_identical'] and row['error'] is None
    return {'preserved_inputs': len(expected), 'modes': len(modes), 'registers': len(patches),
            'reproduced_extracts': sum(r['byte_identical'] for r in extracts),
            'inherited_extract_failures': [r['patch'] for r in extracts if not r['byte_identical']]}


def command_proof(context):
    for name, pinned in context['receipts'].items():
        receipt = read(HERE / f'{name}.proof.json')
        assert sha((HERE / f'{name}.proof.json').read_bytes()) == pinned['sha256']
        assert receipt['revision'] == pinned['revision']
        assert receipt['command'] == pinned['command']
        assert receipt['exit'] == pinned['exit'] and not receipt['invalidated']
        assert sha((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
    required = {'p610-discovery', 'p610-all-sweeps', 'p610-recap-prefork', 'p610-recap-model',
                'p610-legacy-absence', 'p610-format', 'p610-mists-check', 'p610-source-reproduction',
                'p610-generator-fixtures', 'p610-extractor-fixtures', 'p610-validator-fixtures', 'p610-negative'}
    assert set(context['receipts']) == required
    for name in required - {'p610-negative'}:
        assert context['receipts'][name]['exit'] == 0
    assert context['receipts']['p610-negative']['exit'] != 0
    for name in ('p610-discovery', 'p610-all-sweeps', 'p610-recap-prefork', 'p610-recap-model', 'p610-legacy-absence'):
        log = (HERE / f'{name}.txt').read_text()
        passes = re.findall(r'test result: ok\. (\d+) passed', log)
        assert passes and int(passes[-1]) > 0, name
    summaries = read(HERE / 'p610-sweep-summary.json')
    tests = historical_sweep_tests(ROOT, context['sweep_revision'])
    assert {r['test'] for r in summaries} == {p.stem for p in tests}
    for row in summaries:
        results = read(HERE / row['file'])
        assert row['sha256'] == sha((HERE / row['file']).read_bytes())
        assert row['rows'] == len(results)
        fixture_path = f"tests/data/{row['test'].removesuffix('_publication_sweep')}_sweep_known_gaps.json"
        expected_gaps = set(json.loads(blob(context['sweep_revision'], fixture_path)))
        assert {k for k, v in results.items() if not v['ok']} == expected_gaps
        assert row['gaps'] == len(expected_gaps)
    warnings = [l for l in (HERE / 'p610-mists-check.txt').read_text().splitlines() if l.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in l or '`iced_wgpu` (manifest)' in l for l in warnings), warnings
    negative = read(HERE / 'p610-negative-results.json')
    positive = read(HERE / 'patch_6_1_0_publication_sweep-results.json')
    assert {k for k, v in negative.items() if not v['ok']} == {'p610-negative-control', 'wt-global-api-SendChatMessage-14'}
    retained = {k: v for k, v in negative.items() if k != 'p610-negative-control'}
    assert retained == {k: v for k, v in positive.items() if k != 'wt-global-api-DeathRecap_HasEvents-7'}
    matrix = read(HERE / 'p610-other-validator-matrix.json')
    prior = [p for p in paths_at(context['base_revision'], 'data/patch-api/evidence')
             if Path(p).name in ('validate.py', 'validate_integrated.py')]
    assert {r['path'] for r in matrix} == set(prior)
    for row in matrix:
        assert row['exit'] == 0
        assert sha((HERE / row['log']).read_bytes()) == row['log_sha256']
        assert row['script_sha256'] == sha(blob(context['base_revision'], row['path']))
    assert not subprocess.check_output(['git', 'diff', '--name-only', context['base_revision'],
                                       context['sweep_revision'], '--', 'src'], cwd=ROOT)
    return {'sweeps': len(summaries), 'prior_validators': len(matrix)}


def retirements(context):
    scans = read(HERE / 'p610-retirement-scans.json')
    assert scans['members'] == scans['scans'] == [] and scans['tool'] == '/usr/bin/grep'
    for row in scans['later_registers'].values():
        paths = [p for p in paths_at(row['revision'], 'data/patch-api/sources') if p.endswith('-wikitext-register.json')]
        assert set(row['registers']) == set(paths) and row['readditions'] == []
    for row in read(HERE / 'p610-named-api-scans.json'):
        assert row['command'][0] == '/usr/bin/grep' and '-w' in row['command']
        assert row['exit'] in (0, 1)
        assert row['sha256'] == sha((HERE / row['file']).read_bytes())
    for name in ('index.md', 'log.md'):
        path = f'docs/wiki/{name}'
        before = blob(context['base_revision'], path).decode()
        after = blob(context['documentation_revision'], path).decode()
        assert before in after and len(after.splitlines()) >= len(before.splitlines())


def main():
    context = read(HERE / 'p610-proof-context.json')
    result = {'source': source_accounting(context), 'preservation': historical_preservation(context),
              'verification': command_proof(context)}
    retirements(context)
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
