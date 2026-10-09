#!/usr/bin/env python3
"""Replay sealed own historical evidence, not current-head/native acceptance.

Stdlib only. Reads this evidence directory and its archived selected blobs;
never Git, target, caches, imported repo helpers or current ledgers/fixtures.
"""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import types

HERE = Path(__file__).resolve().parent
MANIFEST_SHA256 = '8ddf62eec48165353245af754f82833d0c1adb2be054c90b6ccf0a3deb26ee04'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(name):
    return json.loads((HERE / name).read_bytes())


def verify_seals():
    raw = (HERE / 'historical-inputs.json').read_bytes()
    assert digest(raw) == MANIFEST_SHA256, 'manifest seal'
    manifest = json.loads(raw)
    for name, seal in manifest['sealed_files'].items():
        assert Path(name).name == name, 'sealed path boundary'
        content = (HERE / name).read_bytes()
        assert digest(content) == seal['sha256'], 'input seal: ' + name
        assert len(content) == seal['bytes'], 'size seal: ' + name
        assert len(content) <= 5_000_000, 'oversized input: ' + name
    archive = (HERE / 'historical-blobs.json.gz').read_bytes()
    assert digest(archive) == manifest['archive_sha256'], 'archive seal'
    assert len(archive) == manifest['archive_bytes'] <= 5_000_000, 'archive size seal'
    blobs = json.loads(gzip.decompress(archive))
    assert set(blobs) == set(manifest['archived_paths']), 'archive path seal'
    for path, content in blobs.items():
        assert digest(content.encode()) == manifest['archived_paths'][path], 'blob seal: ' + path
    return manifest, blobs


def load_module(blobs, path):
    module = types.ModuleType('historical_' + Path(path).stem)
    module.__file__ = str(HERE / 'snapshot' / path)
    exec(compile(blobs[path], path, 'exec'), module.__dict__)
    return module


def serialized(value):
    return json.dumps(value, indent=2, ensure_ascii=False) + '\n'


def default_register(module, raw, source):
    entries, counts = [], []
    buckets = module.split_sections(raw)
    for section in module.SECTIONS.values():
        category = 'cvars' if section == 'commands' else section
        rows, headers = module.parse_section(category, buckets.get(section, []))
        if section == 'commands':
            for header in headers:
                header['section'] = 'commands'
        entries.extend(rows)
        counts.extend(headers)
    return {'schema': 'patch-api-wikitext-register/v1', 'patch': '3.3.3',
            'source': source, 'header_counts': counts, 'entries': entries}


def verify_source(blobs):
    pin = read_json('source-pin.json')
    response = (HERE / 'source-response.json').read_bytes()
    assert digest(response) == pin['response_sha256'], 'pinned response'
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
        261776, 'Patch 3.3.3/API changes', 2531935, '2010-05-13T22:15:52Z'), 'historical retail identity'
    raw = (HERE / 'historical-source.wikitext').read_bytes()
    assert revision['slots']['main']['*'].encode() == raw, 'exact response content'
    assert digest(raw) == pin['wikitext_sha256'], 'source hash'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'
    provenance = read_json('historical-provenance.json')
    assert provenance['sha256'] == digest(raw) and provenance['revid'] == pin['revid'], 'provenance'
    assert provenance['client_line'] == 'retail', 'source client line'
    assert provenance['generator_flags'] == ['--historical-api-headings', '--client-line', 'retail'], 'frozen flags'
    assert provenance['extractor_flags'] == [], 'frozen extract flags'
    generator = load_module(blobs, 'tools/gen_patch_wikitext_register.py')
    register = read_json('historical-register.json')
    default = default_register(generator, raw.decode(), register['source'])
    baseline = load_module(blobs, 'baseline/gen_patch_wikitext_register.py')
    assert serialized(default) == serialized(default_register(baseline, raw.decode(), register['source'])), 'default byte compatibility'
    generated = dict(default, entries=generator.parse_historical_api_headings(raw.decode()), client_line='retail')
    assert serialized(generated) == (HERE / 'historical-register.json').read_text(), 'register byte reproduction'
    assert register['source'] == {'path': 'data/patch-api/sources/3.3.3-api-changes.wikitext',
                                  'revid': pin['revid'], 'sha256': digest(raw)}, 'register source identity'
    extractor = load_module(blobs, 'tools/extract_patch_non_inventory.py')
    text = extractor.extract_text(raw.decode())
    assert text == (HERE / 'historical-extract.txt').read_text(), 'extract byte reproduction'
    return register, extractor.seed_rows(text, '3.3.3'), raw.decode()


