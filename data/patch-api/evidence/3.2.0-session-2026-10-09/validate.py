#!/usr/bin/env python3
"""Replay original 3.2.0 development evidence; never read current checkout inputs."""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import types

MANIFEST_SHA256 = '376ad0f879b991d68e0dc29541bb15050e9841935d2684da5195d8388e80a4c8'
SOURCES = 'data/patch-api/sources/'


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def digest(content):
    return hashlib.sha256(content).hexdigest()


def load_json(path):
    return json.loads(path.read_text())


def verify_seals(here):
    raw = (here / 'historical-inputs.json').read_bytes()
    require(digest(raw) == MANIFEST_SHA256, 'historical manifest seal')
    manifest = json.loads(raw)
    require(manifest['patch'] == '3.2.0' and manifest['client_line'] == 'retail',
            'historical retail identity')
    for relative, seal in manifest['external'].items():
        content = (here / relative).read_bytes()
        require(digest(content) == seal['sha256'], 'sealed input: ' + relative)
        require(len(content) == seal['bytes'], 'sealed size: ' + relative)
    compressed = (here / 'historical-blobs.json.gz').read_bytes()
    require(digest(compressed) == manifest['archive_sha256'], 'historical archive seal')
    archive = json.loads(gzip.decompress(compressed))
    require(set(archive) == set(manifest['archived']), 'archived path accounting')
    for path, content in archive.items():
        encoded = content.encode()
        seal = manifest['archived'][path]
        require(digest(encoded) == seal['sha256'], 'archived input: ' + path)
        require(len(encoded) == seal['bytes'], 'archived size: ' + path)
    return manifest, archive


def snapshot_module(here, archive, path):
    module = types.ModuleType('p320_' + Path(path).stem)
    module.__file__ = str(here / 'snapshot' / path)
    exec(compile(archive[path], path, 'exec'), module.__dict__)
    return module


def reproduce_source(here, archive):
    pin = load_json(here / 'source-pin.json')
    response_bytes = (here / 'source-response.json').read_bytes()
    require(digest(response_bytes) == pin['response_sha256'], 'response digest')
    page = json.loads(response_bytes)['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    require(page['pageid'] == pin['pageid'] == 499137 and
            page['title'] == pin['title'] == 'Patch 3.2.0/API changes', 'source identity')
    require(revision['revid'] == pin['revid'] == 4812266 and
            revision['timestamp'] == pin['timestamp'] == '2020-09-04T21:33:02Z',
            'source revision')
    raw = archive[SOURCES + '3.2.0-api-changes.wikitext']
    require(revision['slots']['main']['*'] == raw, 'response/raw content identity')
    require(digest(raw.encode()) == pin['wikitext_sha256'] and
            len(raw.encode()) == pin['wikitext_bytes'], 'source pin')
    provenance = json.loads(archive[SOURCES + '3.2.0-api-changes.provenance.json'])
    require(provenance['generator_flags'] == ['--wrath-retail-change-bullets', '--client-line', 'retail']
            and provenance['extractor_flags'] == ['--numbered-reflist'], 'frozen tool flags')
    require(provenance['sha256'] == pin['wikitext_sha256'] and
            provenance['revid'] == pin['revid'], 'provenance identity')
    generator = snapshot_module(here, archive, 'tools/gen_patch_wikitext_register.py')
    extractor = snapshot_module(here, archive, 'tools/extract_patch_non_inventory.py')
    generated = {
        'schema': 'patch-api-wikitext-register/v1', 'patch': '3.2.0',
        'source': {'path': SOURCES + '3.2.0-api-changes.wikitext',
                   'revid': pin['revid'], 'sha256': pin['wikitext_sha256']},
        'header_counts': [], 'entries': generator.parse_wrath_retail_change_bullets(raw),
        'client_line': 'retail',
    }
    require(json.dumps(generated, indent=2, ensure_ascii=False) + '\n' ==
            archive[SOURCES + '3.2.0-wikitext-register.json'], 'register reproduction')
    text = extractor.extract_text(raw, numbered_reflist=True)
    require(text == archive[SOURCES + '3.2.0-api-changes.txt'], 'full extract reproduction')
    # Bounded actual prior-output control; no unchanged-all-pages claim.
    prior = json.loads(archive[SOURCES + '4.1.0-wikitext-register.json'])
    prior_raw = archive[SOURCES + '4.1.0-api-changes.wikitext']
    require(generator.parse_cataclysm_change_bullets(prior_raw) == prior['entries'],
            'prior 4.1.0 register control')
    require(extractor.extract_text(prior_raw) == archive[SOURCES + '4.1.0-api-changes.txt'],
            'prior 4.1.0 extract control')
    return generated, extractor.seed_rows(text, '3.2.0'), raw


