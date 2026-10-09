#!/usr/bin/env python3
"""Replay sealed 2.1.0 SOURCE history; never read current checkout or Git."""
from functools import lru_cache
import gzip
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import types

HERE = Path(__file__).resolve().parent
MANIFEST_SHA256 = '5cad83b824fe801c48079998bb96e9d142f3eba76e13a7053ef2a9d27ad4f640'
CACHE = 'data/patch-api/source-cache/legacy-2026-10-09/'
SOURCES = 'data/patch-api/sources/'


def digest(content):
    return hashlib.sha256(content).hexdigest()


def read_json(path):
    return json.loads(path.read_bytes())


def check_seals():
    content = (HERE / 'historical-inputs.json').read_bytes()
    assert digest(content) == MANIFEST_SHA256, 'historical manifest seal'
    manifest = json.loads(content)
    for name, expected in manifest['sealed_files'].items():
        content = (HERE / name).read_bytes()
        assert digest(content) == expected['sha256'], 'sealed input: ' + name
        assert len(content) == expected['bytes'] and len(content) < 5_000_000, 'sealed size: ' + name
    bundle = snapshots()
    assert set(bundle) == set(manifest['snapshot_sha256']), 'snapshot set'
    for name, content in bundle.items():
        assert digest(content.encode()) == manifest['snapshot_sha256'][name], 'snapshot: ' + name
        assert len(content.encode()) < 5_000_000, 'oversized snapshot: ' + name
    return manifest


@lru_cache(maxsize=1)
def snapshots():
    return json.loads(gzip.decompress((HERE / 'historical-bundle.json.gz').read_bytes()))


@lru_cache(maxsize=None)
def load_tool(path):
    module = types.ModuleType('p210_' + Path(path).stem)
    module.__file__ = str(HERE / path)
    exec(compile(snapshots()[path], path, 'exec'), module.__dict__)
    return module


def source_identity():
    pin = read_json(HERE / 'source-pin.json')
    identity = ('Patch 2.1.0/API changes', 279373, 6767102, '2026-07-09T22:15:03Z')
    keys = ('title', 'pageid', 'revid', 'timestamp')
    assert tuple(pin[key] for key in keys) == identity, 'literal source identity'
    frozen = json.loads(snapshots()[CACHE + 'manifest.json'])
    assert next(page for page in frozen['pages'] if page['version'] == '2.1.0') == pin, 'frozen manifest pin'
    registry = json.loads(snapshots()[SOURCES + 'api-change-pages-remaining.json'])
    assert len(registry['pages']) == 101 and registry['pages'][-1]['version'] == '1.0.0', 'full registry through 1.0.0'
    registered = next(page for page in registry['pages'] if page['version'] == '2.1.0')
    assert tuple(registered[key] for key in keys) == identity, 'full registry identity'
    response = (HERE / 'source-response.json').read_bytes()
    assert digest(response) == pin['response_sha256'], 'response pin'
    page = json.loads(response)['query']['pages']['279373']
    revision = page['revisions'][0]
    assert (page['title'], page['pageid'], revision['revid'], revision['timestamp']) == identity, 'response identity'
    raw = (HERE / 'source.wikitext').read_bytes()
    assert raw == revision['slots']['main']['*'].encode(), 'exact response raw body'
    assert digest(raw) == pin['wikitext_sha256'] and len(raw) == pin['wikitext_bytes'], 'raw pin'
    provenance = read_json(HERE / 'provenance.json')
    assert provenance == dict(pin, sha256=digest(raw), client_line='retail',
                              generator_flags=['--legacy-retail-profiling-summary', '--client-line', 'retail'],
                              extractor_flags=[]), 'literal recorded flags'
    return raw.decode()


def expected_register(raw):
    generator = load_tool('tools/gen_patch_wikitext_register.py')
    return {'schema': 'patch-api-wikitext-register/v1', 'patch': '2.1.0',
            'source': {'path': SOURCES + '2.1.0-api-changes.wikitext',
                       'revid': 6767102, 'sha256': digest(raw.encode())},
            'header_counts': [], 'entries': generator.parse_legacy_retail_profiling_summary(raw),
            'client_line': 'retail'}


def verify_accounting(register, raw, text, ledger, gaps):
    pin = read_json(HERE / 'source-pin.json')
    assert digest(raw.encode()) == pin['wikitext_sha256'], 'literal raw source'
    expected = expected_register(raw)
    assert register == expected, 'every literal inventory occurrence'
    extractor = load_tool('tools/extract_patch_non_inventory.py')
    assert text == extractor.extract_text(raw), 'default extract'
    accounting = load_tool('tools/build_patch_2_1_0_accounting.py')
    expected_ledger, expected_gaps = accounting.account(expected, raw, text)
    assert ledger == expected_ledger, 'full literal ledger/signature/header/status accounting'
    assert gaps == expected_gaps, 'every original source contract gap'
    assert all(row['capabilities'] == [] for row in [*ledger['source_rows'], *ledger['signature_rows']]), 'invented behavioral credit'
    assert ledger['runtime_observations'] == ledger['native_observations'] == [], 'invented observations'
    return accounting.counts(expected, ledger, gaps)


