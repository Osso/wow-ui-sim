#!/usr/bin/env python3
"""Sealed historical source-only replay. No Git, target, native or current-file credit."""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import types

HERE = Path(__file__).resolve().parent
MANIFEST_SHA256 = 'e2ff038493ccdbe9ba04ec35c301c82aaee6a127a1ec905fa237fb6ce22cc525'
SOURCES = 'data/patch-api/sources/'
CACHE = 'data/patch-api/source-cache/legacy-2026-10-09/'


def digest(content):
    return hashlib.sha256(content).hexdigest()


def read_json(path):
    return json.loads(path.read_bytes())


def verify_seals(here):
    content = (here / 'historical-inputs.json').read_bytes()
    assert digest(content) == MANIFEST_SHA256, 'historical manifest seal'
    manifest = json.loads(content)
    for name, seal in manifest['sealed_files'].items():
        assert Path(name).name == name, 'historical input outside evidence'
        content = (here / name).read_bytes()
        assert digest(content) == seal['sha256'], 'sealed input: ' + name
        assert len(content) == seal['bytes'], 'sealed size: ' + name
        assert len(content) <= 5_000_000, 'oversized historical input: ' + name
    bundle = json.loads(gzip.decompress((here / 'historical-bundle.json.gz').read_bytes()))
    assert set(bundle) == set(manifest['snapshot_sha256']), 'snapshot set'
    for name, content in bundle.items():
        assert digest(content.encode()) == manifest['snapshot_sha256'][name], 'snapshot: ' + name
    return manifest, bundle


def load_module(bundle, path):
    module = types.ModuleType('p303_' + Path(path).stem)
    module.__file__ = str(HERE / path)
    exec(compile(bundle[path], path, 'exec'), module.__dict__)
    return module


