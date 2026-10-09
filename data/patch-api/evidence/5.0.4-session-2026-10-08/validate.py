"""Read-only 5.0.4 proof with compact historical Git tree/blob pins."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def read_json(path):
    return json.loads(path.read_bytes())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_git_object(spec):
    return subprocess.check_output(['git', 'show', spec], cwd=ROOT)


def read_pinned_json(final, path):
    return json.loads(read_git_object(final['source_tree'] + ':' + path))


def validate_seals(final):
    for name, expected in final['owned_artifacts'].items():
        assert digest((HERE / name).read_bytes()) == expected, 'own evidence drift: ' + name
    for path, expected in final['current_source_guards'].items():
        assert digest((ROOT / path).read_bytes()) == expected, 'source drift: ' + path
    for path, blob in final['historical_blobs'].items():
        assert read_git_object(blob), 'missing historical blob: ' + path
    for path, blob in final['prior_validators'].items():
        assert digest((ROOT / path).read_bytes()) == digest(read_git_object(blob)), path


def validate_inventory(final, register, ledger, observed, fixture):
    entries = register['entries']
    assert len(entries) == 626 and len({e['id'] for e in entries}) == 626
    assert len([e for e in entries if e['id'].startswith('wt-')]) == 77
    assert len([e for e in entries if e['id'].startswith('diff-wt-')]) == 549
    assert set(observed) == {e['id'] for e in entries}
    assert {key for key, value in observed.items() if not value['ok']} == set(fixture)
    assert len(fixture) == 159
    assert all(c['header_count'] == c['parsed_count'] for c in register['header_counts'])
    rows = ledger['source_rows']
    assert len(rows) == 704 and len({r['source_id'] for r in rows}) == 704
    extractor = {'__name__': 'p504_historical_extractor',
                 '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    source = read_git_object(final['historical_blobs']['tools/extract_patch_non_inventory.py'])
    exec(compile(source, 'historical_extractor', 'exec'), extractor)
    text = read_git_object(final['source_tree'] + ':5.0.4-api-changes.txt').decode()
    prose = extractor['seed_rows'](text, '5.0.4')
    signatures = {'signature-' + e['id'] for e in entries if 'signature' in e}
    assert {r['source_id'] for r in rows} == set(observed) | {r['source_id'] for r in prose} | signatures
    assert all(r['note'] for r in rows)
    assert len(signatures) == 5
    for e in entries:
        if 'signature' in e:
            row = next(r for r in rows if r['source_id'] == 'signature-' + e['id'])
            assert row['signature'] == e['signature'] and row['status'] == 'audit-pending'


def validate_accounting(final, ledger, observed, fixture):
    rows = ledger['source_rows']
    summary = read_json(HERE / 'accounting-summary.json')
    assert summary['total_ids'] == len(rows)
    assert summary['publication_mismatches'] == len(fixture)
    assert {status: sum(r['status'] == status for r in rows)
            for status in summary['statuses']} == summary['statuses']
    review = read_json(HERE / 'gap-review.json')
    assert {r['source_id'] for r in review['publication_mismatches']} == set(fixture)
    assert len(review['prose_pending']) == 61 and len(review['signatures_pending']) == 5
    assert review['no_new_retirements'] is True and final['retirements'] == []
    modeled = [r for r in rows if 'pet-type-state-read' in r['capabilities']]
    assert len(modeled) == 1 and observed[modeled[0]['source_id']]['ok']
    negative = read_json(HERE / 'negative-results.json')
    expectation = read_json(HERE / 'negative-expectation.json')
    assert set(negative) == set(observed)
    assert {k for k, v in negative.items() if not v['ok']} == set(fixture) | {expectation['source_id']}
    assert len([v for v in negative.values() if not v['ok']]) == 160


def validate_reproduction(final):
    rows = read_json(HERE / 'reproduction.json')
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only',
                                    final['source_tree']], cwd=ROOT, text=True).splitlines()
    required = {n for n in names if n.endswith('-wikitext-register.json')}
    assert required == {Path(r['path']).name for r in rows}
    assert len(rows) == 67 and all(r['register_identical'] for r in rows)
    assert {r['patch'] for r in rows if not r['extract_identical']} == {'12.0.5', '12.0.7', '12.1.0'}
    for row in rows:
        register = read_git_object(final['source_tree'] + ':' + Path(row['path']).name)
        extract = read_git_object(final['source_tree'] + ':' + row['patch'] + '-api-changes.txt')
        assert digest(register) == row['register_sha256']
        assert digest(extract) == row['extract_sha256']
    prov = read_pinned_json(final, '5.0.4-api-changes.provenance.json')
    assert prov['revid'] == 3706382 and prov['toc'] == 50001 and prov['client_line'] == 'retail'
    for label, meta, path in [('source', prov, '5.0.4-api-changes.wikitext'),
                             ('diff', prov['diff_source'], None),
                             ('parent', prov['toc_source'], None)]:
        response = read_json(HERE / (label + '-response.json'))
        revision = response['query']['pages'][str(meta['pageid'])]['revisions'][0]
        assert revision['revid'] == meta['revid']
        raw = revision['slots']['main']['*'].encode()
        expected = read_git_object(final['source_tree'] + ':' + path) if path else (ROOT / meta['path']).read_bytes()
        assert raw == expected and digest(raw) == meta['sha256']
    assert re.search(r'^\|toc\s*=\s*50001\s*$', (HERE / 'parent.wikitext').read_text(), re.M)


def validate_scans(register):
    scans = read_json(HERE / 'removed-consumer-scans.json')
    removed = {e['id'] for e in register['entries'] if e['direction'] == 'removed'}
    assert {r['id'] for r in scans['occurrences']} == removed and len(removed) == 121
    assert all(set(r['hits']) == {'cache_qualified', 'cache_bare', 'callers_qualified', 'callers_bare'}
               for r in scans['occurrences'])
    later = read_json(HERE / 'later-retail-register-scan.json')
    for row in later['registers']:
        tree = later['pins'][row['revision']]
        raw = read_git_object(tree + ':' + Path(row['path']).name)
        assert digest(raw) == row['sha256']
        assert json.loads(raw).get('client_line', 'retail') == 'retail'
    republished = {e['symbol'] for r in later['registers'] for e in r['readditions']}
    assert republished == {'GetExpertisePercent', 'isRaidFinderDungeonDisplayable',
                          'Cooldown:GetDrawEdge', 'Cooldown:SetDrawEdge'}


def validate_receipts(final):
    for row in final['commands']:
        assert row['exit'] == row['expected_exit'] and row['invalidated'] is False, row['name']
        assert digest((HERE / row['log']).read_bytes()) == row['log_sha256']
    fixtures = read_json(HERE / 'python-fixtures.json')
    assert all(r['exit'] == 0 for r in fixtures)
    assert sum(int(re.search(r'Ran (\d+) tests?', r['stderr'])[1]) for r in fixtures) == 101
    assert read_json(HERE / 'lua-errors.receipt.json')['stdout'].strip() == '[]'
    warnings = re.findall(r'^warning: (.*)$', (HERE / 'mists-check.log').read_text(), re.M)
    assert all(w.startswith(('iced-wgpu-patched/', '`iced_wgpu` (manifest)')) for w in warnings), warnings
    others = json.loads(gzip.decompress((HERE / 'other-retail-sweep-results.json.gz').read_bytes()))
    sweep = read_json(HERE / 'retail-sweep-summary.json')
    assert len(others) == 61 and sweep['pages'] == 62 and sweep['tests'] == 63
    own = read_json(HERE / 'patch_5_0_4_publication_sweep-results.json')
    assert sum(len(value) for value in others.values()) + len(own) == sweep['observations'] == 10779


def validate():
    final = read_json(HERE / 'finalization.json')
    validate_seals(final)
    register = read_pinned_json(final, '5.0.4-wikitext-register.json')
    ledger = read_pinned_json(final, '5.0.4-page-coverage.json')
    fixture = json.loads(read_git_object(final['historical_blobs']['tests/data/patch_5_0_4_sweep_known_gaps.json']))
    observed = read_json(HERE / 'patch_5_0_4_publication_sweep-results.json')
    validate_inventory(final, register, ledger, observed, fixture)
    validate_accounting(final, ledger, observed, fixture)
    validate_reproduction(final)
    validate_scans(register)
    validate_receipts(final)
    print(json.dumps({'status': 'PASS', 'inventory': 626, 'accounted_ids': 704,
                      'publication_mismatches': 159, 'retirements': 0}))


if __name__ == '__main__':
    validate()
