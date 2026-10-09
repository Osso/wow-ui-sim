#!/usr/bin/env python3
"""Own sealed source/accounting replay, not runtime/native/current-head acceptance.

Only files beside this validator and its compact archive are read. No Git,
subprocess, target, mutable source coverage, or current successor dependency.
"""
import base64
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import types

HERE = Path(__file__).resolve().parent
MANIFEST_SHA256 = 'a33186896972499e4f01b635994f0530f7e573566adde9849716c92d2c6371cd'


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read_json(name):
    return json.loads((HERE / name).read_bytes())


def serialized(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()


def verify_seals():
    raw = (HERE / 'historical-inputs.json').read_bytes()
    assert digest(raw) == MANIFEST_SHA256, 'historical manifest seal'
    manifest = json.loads(raw)
    for name, seal in manifest['sealed_files'].items():
        content = (HERE / name).read_bytes()
        assert digest(content) == seal['sha256'], 'input seal: ' + name
        assert len(content) == seal['bytes'] <= 5_000_000, 'input size seal: ' + name
    bundle = json.loads(gzip.decompress((HERE / 'historical-bundle.json.gz').read_bytes()))
    assert set(bundle) == set(manifest['snapshot_sha256']), 'archive path seal'
    for path, content in bundle.items():
        assert digest(content.encode()) == manifest['snapshot_sha256'][path], 'snapshot seal: ' + path
    return manifest, bundle


def load_module(bundle, path):
    module = types.ModuleType('p310_' + Path(path).stem)
    module.__file__ = str(HERE / 'snapshot' / path)
    exec(compile(bundle[path], path, 'exec'), module.__dict__)
    return module


def verify_source(bundle):
    pin = read_json('source-pin.json')
    assert (pin['pageid'], pin['revid'], pin['timestamp'], pin['title']) == (
        233195, 2259594, '2013-05-22T17:34:42Z', 'Patch 3.1.0/API changes'), 'literal source identity'
    frozen = json.loads(bundle['data/patch-api/source-cache/legacy-2026-10-09/manifest.json'])
    assert next(p for p in frozen['pages'] if p['version'] == '3.1.0') == pin, 'frozen manifest identity'
    registry = json.loads(bundle['data/patch-api/sources/api-change-pages-remaining.json'])
    assert registry['pages'][-1]['version'] == '1.0.0', 'registry cutoff'
    page_pin = next(p for p in registry['pages'] if p['version'] == '3.1.0')
    assert all(page_pin[k] == pin[k] for k in ('title', 'pageid', 'revid', 'timestamp')), 'registry identity'
    response = (HERE / 'source-response.json').read_bytes()
    assert digest(response) == pin['response_sha256'], 'response pin'
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
        pin['pageid'], pin['title'], pin['revid'], pin['timestamp']), 'response identity'
    raw = (HERE / 'historical-source.wikitext').read_bytes()
    assert raw == revision['slots']['main']['*'].encode(), 'literal source body'
    assert digest(raw) == pin['wikitext_sha256'] and len(raw) == pin['wikitext_bytes'], 'source pin'
    provenance = read_json('historical-provenance.json')
    assert provenance['generator_flags'] == ['--legacy-function-labels', '--client-line', 'retail'], 'generator flags'
    assert provenance['extractor_flags'] == [] and provenance['client_line'] == 'retail', 'extract/client flags'
    assert provenance['sha256'] == digest(raw) and provenance['revid'] == pin['revid'], 'provenance'
    generator = load_module(bundle, 'tools/gen_patch_wikitext_register.py')
    register = dict(schema='patch-api-wikitext-register/v1', patch='3.1.0',
        source=dict(path='data/patch-api/sources/3.1.0-api-changes.wikitext', revid=pin['revid'], sha256=digest(raw)),
        header_counts=[], entries=generator.parse_legacy_function_labels(raw.decode()), client_line='retail')
    assert serialized(register) == (HERE / 'historical-register.json').read_bytes(), 'register byte reproduction'
    baseline = load_module(bundle, 'baseline/gen_patch_wikitext_register.py')
    for module in (generator, baseline):
        sections = module.split_sections(raw.decode())
        entries, counts = [], []
        for section in module.SECTIONS.values():
            rows, headers = module.parse_section('cvars' if section == 'commands' else section, sections.get(section, []))
            entries.extend(rows)
            counts.extend(headers)
        assert entries == [] and counts == [], 'default inventory compatibility'
    extractor = load_module(bundle, 'tools/extract_patch_non_inventory.py')
    text = extractor.extract_text(raw.decode())
    assert text.encode() == (HERE / 'historical-extract.txt').read_bytes(), 'extract byte reproduction'
    return register, extractor.seed_rows(text, '3.1.0'), raw.decode()


