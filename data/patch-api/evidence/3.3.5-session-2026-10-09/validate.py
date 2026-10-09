#!/usr/bin/env python3
"""Replay original 3.3.5 accounting, not current-head/runtime/native acceptance.

Only frozen own source and historical sidecars/bundle are inputs. No Git,
target, live coverage/gaps, mutable later registers or subprocess dependency.
"""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import types

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCES = Path('data/patch-api/sources')
MANIFEST_SHA256 = 'ae545b4396125f7d634a49812978e319ffa4cb4bc959387b940b43eca125533c'


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read_json(path):
    return json.loads(path.read_bytes())


def verify_seals(root, here):
    raw = (here / 'historical-inputs.json').read_bytes()
    assert digest(raw) == MANIFEST_SHA256, 'historical manifest seal'
    manifest = json.loads(raw)
    for relative, seal in manifest['sealed_files'].items():
        content = (root / relative).read_bytes()
        assert digest(content) == seal['sha256'], 'sealed input: ' + relative
        assert len(content) == seal['bytes'], 'sealed size: ' + relative
        assert len(content) <= 5_000_000, 'oversized historical input: ' + relative
    bundle = json.loads(gzip.decompress((here / 'historical-bundle.json.gz').read_bytes()))
    assert set(bundle) == set(manifest['snapshot_sha256']), 'historical snapshot set'
    for path, text in bundle.items():
        assert digest(text.encode()) == manifest['snapshot_sha256'][path], 'snapshot: ' + path
    return manifest, bundle


def load_module(root, bundle, path):
    module = types.ModuleType('p335_' + Path(path).stem)
    module.__file__ = str(root / path)
    exec(compile(bundle[path], path, 'exec'), module.__dict__)
    return module


def reproduce_source(root, here, bundle):
    pin = read_json(here / 'source-pin.json')
    assert (pin['title'], pin['pageid'], pin['revid'], pin['timestamp']) == (
        'Patch 3.3.5/API changes', 25049, 247986, '2010-07-10T15:47:20Z'), 'literal registry identity'
    registry = json.loads(bundle[str(SOURCES / 'api-change-pages-remaining.json')])
    assert registry['pages'][-1]['version'] == '1.0.0', 'registry cutoff'
    registered = next(p for p in registry['pages'] if p['version'] == '3.3.5')
    assert all(registered[k] == pin[k] for k in ('title', 'pageid', 'revid', 'timestamp')), 'registry identity'
    frozen = json.loads(bundle['data/patch-api/source-cache/legacy-2026-10-09/manifest.json'])
    assert next(p for p in frozen['pages'] if p['version'] == '3.3.5') == pin, 'literal manifest record'
    response = (here / 'source-response.json').read_bytes()
    assert digest(response) == pin['response_sha256'], 'source response hash'
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert (page['title'], page['pageid'], revision['revid'], revision['timestamp']) == (
        pin['title'], pin['pageid'], pin['revid'], pin['timestamp']), 'response identity'
    raw = (root / SOURCES / '3.3.5-api-changes.wikitext').read_bytes()
    assert raw == revision['slots']['main']['*'].encode(), 'literal source body'
    assert digest(raw) == pin['wikitext_sha256'] and len(raw) == pin['wikitext_bytes'], 'source pin'
    provenance = read_json(root / SOURCES / '3.3.5-api-changes.provenance.json')
    assert provenance['generator_flags'] == ['--legacy-section-lists', '--client-line', 'retail'], 'frozen generator flags'
    assert provenance['extractor_flags'] == [] and provenance['client_line'] == 'retail', 'frozen extractor/client flags'
    assert provenance['sha256'] == digest(raw) and provenance['revid'] == pin['revid'], 'provenance'
    generator = load_module(root, bundle, 'tools/gen_patch_wikitext_register.py')
    generated = {'schema': 'patch-api-wikitext-register/v1', 'patch': '3.3.5',
                 'source': {'path': str(SOURCES / '3.3.5-api-changes.wikitext'),
                            'revid': pin['revid'], 'sha256': digest(raw)},
                 'header_counts': [], 'entries': generator.parse_legacy_section_lists(raw.decode()),
                 'client_line': 'retail'}
    register_bytes = (root / SOURCES / '3.3.5-wikitext-register.json').read_bytes()
    assert (json.dumps(generated, indent=2, ensure_ascii=False) + '\n').encode() == register_bytes, 'register reproduction'
    extractor = load_module(root, bundle, 'tools/extract_patch_non_inventory.py')
    text = extractor.extract_text(raw.decode())
    assert text.encode() == (root / SOURCES / '3.3.5-api-changes.txt').read_bytes(), 'extract reproduction'
    return generated, extractor.seed_rows(text, '3.3.5'), raw.decode(), text


