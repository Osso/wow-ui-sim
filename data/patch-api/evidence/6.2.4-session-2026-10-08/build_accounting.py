"""Account for each pinned register occurrence and retained plaintext statement."""
import hashlib
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


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def classify_statement(literal, matching_entries):
    publication = [{'kind': 'current-publication', 'proof': 'tests/patch_6_2_4_publication_sweep.rs'}]
    if matching_entries:
        return ('bounded-coverage', publication,
                'Named occurrences match register publication/absence observations only; '
                'later 8.2.5 removals supersede the three GameAccount additions. '
                'Historical signatures, tuple outputs and native rename behavior are not credited.')
    if literal == 'Examples:':
        return 'metadata-only', [], 'Label for the three following explicit rename pairs; no runtime credit.'
    if literal.startswith('*Presence IDs'):
        return ('audit-pending', [],
                'Current C_BattleNet backing distinguishes parent account IDs, game-account IDs and GUIDs. '
                'tests/patch_6_2_4_behavior.rs bounds distinct output IDs, index-vs-ID and wrong-GUID lookups. '
                'The three numeric-ID BN* query globals are superseded by 8.2.5 removals; '
                'no exhaustive historical all-functions strict numeric-input/invalid-ID contract or native capture. '
                'Do not reintroduce retired globals or alias numeric IDs to friend indices.')
    if 'translate from' in literal or literal.startswith('bnetIDAccount ='):
        return ('audit-pending', [],
                'select(17, BNGetGameAccountInfo(gameID)) is an explicit historical tuple contract. '
                'Current API queries return tables through GUID-based C_BattleNet backing; '
                'the BNGetGameAccountInfo global is absent after 8.2.5 removal. '
                'No current numeric gameID-to-parent lookup or 17-return tuple is modeled; '
                'successor identity/count tests are not proof of this tuple. No compatibility shim added.')
    if 'active bnetIDGameAccount' in literal or literal.startswith('bnetIDGameAccount ='):
        return ('audit-pending', [],
                'select(6, BNGetFriendInfoByID(accountID)) is an explicit historical active-account contract. '
                'BNGetFriendInfoByID is removed in 8.2.5. Current GetFriendAccountInfo takes a friend index '
                'and selects the first game account; optional account selection is ignored. '
                'No modeled historical active-account selection or six-position tuple; do not equate index with account ID.')
    if literal.startswith('*The new Battle.net architecture'):
        return ('audit-pending', [],
                'Three named Toon-to-GameAccount pairs are publication-accounted separately. '
                'Unquantified many-functions architecture transition supplies no exhaustive native input/output contract; '
                'current separate parent/game backing is bounded, not historical migration parity.')
    if literal.startswith('* Support for conversations'):
        return ('bounded-coverage', publication,
                'Both named C-function removals and every explicit conversation-related removal in the list '
                'are raw/lookup absent in cached retail. Current non-documentation cached consumers are absent. '
                'Unenumerated etc. and all historical Lua/XML-reference removal are not independently reconstructed.')
    if literal.startswith('* The “realmName”'):
        return ('audit-pending', [],
                'realmName CVar value/default absence is observed. GetRealmName remains a temporary '
                'constant SimulatedRealm in client_info_defaults.rs, not an authoritative current-realm backing model. '
                'The page provides no realm-selection/update fixture, and variable/field consumers with the same name '
                'are retained. No existing CVar/global consumers retired; no constant promoted as modeled realm behavior.')
    if literal.startswith('[Transcluded source:'):
        return ('metadata-only', [],
                'Automated diff subpage is referenced but not expanded by this page audit; '
                'no subpage API inventory or native contract credit.')
    raise ValueError(f'unaccounted retained statement: {literal}')


def main():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    register_path = SOURCES / '6.2.4-wikitext-register.json'
    register = read(register_path)
    results = read(HERE / 'p624-discovery-results.json')
    text_path = SOURCES / '6.2.4-api-changes.txt'
    text = text_path.read_text()
    raw = (SOURCES / '6.2.4-api-changes.wikitext').read_text().splitlines()
    rows, gaps = [], []
    for entry in register['entries']:
        gap = not results[entry['id']]['ok']
        note = 'Current raw/lookup publication or absence only; no historical signature/output/native parity.'
        if results[entry['id']]['expected']['superseded_by']:
            note = 'Current absence after later 8.2.5 removal; not historical added-global behavior.'
        rows.append({'source_id': entry['id'], 'source': entry['symbol'],
                     'status': 'audit-pending' if gap else 'bounded-coverage',
                     'capabilities': [] if gap else [{'kind': 'publication', 'proof': 'tests/patch_6_2_4_publication_sweep.rs'}],
                     'note': note})
        if gap:
            gaps.append({'source_id': entry['id'], 'symbol': entry['symbol'], 'reason': note, 'observation': results[entry['id']]})
    scout, pending = [], []
    for row in extractor.seed_rows(text, '6.2.4'):
        number = int(row['source_id'].rsplit('-', 1)[-1])
        literal = text.splitlines()[number - 1].strip()
        matching = [entry for entry in register['entries'] if entry['wikitext_line'] == number]
        if row['status'] == 'metadata-only':
            status, capabilities, reason = 'metadata-only', [], 'Source title/heading; no runtime credit.'
        else:
            status, capabilities, reason = classify_statement(literal, matching)
        # The realm prose mixes an inventory absence with an unmodeled getter contract.
        if literal.startswith('* The “realmName”'):
            status, capabilities, reason = classify_statement(literal, [])
        row.update(status=status, capabilities=capabilities, note=reason)
        rows.append(row)
        scout.append({'source_id': row['source_id'], 'extract_line': number,
                      'literal': text.splitlines()[number - 1], 'wikitext_line': number,
                      'raw_literal': raw[number - 1], 'status': status, 'reason': reason})
        if status == 'audit-pending':
            pending.append({'source_id': row['source_id'], 'literal': literal, 'reason': reason})
    write_json(SOURCES / '6.2.4-page-coverage.json', {
        'schema': 'patch-page-coverage/v1', 'patch': '6.2.4',
        'source_sha256': hashlib.sha256(register_path.read_bytes()).hexdigest(),
        'non_inventory_source': {'path': str(text_path.relative_to(ROOT)),
                                 'sha256': hashlib.sha256(text_path.read_bytes()).hexdigest()},
        'proof_policy': 'Current superseded publication/absence and bounded successor backing only; no native/historical tuple parity.',
        'source_rows': rows})
    write_json(HERE / 'p624-extract-scout.json', scout)
    write_json(HERE / 'p624-gap-review.json', {'publication_gaps': gaps, 'unmodeled_source_contracts': pending})
    print(json.dumps({'inventory': len(register['entries']), 'extract': len(scout),
                      'ledger': len(rows), 'publication_gaps': len(gaps), 'pending_prose': len(pending)}))


if __name__ == '__main__':
    main()
