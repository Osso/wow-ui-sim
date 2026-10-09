#!/usr/bin/env python3
"""Frozen Retail 3.3.0 accounting replay. No Git, target, network or current files."""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import types

HERE = Path(__file__).resolve().parent
MANIFEST_SHA256 = 'dd30a2f0b2e906b82b07e64bca09b8fd8a006d6d1aa56c04963ebe80e1b2ad76'
CLOSURE_MANIFEST_SHA256 = 'dcfc019ddc8d624b4391f7bf5506944916f64130787cafc0c75597eeb8e631b1'
SOURCES = 'data/patch-api/sources/'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(path):
    return json.loads(path.read_bytes())


def verify_manifest(directory, expected):
    raw = (directory / 'historical-inputs.json').read_bytes()
    assert digest(raw) == expected, 'manifest seal'
    manifest = json.loads(raw)
    for relative, seal in manifest['sealed_files'].items():
        data = (directory / relative).read_bytes()
        assert digest(data) == seal['sha256'], 'sealed input: ' + relative
        assert len(data) == seal['bytes'], 'sealed size: ' + relative
    archive = (directory / 'historical-blobs.json.gz').read_bytes()
    assert digest(archive) == manifest['archive_sha256'], 'archive seal'
    blobs = json.loads(gzip.decompress(archive))
    assert set(blobs) == set(manifest['archived_files']), 'archive path seal'
    for path, data in blobs.items():
        assert digest(data.encode()) == manifest['archived_files'][path], 'blob seal: ' + path
    return manifest, blobs


def load_module(blobs, path):
    module = types.ModuleType('snapshot_' + Path(path).stem)
    module.__file__ = str(HERE / path)
    exec(compile(blobs[path], path, 'exec'), module.__dict__)
    return module


def verify_source(blobs):
    pin = read_json(HERE / 'source-pin.json')
    response = (HERE / 'source-response.json').read_bytes()
    assert digest(response) == pin['response_sha256'], 'response hash'
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
        pin['pageid'], pin['title'], pin['revid'], pin['timestamp']), 'source identity'
    raw = blobs[SOURCES + '3.3.0-api-changes.wikitext']
    assert revision['slots']['main']['*'] == raw, 'response/source equality'
    assert digest(raw.encode()) == pin['wikitext_sha256'], 'wikitext hash'
    assert len(raw.encode()) == pin['wikitext_bytes'], 'source size'
    provenance = json.loads(blobs[SOURCES + '3.3.0-api-changes.provenance.json'])
    assert provenance['generator_flags'] == ['--wrath-retail-summary', '--client-line', 'retail']
    assert provenance['extractor_flags'] == ['--wrath-summary-markup']
    generator = load_module(blobs, 'tools/gen_patch_wikitext_register.py')
    extractor = load_module(blobs, 'tools/extract_patch_non_inventory.py')
    generated = dict(schema='patch-api-wikitext-register/v1', patch='3.3.0',
                     source=dict(path=SOURCES + '3.3.0-api-changes.wikitext',
                                 revid=pin['revid'], sha256=pin['wikitext_sha256']),
                     header_counts=[], entries=generator.parse_wrath_retail_summary(raw),
                     client_line='retail')
    assert json.dumps(generated, indent=2, ensure_ascii=False) + '\n' == blobs[
        SOURCES + '3.3.0-wikitext-register.json'], 'register reproduction'
    text = extractor.extract_text(raw, wrath_summary_markup=True)
    assert text == blobs[SOURCES + '3.3.0-api-changes.txt'], 'full extraction reproduction'
    assert extractor.extract_text(raw) == (HERE / 'base-default-extract.txt').read_text(), (
        'default own extract regression')
    return generated, extractor.seed_rows(text, '3.3.0')


def default_outputs(generator, extractor, raw):
    try:
        buckets = generator.split_sections(raw)
        entries, counts = [], []
        for section in generator.SECTIONS.values():
            a, b = generator.parse_section('cvars' if section == 'commands' else section,
                                           buckets.get(section, []))
            entries += a
            counts += b
        parsed = ('ok', entries, counts)
    except (ValueError, KeyError, IndexError) as error:
        parsed = (type(error).__name__, str(error))
    try:
        text = ('ok', extractor.extract_text(raw))
    except (ValueError, KeyError, IndexError) as error:
        text = (type(error).__name__, str(error))
    return parsed, text


def verify_old_defaults(blobs):
    old = [load_module(blobs, 'baseline/' + name) for name in (
        'gen_patch_wikitext_register.py', 'extract_patch_non_inventory.py')]
    new = [load_module(blobs, 'tools/' + name) for name in (
        'gen_patch_wikitext_register.py', 'extract_patch_non_inventory.py')]
    results = read_json(HERE / 'default-regression-results.json')
    paths = {row['path'] for row in results}
    archived = {path for path in blobs if path.startswith(SOURCES)
                and path.endswith('-api-changes.wikitext')}
    assert paths == archived, 'default regression source set'
    for row in results:
        raw = blobs[row['path']]
        assert digest(raw.encode()) == row['sha256'], 'default source hash'
        before = default_outputs(*old, raw)
        after = default_outputs(*new, raw)
        assert before == after, 'old output changed: ' + row['path']
        assert row['default_equal'] and row['register_result'] == before[0][0]
        assert row['extract_result'] == before[1][0]
    return len(results)