def derive_literal_inventory(raw):
    occurrences, signatures, commands = [], [], []
    for number, line in enumerate(raw.splitlines(), 1):
        if not re.match(r"^\*\s+''(?:new|updated|removed|undocumented)''", line):
            if 'UI developers' in line:
                commands.extend({'source_id': f'command-{name}-{number}',
                                 'symbol': name, 'literal': name, 'wikitext_line': number}
                                for name in re.findall(r'/(?:dump|eventtrace|framestack|reload)\b', line))
            continue
        api = re.search(r'\{\{api\|(?:t=(e)\|)?([^{}|]+)\}\}', line)
        array = re.search(r'"([A-Za-z_][A-Za-z0-9_]*)"\s*array\b', line)
        require(api is not None or array is not None, 'unaccounted labelled source line')
        literal = api[2] if api else array[1]
        symbol = literal.split('(', 1)[0]
        section = 'events' if api and api[1] else 'global-api'
        source_id = f'wt-{section}-{symbol}-{number}'
        occurrences.append({'source_id': source_id, 'symbol': symbol, 'wikitext_line': number})
        if '(' in literal:
            signatures.append({'source_id': f'signature-{symbol}-{number}', 'symbol': symbol,
                               'literal': literal, 'wikitext_line': number, 'inventory_id': source_id})
    return {'api_event_array_occurrences': occurrences,
            'slash_command_occurrences': commands, 'call_signatures': signatures}


def verify_results(here, archive, register):
    pins = load_json(here / 'later-register-pins.json')
    require(pins['pending'] == ['3.3.0', '3.3.3', '3.3.5', '4.0.1'], 'queued retail order')
    latest = {}
    versions = []
    for pin in pins['pins']:
        content = archive[pin['path']]
        require(digest(content.encode()) == pin['sha256'], 'later register pin')
        later = json.loads(content)
        require(later.get('client_line', 'retail') == 'retail', 'Classic successor credit')
        versions.append(tuple(int(value) for value in later['patch'].split('.')))
        for row in later['entries']:
            if row['direction'] != 'changed':
                latest[row['symbol']] = row
    require(versions == sorted(set(versions)) and all(v >= (4, 1, 0) for v in versions),
            'actual retail successor order')
    observed = load_json(here / 'own-sweep-green-results.json')
    red = load_json(here / 'own-sweep-red-results.json')
    known = load_json(here / 'historical-known-gaps.json')
    entries = {row['id']: row for row in register['entries']}
    require(len(entries) == len(register['entries']), 'duplicate inventory IDs')
    require(set(observed) == set(red) == set(entries), 'publication result accounting')
    gaps = {key for key, result in observed.items() if not result['ok']}
    require(len(known) == len(set(known)) and gaps == set(known), 'historical known gaps')
    require({key for key, result in red.items() if not result['ok']} == gaps, 'RED gap boundary')
    for key, row in entries.items():
        newer = latest.get(row['symbol'])
        own_removed = row['direction'] == 'removed'
        removed = newer['direction'] == 'removed' if newer else own_removed
        expectation = observed[key]['expected']
        require(expectation['symbol'] == row['symbol'] and
                expectation['section'] == row['section'] and
                expectation['direction'] == row['direction'] and
                expectation['publication'] == ('absent' if removed else 'published') and
                expectation['superseded_by'] == (newer['id'] if newer and removed != own_removed else None),
                'historical publication expectation: ' + key)
    negative = load_json(here / 'negative-results.json')
    negative_register = load_json(here / 'negative-register.json')
    negative_ids = {row['id'] for row in negative_register['entries']}
    require(set(negative) == negative_ids and len(negative) == len(observed), 'negative inventory boundary')
    negative_gaps = {key for key, result in negative.items() if not result['ok']}
    require(negative_gaps == gaps | {'p320-negative-control'}, 'negative gap boundary')
    for key in set(negative) & set(observed):
        require(negative[key] == observed[key], 'negative unrelated observation drift: ' + key)
    return observed, gaps, negative_gaps, len(versions)


