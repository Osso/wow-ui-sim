"""Read-only historical 5.4.8 proof; shared inputs are Git blobs, never moving files."""
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


def read(name):
    return json.loads((HERE / name).read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', revision + ':' + path)


def paths(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def historical_registers(revision):
    return [p for p in paths(revision, 'data/patch-api/sources')
            if p.endswith('-wikitext-register.json')]


def historical_namespace(revision, filename):
    namespace = {'__name__': 'historical_tool', '__file__': str(ROOT / filename)}
    exec(compile(blob(revision, filename), filename, 'exec'), namespace)
    return namespace


def source_accounting(context):
    revision = context['accounting_revision']
    def source(name):
        return blob(revision, 'data/patch-api/sources/5.4.8-' + name)
    provenance = json.loads(source('api-changes.provenance.json'))
    page = read('p548-fetch.json')['query']['pages']['177816']
    pinned = page['revisions'][0]
    raw = source('api-changes.wikitext')
    assert page['title'] == 'Patch 5.4.8/API changes'
    assert provenance['revid'] == pinned['revid'] == 1736772
    assert provenance['timestamp'] == pinned['timestamp']
    assert provenance['sha256'] == sha((HERE / 'p548-fetch.json').read_bytes())
    assert raw.decode() == pinned['slots']['main']['*']
    assert provenance['wikitext_sha256'] == sha(raw)
    parent = read('p548-patch-fetch.json')['query']['pages']['123196']['revisions'][0]
    confirmation = read('p548-retail-confirmation.json')
    assert confirmation['revid'] == parent['revid']
    assert confirmation['fetch_sha256'] == sha((HERE / 'p548-patch-fetch.json').read_bytes())
    assert '|toc = 50400' in parent['slots']['main']['*']
    assert '|Release = May 20, 2014' in parent['slots']['main']['*']
    register = json.loads(source('wikitext-register.json'))
    generator = historical_namespace(context['tool_revision'], 'tools/gen_patch_wikitext_register.py')
    assert register['entries'] == generator['parse_combat_restriction_bullets'](raw.decode())
    cvars = re.findall(r'^\*\[\[CVar ([^|\]]+)\|', raw.decode(), re.M)
    assert [r['symbol'] for r in register['entries'] if r['section'] == 'cvars'] == cvars
    assert [r['symbol'] for r in register['entries'] if r['section'] == 'global-api'] == ['SetUIVisibility']
    assert register['source']['sha256'] == sha(raw)
    assert register['source']['revid'] == pinned['revid']
    assert all(r['direction'] == 'changed' for r in register['entries'])
    extractor = historical_namespace(context['tool_revision'], 'tools/extract_patch_non_inventory.py')
    text = extractor['extract_text'](raw.decode())
    assert source('api-changes.txt') == text.encode()
    seeded = extractor['seed_rows'](text, '5.4.8')
    ledger = json.loads(source('page-coverage.json'))
    assert ledger['source_sha256'] == sha(source('wikitext-register.json'))
    assert ledger['non_inventory_source']['sha256'] == sha(text.encode())
    expected = {r['id'] for r in register['entries']} | {r['source_id'] for r in seeded}
    rows = ledger['source_rows']
    assert len(rows) == len(expected) == len({r['source_id'] for r in rows})
    assert {r['source_id'] for r in rows} == expected
    assert all(r['note'] and r['status'] in ('bounded-coverage', 'audit-pending', 'metadata-only') for r in rows)
    assert all(not r['capabilities'] for r in rows if r['status'] != 'bounded-coverage')
    results = read('patch_5_4_8_publication_sweep-results.json')
    assert set(results) == {r['id'] for r in register['entries']}
    gaps = {key for key, result in results.items() if not result['ok']}
    fixture = json.loads(blob(revision, 'tests/data/patch_5_4_8_sweep_known_gaps.json'))
    assert gaps == set(fixture)
    for row in rows:
        if row['source_id'] in results:
            expected_status = 'bounded-coverage' if results[row['source_id']]['ok'] else 'audit-pending'
            assert row['status'] == expected_status
    pending = read('p548-problematic-contracts.json')
    assert {r['source_id'] for r in pending} == {r['source_id'] for r in rows if r['status'] == 'audit-pending'}
    assert all(r['reason'] for r in pending)
    return {'inventory': len(register['entries']), 'extract': len(seeded),
            'statuses': dict(Counter(r['status'] for r in rows)), 'publication_gaps': len(gaps)}


def reproduction(context):
    registers = read('p548-register-reproduction.json')
    revision = context['reproduction_revision']
    expected = historical_registers(revision)
    assert {r['patch'] for r in registers} == {Path(p).name.removesuffix('-wikitext-register.json') for p in expected}
    assert all(r['exit'] == 0 and r['byte_identical'] for r in registers)
    for row in registers:
        path = 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json'
        assert row['sha256'] == sha(blob(revision, path))
    extracts = read('p548-saved-extract-reproduction.json')
    assert {r['patch'] for r in extracts} == {r['patch'] for r in registers}
    failures = [r['patch'] for r in extracts if not r['byte_identical']]
    inherited = json.loads(blob(context['base_revision'],
        'data/patch-api/evidence/6.1.0-session-2026-10-08/integrated/p610-saved-extract-reproduction.json'))
    assert failures == [r['patch'] for r in inherited if not r['byte_identical']]
    extractor = historical_namespace(context['tool_revision'], 'tools/extract_patch_non_inventory.py')
    for row in extracts:
        prefix = 'data/patch-api/sources/' + row['patch']
        assert row['saved_sha256'] == sha(blob(revision, prefix + '-api-changes.txt'))
        options = {flag.removeprefix('--').replace('-', '_'): True for flag in row['verified_flags']}
        try:
            rendered = extractor['extract_text'](blob(revision, prefix + '-api-changes.wikitext').decode(), **options)
            assert row['sha256'] == sha(rendered.encode()) and row['error'] is None
        except ValueError as error:
            assert row['sha256'] is None and row['error'] == str(error)
    preservation = read('p548-input-preservation.json')
    assert {r['path'] for r in preservation['rows']} == set(paths(context['base_revision'], 'data/patch-api/sources'))
    for row in preservation['rows']:
        assert row['before_sha256'] == sha(blob(context['base_revision'], row['path']))
        assert row['after_sha256'] == sha(blob(preservation['audit_revision'], row['path']))
        assert row['before_sha256'] == row['after_sha256']
    return {'registers': len(registers), 'extracts': len(extracts) - len(failures), 'inherited_extract_failures': failures}


def model_observations(context):
    register = json.loads(blob(context['accounting_revision'],
                              'data/patch-api/sources/5.4.8-wikitext-register.json'))
    names = {r['symbol'] for r in register['entries'] if r['section'] == 'cvars'}
    summary = read('p548-accounting-summary.json')
    active = set(summary['current_readable_cvars'])
    for mode in ['bare', 'cached']:
        observed = read('p548-' + mode + '-combat-observations.json')
        assert set(observed) == names
        assert {name for name, row in observed.items() if row['status'] == 'bounded-model'} == active
        for name in active:
            row = observed[name]
            assert row['events'] == 0
            assert all(row[key] for key in ['global_blocked', 'namespace_blocked', 'uppercase_blocked',
                                            'bitfield_blocked', 'scale_unchanged', 'secure_combat_write',
                                            'insecure_out_of_combat_write'])
    globals_modeled = [row for row in register['entries'] if row['section'] == 'global-api']
    return len(active) + len(globals_modeled)


def queued_registers(context):
    own = json.loads(blob(context['accounting_revision'],
                         'data/patch-api/sources/5.4.8-wikitext-register.json'))
    symbols = {row['symbol'] for row in own['entries']}
    for snapshot in read('p548-later-register-check.json').values():
        revision = snapshot['revision']
        assert {row['path'] for row in snapshot['registers']} == set(historical_registers(revision))
        for row in snapshot['registers']:
            contents = blob(revision, row['path'])
            assert row['sha256'] == sha(contents)
            register = json.loads(contents)
            assert row['patch'] == register['patch']
            assert row['intersections'] == [r for r in register['entries'] if r['symbol'] in symbols]


def proofs(context):
    for name in context['proof_receipts']:
        receipt = read(name)
        assert receipt['exit'] == 0 and not receipt['invalidated'], name
        assert sha((HERE / receipt['log']).read_bytes()) == receipt['log_sha256'], name
        git('rev-parse', '--verify', receipt['revision'] + '^{commit}')
        if receipt['command'][0] == 'cargo':
            changed = git('diff', '--name-only', receipt['revision'], context['accounting_revision'],
                          '--', 'src', 'tests', 'Cargo.toml', 'Cargo.lock', 'build.rs')
            assert not changed, 'proof source scope changed: ' + name
    negative = read('p548-negative.proof.json')
    assert negative['exit'] != 0
    actual = read('patch_5_4_8_publication_sweep-results.json')
    control = read('p548-negative-results.json')
    assert sum(not r['ok'] for r in control.values()) == sum(not r['ok'] for r in actual.values()) + 1
    assert set(control) == set(actual)
    changed_ids = {key for key in actual if actual[key] != control[key]}
    assert changed_ids == {key for key in actual if actual[key]['expected']['symbol'] == 'SetUIVisibility'}
    assert all(actual[key]['ok'] and not control[key]['ok'] for key in changed_ids)
    startup = read('p548-branch-startup.receipt.json')
    baseline = read('p548-master-startup.receipt.json')
    assert startup['exit'] == baseline['exit'] == 0
    assert startup['addons_enabled'] and baseline['addons_enabled']
    assert startup['matches_master'] and startup['errors'] == baseline['errors']
    assert not git('diff', '--name-only', context['base_revision'], baseline['revision'],
                   '--', 'src', 'crates', 'Cargo.toml', 'Cargo.lock')
    summary = read('p548-sweep-summary.json')
    sweep_paths = [p for p in paths(summary['revision'], 'tests') if p.endswith('_publication_sweep.rs')]
    assert {Path(p).stem for p in sweep_paths} == {r['test'] for r in summary['rows']}
    for row in summary['rows']:
        results = read(row['file'])
        assert row['observations'] == len(results)
        assert row['gaps'] == sum(not r['ok'] for r in results.values())
        if row['test'] != 'patch_5_4_8_publication_sweep':
            assert row['unchanged_from_master']
            old = json.loads(blob(context['base_revision'], row['master_path']))
            assert results == old
    warnings = (HERE / 'p548-mists-check.txt').read_text()
    non_vendor = [line for line in warnings.splitlines() if line.startswith('warning:')
                  and 'iced-wgpu-patched/Cargo.toml:' not in line and '`iced_wgpu` (manifest)' not in line]
    assert not non_vendor, non_vendor
    return {'sweep_pages': len(summary['rows']), 'startup_errors': len(startup['errors'])}


def main():
    context = read('p548-proof-context.json')
    for name, digest in context['own_seals'].items():
        assert sha((HERE / name).read_bytes()) == digest, 'own evidence tamper: ' + name
        path = (HERE / name).relative_to(ROOT).as_posix()
        git('ls-files', '--error-unmatch', path)
    source = source_accounting(context)
    reproduced = reproduction(context)
    verified = proofs(context)
    verified['meaningful_models'] = model_observations(context)
    queued_registers(context)
    retirement = read('p548-retirement-scans.json')
    assert retirement['retired_members'] == []
    for row in read('p548-named-scans.json'):
        content = (HERE / row['output']).read_bytes()
        assert row['tool'] == '/usr/bin/grep' and '-w' in row['command']
        assert sha(content) == row['sha256'] and len(content.decode().splitlines()) == row['lines']
    baseline = read('p548-wiki-baseline.json')
    for name, minimum in baseline.items():
        historical = blob(context['accounting_revision'], 'docs/wiki/' + name)
        assert len(historical.decode().splitlines()) >= minimum
    print(json.dumps({'status': 'PASS', **source, **reproduced, **verified}, sort_keys=True))


if __name__ == '__main__':
    main()
