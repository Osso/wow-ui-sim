"""Account for every pinned inventory identity and retained plaintext statement."""
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


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def classify_prose(literal):
    publication = [{'kind': 'publication', 'proof': 'tests/patch_7_1_0_publication_sweep.rs'}]
    if literal.startswith('* New API:'):
        return 'bounded-coverage', publication, 'Physical display publication and existing screen_mode tests; legacy item-level API superseded by 10.2.6 removal, current C_Item base-level/link query tested. No bonus/scaling/native coercion parity claim.'
    if literal.startswith('* New XML frame attributes:'):
        return 'audit-pending', [], 'clipChildren state/inheritance is bounded by tests/patch_7_1_0_behavior.rs. Custom intrinsic declarations enter the registry, but CreateFrame uses a finite widget_type_for_tag alias map and rejects P710ClipIntrinsic. Failed cached probe and exact harness retained; no special-case factory shim. Generalized dynamic tag/loading/security contract remains missing.'
    if literal.startswith('* New frame methods:'):
        return 'bounded-coverage', [{'kind': 'clipping-state', 'proof': 'tests/patch_7_1_0_behavior.rs'}], 'Inherited clipChildren before OnLoad, false override and independent SetClipsChildren/DoesClipChildren state tested. Pixel clipping, hit testing and custom intrinsic factories are not credited by this test.'
    if literal == '* New CVars:':
        return 'metadata-only', [], 'Category heading; named CVar rows are accounted separately, no runtime credit.'
    if literal.startswith(('** NameplatePersonal', '*** 0 =')):
        return 'bounded-coverage', publication, 'Five named CVars superseded by explicit 12.0.0 removals; current absence observed. Historical personal-nameplate visibility/delay/alpha behavior is not implemented or credited.'
    if literal.startswith('* ScrollingMessageFrame'):
        return 'audit-pending', [], 'Page describes a native-to-Lua implementation migration, not new named methods. Current runtime maps ScrollingMessageFrame to a MessageFrame plus the ScrollingMessageFrame Lua template; no pinned 7.1.0 implementation or historical migration parity proof. Linked source is not expanded.'
    if literal.startswith("* 'OnEnter'"):
        return 'audit-pending', [], 'Mouse-entry eligibility statement is explicitly qualified as a confirmed pre-release bug expected to be fixed. No final 7.1.0 client capture establishes the intended policy; do not change shared hover dispatch based on a withdrawn/uncertain rule.'
    if literal.startswith('** This has been confirmed'):
        return 'metadata-only', [], 'Qualification of preceding mouse bug, including author attribution; not evidence of final client behavior.'
    if literal.startswith('* GetItemInfo'):
        return 'audit-pending', [], 'Legacy global superseded by 10.2.6 removal. Current C_Item successor reads binding and expansion from real item catalog; bounded Hearthstone/link/base-level test. itemSetID is constant 0 and isCraftingReagent constant false, not backed by a set/reagent metadata model; four-return semantic parity remains incomplete. Do not add guessed metadata/shims.'
    if literal == '* TitleRegion':
        return 'audit-pending', [], 'Bare removal gives no factory/method/signature contract. No exact TitleRegion retail/src/tests consumer or later master/p720 re-addition. WidgetType has no TitleRegion and unknown runtime CreateFrame kinds are rejected; neither fact establishes the old native region factory/method contract. Related GetTitleRegion names occur in current Lua mixins, not a native TitleRegion factory; no methods or widget fallback retired.'
    if literal == '[References list; not expanded]':
        return 'metadata-only', [], 'Reference expansion is outside this pinned page; no runtime credit.'
    raise ValueError(f'unaccounted prose: {literal}')


def main():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    register_path = SOURCES / '7.1.0-wikitext-register.json'
    register = read(register_path)
    results = read(HERE / 'patch_7_1_0_publication_sweep-results.json')
    text_path = SOURCES / '7.1.0-api-changes.txt'
    text = text_path.read_text()
    raw = (SOURCES / '7.1.0-api-changes.wikitext').read_text().splitlines()
    rows, gaps = [], []
    for entry in register['entries']:
        gap = not results[entry['id']]['ok']
        note = 'Current publication/lookup only; no historical signatures, output, secrecy or behavioral parity claim.'
        if results[entry['id']]['expected']['superseded_by']:
            note = 'Current absence/deprecation fallback after ' + results[entry['id']]['expected']['superseded_by'] + '; not historical behavior.'
        rows.append({'source_id': entry['id'], 'source': entry['symbol'],
                     'status': 'audit-pending' if gap else 'bounded-coverage',
                     'capabilities': [] if gap else [{'kind': 'publication', 'proof': 'tests/patch_7_1_0_publication_sweep.rs'}],
                     'note': note})
        if gap:
            gaps.append({'source_id': entry['id'], 'symbol': entry['symbol'], 'reason': note, 'observation': results[entry['id']]})
    scout, pending = [], []
    for row in extractor.seed_rows(text, '7.1.0'):
        number = int(row['source_id'].rsplit('-', 1)[-1])
        literal = text.splitlines()[number - 1]
        if row['status'] == 'metadata-only':
            status, capabilities, reason = 'metadata-only', [], 'Source title/heading; no runtime credit.'
        else:
            status, capabilities, reason = classify_prose(literal)
        row.update(status=status, capabilities=capabilities, note=reason)
        rows.append(row)
        scout.append({'source_id': row['source_id'], 'extract_line': number, 'literal': literal,
                      'wikitext_line': number, 'raw_literal': raw[number - 1], 'status': status, 'reason': reason})
        if status == 'audit-pending':
            pending.append({'source_id': row['source_id'], 'literal': literal, 'reason': reason})
    dump(SOURCES / '7.1.0-page-coverage.json', {
        'schema': 'patch-page-coverage/v1', 'patch': '7.1.0',
        'source_sha256': hashlib.sha256(register_path.read_bytes()).hexdigest(),
        'non_inventory_source': {'path': str(text_path.relative_to(ROOT)), 'sha256': hashlib.sha256(text_path.read_bytes()).hexdigest()},
        'proof_policy': 'Publication, superseded absence and bounded existing-model queries only; no native/historical parity.',
        'source_rows': rows})
    dump(HERE / 'p710-extract-scout.json', scout)
    dump(HERE / 'p710-gap-review.json', {'publication_gaps': gaps, 'unmodeled_source_contracts': pending})
    summaries = []
    for path in sorted(HERE.glob('patch_*_publication_sweep-results.json')):
        observations = read(path)
        summaries.append({'test': path.name.removesuffix('-results.json'), 'file': path.name,
                          'rows': len(observations), 'ok': sum(row['ok'] for row in observations.values()),
                          'gaps': sum(not row['ok'] for row in observations.values())})
    dump(HERE / 'p710-sweep-summary.json', summaries)
    print(json.dumps({'inventory': len(register['entries']), 'extract': len(scout),
                      'ledger': len(rows), 'gaps': len(gaps), 'pending_prose': len(pending)}))


if __name__ == '__main__':
    main()