def verify_inventory(here, manifest, bundle, register):
    entries = {e['id']: e for e in register['entries']}
    observed = read_json(here / 'own-sweep-green-results.json')
    red = read_json(here / 'own-sweep-red-results.json')
    assert red == observed, 'discovery/known-gap observations differ'
    assert set(observed) == set(entries), 'publication inventory'
    known = set(read_json(here / 'historical-known-gaps.json'))
    actual = {key for key, value in observed.items() if not value['ok']}
    assert actual == known, 'original gap fixture'
    latest = {}
    for path in manifest['later_registers']:
        later = json.loads(bundle[path])
        assert later.get('client_line', 'retail') == 'retail', 'Classic successor credit'
        assert tuple(map(int, later['patch'].split('.'))) >= (4, 1, 0), 'queued/Classic successor credit'
        for entry in later['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    for source_id, entry in entries.items():
        newer = latest.get(entry['symbol'])
        own_removed = entry['direction'] == 'removed'
        removed = newer['direction'] == 'removed' if newer else own_removed
        superseded = newer['id'] if newer and removed != own_removed else None
        expected = observed[source_id]['expected']
        assert expected['symbol'] == entry['symbol'] and expected['section'] == entry['section'], 'probe identity'
        assert expected['publication'] == ('absent' if removed else 'published'), 'historical expectation'
        assert expected['superseded_by'] == superseded, 'historical supersession'
    negative_register = read_json(here / 'negative-register.json')
    negative = read_json(here / 'negative-results.json')
    negative_entries = {e['id']: e for e in negative_register['entries']}
    replaced = set(entries) - set(negative_entries)
    assert len(replaced) == 1 and all(observed[key]['ok'] for key in replaced), 'negative replaced row'
    assert set(negative_entries) - set(entries) == {'p335-negative-control'}, 'negative added row'
    assert negative_entries['p335-negative-control']['symbol'] == 'P335DefinitelyMissingGlobal', 'negative identity'
    assert set(negative) == set(negative_entries) and len(negative) == len(observed), 'negative boundary'
    negative_gaps = {key for key, value in negative.items() if not value['ok']}
    assert negative_gaps == known | {'p335-negative-control'}, 'negative gap accounting'
    return observed, known, negative_gaps


def verify_accounting(here, register, supplemental, raw, text, observed, known):
    ledger = read_json(here / 'historical-page-coverage.json')
    rows = ledger['source_rows']
    by_id = {r['source_id']: r for r in rows}
    entries = {e['id']: e for e in register['entries']}
    extra = {r['source_id'] for r in supplemental}
    fragments = {'signature-' + e['id']: e for e in entries.values()
                 if 'signature' in e or 'returns' in e}
    assert len(rows) == len(by_id) and len(entries) == len(register['entries']), 'duplicate IDs'
    assert set(by_id) == set(entries) | extra | set(fragments), 'complete page accounting'
    assert ledger['source_sha256'] == digest(raw.encode()), 'ledger source boundary'
    assert ledger['non_inventory_source']['sha256'] == digest(text.encode()), 'ledger extract boundary'
    statuses = {'publication-gap', 'superseded-publication', 'absence-only',
                'publication-only', 'metadata-only', 'semantic-gap', 'signature-gap'}
    assert all(r['note'] and r['status'] in statuses for r in rows), 'missing disposition'
    for key, entry in entries.items():
        result = observed[key]
        status = ('publication-gap' if key in known else
                  'superseded-publication' if result['expected']['superseded_by'] else
                  'absence-only' if entry['direction'] == 'removed' else 'publication-only')
        assert by_id[key]['status'] == status, 'publication disposition'
        assert bool(by_id[key]['capabilities']) == (status in ('absence-only', 'publication-only')), 'publication overclaim'
    for key, entry in fragments.items():
        row = by_id[key]
        assert row['status'] == 'signature-gap' and row['capabilities'] == [], 'signature overclaim'
        assert row.get('literal_signature') == entry.get('signature'), 'signature fragment'
        assert row.get('literal_returns') == entry.get('returns'), 'return fragment'
    semantic = {r['source_id'] for r in supplemental
                if 'NotifyInspect' in text.splitlines()[int(r['source_id'].rsplit('-', 1)[1]) - 1]}
    for key in extra:
        assert by_id[key]['capabilities'] == [], 'extract overclaim'
        assert by_id[key]['status'] == ('semantic-gap' if key in semantic else 'metadata-only'), 'extract disposition'
    headers = [line for line in text.splitlines() if line.startswith('==')]
    assert not register['header_counts'], 'invented numerical headers'
    return {'ledger_rows': len(rows), 'ledger_statuses': dict(Counter(r['status'] for r in rows)),
            'extract_rows': len(extra), 'semantic_gaps': len(semantic),
            'signature_rows': len(fragments), 'named_headers': len(headers),
            'publication_rows': len(entries), 'publication_ok': len(entries) - len(known),
            'publication_gaps': len(known)}


def command_summaries(here):
    summaries = {}
    for command in read_json(here / 'command-ledger.json')['commands']:
        log = (here / command['artifact']).read_text()
        assert command['revision'] and command['scope'] and command['argv'], 'command boundary'
        if command['exit'] == 0:
            rust = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed', log)
            python = re.search(r'Ran (\d+) test.*\n\nOK', log, re.S)
            assert rust or python, 'successful command receipt'
            assert all(int(failed) == 0 for _, failed in rust), 'failed success receipt'
            count = sum(int(n) for n, _ in rust) if rust else int(python[1])
            summaries[command['artifact']] = {'passed': count, 'exit': command['exit']}
        else:
            assert 'panicked' in log or 'FAILED (failures=' in log, 'failure receipt'
            summaries[command['artifact']] = {'exit': command['exit']}
    return summaries


def validate(root=ROOT, here=HERE):
    manifest, bundle = verify_seals(root, here)
    register, supplemental, raw, text = reproduce_source(root, here, bundle)
    observed, known, negative = verify_inventory(here, manifest, bundle, register)
    counts = verify_accounting(here, register, supplemental, raw, text, observed, known)
    scans = read_json(here / 'whole-tree-retirement-scans.json')
    assert all(not scan['hits'] for scan in scans.values()), 'removed-name consumers'
    counts.update(negative_gaps=len(negative), later_registers=len(manifest['later_registers']),
                  sealed_inputs=len(manifest['sealed_files']), command_summaries=command_summaries(here))
    return counts


if __name__ == '__main__':
    print(json.dumps(validate(), indent=2, sort_keys=True))
