"""Account for every inventory row, retained prose line and omitted diff caption."""
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def extract_rows():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    rows = module.seed_rows((SOURCES / '5.4.7-api-changes.txt').read_text(), '5.4.7')
    notes = {
        'prose-undated-004': 'CHAT_MSG_ADDON arg4 must identify the inbound sender with realm. Existing legacy synthetic echo in message_verbs.rs instead copies the outbound recipient into arg4; namespace outbound intent is not inbound delivery. No inbound sender/realm producer is modeled. Retained as a concrete payload mismatch, not patched with fabricated event injection.',
        'prose-undated-005': 'CHAT_MSG_WHISPER arg2 requires an incoming author-name/realm producer. send_chat_message logs outbound messages without inbound events; GM Chat consumer tests inject fixtures rather than modeling remote delivery. Registration does not establish author normalization.',
        'prose-undated-006': 'Source explicitly says other CHAT_MSG_* events were not checked. No concrete event list or universal realm-normalization contract can be inferred; uncertainty retained literally.',
    }
    for row in rows:
        if row['source_id'] in notes:
            row['note'] = notes[row['source_id']]
    raw = (SOURCES / '5.4.7-api-changes.wikitext').read_text()
    for number, line in enumerate(raw.splitlines(), 1):
        if line.startswith('|+'):
            rows.append({'source_id': f'source-caption-{number:03}', 'status': 'metadata-only',
                         'capabilities': [], 'note': 'Diff build context, not runtime credit: ' + line[2:].strip(),
                         'wikitext_line': number})
    return rows


def main():
    register = read(SOURCES / '5.4.7-wikitext-register.json')
    observed = read(HERE / 'p547-discovery-results.json')
    assert set(observed) == {entry['id'] for entry in register['entries']}
    rows, gaps = [], []
    for entry in register['entries']:
        result = observed[entry['id']]
        ok = result['ok']
        note = 'Exact current publication/absence after retail later-register supersession only; no historical signature/output/security/domain parity.'
        if not ok:
            assert entry['symbol'] in ('CHARACTER_UPGRADE_ABORTED', 'CHARACTER_UPGRADE_STARTED'), entry
            note = ('Current strict retail event catalog rejects ' + entry['symbol'] +
                    '; source-era character-boost start/abort lifecycle and its service producer are unmodeled. Do not make a historical name registerable without modeling the relevant lifecycle, or reuse the unrelated modern boost/store completion callback.')
            gaps.append({'source_id': entry['id'], 'symbol': entry['symbol'], 'reason': note, 'observed': result})
        rows.append({'source_id': entry['id'], 'status': 'bounded-coverage' if ok else 'audit-pending',
                     'capabilities': ['publication-absence'] if ok else [], 'note': note})
    rows.extend(extract_rows())
    assert len({row['source_id'] for row in rows}) == len(rows)
    write(SOURCES / '5.4.7-page-coverage.json', {
        'patch': '5.4.7', 'source_revid': register['source']['revid'],
        'proof_policy': 'Publication/absence is distinct from bounded existing backing behavior and pending 2014 contracts.',
        'source_rows': rows,
        'non_inventory_source': {'path': 'data/patch-api/sources/5.4.7-api-changes.txt'},
    })
    write(ROOT / 'tests/data/patch_5_4_7_sweep_known_gaps.json', [row['source_id'] for row in gaps])
    write(HERE / 'p547-gap-review.json', gaps)
    write(HERE / 'p547-accounting-summary.json', {
        'inventory': len(register['entries']), 'total_ids': len(rows),
        'publication_gaps': len(gaps),
        'statuses': {status: sum(row['status'] == status for row in rows)
                     for status in sorted({row['status'] for row in rows})},
        'pending_prose': [row['source_id'] for row in rows if row['status'] == 'audit-pending'
                          and not row['source_id'].startswith('wt-')],
    })
    print(json.dumps(read(HERE / 'p547-accounting-summary.json')))


if __name__ == '__main__':
    main()
