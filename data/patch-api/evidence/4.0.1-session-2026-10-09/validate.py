"""Validate only this audit's sealed historical inputs and targeted evidence."""
import argparse
import base64
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import types

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
PREFIX = 'data/patch-api/sources/4.0.1-'


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read_json(path):
    return json.loads(path.read_text())


def sealed_archives(evidence, context):
    for name, expected in context['evidence_sha256'].items():
        path = evidence / name
        assert path.stat().st_size < 5_000_000, 'oversized evidence: ' + name
        assert digest(path.read_bytes()) == expected, 'evidence seal: ' + name
    archives = {}
    for name, expected in context['archives'].items():
        raw = (evidence / name).read_bytes()
        assert len(raw) < 5_000_000, 'oversized archive: ' + name
        assert digest(raw) == expected, 'archive seal: ' + name
        archives[name] = json.loads(gzip.decompress(raw))
    return archives


def decode_inputs(archive):
    return {path: base64.b64decode(row['content'], validate=True)
            for path, row in archive.items()}


def load_pinned_helper(inputs):
    # Existing Git object/tree helper contract; execute its retained version,
    # not a mutable later-audit tool. None of its Git commands are invoked.
    module = types.ModuleType('p401_pinned_tree_helper')
    code = inputs['tools/patch_audit_pin_trees.py']
    exec(compile(code, 'historical/patch_audit_pin_trees.py', 'exec'), module.__dict__)
    return module


def check_historical_scope(context, archive, scope_archive, inputs):
    helper = load_pinned_helper(inputs)
    for oid, rows in scope_archive['trees'].items():
        entries = {name: (mode, kind, blob) for name, mode, kind, blob in rows}
        assert len(entries) == len(rows), 'duplicate scope path'
        assert helper.tree_id(entries) == oid, 'historical scope tree: ' + oid
    for revision, scope in scope_archive['revisions'].items():
        commit = base64.b64decode(scope['commit'], validate=True)
        assert helper.object_id('commit', commit) == scope['commit_id'], revision
        assert scope['commit_id'].startswith(revision), revision
        root = {name: (mode, kind, oid) for name, mode, kind, oid in scope['root']}
        tree = commit.splitlines()[0].decode().removeprefix('tree ')
        assert helper.tree_id(root) == tree, 'historical commit root: ' + revision
        for directory, oid in scope['scope_trees'].items():
            assert root[directory] == ('040000', 'tree', oid), directory
            assert oid in scope_archive['trees'], directory
    source_scope = scope_archive['revisions'][context['source_revision']]
    root = {name: (mode, kind, oid) for name, mode, kind, oid in source_scope['root']}
    for path, raw in inputs.items():
        row = archive[path]
        assert digest(raw) == row['sha256'], 'input SHA: ' + path
        assert helper.object_id('blob', raw) == row['blob_id'], 'input blob: ' + path
        directory, separator, relative = path.partition('/')
        if separator:
            tree = source_scope['scope_trees'][directory]
            rows = {name: (mode, kind, oid) for name, mode, kind, oid
                    in scope_archive['trees'][tree]}
            assert rows[relative][2] == row['blob_id'], 'scope membership: ' + path
        else:
            assert root[path][2] == row['blob_id'], 'scope membership: ' + path


def check_source(evidence, context, inputs):
    provenance = json.loads(inputs[PREFIX + 'api-changes.provenance.json'])
    pin = read_json(evidence / 'source-pin.json')
    response = read_json(evidence / 'source-response.json')
    page = response['query']['pages'][str(context['pageid'])]
    fetched = page['revisions'][0]
    raw = inputs[PREFIX + 'api-changes.wikitext']
    assert page['pageid'] == pin['pageid'] == provenance['pageid'] == context['pageid']
    assert page['title'] == pin['title'] == provenance['title'] == 'Patch 4.0.1/API changes'
    assert fetched['revid'] == pin['revid'] == provenance['revid'] == context['source_revid']
    assert fetched['timestamp'] == pin['timestamp'] == provenance['timestamp'] == '2012-09-06T23:25:23Z'
    assert fetched['slots']['main']['*'].encode() == raw, 'response source bytes'
    assert digest(raw) == pin['wikitext_sha256'] == provenance['sha256']
    assert digest((evidence / 'source-response.json').read_bytes()) == pin['response_sha256']
    assert (ROOT / (PREFIX + 'api-changes.wikitext')).read_bytes() == raw, 'pinned working source drift'
    assert provenance['http_status'] == 200  # Supplied receipt/requirement; no new HTTP request.
    register = json.loads(inputs[PREFIX + 'wikitext-register.json'])
    assert register['source']['sha256'] == digest(raw)
    assert register['source']['revid'] == context['source_revid']
    assert register['client_line'] == provenance['client_line'] == 'retail'
    labeled = [(number, direction, name) for number, line in enumerate(raw.decode().splitlines(), 1)
               if (match := re.fullmatch(r': (NEW|REMOVED) \{\{api\|(?:t=e\|)?([^{}|]+)\}\}', line))
               for direction, name in [(match[1], match[2])]]
    inventory = [(row['wikitext_line'], 'NEW' if row['direction'] == 'added' else 'REMOVED', row['symbol'])
                 for row in register['entries'] if row['direction'] != 'changed']
    assert inventory == labeled, 'whole labeled source inventory'
    return provenance, register


