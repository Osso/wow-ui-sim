"""Read-only historical retail 5.3.0 proof, portable to later audits."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers


def read(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', revision + ':' + path], cwd=ROOT)


def main():
    seal = read(HERE / 'p530-seal.json')
    for name, expected in seal['own_files'].items():
        assert digest((HERE / name).read_bytes()) == expected, ('own artifact drift', name)
    revision = seal['audit_revision']
    for path, expected in seal['shared_files'].items():
        assert digest(blob(revision, path)) == expected, ('historical input drift', path)
    registers = historical_registers(ROOT, revision)
    reproduction = read(HERE / 'p530-register-reproduction.json')
    assert {row['patch'] for row in reproduction} == {
        path.name.removesuffix('-wikitext-register.json') for path in registers}
    assert all(row['byte_identical'] and row['exit'] == 0 for row in reproduction)
    extracts = read(HERE / 'p530-saved-extract-reproduction.json')
    inherited = json.loads(blob(seal['base_revision'],
        'data/patch-api/evidence/5.4.7-session-2026-10-08/integrated/p547-saved-extract-reproduction.json'))
    failures = {row['patch']: row['error'] for row in extracts if not row['byte_identical']}
    assert failures == {row['patch']: row['error'] for row in inherited if not row['byte_identical']}
    register = json.loads(blob(revision, 'data/patch-api/sources/5.3.0-wikitext-register.json'))
    assert register['client_line'] == 'retail'
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    observations = read(HERE / 'patch_5_3_0_publication_sweep-results.json')
    entries = {row['id']: row for row in register['entries']}
    assert set(observations) == set(entries)
    known = json.loads(blob(revision, 'tests/data/patch_5_3_0_sweep_known_gaps.json'))
    assert set(known) == {name for name, row in observations.items() if not row['ok']}
    coverage = json.loads(blob(revision, 'data/patch-api/sources/5.3.0-page-coverage.json'))['source_rows']
    ids = [row['source_id'] for row in coverage]
    assert len(ids) == len(set(ids)) and set(entries) <= set(ids)
    assert all(row['note'] and row['status'] in ('bounded-coverage', 'audit-pending', 'metadata-only')
               for row in coverage)
    assert {row['source_id'] for row in read(HERE / 'p530-gap-review.json')} == set(known)
    for name, expected_exit in seal['proofs'].items():
        receipt = read(HERE / (name + '.proof.json'))
        assert receipt['exit'] == expected_exit and not receipt['invalidated'], name
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256'], name
        assert receipt['command'], name
        for path, expected in seal['proof_inputs'][name].items():
            assert digest(blob(receipt['revision'], path)) == expected, (name, path)
    negative = read(HERE / 'p530-negative-results.json')
    assert {name for name, row in negative.items() if not row['ok']} == set(known) | {seal['negative_id']}
    assert set(negative) == set(observations)
    for row in read(HERE / 'p530-scan-receipts.json'):
        assert row['tool'].startswith('/usr/bin/grep') and row['exit'] in (0, 1)
        assert row['matches'] == len((HERE / row['log']).read_text().splitlines())
    later = read(HERE / 'p530-later-retirement-check.json')
    for row in later:
        data = blob(row['revision'], row['path'])
        assert digest(data) == row['sha256']
        assert not any(hit['direction'] == 'added' for hit in row['hits'])
    # Shared snapshots only, never live wiki/source comparisons.
    for path, count in read(HERE / 'wiki-baseline.json').items():
        assert len(blob(revision, path).decode().splitlines()) >= count
    for name in ('p530-fetch.json', 'p530-diff-fetch.json', 'p530-parent-fetch.json'):
        page = next(iter(read(HERE / name)['query']['pages'].values()))
        assert 'missing' not in page and page['revisions'][0]['slots']['main']['*']
    for name, path, pageid, revid in [
        ('p530-fetch.json', 'data/patch-api/sources/5.3.0-api-changes.wikitext', 423274, 4065122),
        ('p530-diff-fetch.json', 'data/patch-api/sources/5.3.0-api-changes.diff.wikitext', 330804, 3188422),
    ]:
        page = next(iter(read(HERE / name)['query']['pages'].values()))
        source = page['revisions'][0]
        assert page['pageid'] == pageid and source['revid'] == revid
        assert source['slots']['main']['*'].encode() == blob(revision, path)
    parent_page = next(iter(read(HERE / 'p530-parent-fetch.json')['query']['pages'].values()))
    assert parent_page['pageid'] == 38992 and parent_page['revisions'][0]['revid'] == 6611399
    parent = (HERE / 'p530-parent.wikitext').read_text()
    assert parent_page['revisions'][0]['slots']['main']['*'] == parent
    assert '|toc = 50300' in parent and '|Release = May 21, 2013' in parent
    print(json.dumps({'status': 'PASS', 'inventory': len(entries), 'publication_gaps': len(known),
                      'accounted_ids': len(ids), 'registers': len(registers),
                      'inherited_extract_failures': sorted(failures)}))


if __name__ == '__main__':
    main()