def generate_bytes(raw, patch, revid, source_path, flags):
    """Run archived CLI entrypoint with recorded flags; only disposable file IO."""
    generator = load_tool('tools/gen_patch_wikitext_register.py')
    with tempfile.TemporaryDirectory(prefix='p210-template-') as directory:
        root = Path(directory)
        source = root / source_path
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text(raw)
        output = root / 'register.json'
        old_argv, old_file = sys.argv, generator.__file__
        try:
            generator.__file__ = str(root / 'tools/gen_patch_wikitext_register.py')
            sys.argv = [generator.__file__, patch, str(source), str(revid), str(output), *flags]
            generator.main()
        finally:
            sys.argv, generator.__file__ = old_argv, old_file
        return output.read_bytes()


def reproduce_defaults_and_templates(raw):
    own = generate_bytes(raw, '2.1.0', 6767102, CACHE + '2.1.0-wikitext.txt', [])
    assert own == (HERE / 'default-2.1.0-register.json').read_bytes(), 'own default bytes'
    bundle = snapshots()
    other = generate_bytes(bundle[CACHE + '3.4.0-wikitext.txt'], '3.4.0', 165668, CACHE + '3.4.0-wikitext.txt', [])
    assert other == (HERE / 'default-3.4.0-register.json').read_bytes(), 'Wrath template default bytes'
    extractor = load_tool('tools/extract_patch_non_inventory.py')
    for patch in ('3.0.3', '3.0.8', '3.1.0'):
        prefix = SOURCES + patch
        provenance = json.loads(bundle[prefix + '-api-changes.provenance.json'])
        source_path = prefix + '-api-changes.wikitext'
        source = bundle[source_path]
        generated = generate_bytes(source, patch, provenance['revid'], source_path, provenance['generator_flags'])
        assert generated == bundle[prefix + '-wikitext-register.json'].encode(), 'recorded register: ' + patch
        flags = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
        assert extractor.extract_text(source, **flags) == bundle[prefix + '-api-changes.txt'], 'recorded extract: ' + patch


def verify_successors(manifest, register):
    boundary = read_json(HERE / 'successor-boundary.json')
    assert boundary['later_registers'] == manifest['later_registers'], 'literal actual successor set'
    names = {row['symbol'] for row in register['entries']}
    overlaps, versions = [], []
    for path in boundary['later_registers']:
        later = json.loads(snapshots()[path])
        version = tuple(map(int, later['patch'].split('.')))
        assert version > (2, 1, 0) and version[:2] not in ((2, 5), (3, 4), (4, 4), (5, 5)), 'Classic successor credit'
        assert later.get('client_line', 'retail') == 'retail', 'nonretail successor credit'
        versions.append(version)
        overlaps.extend({'register': path, 'entry': row} for row in later['entries'] if row['symbol'] in names)
    assert versions == sorted(set(versions)), 'actual successor order/duplicates'
    assert overlaps == boundary['actual_same_symbol_overlaps'], 'exact literal overlaps'
    expected_queued = ['2.2.0', '2.3.0', '2.4.0', '2.4.2', '3.0.2', '3.0.3', '3.0.8']
    assert [row['patch'] for row in boundary['queued_registers']] == expected_queued, 'queued order'
    for row in boundary['queued_registers']:
        assert row == {'patch': row['patch'], 'path': SOURCES + row['patch'] + '-wikitext-register.json',
                       'client_line': 'retail', 'state': 'queued-placeholder-main-adds-ordered',
                       'supersession_credit': False}, 'queued main ownership/no credit'
        assert CACHE + row['patch'] + '-wikitext.txt' in snapshots(), 'queued literal source retained'
    assert boundary['source_only_no_closures'] is True, 'source-only successor limit'
    return {'actual_retail_successors': len(versions), 'same_symbol_overlap_occurrences': len(overlaps),
            'queued_main_owned': len(expected_queued)}


def verify_commands():
    commands = read_json(HERE / 'command-ledger.json')
    assert [row['exit_code'] for row in commands] == [1, 1, 0, 1], 'original development exits'
    for row in commands:
        log = (HERE / row['log']).read_bytes()
        assert digest(log) == row['log_sha256'], 'command log seal: ' + row['log']
        assert row['argv'] and row['revision'] and row['cwd'].endswith('wow-ui-sim-p210-source'), 'command identity'
        assert row['scope'] and row['later_changes_invalidate'], 'proof scope'
    assert b'Ran 3 tests' in (HERE / 'source-green.log').read_bytes() and b'\nOK\n' in (HERE / 'source-green.log').read_bytes(), 'original parser/template GREEN'
    assert b'own historical validator is missing' in (HERE / 'validator-red.log').read_bytes(), 'original copied-replay RED'
    return len(commands)


def main():
    manifest = check_seals()
    raw = source_identity()
    register = read_json(HERE / 'register.json')
    text = (HERE / 'extract.txt').read_text()
    result = verify_accounting(register, raw, text, read_json(HERE / 'historical-page-coverage.json'),
                               read_json(HERE / 'historical-known-gaps.json'))
    assert result == read_json(HERE / 'source-counts.json'), 'derived source counts'
    serialized = (json.dumps(expected_register(raw), indent=2, ensure_ascii=False) + '\n').encode()
    assert serialized == (HERE / 'register.json').read_bytes(), 'exact opt-in register bytes'
    reproduce_defaults_and_templates(raw)
    result.update(verify_successors(manifest, register))
    result.update({'sealed_files': len(manifest['sealed_files']), 'snapshots': len(snapshots()),
                   'command_receipts': verify_commands(), 'proof_boundary': 'SOURCE-only'})
    print(json.dumps(result, sort_keys=True))


if __name__ == '__main__':
    main()