def verify_publication(manifest, bundle, register):
    entries = {row['id']: row for row in register['entries']}
    assert len(entries) == len(register['entries']), 'duplicate inventory ID'
    observed = read_json('own-sweep-green-results.json')
    assert observed == read_json('own-sweep-red-results.json'), 'discovery observations changed'
    assert set(observed) == set(entries), 'complete publication observation'
    known = read_json('historical-known-gaps.json')
    actual = {key for key, value in observed.items() if not value['ok']}
    assert len(known) == len(set(known)) and set(known) == actual, 'exact original known gaps'
    latest, versions = {}, []
    for path in manifest['later_registers']:
        later = json.loads(bundle[path])
        assert later.get('client_line', 'retail') == 'retail', 'Classic successor credit forbidden'
        version = tuple(map(int, later['patch'].split('.')))
        assert version >= (4, 1, 0), 'queued successor credit forbidden'
        versions.append(version)
        for entry in later['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    assert versions == sorted(versions), 'retail successor order'
    assert manifest['pending_successors'] == ['3.2.0', '3.3.0', '3.3.3', '3.3.5', '4.0.1'], 'queued retail successors'
    for key, entry in entries.items():
        newer = latest.get(entry['symbol'])
        own_removed = entry['direction'] == 'removed'
        removed = newer['direction'] == 'removed' if newer else own_removed
        expected = observed[key]['expected']
        assert expected['publication'] == ('absent' if removed else 'published'), 'publication expectation'
        superseded = newer['id'] if newer and own_removed != removed else None
        assert expected['superseded_by'] == superseded, 'historical supersession'
    negative = read_json('negative-results.json')
    control = read_json('negative-control.json')
    replaced = control['replaced_id']
    assert replaced in entries and observed[replaced]['ok'], 'negative replaces only a matching occurrence'
    assert set(negative) == (set(entries) - {replaced}) | {'p310-negative-control'}, 'negative observation inventory'
    assert {key for key, value in negative.items() if not value['ok']} == actual | {'p310-negative-control'}, 'negative gap boundary'
    return entries, observed, actual


def verify_accounting(register, extracts, raw, entries, observed, gaps):
    coverage = read_json('historical-page-coverage.json')['source_rows']
    rows = {row['source_id']: row for row in coverage}
    assert len(rows) == len(coverage), 'duplicate ledger ID'
    raw_ids = {f'raw-line-{i:03}' for i, line in enumerate(raw.splitlines(), 1) if line.strip()}
    signatures = read_json('historical-signatures.json')['signatures']
    fragments = []
    for number, line in enumerate(raw.splitlines(), 1):
        for ordinal, call in enumerate(re.finditer(r'([A-Za-z_]\w*(?::[A-Za-z_]\w*)?)\([^)]*\)', line), 1):
            fragments.append((f'signature-{number:03}-{ordinal}', call[1], call[0], number, line))
    assert [(s['source_id'], s['symbol'], s['fragment'], s['wikitext_line'], s['source_text']) for s in signatures] == fragments, 'complete literal signature fragments'
    signature_ids = {s['source_id'] for s in signatures}
    extract_ids = {row['source_id'] for row in extracts}
    assert set(rows) == set(entries) | raw_ids | signature_ids | extract_ids, 'complete nonblank/fragment/extract/identity accounting'
    assert all(row['note'] and row['status'] in ('bounded-coverage', 'audit-pending', 'metadata-only') for row in rows.values()), 'explicit dispositions'
    for key in entries:
        assert rows[key]['status'] == ('audit-pending' if key in gaps else 'bounded-coverage'), 'publication status'
        assert rows[key]['capabilities'] == (['publication-only'] if observed[key]['ok'] else []), 'publication-only credit'
    for key in raw_ids:
        row = rows[key]
        assert row['source_text'] == raw.splitlines()[row['wikitext_line'] - 1], 'literal raw line'
    assert all(not rows[key]['capabilities'] for key in raw_ids | signature_ids | extract_ids), 'native/model/signature overclaim'
    assert all(rows[key]['status'] == 'audit-pending' for key in signature_ids), 'signature limit retention'
    return dict(publication_rows=len(entries), publication_matches=len(entries) - len(gaps), publication_gaps=len(gaps),
                negative_gaps=len(gaps) + 1, raw_nonblank=len(raw_ids), extract_rows=len(extract_ids),
                signature_fragments=len(signature_ids), ledger_rows=len(rows), modeled_closures=0,
                ledger_statuses=dict(Counter(row['status'] for row in rows.values())))


def decode_tree(encoded, tree_id):
    raw = base64.b64decode(encoded)
    assert hashlib.sha1(b'tree ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == tree_id, 'Git tree object pin'
    records = {}
    position = 0
    while position < len(raw):
        end = raw.index(b'\0', position)
        mode, name = raw[position:end].split(b' ', 1)
        records[name.decode()] = (mode.decode(), raw[end + 1:end + 21].hex())
        position = end + 21
    return records


def verify_commands(bundle):
    commands = read_json('command-ledger.json')['commands']
    for row in commands:
        assert row['revision'] and row['argv'] and row['scope'], 'exact command identity'
        log = (HERE / row['artifact']).read_text()
        assert row['exit_code'] == row['expected_exit_code'], 'retained exit status'
        assert row['required_log_text'] in log, 'retained command result'
    scopes = json.loads(bundle['proof-scopes.json'])
    assert {row['revision'] for row in commands} <= set(scopes), 'receipt revisions retained'
    for revision, scope in scopes.items():
        commit = base64.b64decode(scope['commit_base64'])
        assert hashlib.sha1(b'commit ' + str(len(commit)).encode() + b'\0' + commit).hexdigest() == revision, 'commit object pin'
        assert commit.startswith(('tree ' + scope['root_tree']).encode()), 'commit tree pin'
        trees = {oid: decode_tree(encoded, oid) for oid, encoded in scope['tree_objects'].items()}
        for path, seal in scope['selected_files'].items():
            oid = scope['root_tree']
            for part in path.split('/'):
                _, oid = trees[oid][part]
            assert oid == seal['git_blob'], 'selected path belongs to pinned commit: ' + path
            raw = bundle[seal['snapshot_path']].encode()
            assert digest(raw) == seal['sha256'], 'selected file SHA pin: ' + path
            assert hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == seal['git_blob'], 'selected Git blob pin: ' + path
    return len(commands), len(scopes)


def main():
    manifest, bundle = verify_seals()
    register, extracts, raw = verify_source(bundle)
    entries, observed, gaps = verify_publication(manifest, bundle, register)
    summary = verify_accounting(register, extracts, raw, entries, observed, gaps)
    summary['retained_commands'], summary['pinned_revisions'] = verify_commands(bundle)
    summary['sealed_files'] = len(manifest['sealed_files'])
    summary['archived_files'] = len(bundle)
    summary['pending_successors'] = manifest['pending_successors']
    print(json.dumps(summary, sort_keys=True))


if __name__ == '__main__':
    main()
