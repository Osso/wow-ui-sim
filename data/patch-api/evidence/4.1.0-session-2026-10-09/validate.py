#!/usr/bin/env python3
"""Own historical receipt validator; never a current-head runtime acceptance gate."""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys
import types

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_pin_trees import object_id, tree_id

MANIFEST_SHA256 = '34d382e9e9f20eda25c728ed71e93737f44944c0b675cea106b6859a20de7005'
SOURCES = Path('data/patch-api/sources')


def read_json(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify_seals(root, here):
    raw = (here / 'historical-inputs.json').read_bytes()
    assert digest(raw) == MANIFEST_SHA256, 'historical manifest seal'
    manifest = json.loads(raw)
    for relative, seal in manifest['sealed_files'].items():
        content = (root / relative).read_bytes()
        assert digest(content) == seal['sha256'], 'sealed input: ' + relative
        assert len(content) == seal['bytes'], 'sealed size: ' + relative
    archive = (here / 'historical-blobs.json.gz').read_bytes()
    assert digest(archive) == manifest['archive_sha256'], 'historical archive seal'
    blobs = json.loads(gzip.decompress(archive))
    for oid, content in blobs.items():
        assert object_id('blob', content.encode()) == oid, 'historical blob: ' + oid
    used = set()
    for scope in manifest['scopes']:
        assert tree_id(scope['entries']) == scope['selected_tree'], 'historical selected tree'
        for mode, kind, oid in scope['entries'].values():
            assert kind == 'blob' and mode in ('100644', '100755'), 'historical mode/type'
            assert oid in blobs, 'missing historical blob: ' + oid
            used.add(oid)
    assert used == set(blobs), 'unreferenced historical blobs'
    for path in here.rglob('*'):
        if path.is_file():
            assert path.stat().st_size <= 5_000_000, 'oversized evidence: ' + str(path)
    return manifest, blobs


def snapshot_content(scope, blobs, path):
    return blobs[scope['entries'][path][2]]


def load_snapshot_module(scope, blobs, path):
    module = types.ModuleType('p410_' + Path(path).stem)
    module.__file__ = str(ROOT / path)
    exec(compile(snapshot_content(scope, blobs, path), path, 'exec'), module.__dict__)
    return module


def verify_source_and_reproduction(root, here, scope, blobs):
    pin = read_json(here / 'source-pin.json')
    response_bytes = (here / 'source-response.json').read_bytes()
    assert digest(response_bytes) == pin['response_sha256'], 'response digest'
    response = json.loads(response_bytes)
    page = response['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert page['title'] == pin['title'] and page['pageid'] == pin['pageid'], 'source page identity'
    assert revision['revid'] == pin['revid'] and revision['timestamp'] == pin['timestamp'], 'source revision'
    raw = (root / SOURCES / '4.1.0-api-changes.wikitext').read_bytes()
    assert revision['slots']['main']['*'].encode() == raw, 'response content differs'
    assert digest(raw) == pin['wikitext_sha256'] and len(raw) == pin['wikitext_bytes'], 'source pin'
    provenance = read_json(root / SOURCES / '4.1.0-api-changes.provenance.json')
    assert provenance['revid'] == pin['revid'] and provenance['sha256'] == digest(raw), 'provenance'
    assert provenance['generator_flags'] == ['--cataclysm-change-bullets', '--client-line', 'retail'], 'frozen generator flags'
    assert provenance['extractor_flags'] == [], 'frozen extractor flags'
    generator = load_snapshot_module(scope, blobs, 'tools/gen_patch_wikitext_register.py')
    generated = {
        'schema': 'patch-api-wikitext-register/v1', 'patch': '4.1.0',
        'source': {'path': str(SOURCES / '4.1.0-api-changes.wikitext'),
                   'revid': pin['revid'], 'sha256': digest(raw)},
        'header_counts': [], 'entries': generator.parse_cataclysm_change_bullets(raw.decode()),
        'client_line': 'retail',
    }
    register = read_json(root / SOURCES / '4.1.0-wikitext-register.json')
    assert json.dumps(generated, indent=2, ensure_ascii=False) + '\n' == (
        root / SOURCES / '4.1.0-wikitext-register.json').read_text(), 'register reproduction'
    extractor = load_snapshot_module(scope, blobs, 'tools/extract_patch_non_inventory.py')
    text = extractor.extract_text(raw.decode())
    assert text == (root / SOURCES / '4.1.0-api-changes.txt').read_text(), 'extract reproduction'
    return register, extractor.seed_rows(text, '4.1.0'), raw.decode()


def signature_ids(raw):
    ids = set()
    for number, line in enumerate(raw.splitlines(), 1):
        if 'new third parameter' in line:
            for name in sorted(set(re.findall(r'\bCOMBAT_LOG_EVENT(?:_UNFILTERED)?\b', line))):
                ids.add(f'signature-{name}-{number}')
    return ids


def verify_accounting(root, here, manifest, scope, blobs, register, supplemental, raw):
    rows = read_json(root / SOURCES / '4.1.0-page-coverage.json')['source_rows']
    by_id = {row['source_id']: row for row in rows}
    entries = {entry['id']: entry for entry in register['entries']}
    source_ids = {row['source_id'] for row in supplemental}
    signatures = signature_ids(raw)
    assert len(by_id) == len(rows) and len(entries) == len(register['entries']), 'duplicate source IDs'
    assert set(by_id) == set(entries) | source_ids | signatures, 'complete source accounting'
    assert all(row['note'] and row['status'] in ('bounded-coverage', 'metadata-only', 'audit-pending') for row in rows), 'missing row disposition'
    assert all(not by_id[name]['capabilities'] for name in signatures), 'signature parity overclaim'
    observed = read_json(here / 'own-sweep-green-results.json')
    known = set(read_json(root / 'tests/data/patch_4_1_0_sweep_known_gaps.json'))
    actual = {key for key, value in observed.items() if not value['ok']}
    assert set(observed) == set(entries), 'sweep inventory'
    assert actual == known, 'known gap accounting'
    latest = {}
    for path in manifest['later_registers']:
        later = json.loads(snapshot_content(scope, blobs, path))
        assert later.get('client_line', 'retail') == 'retail', 'Classic successor credit'
        for entry in later['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    for source_id, entry in entries.items():
        newer = latest.get(entry['symbol'])
        own_removed = entry['direction'] == 'removed'
        removed = newer['direction'] == 'removed' if newer else own_removed
        superseded = newer['id'] if newer and removed != own_removed else None
        expected = observed[source_id]['expected']
        assert expected['publication'] == ('absent' if removed else 'published'), 'historical expectation'
        assert expected['superseded_by'] == superseded, 'historical supersession'
        row = by_id[source_id]
        assert row['status'] == ('audit-pending' if source_id in known else 'bounded-coverage'), 'publication disposition'
        assert bool(row['capabilities']) == observed[source_id]['ok'], 'publication capability credit'
    negative = read_json(here / 'negative-results.json')
    negative_gaps = {key for key, value in negative.items() if not value['ok']}
    assert negative_gaps == known | {'p410-negative-control'}, 'negative gap accounting'
    assert len(negative) == len(observed), 'negative row boundary'
    return {
        'publication_rows': len(entries), 'publication_gaps': len(known),
        'publication_ok': len(entries) - len(known), 'extract_rows': len(source_ids),
        'signature_rows': len(signatures), 'ledger_rows': len(rows),
        'ledger_statuses': dict(Counter(row['status'] for row in rows)),
        'prose_gaps': sum(by_id[name]['status'] == 'audit-pending' for name in source_ids),
        'negative_gaps': len(negative_gaps),
    }


def verify_recorded_commands(here):
    commands = read_json(here / 'command-ledger.json')['commands']
    summaries = {}
    for command in commands:
        log = (here / command['artifact']).read_text()
        passed = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed', log)
        if command['artifact'] == 'parser-green.txt':
            match = re.search(r'Ran (\d+) test.*\n\nOK', log, re.S)
            assert command['exit'] == 0 and match, 'parser receipt'
            summaries[command['artifact']] = {'passed': int(match[1]), 'exit': command['exit']}
        elif command['exit'] == 0:
            assert passed and all(int(failed) == 0 for _, failed in passed), 'successful command receipt'
            summaries[command['artifact']] = {'passed': sum(int(count) for count, _ in passed), 'exit': command['exit']}
        else:
            assert 'error:' in log or 'panicked' in log, 'failure receipt'
            summaries[command['artifact']] = {'exit': command['exit']}
    return summaries


def validate(root=ROOT, here=HERE):
    manifest, blobs = verify_seals(root, here)
    scope = manifest['scopes'][-1]
    register, supplemental, raw = verify_source_and_reproduction(root, here, scope, blobs)
    summary = verify_accounting(root, here, manifest, scope, blobs, register, supplemental, raw)
    summary.update(sealed_files=len(manifest['sealed_files']), historical_blobs=len(blobs),
                   historical_scopes=len(manifest['scopes']),
                   later_retail_registers=len(manifest['later_registers']),
                   recorded_commands=verify_recorded_commands(here),
                   proof='historical receipts only; no native parity or current-head acceptance')
    return summary


if __name__ == '__main__':
    print(json.dumps(validate(), sort_keys=True))