def verify_accounting(manifest, blobs, register, extract, raw):
    entries = {row['id']: row for row in register['entries']}
    assert len(entries) == len(register['entries']), 'duplicate inventory ID'
    signatures = read_json('historical-signatures.json')['signatures']
    signature_ids = {f"signature-{entry['symbol']}-{entry['wikitext_line']}" for entry in entries.values()}
    assert {row['source_id'] for row in signatures} == signature_ids, 'complete signature inventory'
    assert len(signatures) == len(signature_ids), 'duplicate signature ID'
    for row in signatures:
        assert row['source_text'] == raw.splitlines()[row['wikitext_line'] - 1], 'literal signature source'
        assert row['note'], 'signature disposition'
    coverage = read_json('historical-page-coverage.json')['source_rows']
    rows = {row['source_id']: row for row in coverage}
    extract_ids = {row['source_id'] for row in extract}
    assert len(rows) == len(coverage), 'duplicate ledger ID'
    assert set(rows) == set(entries) | extract_ids | signature_ids, 'complete ledger accounting'
    assert all(row['note'] and row['status'] in ('bounded-coverage', 'audit-pending', 'metadata-only')
               for row in rows.values()), 'explicit dispositions'
    known = read_json('historical-known-gaps.json')
    assert len(known) == len(set(known)), 'duplicate gap ID'
    observed = read_json('own-sweep-green-results.json')
    assert observed == read_json('own-sweep-red-results.json'), 'discovery observation preservation'
    assert set(observed) == set(entries), 'observed inventory'
    actual = {key for key, value in observed.items() if not value['ok']}
    assert actual == set(known), 'historical gap equality'
    declarations = re.findall(r'include_str!\("\.\./(data/patch-api/sources/[^"\n]+)"\)',
                              blobs['tests/patch_3_3_3_publication_sweep.rs'].split('later_registers:', 1)[1])
    assert declarations == manifest['later_registers'], 'exact archived successor set'
    latest, versions = {}, []
    for path in declarations:
        later = json.loads(blobs[path])
        assert later.get('client_line', 'retail') == 'retail', 'Classic supersession forbidden'
        versions.append(tuple(int(n) for n in later['patch'].split('.')))
        for entry in later['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    assert versions == sorted(versions) and versions[0] == (4, 1, 0), 'retail successor order'
    assert manifest['pending_successors'] == ['3.3.5', '4.0.1'], 'pending successor order'
    for source_id, entry in entries.items():
        newer = latest.get(entry['symbol'])
        own_removed = entry['direction'] == 'removed'
        removed = newer['direction'] == 'removed' if newer else own_removed
        expected = observed[source_id]['expected']
        assert expected['publication'] == ('absent' if removed else 'published'), 'publication expectation'
        superseded = newer['id'] if newer and removed != own_removed else None
        assert expected['superseded_by'] == superseded, 'historical supersession'
        assert rows[source_id]['status'] == ('audit-pending' if source_id in actual else 'bounded-coverage'), 'publication disposition'
        assert bool(rows[source_id]['capabilities']) == observed[source_id]['ok'], 'publication credit'
    negative = read_json('negative-results.json')
    assert {key for key, value in negative.items() if not value['ok']} == actual | {'p333-negative-control'}, 'negative gap accounting'
    missing_id = next(key for key, entry in entries.items() if entry['symbol'] == 'GetPetSpellBonusDamage')
    assert set(negative) == (set(observed) - {missing_id}) | {'p333-negative-control'}, 'negative inventory boundary'
    model_ids = {key for key in signature_ids if 'modeled-scalar-state-read' in rows[key]['capabilities']}
    assert all(rows[key]['status'] == ('bounded-coverage' if key in model_ids else 'audit-pending')
               for key in signature_ids), 'signature limits'
    assert all(not rows[key]['capabilities'] for key in signature_ids - model_ids), 'native/signature overclaim'
    prose = {row['source_id'] for row in extract if rows[row['source_id']]['status'] == 'audit-pending'}
    assert all(not rows[key]['capabilities'] for key in extract_ids), 'duplicate extract credit'
    return {'publication_rows': len(entries), 'publication_ok': len(entries) - len(actual),
            'publication_gaps': len(actual), 'negative_gaps': sum(not row['ok'] for row in negative.values()),
            'ledger_rows': len(rows), 'ledger_statuses': dict(Counter(row['status'] for row in rows.values())),
            'signature_rows': len(signature_ids), 'modeled_scalar_reads': len(model_ids),
            'extract_rows': len(extract_ids), 'prose_gaps': len(prose),
            'declared_numerical_headers': len(register['header_counts']), 'later_registers': len(declarations)}


def verify_commands():
    summaries = {}
    for command in read_json('command-ledger.json')['commands']:
        log = (HERE / command['artifact']).read_text()
        assert command['revision'] and command['scope'] and command['argv'], 'command identity'
        rust = re.search(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed;', log)
        python = re.search(r'Ran (\d+) tests? in [^\n]+\n\n(OK|FAILED)', log)
        assert rust or python, 'test receipt summary: ' + command['artifact']
        if rust:
            passed, failed = int(rust[2]), int(rust[3])
            assert (command['exit'] == 0) == (rust[1] == 'ok' and failed == 0), 'Rust receipt exit'
        else:
            passed = int(python[1]) if python[2] == 'OK' else 0
            failed = 0 if python[2] == 'OK' else int(python[1])
            assert (command['exit'] == 0) == (python[2] == 'OK'), 'Python receipt exit'
        summaries[command['artifact']] = {'passed': passed, 'failed': failed, 'exit': command['exit']}
    return summaries


def validate():
    manifest, blobs = verify_seals()
    register, extract, raw = verify_source(blobs)
    counts = verify_accounting(manifest, blobs, register, extract, raw)
    counts['command_summaries'] = verify_commands()
    counts['sealed_files'] = len(manifest['sealed_files'])
    counts['archive_bytes'] = manifest['archive_bytes']
    return counts


if __name__ == '__main__':
    print(json.dumps(validate(), indent=2, sort_keys=True))
