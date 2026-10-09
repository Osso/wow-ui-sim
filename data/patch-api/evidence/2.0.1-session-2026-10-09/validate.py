#!/usr/bin/env python3
"""Frozen SOURCE-only replay. All inputs come from this evidence directory.

No Git/target/runtime/current source files or network; seals are local integrity
controls, not signatures supplied by an external trusted authority.
"""
import ast
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

MANIFEST_SHA256 = '804da368022a4c27435e9d03d41321d8fb30b946f825cc3a9c028f3125560d41'
CACHE = 'data/patch-api/source-cache/legacy-2026-10-09/'
SOURCES = 'data/patch-api/sources/'
HERE = Path(__file__).resolve().parent


def digest(value):
    return hashlib.sha256(value).hexdigest()


def load_inputs(here):
    content = (here / 'historical-inputs.json').read_bytes()
    assert digest(content) == MANIFEST_SHA256, 'historical manifest seal'
    manifest = json.loads(content)
    for name, seal in manifest['sealed_files'].items():
        content = (here / name).read_bytes()
        assert digest(content) == seal['sha256'], 'sealed input: ' + name
        assert len(content) == seal['bytes'] < 5_000_000, 'sealed input size: ' + name
    bundle = json.loads(gzip.decompress((here / 'historical-bundle.json.gz').read_bytes()))
    assert set(bundle) == set(manifest['snapshot_sha256']), 'snapshot set'
    for name, value in bundle.items():
        assert digest(value.encode()) == manifest['snapshot_sha256'][name], 'snapshot seal: ' + name
    return manifest, bundle


def load_tool(bundle, path):
    module = types.ModuleType('p201_' + Path(path).stem)
    module.__file__ = str(HERE / path)
    exec(compile(bundle[path], path, 'exec'), module.__dict__)
    return module


def build_gaps(expected):
    gaps = []
    for group, contract in [('source_rows', 'literal-prose'), ('occurrences', 'literal-mention'),
                            ('signature_rows', 'signature-fragment'),
                            ('references', 'unexpanded-reference')]:
        for row in expected[group]:
            if row['status'] == 'UNPROVEN':
                gaps.append({'id': group + '-' + row['id'], 'source_id': row['id'],
                             'line': row['line'], 'contract': contract,
                             'reason': 'UNPROVEN: ' + row['limit'], 'capabilities': []})
    return gaps


def verify_accounting(ledger, gaps, expected):
    assert ledger == expected, 'complete literal ledger / signatures / headers / proof limits'
    assert gaps == build_gaps(expected), 'every literal UNPROVEN contract required'
    for group in ['source_rows', 'occurrences', 'signature_rows', 'headers', 'references']:
        assert len({row['id'] for row in ledger[group]}) == len(ledger[group]), 'duplicate ID: ' + group
    assert all(not row['capabilities'] for row in ledger['source_rows']), 'invented credit'
    return ledger['counts']


def verify_pin(here, bundle):
    pin = json.loads((here / 'source-pin.json').read_bytes())
    identity = ('Patch 2.0.1/API changes', 324401, 3129557, '2019-06-15T12:04:30Z')
    keys = ('title', 'pageid', 'revid', 'timestamp')
    assert tuple(pin[key] for key in keys) == identity, 'literal source identity'
    manifest = json.loads(bundle[CACHE + 'manifest.json'])
    assert next(p for p in manifest['pages'] if p['version'] == '2.0.1') == pin, 'manifest pin'
    registry_raw = bundle[manifest['registry_path']].encode()
    assert digest(registry_raw) == manifest['registry_sha256'], 'full registry hash'
    registry = json.loads(registry_raw)
    assert len(registry['pages']) == 101 and registry['pages'][-1]['version'] == '1.0.0', 'registry cutoff'
    registered = next(p for p in registry['pages'] if p['version'] == '2.0.1')
    assert tuple(registered[key] for key in keys) == identity, 'full registry source identity'
    response = (here / 'source-response.json').read_bytes()
    assert digest(response) == pin['response_sha256'], 'response digest'
    page = json.loads(response)['query']['pages']['324401']
    revision, = page['revisions']
    assert (page['title'], page['pageid'], revision['revid'], revision['timestamp']) == identity, 'response identity'
    raw = (here / 'source.wikitext').read_bytes()
    assert raw == revision['slots']['main']['*'].encode(), 'exact response raw body'
    assert len(raw) == pin['wikitext_bytes'] and digest(raw) == pin['wikitext_sha256'], 'raw digest/size'
    return pin, raw


def verify_successors(here, bundle, manifest, ledger):
    boundary = json.loads((here / 'successor-boundary.json').read_bytes())
    actual = boundary['actual_later_retail_registers']
    assert actual == manifest['actual_later_retail_registers'], 'actual successor set'
    queued = ['2.1.0', '2.2.0', '2.3.0', '2.4.0', '2.4.2', '3.0.2', '3.0.3', '3.0.8']
    assert boundary['queued_main_owned'] == [
        {'patch': patch, 'client_line': 'historical-retail',
         'state': 'main-owned-not-used-for-supersession', 'supersession_credit': False}
        for patch in queued], 'queued history/ownership'
    names = {row['symbol'] for row in ledger['occurrences']
             if row['kind'] not in ['context-identifier', 'language-example']}
    overlaps, patches = [], []
    for path in actual:
        register = json.loads(bundle[path])
        patch = register['patch']
        assert patch not in queued and not patch.startswith(('2.5.', '3.4.', '5.5.')), 'queued/Classic credit'
        assert register.get('client_line', 'retail') == 'retail', 'foreign client history'
        version = tuple(map(int, patch.split('.')))
        assert version > (2, 0, 1), 'non-successor input'
        patches.append(version)
        overlaps.extend({'path': path, 'entry': row} for row in register['entries'] if row['symbol'] in names)
    assert patches == sorted(set(patches)), 'successor order/multiplicity'
    assert overlaps == boundary['actual_same_symbol_mentions'], 'same-name mentions'
    assert boundary['supersession_credit'] == 0, 'invented retirement/model credit'
    return len(actual), len(overlaps)