def verify_accounting(here, archive, register, extract_rows, raw):
    literal = derive_literal_inventory(raw)
    require(literal == load_json(here / 'literal-inventory.json'), 'complete literal inventory')
    coverage = load_json(here / 'historical-page-coverage.json')
    require(coverage['source_sha256'] == digest(archive[SOURCES + '3.2.0-wikitext-register.json'].encode()),
            'ledger register digest')
    rows = coverage['source_rows']
    by_id = {row['source_id']: row for row in rows}
    inventory = {row['id'] for row in register['entries']}
    extract = {row['source_id'] for row in extract_rows}
    signatures = {row['source_id'] for row in literal['call_signatures']}
    commands = {row['source_id'] for row in literal['slash_command_occurrences']}
    require(len(by_id) == len(rows) and set(by_id) == inventory | extract | signatures | commands,
            'complete ledger accounting')
    observed, gaps, negative_gaps, later_count = verify_results(here, archive, register)
    for key, row in by_id.items():
        require(row['note'] and row['status'] in ('audit-pending', 'metadata-only', 'bounded-coverage'),
                'missing disposition: ' + key)
        if key in inventory:
            require(row['status'] == ('bounded-coverage' if observed[key]['ok'] else 'audit-pending') and
                    row['capabilities'] == (['publication-absence'] if observed[key]['ok'] else []),
                    'publication-only credit: ' + key)
        else:
            require(not row['capabilities'] and row['status'] != 'bounded-coverage',
                    'unsupported behavioral credit: ' + key)
    require(all(by_id[key]['status'] == 'audit-pending' for key in signatures | commands),
            'signature/command parity overclaim')
    return {'publication_rows': len(inventory), 'publication_ok': len(inventory) - len(gaps),
            'publication_gaps': len(gaps), 'extract_rows': len(extract),
            'extract_pending': sum(by_id[key]['status'] == 'audit-pending' for key in extract),
            'signature_rows': len(signatures), 'slash_command_rows': len(commands),
            'ledger_rows': len(rows), 'ledger_statuses': dict(Counter(row['status'] for row in rows)),
            'negative_gaps': len(negative_gaps), 'later_retail_registers': later_count}


def verify_commands(here):
    commands = load_json(here / 'command-ledger.json')['commands']
    summaries = {}
    for command in commands:
        text = (here / command['artifact']).read_text()
        require(command['revision'] and command['scope'], 'unscoped command')
        if command['exit'] == 0 and command['argv'][:2] == ['cargo', 'fmt']:
            require(not text.strip(), 'formatter receipt')
            summaries[command['artifact']] = {'exit': 0}
            continue
        if command['exit'] == 0:
            rust = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed', text)
            python = re.search(r'Ran (\d+) test.*?\n\nOK\b', text, re.S)
            require((rust and all(int(failed) == 0 for _, failed in rust)) or python,
                    'successful test receipt: ' + command['artifact'])
            passed = sum(int(count) for count, _ in rust) if rust else int(python[1])
            summaries[command['artifact']] = {'exit': 0, 'passed': passed}
        else:
            require('FAILED' in text or 'error:' in text, 'failure receipt: ' + command['artifact'])
            summaries[command['artifact']] = {'exit': command['exit']}
    return summaries


def validate(here):
    manifest, archive = verify_seals(here)
    register, extract_rows, raw = reproduce_source(here, archive)
    summary = verify_accounting(here, archive, register, extract_rows, raw)
    summary['commands'] = verify_commands(here)
    summary['archived_files'] = len(archive)
    summary['sealed_external_files'] = len(manifest['external'])
    return summary


if __name__ == '__main__':
    print(json.dumps(validate(Path(__file__).resolve().parent), sort_keys=True))
