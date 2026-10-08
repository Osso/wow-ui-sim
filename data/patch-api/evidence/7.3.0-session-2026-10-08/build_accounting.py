"""Account for every table occurrence and retained prose line, with bounded credit."""
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PROSE = {
    4: ('bounded-coverage', ['cached-console-addon'], 'Actual current Blizzard_Console load, frame publication and command-history behavior covered by existing targeted prefork cases; not historical 2017 implementation parity.'),
    5: ('bounded-coverage', ['numeric-soundkit-entry', 'sound-request-model'], 'Concrete SOUNDKIT.IG_INVENTORY_ROTATE_CHARACTER = 861 reaches request state. Legacy sound-name rejection is tested. Complete historical key/name similarity is unspecified: the source supplies no old-name catalog or exhaustive mapping.'),
    6: ('bounded-coverage', ['current-retail-table-publication'], 'Both named namespaces are raw/lookup tables. No member list is given; unspecified artifact-forge/console members and native behavior are not inferred from table publication.'),
    7: ('audit-pending', ['actual-inspector-window-lifecycle'], 'Actual Blizzard_DebugTools inspector focuses concrete root/child tables, navigates backward and closes. Exact /tinspect slash dispatch and dangerous-script consent gating are not proved by calling DisplayTableInspectorWindow directly; no consent shim or vendor override was added.'),
    10: ('bounded-coverage', ['numeric-sound-input-change', 'shared-sound-request-model'], 'Bare C_Sound/global and cached Blizzard alias use one sound-kit request model. IDs 861/839 update state; an old string name errors without replacing the accepted request. Audio fidelity, optional channels/handles and exhaustive numeric validation parity are not claimed.'),
}


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    assert read(HERE / 'p730-model-green.proof.json')['exit'] == 0
    register_path = SOURCES / '7.3.0-wikitext-register.json'
    register = read(register_path)
    results = read(HERE / 'p730-model-green-results.json')
    known = set(read(ROOT / 'tests/data/patch_7_3_0_sweep_known_gaps.json'))
    assert {key for key, row in results.items() if not row['ok']} == known
    dump(HERE / 'patch_7_3_0_publication_sweep-results.json', results)
    rows = [{'source_id': row['id'], 'status': 'audit-pending' if row['id'] in known else 'partial-development-green',
             'capabilities': [] if row['id'] in known else ['current-retail-table-publication'],
             'note': 'Raw and ordinary lookup table publication only; no fabricated callable, member/signature/output/security or complete historical parity claim.'}
            for row in register['entries']]
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text_path = SOURCES / '7.3.0-api-changes.txt'
    text = text_path.read_text()
    raw_lines = (SOURCES / '7.3.0-api-changes.wikitext').read_text().splitlines()
    scout = []
    for seed in extractor.seed_rows(text, '7.3.0'):
        number = int(seed['source_id'].rsplit('-', 1)[1])
        if seed['status'] != 'metadata-only':
            status, caps, note = PROSE[number]
            seed.update(status=status, capabilities=caps, note=note)
        rows.append(seed)
        scout.append({'source_id': seed['source_id'], 'extract_line': number,
                      'wikitext_line': number, 'literal': text.splitlines()[number - 1],
                      'raw_literal': raw_lines[number - 1], 'status': seed['status'],
                      'reason': seed['note']})
    assert len(rows) == len({row['source_id'] for row in rows})
    dump(SOURCES / '7.3.0-page-coverage.json', {
        'schema': 'patch-api-page-coverage/v1', 'patch': '7.3.0',
        'source': str(register_path.relative_to(ROOT)), 'source_sha256': digest(register_path),
        'non_inventory_source': {'path': str(text_path.relative_to(ROOT)), 'sha256': digest(text_path)},
        'proof_policy': 'Complete source occurrence accounting. Bounded current behavior is not native/historical parity. 7.3.2 integration placeholder remains. No retirements or shims.',
        'source_rows': rows,
    })
    dump(HERE / 'p730-gap-review.json', [{'source_id': row['id'], 'reason': 'Publication failure requires a real backing model, not a synthetic namespace.'}
                                         for row in register['entries'] if row['id'] in known])
    dump(HERE / 'p730-extract-scout.json', scout)
    dump(HERE / 'p730-problematic-contracts.json', [
        {'source_id': 'prose-undated-005', 'scope': 'Historical SOUNDKIT key/name catalog', 'reason': PROSE[5][2]},
        {'source_id': 'prose-undated-007', 'scope': '/tinspect command routing and consent', 'reason': PROSE[7][2]},
    ])
    dump(HERE / 'p730-accounting-summary.json', {
        'inventory_rows': len(register['entries']), 'inventory_ok': len(results) - len(known),
        'inventory_gaps': len(known), 'extract_rows': len(scout), 'ledger_rows': len(rows),
        'ledger_statuses': dict(Counter(row['status'] for row in rows)),
        'pending_prose': sum(row['status'] == 'audit-pending' for row in scout),
    })
    print((HERE / 'p730-accounting-summary.json').read_text())


if __name__ == '__main__':
    main()