def normalize_log(value, root):
    value = value.replace(str(root), '<ROOT>')
    return re.sub(r'Ran (\d+) tests in [0-9.]+s', r'Ran \1 tests in <duration>s', value)


def verify_original_commands(here, bundle, root):
    receipts = json.loads((here / 'original-command-ledger.json').read_bytes())
    for receipt in receipts:
        if not receipt.get('scope_snapshots'):
            continue
        revision_root = root / 'revisions' / receipt['revision']
        for key in receipt['scope_snapshots']:
            relative = key.split('/', 2)[2]
            destination = revision_root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(bundle[key])
        source = revision_root / CACHE / '2.0.1-wikitext.txt'
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_bytes((here / 'source.wikitext').read_bytes())
        result = subprocess.run([sys.executable, '-B', *receipt['command'][2:]],
                                cwd=revision_root, capture_output=True, text=True,
                                env=dict(os.environ, PATH='/p201-no-git', PYTHONPATH=''))
        assert result.returncode == receipt['exit_code'], 'historical command exit: ' + receipt['log']
        expression = ast.parse((here / receipt['log']).read_text(), mode='eval').body
        logged = {keyword.arg: ast.literal_eval(keyword.value) for keyword in expression.keywords}
        assert normalize_log(result.stderr, revision_root) == normalize_log(
            logged['stderr'], receipt['cwd']), 'historical command result: ' + receipt['log']
        assert result.stdout == logged['stdout'], 'historical stdout: ' + receipt['log']
    return sum(bool(receipt.get('scope_snapshots')) for receipt in receipts)


def verify_recorded_outputs(here, bundle, root, pin):
    for path in ['tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py']:
        dest = root / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(bundle[path])
    raw_path = root / pin['wikitext_path']
    raw_path.parent.mkdir(parents=True, exist_ok=True)
    raw_path.write_bytes((here / 'source.wikitext').read_bytes())
    cases = [('2.0.1', raw_path, pin['revid'], [], (here / 'default-register.json').read_bytes())]
    for patch in ['3.0.2', '3.0.3', '3.0.8']:
        source = root / SOURCES / (patch + '-api-changes.wikitext')
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text(bundle[SOURCES + patch + '-api-changes.wikitext'])
        provenance = json.loads(bundle[SOURCES + patch + '-api-changes.provenance.json'])
        revid = provenance.get('revid') or provenance['source']['revid']
        cases.append((patch, source, revid, provenance['generator_flags'],
                      bundle[SOURCES + patch + '-wikitext-register.json'].encode()))
        flags = {flag.removeprefix('--').replace('-', '_'): True
                 for flag in provenance['extractor_flags'] if flag != '--text-only'}
        extractor = load_tool(bundle, 'tools/extract_patch_non_inventory.py')
        assert extractor.extract_text(source.read_text(), **flags).encode() == bundle[
            SOURCES + patch + '-api-changes.txt'].encode(), 'recorded extract: ' + patch
    for patch, source, revid, flags, expected in cases:
        output = root / (patch + '-output.json')
        result = subprocess.run([
            sys.executable, '-I', '-B', str(root / 'tools/gen_patch_wikitext_register.py'),
            patch, str(source), str(revid), str(output), *flags,
        ], cwd=root, capture_output=True, text=True,
            env=dict(os.environ, PATH='/p201-no-git', PYTHONPATH=''))
        assert result.returncode == 0, 'recorded generator: ' + patch + result.stderr
        assert output.read_bytes() == expected, 'recorded register bytes: ' + patch
    return len(cases)


def replay(here):
    manifest, bundle = load_inputs(here)
    pin, raw = verify_pin(here, bundle)
    auditor = load_tool(bundle, 'tools/audit_patch_2_0_1_source.py')
    expected = auditor.account(raw.decode())
    ledger = json.loads((here / 'historical-page-coverage.json').read_bytes())
    gaps = json.loads((here / 'historical-known-gaps.json').read_bytes())
    counts = dict(verify_accounting(ledger, gaps, expected))
    serialized = (json.dumps(expected, indent=2, ensure_ascii=False) + '\n').encode()
    assert serialized == (here / 'historical-page-coverage.json').read_bytes(), 'exact serialized ledger'
    extractor = load_tool(bundle, 'tools/extract_patch_non_inventory.py')
    assert extractor.extract_text(raw.decode()).encode() == (here / 'extract.txt').read_bytes(), 'default extract'
    counts['actual_successors'], counts['same_symbol_mentions'] = verify_successors(here, bundle, manifest, ledger)
    with tempfile.TemporaryDirectory(prefix='p201-archived-tools-') as directory:
        root = Path(directory)
        counts['recorded_register_replays'] = verify_recorded_outputs(here, bundle, root, pin)
        counts['original_command_replays'] = verify_original_commands(here, bundle, root)
        assert not (root / '.git').exists() and not (root / 'target').exists(), 'Git/target dependency'
    counts['sealed_files'] = len(manifest['sealed_files'])
    counts['snapshots'] = len(bundle)
    counts['gap_records'] = len(gaps)
    return counts


if __name__ == '__main__':
    print(json.dumps(replay(HERE), sort_keys=True))