def check_reproduction(context, inputs, provenance):
    with tempfile.TemporaryDirectory(prefix='p401-reproduce-', dir=ROOT / 'target') as directory:
        snapshot = Path(directory)
        for path, raw in inputs.items():
            destination = snapshot / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(raw)
        command = [sys.executable, '-B', str(snapshot / 'tools/gen_patch_wikitext_register.py'),
                   '4.0.1', str(snapshot / (PREFIX + 'api-changes.wikitext')),
                   str(context['source_revid']), str(snapshot / 'generated.json'),
                   *provenance['generator_flags']]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        assert result.returncode == 0, result.stderr
        assert (snapshot / 'generated.json').read_bytes() == inputs[PREFIX + 'wikitext-register.json'], 'register reproduction'
        command = [sys.executable, '-B', str(snapshot / 'tools/extract_patch_non_inventory.py'),
                   '--patch', '4.0.1', '--text-only', '--check', *provenance['extractor_flags']]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        assert result.returncode == 0, result.stderr


def check_recorded_proofs(evidence, context, scopes):
    for label, proof in context['proofs'].items():
        assert read_json(evidence / (label + '.proof.json')) == proof, 'proof drift: ' + label
        log = (evidence / proof['log']).read_bytes()
        assert digest(log) == proof['log_sha256'], 'proof log: ' + label
        assert proof['revision'] in scopes['revisions'], 'proof revision: ' + label
        assert proof['cwd'] == '/home/osso/.worktrees/wow-ui-sim-p401-source', label
        assert proof['command'][0] in ('python3', 'cargo'), label
        assert proof['scope'], label
    for label in ('parser-green', 'generator-regression', 'extractor-regression'):
        proof = context['proofs'][label]
        log = (evidence / proof['log']).read_text()
        assert proof['exit'] == 0 and re.search(r'Ran [1-9]\d* tests', log) and '\nOK' in log, label
    for label in ('retail-factory-green', 'retail-prefork-green', 'mists-factory-control'):
        proof = context['proofs'][label]
        assert proof['exit'] == 0, label
        log = (evidence / proof['log']).read_text()
        assert re.search(r'test result: ok\. [1-9]\d* passed; 0 failed;', log), label
    for label in ('parser-red-assertions', 'retirement-red', 'publication-default-red', 'publication-negative'):
        proof = context['proofs'][label]
        assert proof['exit'] != 0, label
        assert 'AssertionError' in (evidence / proof['log']).read_text() or 'panicked' in (evidence / proof['log']).read_text(), label
    blocked = context['proofs']['mists-prefork-control']
    assert blocked['exit'] != 0 and 'requires the features: `client-retail`' in (evidence / blocked['log']).read_text()
    headless = context['proofs']['publication-red']
    assert headless['exit'] != 0 and 'could not compile' in (evidence / headless['log']).read_text()