def verify_accounting(manifest, blobs, register, supplemental):
    rows = read_json(HERE / 'historical-page-coverage.json')['source_rows']
    by_id = {row['source_id']: row for row in rows}
    entries = {row['id']: row for row in register['entries']}
    supplemental_ids = {row['source_id'] for row in supplemental}
    signatures = {f"signature-{row['symbol']}-{row['wikitext_line']}" for row in entries.values()
                  if row['annotation'].startswith('* NEW - ')
                  and row['section'] not in ['events', 'framexml']}
    assert len(by_id) == len(rows) and len(entries) == len(register['entries']), 'duplicate IDs'
    assert set(by_id) == set(entries) | supplemental_ids | signatures, 'complete literal accounting'
    assert all(row['note'] and row['status'] in (
        'bounded-coverage', 'metadata-only', 'audit-pending') for row in rows), 'row dispositions'
    assert all(not by_id[name]['capabilities'] for name in signatures), 'signature overclaim'
    observed = read_json(HERE / 'own-sweep-green-results.json')
    known = set(read_json(HERE / 'historical-known-gaps.json'))
    assert set(observed) == set(entries), 'observed register set'
    assert {key for key, row in observed.items() if not row['ok']} == known, 'gap accounting'
    latest = {}
    versions = []
    for path in manifest['later_registers']:
        later = json.loads(blobs[path])
        assert later.get('client_line', 'retail') == 'retail', 'Classic successor credit'
        versions.append(tuple(map(int, later['patch'].split('.'))))
        for row in later['entries']:
            if row['direction'] != 'changed':
                latest[row['symbol']] = row
    assert versions == sorted(versions) and len(set(versions)) == len(versions), 'successor order'
    for source_id, entry in entries.items():
        newer = latest.get(entry['symbol'])
        own_removed = entry['direction'] == 'removed'
        removed = newer['direction'] == 'removed' if newer else own_removed
        superseded = newer['id'] if newer and removed != own_removed else None
        assert observed[source_id]['expected']['publication'] == (
            'absent' if removed else 'published'), 'historical expectation'
        assert observed[source_id]['expected']['superseded_by'] == superseded, 'supersession'
        assert by_id[source_id]['status'] == (
            'audit-pending' if source_id in known else 'bounded-coverage'), 'publication status'
        assert bool(by_id[source_id]['capabilities']) == observed[source_id]['ok'], 'capability credit'
    negative = read_json(HERE / 'negative-results.json')
    negative_gaps = {key for key, row in negative.items() if not row['ok']}
    negative_register = read_json(HERE / 'negative-register.json')
    assert set(negative) == {row['id'] for row in negative_register['entries']}
    assert len(negative) == len(entries), 'negative boundary'
    assert negative_gaps == known | {'p330-negative-control'}, 'negative detection'
    return dict(publication_rows=len(entries), publication_ok=len(entries) - len(known),
                publication_gaps=len(known), extract_rows=len(supplemental_ids),
                signature_rows=len(signatures), ledger_rows=len(rows),
                ledger_statuses=dict(Counter(row['status'] for row in rows)),
                prose_gaps=sum(by_id[name]['status'] == 'audit-pending' for name in supplemental_ids),
                negative_gaps=len(negative_gaps), later_retail_registers=len(versions))


def verify_receipts(directory):
    summaries = {}
    for command in read_json(directory / 'command-ledger.json')['commands']:
        log = (directory / command['artifact']).read_text()
        if command['exit'] == 0:
            passed = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed', log)
            python = re.search(r'Ran (\d+) tests?.*\n\nOK', log, re.S)
            assert passed or python, 'success receipt: ' + command['artifact']
            assert all(int(failed) == 0 for _, failed in passed), 'receipt failure'
            summaries[command['artifact']] = (sum(int(count) for count, _ in passed)
                                              if passed else int(python[1]))
        else:
            assert 'FAILED' in log or 'error:' in log or 'panicked' in log, 'failure receipt'
    return summaries


def validate():
    manifest, blobs = verify_manifest(HERE, MANIFEST_SHA256)
    register, supplemental = verify_source(blobs)
    summary = verify_accounting(manifest, blobs, register, supplemental)
    summary['default_regression_pages'] = verify_old_defaults(blobs)
    summary['recorded_passes'] = verify_receipts(HERE)
    closure_manifest, closure_blobs = verify_manifest(HERE / 'closures', CLOSURE_MANIFEST_SHA256)
    closures = read_json(HERE / 'closures/coverage-update.json')['closures']
    assert all(row['source_id'] in {item['source_id'] for item in
                   read_json(HERE / 'historical-page-coverage.json')['source_rows']}
               and row['proof'] and row['scope'] for row in closures), 'closure accounting'
    summary['real_model_closures'] = len(closures)
    summary['closure_recorded_passes'] = verify_receipts(HERE / 'closures')
    summary['sealed_files'] = len(manifest['sealed_files']) + len(closure_manifest['sealed_files'])
    summary['archived_files'] = len(blobs) + len(closure_blobs)
    summary['proof'] = 'frozen development receipts only; main owns integration and acceptance'
    return summary


if __name__ == '__main__':
    print(json.dumps(validate(), sort_keys=True))
