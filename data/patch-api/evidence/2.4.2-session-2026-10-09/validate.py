#!/usr/bin/env python3
"""Immutable historical 2.4.2 replay; no Git, target or current checkout reads."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import types
from collections import Counter

HERE = Path(__file__).resolve().parent
MANIFEST_SHA256 = 'dae8a4298ab6d31fc72cd2a6c9e6613e1043dab0316707e0f0378e00ff50309b'
SOURCES = 'data/patch-api/sources/'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(path):
    return json.loads(path.read_bytes())


def load_module(root, relative, name):
    module = types.ModuleType(name)
    module.__file__ = str(root / relative)
    exec(compile((root / relative).read_text(), module.__file__, 'exec'), module.__dict__)
    return module


def verify_seals():
    raw = (HERE / 'historical-manifest.json').read_bytes()
    assert digest(raw) == MANIFEST_SHA256, 'manifest seal'
    manifest = json.loads(raw)
    for relative, expected in manifest['files'].items():
        data = (HERE / relative).read_bytes()
        assert digest(data) == expected, 'seal: ' + relative
    blobs = json.loads(gzip.decompress((HERE / 'historical-blobs.json.gz').read_bytes()))
    assert set(blobs) == set(manifest['blobs']), 'archive path set'
    for relative, expected in manifest['blobs'].items():
        assert digest(blobs[relative].encode()) == expected, 'blob seal: ' + relative
        assert not Path(relative).is_absolute() and '..' not in Path(relative).parts, 'unsafe blob path'
    return manifest, blobs


def verify_source(root):
    pin = read_json(HERE / 'source-pin.json')
    assert digest((HERE / 'source-manifest.json').read_bytes()) == pin['manifest_sha256']
    assert digest((HERE / 'source-response.json').read_bytes()) == pin['response_sha256']
    manifest = read_json(HERE / 'source-manifest.json')
    frozen = next(row for row in manifest['pages'] if row['version'] == '2.4.2')
    for key in ('pageid', 'revid', 'timestamp', 'wikitext_sha256', 'response_sha256'):
        assert frozen[key] == pin[key], 'manifest identity: ' + key
    page = read_json(HERE / 'source-response.json')['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
        81145, 'Patch 2.4.2/API changes', 803933, '2021-12-28T02:03:05Z'), 'frozen retail identity'
    raw = (root / (SOURCES + '2.4.2-api-changes.wikitext')).read_text()
    assert raw == revision['slots']['main']['*'], 'response/raw equality'
    assert digest(raw.encode()) == pin['wikitext_sha256'] and len(raw.encode()) == pin['wikitext_bytes']
    registry_path = manifest['registry_path']
    assert digest((root / registry_path).read_bytes()) == manifest['registry_sha256']
    registry = read_json(root / registry_path)
    versions = [tuple(map(int, row['version'].split('.'))) for row in registry['pages']]
    assert min(versions) == (1, 0, 0), 'registry boundary'
    registered = next(row for row in registry['pages'] if row['version'] == '2.4.2')
    assert registered['revid'] == 803933 and registered['pageid'] == 81145, 'registry pin'
    return raw, len(versions)


def execute_python(root, script, args, output):
    result = subprocess.run([sys.executable, '-B', str(root / script)] + args,
                            cwd=root, env=dict(os.environ, PATH='/nonexistent', PYTHONDONTWRITEBYTECODE='1'),
                            capture_output=True)
    return result.returncode, output.read_bytes() if result.returncode == 0 else result.stderr


def verify_reproduction(root):
    provenance = read_json(root / (SOURCES + '2.4.2-api-changes.provenance.json'))
    output = root / 'scratch-register.json'
    args = ['2.4.2', SOURCES + '2.4.2-api-changes.wikitext', '803933', str(output)]
    code, data = execute_python(root, 'tools/gen_patch_wikitext_register.py',
                                args + provenance['generator_flags'], output)
    assert code == 0 and data == (root / (SOURCES + '2.4.2-wikitext-register.json')).read_bytes(), 'own register replay'
    output = root / (SOURCES + '2.4.2-api-changes.txt')
    original = output.read_bytes()
    code, data = execute_python(root, 'tools/extract_patch_non_inventory.py',
                                ['--patch', '2.4.2', '--text-only'] + provenance['extractor_flags'], output)
    assert code == 0 and data == original, 'own extract replay'


def default_output(generator, extractor, raw):
    try:
        buckets, entries, counts = generator.split_sections(raw), [], []
        for section in generator.SECTIONS.values():
            a, b = generator.parse_section('cvars' if section == 'commands' else section,
                                           buckets.get(section, []))
            entries.extend(a)
            counts.extend(b)
        inventory = ('ok', entries, counts)
    except (ValueError, KeyError, IndexError) as error:
        inventory = (type(error).__name__, str(error))
    try:
        text = ('ok', extractor.extract_text(raw))
    except (ValueError, KeyError, IndexError) as error:
        text = (type(error).__name__, str(error))
    return inventory, text


def verify_parser_preservation(root, manifest):
    old = [load_module(root, 'baseline/' + name, 'baseline_' + name) for name in (
        'gen_patch_wikitext_register.py', 'extract_patch_non_inventory.py')]
    new = [load_module(root, 'tools/' + name, 'current_' + name) for name in (
        'gen_patch_wikitext_register.py', 'extract_patch_non_inventory.py')]
    for path in manifest['default_sources']:
        raw = (root / path).read_text()
        assert default_output(*old, raw) == default_output(*new, raw), 'default changed: ' + path
    results = []
    for relative in manifest['recorded_provenances']:
        patch = Path(relative).name.removesuffix('-api-changes.provenance.json')
        provenance = read_json(root / relative)
        raw = SOURCES + patch + '-api-changes.wikitext'
        register = root / (SOURCES + patch + '-wikitext-register.json')
        source = read_json(register)['source']
        out = root / 'recorded-register.json'
        args = [patch, raw, str(source['revid']), str(out)] + provenance.get('generator_flags', [])
        before = execute_python(root, 'baseline/gen_patch_wikitext_register.py', args, out)
        after = execute_python(root, 'tools/gen_patch_wikitext_register.py', args, out)
        # Error text's script name may differ; preserve error boundary without false success.
        assert before[0] == after[0], 'recorded register exit changed: ' + patch
        if before[0] == 0:
            assert before[1] == after[1], 'recorded register bytes changed: ' + patch
        text = root / (SOURCES + patch + '-api-changes.txt')
        stored_text = text.read_bytes() if text.exists() else None
        args = ['--patch', patch, '--text-only'] + provenance.get('extractor_flags', [])
        before_text = execute_python(root, 'baseline/extract_patch_non_inventory.py', args, text)
        after_text = execute_python(root, 'tools/extract_patch_non_inventory.py', args, text)
        assert before_text[0] == after_text[0], 'recorded extract exit changed: ' + patch
        if before_text[0] == 0:
            assert before_text[1] == after_text[1], 'recorded extract bytes changed: ' + patch
        results.append(dict(patch=patch, register_exit=after[0], extract_exit=after_text[0],
                            stored_register_equal=after[0] == 0 and after[1] == register.read_bytes(),
                            stored_extract_equal=after_text[0] == 0 and after_text[1] == stored_text))
    return results


def verify_accounting(root, manifest, raw):
    sys.path.insert(0, str(root / 'tools'))
    from patch_2_4_2_accounting import account_source
    register = read_json(root / (SOURCES + '2.4.2-wikitext-register.json'))
    observations = read_json(HERE / 'factory-green-results.json')
    ledger = read_json(HERE / 'historical-page-coverage.json')
    assert ledger == read_json(root / (SOURCES + '2.4.2-page-coverage.json')), 'original ledger equality'
    assert ledger['source_rows'] == account_source(raw, register, observations), 'complete accounting'
    known = set(read_json(HERE / 'historical-known-gaps.json'))
    assert known == {k for k, value in observations.items() if not value['ok']}, 'exact gaps'
    assert known == set(read_json(root / 'tests/data/patch_2_4_2_factory_known_gaps.json'))
    latest = {}
    versions = []
    for path in manifest['later_registers']:
        later = read_json(root / path)
        assert later.get('client_line', 'retail') == 'retail', 'Classic successor exclusion'
        versions.append(later['patch'])
        for entry in later['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    assert versions == ['3.2.0', '3.3.0', '3.3.3', '3.3.5', '4.0.1'], 'bounded actual retail successors'
    for entry in register['entries']:
        newer = latest.get(entry['symbol'])
        absent = bool(newer and newer['direction'] == 'removed')
        expected = observations[entry['id']]['expected']
        assert expected['publication'] == ('absent' if absent else 'published'), 'literal supersession'
        assert expected['superseded_by'] == (newer['id'] if absent else None), 'successor attribution'
    negative = read_json(HERE / 'negative-results.json')
    assert {k for k, value in negative.items() if not value['ok']} == known | {'p242-negative-control'}, 'negative boundary'
    rows = ledger['source_rows']
    return dict(ledger_rows=len(rows), ledger_statuses=dict(Counter(row['status'] for row in rows)),
                raw_rows=sum(row['source_id'].startswith('raw-') for row in rows),
                signature_rows=sum(row['source_id'].startswith('signature-') for row in rows),
                constant_rows=sum(row['source_id'].startswith('constant-') for row in rows),
                contextual_api_rows=sum(row['source_id'].startswith('context-api-') for row in rows),
                publication_rows=len(register['entries']), publication_ok=len(observations) - len(known),
                publication_gaps=len(known), negative_gaps=len(known) + 1,
                native_acceptance='unmeasured', model_scope='existing current namespace amount/separator only')


def verify_receipts():
    receipts = read_json(HERE / 'command-ledger.json')['commands']
    for receipt in receipts:
        log = (HERE / receipt['artifact']).read_text()
        if receipt['exit'] == 0:
            assert 'OK' in log or 'test result: ok.' in log, 'passing receipt: ' + receipt['artifact']
        else:
            assert 'FAILED' in log or 'error:' in log, 'RED receipt: ' + receipt['artifact']
    return len(receipts)


def validate():
    manifest, blobs = verify_seals()
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for relative, data in blobs.items():
            # Runtime source is sealed provenance, never rebuilt or executed by portable replay.
            if relative.startswith('src/') or relative.endswith('.rs'):
                continue
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(data)
        raw, registry_pages = verify_source(root)
        verify_reproduction(root)
        summary = verify_accounting(root, manifest, raw)
        summary['registry_pages'] = registry_pages
        summary['default_source_controls'] = len(manifest['default_sources'])
        summary['recorded_options'] = verify_parser_preservation(root, manifest)
        summary['command_receipts'] = verify_receipts()
        summary['seals'] = len(manifest['files'])
        return summary


if __name__ == '__main__':
    print(json.dumps(validate(), sort_keys=True))
