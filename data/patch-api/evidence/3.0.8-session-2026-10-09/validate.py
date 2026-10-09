#!/usr/bin/env python3
"""Replay only frozen 3.0.8 historical source/development evidence, stdlib only."""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import types

HERE = Path(__file__).resolve().parent
MANIFEST_SHA256 = 'df0531fa57238692c4db1e851f69173950679d05890352b7850f34ab1d9c97d1'


def digest(value):
    return hashlib.sha256(value).hexdigest()


def read_json(name):
    return json.loads((HERE / name).read_bytes())


def load_sealed_inputs():
    raw = (HERE / 'historical-inputs.json').read_bytes()
    assert digest(raw) == MANIFEST_SHA256, 'sealed input: historical-inputs.json'
    manifest = json.loads(raw)
    for name, seal in manifest['sealed_files'].items():
        path = HERE / name
        assert path.is_file(), 'sealed input: ' + name
        payload = path.read_bytes()
        assert len(payload) == seal['bytes'] and digest(payload) == seal['sha256'], 'sealed input: ' + name
    bundle = json.loads(gzip.decompress((HERE / 'historical-bundle.json.gz').read_bytes()))
    return manifest, bundle


def load_archived_module(bundle, name):
    module = types.ModuleType(name)
    # These pinned stdlib-only tools do not execute main when imported.
    module.__file__ = str(HERE / name)
    exec(compile(bundle[name], name, 'exec'), module.__dict__)
    return module


def reproduce_source(bundle):
    pin = read_json('source-pin.json')
    cached = next(p for p in read_json('cache-manifest.json')['pages'] if p['version'] == '3.0.8')
    assert pin == cached, 'manifest identity'
    assert (pin['pageid'], pin['revid'], pin['timestamp']) == (95599, 947144, '2021-12-04T02:32:44Z')
    response = (HERE / 'source-response.json').read_bytes()
    assert digest(response) == pin['response_sha256'], 'response hash'
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert (page['title'], page['pageid'], revision['revid'], revision['timestamp']) == (
        pin['title'], pin['pageid'], pin['revid'], pin['timestamp']), 'response identity'
    raw = (HERE / 'historical-source.wikitext').read_bytes()
    assert raw == revision['slots']['main']['*'].encode(), 'literal source body'
    assert digest(raw) == pin['wikitext_sha256'] and len(raw) == pin['wikitext_bytes'], 'body pin'
    provenance = read_json('historical-provenance.json')
    assert provenance['generator_flags'] == ['--legacy-labeled-summaries', '--client-line', 'retail']
    assert provenance['extractor_flags'] == [] and provenance['client_line'] == 'retail'
    parser = load_archived_module(bundle, 'tools/gen_patch_wikitext_register.py')
    register = read_json('historical-register.json')
    generated = {'schema': 'patch-api-wikitext-register/v1', 'patch': '3.0.8',
                 'source': {'path': 'data/patch-api/sources/3.0.8-api-changes.wikitext',
                            'revid': pin['revid'], 'sha256': digest(raw)},
                 'header_counts': [], 'entries': parser.parse_legacy_labeled_summaries(raw.decode()),
                 'client_line': 'retail'}
    assert (json.dumps(generated, indent=2, ensure_ascii=False) + '\n').encode() == (HERE / 'historical-register.json').read_bytes(), 'register reproduction'
    extractor = load_archived_module(bundle, 'tools/extract_patch_non_inventory.py')
    text = extractor.extract_text(raw.decode())
    assert text.encode() == (HERE / 'historical-extract.txt').read_bytes(), 'extract reproduction'
    return register, raw.decode(), text