def check_accounting(evidence, context, inputs, register):
    ids = {row['id'] for row in register['entries']}
    assert len(ids) == len(register['entries']), 'unique register IDs'
    red = read_json(evidence / 'publication-default-red-results.json')
    green = read_json(evidence / 'publication-green-results.json')
    negative = read_json(evidence / 'publication-negative-results.json')
    known = set(json.loads(inputs['tests/data/patch_4_0_1_sweep_known_gaps.json']))
    assert set(red) == set(green) == set(negative) == ids, 'complete observation IDs'
    assert {key for key, value in green.items() if not value['ok']} == known, 'known-gap equality'
    retired_ids = {row['id'] for row in register['entries'] if row['symbol'] in context['retired_globals']}
    assert {key for key in ids if red[key] != green[key]} == retired_ids, 'bounded retirement delta'
    assert all(not red[key]['ok'] and green[key]['ok'] for key in retired_ids), 'retirement RED/GREEN'
    fake = read_json(evidence / 'fabricated-register.json')
    fake_ids = {row['id'] for row in fake['entries'] if row['symbol'] == 'P401FabricatedMissingAPI'}
    assert len(fake_ids) == 1 and not fake_ids & known, 'negative fixture'
    assert {key for key, value in negative.items() if not value['ok']} == known | fake_ids, 'negative gap delta'
    ledger = json.loads(inputs[PREFIX + 'page-coverage.json'])
    rows = {row['source_id']: row for row in ledger['source_rows']}
    assert len(rows) == len(ledger['source_rows']), 'unique ledger IDs'
    assert ids <= set(rows), 'complete ledger inventory'
    for key in ids:
        assert bool(rows[key]['capabilities']) == green[key]['ok'], 'ledger credit: ' + key
        assert rows[key]['status'] == ('bounded-coverage' if green[key]['ok'] else 'audit-pending'), key
    assert ledger['source_sha256'] == digest(inputs[PREFIX + 'wikitext-register.json'])
    text = inputs[PREFIX + 'api-changes.txt']
    assert ledger['non_inventory_source']['sha256'] == digest(text)
    prose = [line for line in text.decode().splitlines() if line.startswith('* ')]
    pending = [row for key, row in rows.items() if key not in ids and row['status'] == 'audit-pending']
    signatures = [row for key, row in rows.items() if key.startswith('signature-')]
    assert len(pending) == len(prose) + len(signatures), 'prose/signature limit accounting'
    assert all(not row['capabilities'] for row in pending), 'no native behavior credit'
    assert signatures and signatures[0]['wikitext_line'] == 6
    assert all(row['section'] in ('global-api', 'events') for row in register['entries'])
    scans = read_json(evidence / 'retirement-scans.json')
    absent = {value['expected']['symbol'] for value in green.values() if value['expected']['publication'] == 'absent'}
    assert set(scans['candidates']) == absent, 'complete retirement candidates'
    for name in context['retired_globals']:
        assert scans['qualified_and_bare'][name]['same_pattern']
        assert not any(row['scope'] in ('cached-retail', 'tests') for row in scans['matches'].get(name, [])), name
    own_sweep = inputs['tests/patch_4_0_1_publication_sweep.rs'].decode()
    successor_paths = re.findall(r'include_str!\("\.\./(data/patch-api/sources/[^"\n]+-wikitext-register.json)"\)', own_sweep)[1:]
    successors = [json.loads(inputs[path]) for path in successor_paths]
    assert all(row.get('client_line', 'retail') == 'retail' for row in successors), 'Classic successor leak'
    versions = [tuple(map(int, row['patch'].split('.'))) for row in successors]
    assert versions == sorted(versions) and versions[0] == (5, 0, 1), 'retail successor order'
    positions = [own_sweep.index('PENDING ' + patch) for patch in context['pending_successors']]
    assert positions == sorted(positions) and positions[-1] < own_sweep.index('5.0.1-wikitext-register.json')
    return {'inventory': len(ids), 'source_rows': len(rows), 'publication_gaps': len(known),
            'retired_globals': context['retired_globals'], 'prose_limits': len(prose),
            'signature_limits': len(signatures), 'statuses': dict(Counter(row['status'] for row in rows.values())),
            'successor_registers': len(successors), 'sealed_inputs': len(inputs),
            'sealed_evidence': len(context['evidence_sha256']), 'historical_only': True}


def validate(evidence):
    context = read_json(evidence / 'historical-context.json')
    assert digest(Path(__file__).read_bytes()) == context['validator_sha256'], 'historical validator seal'
    archives = sealed_archives(evidence, context)
    archive = archives['historical-inputs.json.gz']
    inputs = decode_inputs(archive)
    scopes = archives['historical-scopes.json.gz']
    check_historical_scope(context, archive, scopes, inputs)
    provenance, register = check_source(evidence, context, inputs)
    check_reproduction(context, inputs, provenance)
    check_recorded_proofs(evidence, context, scopes)
    return check_accounting(evidence, context, inputs, register)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path, default=HERE)
    args = parser.parse_args()
    print('PASS: ' + json.dumps(validate(args.evidence), sort_keys=True))