def reproduce_source(here, bundle):
    pin = read_json(here / 'source-pin.json')
    identity = ('Patch 3.0.3/API changes', 560990, 5407654, '2010-03-28T14:26:47Z')
    keys = ('title', 'pageid', 'revid', 'timestamp')
    assert tuple(pin[key] for key in keys) == identity, 'literal source identity'
    frozen = json.loads(bundle[CACHE + 'manifest.json'])
    assert next(p for p in frozen['pages'] if p['version'] == '3.0.3') == pin, 'frozen manifest pin'
    registry = json.loads(bundle[SOURCES + 'api-change-pages-remaining.json'])
    assert registry['pages'][-1]['version'] == '1.0.0', 'full handoff registry cutoff'
    registered = next(p for p in registry['pages'] if p['version'] == '3.0.3')
    assert tuple(registered[key] for key in keys) == identity, 'full registry pin'
    response = (here / 'source-response.json').read_bytes()
    assert digest(response) == pin['response_sha256'], 'response pin'
    page = json.loads(response)['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert (page['title'], page['pageid'], revision['revid'], revision['timestamp']) == identity, 'response identity'
    raw = (here / 'source.wikitext').read_bytes()
    assert raw == revision['slots']['main']['*'].encode(), 'exact response body'
    assert digest(raw) == pin['wikitext_sha256'] and len(raw) == pin['wikitext_bytes'], 'wikitext pin'
    provenance = read_json(here / 'provenance.json')
    assert provenance == dict(pin, sha256=digest(raw), client_line='retail',
                              generator_flags=['--legacy-cvar-definitions', '--client-line', 'retail'],
                              extractor_flags=[]), 'literal provenance flags'
    generator = load_module(bundle, 'tools/gen_patch_wikitext_register.py')
    generated = {
        'schema': 'patch-api-wikitext-register/v1', 'patch': '3.0.3',
        'source': {'path': SOURCES + '3.0.3-api-changes.wikitext',
                   'revid': pin['revid'], 'sha256': digest(raw)},
        'header_counts': [], 'entries': generator.parse_legacy_cvar_definitions(raw.decode()),
        'client_line': 'retail',
    }
    serialized = (json.dumps(generated, indent=2, ensure_ascii=False) + '\n').encode()
    assert serialized == (here / 'register.json').read_bytes(), 'opt-in register reproduction'
    default = dict(generated, source=dict(generated['source'], path=pin['wikitext_path']), entries=[])
    del default['client_line']
    assert (json.dumps(default, indent=2, ensure_ascii=False) + '\n').encode() == (here / 'default-register.json').read_bytes(), 'default recorded bytes'
    extractor = load_module(bundle, 'tools/extract_patch_non_inventory.py')
    text = extractor.extract_text(raw.decode())
    assert text.encode() == (here / 'extract.txt').read_bytes(), 'default extract reproduction'
    return generated, raw.decode(), text


def verify_accounting(register, raw, text, ledger, gaps):
    definitions = [(number, match[1], match[2], line)
                   for number, line in enumerate(raw.splitlines(), 1)
                   if (match := re.fullmatch(r';\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(\S.*)', line))]
    entries = register['entries']
    assert len(entries) == len(definitions), 'every literal definition inventoried'
    assert register['client_line'] == 'retail' and register['header_counts'] == [], 'invented count/client'
    for entry, (number, name, description, literal) in zip(entries, definitions):
        assert (entry['id'], entry['symbol'], entry['section'], entry['direction'], entry['wikitext_line']) == (
            f'wt-cvars-{name}-{number}', name, 'cvars', 'added', number), 'literal inventory identity'
        assert entry['annotation'] == literal and entry['description'] == description, 'definition prose'
        assert not any(key in entry for key in ('signature', 'returns', 'page_default')), 'fabricated signature/default'
    inventory = {entry['id']: entry for entry in entries}
    raw_rows = {f'raw-3.0.3-{number:03}': (number, line)
                for number, line in enumerate(raw.splitlines(), 1) if line.strip()}
    rows = ledger['source_rows']
    by_id = {row['source_id']: row for row in rows}
    assert len(by_id) == len(rows), 'duplicate ledger ID'
    assert set(by_id) == set(inventory) | set(raw_rows), 'every original source row accounted'
    assert ledger['source_sha256'] == digest(raw.encode()), 'ledger source hash'
    assert ledger['non_inventory_source']['sha256'] == digest(text.encode()), 'ledger extract hash'
    assert ledger['signature_rows'] == [] and ledger['signature_boundary'], 'fabricated callable contract'
    expected_gaps = set()
    for key, entry in inventory.items():
        row = by_id[key]
        assert row['status'] == 'publication-unproven', 'publication overclaim'
        assert row['symbol'] == entry['symbol'] and row['literal'] == entry['annotation'], 'inventory ledger identity'
        assert row['wikitext_line'] == entry['wikitext_line'], 'inventory line'
        expected_gaps.add((key, entry['symbol'], 'publication'))
    definitions_by_line = {number: name for number, name, _, _ in definitions}
    for key, (number, literal) in raw_rows.items():
        row = by_id[key]
        assert (row['wikitext_line'], row['literal'], row['rendered']) == (
            number, literal, text.splitlines()[number - 1]), 'full literal source line'
        if number in definitions_by_line:
            name = definitions_by_line[number]
            assert row['status'] == 'semantic-unproven', 'semantic overclaim'
            assert row['inventory_id'] == f'wt-cvars-{name}-{number}', 'definition cross-reference'
            expected_gaps.add((key, name, 'flag-synchronization'))
        else:
            assert row['status'] == 'metadata-only', 'source/header disposition'
    assert all(row['note'] and row['capabilities'] == [] for row in rows), 'missing disposition or invented credit'
    assert len(gaps) == len(expected_gaps), 'gap multiplicity'
    assert {(gap['source_id'], gap['symbol'], gap['contract']) for gap in gaps} == expected_gaps, 'exact contract gaps'
    assert all(gap['reason'].startswith('UNPROVEN:') for gap in gaps), 'gap reason'
    assert all(by_id[gap['source_id']]['note'] == gap['reason'] for gap in gaps
               if gap['contract'] == 'flag-synchronization'), 'semantic gap reason drift'
    statuses = Counter(row['status'] for row in rows)
    return {
        'inventory_rows': len(inventory), 'raw_nonblank_rows': len(raw_rows),
        'ledger_rows': len(rows), 'ledger_statuses': dict(statuses),
        'signature_rows': len(ledger['signature_rows']), 'contract_gap_records': len(gaps),
        'named_headers': sum(line.startswith('==') for line in raw.splitlines()),
        'semantic_unproven': statuses['semantic-unproven'],
        'publication_unproven': statuses['publication-unproven'], 'meaningful_closures': 0,
    }


def verify_successors(here, bundle, manifest, register):
    boundary = read_json(here / 'successor-boundary.json')
    assert boundary['later_registers'] == manifest['later_registers'], 'actual retail successor set'
    names = {entry['symbol'] for entry in register['entries']}
    overlaps = []
    patches = []
    for path in boundary['later_registers']:
        later = json.loads(bundle[path])
        patch = tuple(map(int, later['patch'].split('.')))
        assert later.get('client_line', 'retail') == 'retail', 'Classic successor credit'
        assert patch >= (3, 3, 3) and not later['patch'].startswith('3.4.'), 'queued/Classic successor credit'
        patches.append(patch)
        overlaps.extend({'register': path, 'entry': entry} for entry in later['entries']
                        if entry['symbol'] in names)
    assert patches == sorted(set(patches)), 'successor order/duplicates'
    assert overlaps == boundary['actual_same_symbol_overlaps'], 'actual literal overlaps'
    queued = boundary['queued_registers']
    assert [entry['patch'] for entry in queued] == ['3.0.8', '3.1.0', '3.2.0', '3.3.0'], 'queued order'
    for entry in queued:
        assert entry['client_line'] == 'retail' and not entry['supersession_credit'], 'queued credit'
        assert entry['state'] == 'queued-placeholder-main-adds-ordered', 'queued ownership'
        assert entry['path'] == SOURCES + entry['patch'] + '-wikitext-register.json', 'queued register path'
        raw = bundle[CACHE + entry['patch'] + '-wikitext.txt']
        mentioned = [name for name in sorted(names) if re.search(r'\b' + re.escape(name) + r'\b', raw)]
        assert mentioned == entry['literal_name_mentions'], 'queued literal overlap'
    return len(patches), len(overlaps), len(queued)


def verify_backing_review(here, bundle, register):
    review = read_json(here / 'backing-state-review.json')
    assert review['meaningful_closures'] == [], 'model closure overclaim'
    names = {entry['symbol'].lower() for entry in register['entries']}
    for path, receipt in review['files'].items():
        text = bundle[path]
        assert digest(text.encode()) == receipt['sha256'], 'backing source boundary'
        hits = [{'line': number, 'text': line} for number, line in enumerate(text.splitlines(), 1)
                if any(name in line.lower() for name in names)]
        assert hits == receipt['literal_name_hits'], 'static name scan'
        assert receipt['scope'], 'static scan proof boundary'


def command_summaries(here):
    summaries = []
    for command in read_json(here / 'command-ledger.json')['commands']:
        log = (here / command['artifact']).read_text()
        assert command['argv'] and command['scope'] and command['revision'], 'command scope'
        count = re.search(r'Ran (\d+) tests?', log)
        assert count, 'missing development test count'
        if command['exit'] == 0:
            assert re.search(r'\nOK\s*$', log), 'missing GREEN receipt'
        else:
            assert 'FAILED (failures=' in log, 'missing RED receipt'
        summaries.append({'revision': command['revision'], 'artifact': command['artifact'],
                          'exit': command['exit'], 'tests': int(count[1])})
    return summaries


def validate(here=HERE):
    manifest, bundle = verify_seals(here)
    register, raw, text = reproduce_source(here, bundle)
    counts = verify_accounting(register, raw, text, read_json(here / 'historical-page-coverage.json'),
                               read_json(here / 'historical-known-gaps.json'))
    actual, overlap, queued = verify_successors(here, bundle, manifest, register)
    verify_backing_review(here, bundle, register)
    counts.update(sealed_inputs=len(manifest['sealed_files']), later_registers=actual,
                  actual_symbol_overlaps=overlap, queued_registers=queued,
                  command_summaries=command_summaries(here))
    return counts


if __name__ == '__main__':
    print(json.dumps(validate(), indent=2, sort_keys=True))
