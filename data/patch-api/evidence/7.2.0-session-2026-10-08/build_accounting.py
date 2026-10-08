"""Build exact inventory/extract accounting from passing bounded proof."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PROSE = {
    4: ('bounded-coverage', ['cached-addon-loading'], 'All three named current-retail addons load through C_AddOns; no historical 2017 addon implementation or complete documentation/contribution/deprecation parity claimed.'),
    5: ('bounded-coverage', ['mask-attachment-lifecycle'], 'CreateMaskTexture returns MaskTexture; duplicate attachment stays one, retrieval returns the same object, removal clears attachment. GPU masking/native 2017 visual parity not claimed.'),
    6: ('audit-pending', ['independent-vertex-offset-storage'], 'SetVertexOffset stores independent concrete corner offsets with readback; complete grep finds only setters/getters/default storage, no renderer consumer. Visual quad deformation remains unimplemented; no shim or vendor override added.'),
    10: ('audit-pending', ['equipment-set-lifecycle'], 'Existing C_EquipmentSet backing state supports create, rename, specialization assignment and delete in cached UI. Page gives no old-member catalog or exact deprecated-wrapper contract; historical Lua migration compatibility cannot be exhaustively established from this source. Current namespace presence is not full migration parity.'),
    14: ('audit-pending', [], 'Voice chat is a domain-level removal summary without any named API member, signature or scope. Current voice systems also appear in later registers; no unspecified member retirement inferred or current consumer disabled.'),
    15: ('audit-pending', [], 'Mac movie recording is a domain-level removal summary without any named API member or platform contract. No macOS recording backing system/native probe is available; no guessed member retirement or fake implementation added.'),
}


def read(path):
    return json.loads(path.read_text())


def dump(path, data):
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    assert read(HERE / 'p720-own-green.proof.json')['exit'] == 0
    register_path = SOURCES / '7.2.0-wikitext-register.json'
    register = read(register_path)
    results = read(HERE / 'p720-own-green-results.json')
    known = set(read(ROOT / 'tests/data/patch_7_2_0_sweep_known_gaps.json'))
    assert {key for key, row in results.items() if not row['ok']} == known
    dump(HERE / 'patch_7_2_0_publication_sweep-results.json', results)
    rows = [{'source_id': entry['id'], 'status': 'audit-pending' if entry['id'] in known else 'bounded-coverage',
             'capabilities': [] if entry['id'] in known else ['current-retail-publication'],
             'note': 'Current-retail publication only, constructed through the appropriate region factory where applicable. Behavioral limits are accounted separately in the prose rows.'}
            for entry in register['entries']]
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text_path = SOURCES / '7.2.0-api-changes.txt'
    text = text_path.read_text()
    raw_lines = (SOURCES / '7.2.0-api-changes.wikitext').read_text().splitlines()
    scout = []
    for seed in extractor.seed_rows(text, '7.2.0'):
        number = int(seed['source_id'].rsplit('-', 1)[1])
        if seed['status'] != 'metadata-only':
            if number == 18:
                seed.update(status='metadata-only', note='Unexpanded reference-list marker; page supplies no inline citations or additional runtime contract.')
            else:
                status, caps, reason = PROSE[number]
                seed.update(status=status, capabilities=caps, note=reason)
        rows.append(seed)
        scout.append({'source_id': seed['source_id'], 'extract_line': number,
                      'wikitext_line': number, 'literal': text.splitlines()[number - 1],
                      'raw_literal': raw_lines[number - 1], 'status': seed['status'], 'reason': seed['note']})
    dump(SOURCES / '7.2.0-page-coverage.json', {
        'schema': 'patch-api-page-coverage/v1', 'patch': '7.2.0',
        'source': str(register_path.relative_to(ROOT)), 'source_sha256': digest(register_path),
        'non_inventory_source': {'path': str(text_path.relative_to(ROOT)), 'sha256': digest(text_path)},
        'proof_policy': 'Account every occurrence; source prose duplicates retain their separate substantive contract. Bounded current behavior is not historical/native parity. No member retirements. 7.2.5 integration placeholder remains.',
        'source_rows': rows,
    })
    dump(HERE / 'p720-extract-scout.json', scout)
    dump(HERE / 'p720-gap-review.json', [{'source_id': entry['id'], 'reason': 'Real publication gap; no synthetic shim added.'}
                                        for entry in register['entries'] if entry['id'] in known])
    dump(HERE / 'p720-problematic-contracts.json', [
        {'source_id': row['source_id'], 'reason': row['reason']} for row in scout if row['status'] == 'audit-pending'])
    dump(HERE / 'p720-accounting-summary.json', {
        'inventory_rows': len(register['entries']), 'inventory_ok': len(results) - len(known),
        'inventory_gaps': len(known), 'extract_rows': len(scout), 'ledger_rows': len(rows),
        'ledger_statuses': dict(Counter(row['status'] for row in rows)),
        'pending_prose': sum(row['status'] == 'audit-pending' for row in scout),
    })
    print((HERE / 'p720-accounting-summary.json').read_text())


if __name__ == '__main__':
    main()