def reconcile_observations(bundle, register):
    entries = {e['id']: e for e in register['entries']}
    observed = read_json('own-sweep-green-results.json')
    assert observed == read_json('own-sweep-red-results.json'), 'discovery/known-gap observations'
    assert set(entries) == set(observed), 'inventory/results identity'
    known = set(read_json('historical-known-gaps.json'))
    assert known == {key for key, value in observed.items() if not value['ok']}, 'original gaps'
    successors = read_json('historical-successors.json')
    assert successors['pending_main_only_oldest_first'] == ['3.1.0', '3.2.0', '3.3.0']
    latest = {}
    versions = []
    for path in successors['actual_retail_oldest_first']:
        later = json.loads(bundle[path])
        assert later.get('client_line', 'retail') == 'retail', 'Classic supersession'
        version = tuple(map(int, later['patch'].split('.')))
        assert version >= (3, 3, 3) and version[:2] != (3, 4), 'queued/Classic supersession'
        versions.append(version)
        for entry in later['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    assert versions == sorted(versions) and len(set(versions)) == len(versions), 'successor order'
    for key, entry in entries.items():
        newer = latest.get(entry['symbol'])
        own_removed = entry['direction'] == 'removed'
        removed = newer['direction'] == 'removed' if newer else own_removed
        superseded = newer['id'] if newer and removed != own_removed else None
        expectation = observed[key]['expected']
        assert expectation['symbol'] == entry['symbol'] and expectation['section'] == entry['section']
        assert expectation['publication'] == ('absent' if removed else 'published')
        assert expectation['superseded_by'] == superseded
    negative_entries = {e['id']: e for e in read_json('negative-register.json')['entries']}
    replaced = set(entries) - set(negative_entries)
    assert len(replaced) == 1 and all(observed[key]['ok'] for key in replaced), 'negative replaced row'
    assert set(negative_entries) - set(entries) == {'p308-negative-control'}
    assert negative_entries['p308-negative-control']['symbol'] == 'P308DefinitelyMissingGlobal'
    negative = read_json('negative-results.json')
    assert set(negative) == set(negative_entries) and len(negative) == len(observed)
    negative_gaps = {key for key, value in negative.items() if not value['ok']}
    assert negative_gaps == known | {'p308-negative-control'}, 'negative exact gap boundary'
    return observed, known, negative_gaps, successors


def reconcile_accounting(register, raw, text, observed):
    ledger = read_json('historical-page-coverage.json')
    rows = ledger['source_rows']
    by_id = {r['source_id']: r for r in rows}
    inventory = {e['id']: e for e in register['entries']}
    raw_lines = {f'raw-source-{i:03}': line for i, line in enumerate(raw.splitlines(), 1) if line.strip()}
    extract_lines = {f'extract-source-{i:03}': line for i, line in enumerate(text.splitlines(), 1) if line.strip()}
    signature_ids = {'signature-' + key for key in inventory}
    assert len(by_id) == len(rows) and len(inventory) == len(register['entries']), 'duplicate IDs'
    assert set(by_id) == set(inventory) | signature_ids | set(raw_lines) | set(extract_lines), 'complete accounting'
    assert ledger['source_sha256'] == digest(raw.encode())
    assert ledger['non_inventory_source']['sha256'] == digest(text.encode())
    for key, literal in (raw_lines | extract_lines).items():
        assert by_id[key]['literal'] == literal, 'literal line loss: ' + key
    for key, entry in inventory.items():
        bounded = observed[key]['ok'] and entry['section'] != 'scriptobjects'
        assert by_id[key]['status'] == ('bounded-coverage' if bounded else 'audit-pending')
        assert bool(by_id[key]['capabilities']) == bounded, 'publication overclaim'
        fragment = by_id['signature-' + key]
        assert fragment['literal_signature'] == entry.get('signature')
        assert fragment['literal_returns'] == entry.get('returns')
        assert fragment['status'] == 'audit-pending' and fragment['capabilities'] == [], 'signature overclaim'
    assert all(row['note'] and row['status'] in ('bounded-coverage', 'audit-pending', 'metadata-only') for row in rows)
    assert all(by_id[key]['capabilities'] == [] for key in extract_lines), 'rendered duplicate credit'
    return {'ledger_rows': len(rows), 'statuses': dict(Counter(r['status'] for r in rows)),
            'raw_nonblank_rows': len(raw_lines), 'extract_nonblank_rows': len(extract_lines),
            'header_rows': sum(line.startswith('==') for line in raw_lines.values()),
            'explicit_signature_rows': sum('signature' in e for e in inventory.values()),
            'unspecified_signature_rows': sum('signature' not in e for e in inventory.values()),
            'inventory_rows': len(inventory)}


def read_command_summaries():
    summaries = {}
    for receipt in read_json('command-ledger.json')['commands']:
        assert receipt['revision'] and receipt['scope'] and receipt['argv'] and receipt['cwd']
        log = (HERE / receipt['artifact']).read_text()
        if receipt['exit'] == 0:
            rust = re.search(r'test result: ok\. (\d+) passed; 0 failed', log)
            python = re.search(r'Ran (\d+) test.*\n\nOK', log, re.S)
            assert rust or python, 'successful test receipt: ' + receipt['artifact']
            summaries[receipt['artifact']] = {'exit': 0, 'passed': int((rust or python)[1])}
        else:
            assert 'FAILED' in log or 'could not compile' in log, 'failed receipt'
            summaries[receipt['artifact']] = {'exit': receipt['exit']}
    return summaries


def validate():
    manifest, bundle = load_sealed_inputs()
    register, raw, text = reproduce_source(bundle)
    observed, known, negative, successors = reconcile_observations(bundle, register)
    counts = reconcile_accounting(register, raw, text, observed)
    counts.update(publication_matches=len(observed) - len(known), publication_gaps=len(known),
                  negative_gaps=len(negative), sealed_inputs=len(manifest['sealed_files']),
                  actual_retail_successors=len(successors['actual_retail_oldest_first']),
                  command_summaries=read_command_summaries())
    return counts


if __name__ == '__main__':
    print(json.dumps(validate(), indent=2, sort_keys=True))
