"""Account for inventory, main-page prose and separately pinned diff captions."""
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


def gap_reason(entry):
    symbol = entry['symbol']
    if symbol.startswith('Browser'):
        return ('No embedded-browser engine/backing for the 2013 support website. ' + symbol +
                ' is not supported by the current Browser object publication. Generic frame '
                'focus methods and two inert navigation methods do not establish browser '
                'cache/cookies/history/ticket/connection/zoom behavior. No inert aliases added.')
    if entry['section'] == 'events':
        return ('Strict current retail catalog rejects ' + symbol +
                '. Its historical battleground, PvP-role, bonus-roll, web-ticket or browser '
                'service producer is not modeled by this audit. Registration alone would '
                'not establish lifecycle/payload parity; no catalog-only workaround added.')
    if symbol in ('AcknowledgeSurvey', 'HideKnowledgeBase', 'ShowKnowledgeBase'):
        return ('Historical support/knowledge-base/survey service has no state-backed lifecycle '
                'or embedded-browser integration. Current help-panel consumers do not supply '
                '2013 survey acknowledgement or old knowledge-base semantics; no shim added.')
    if symbol in ('GetItemUpgradeEffect', 'GetNumItemUpgradeEffects'):
        return ('Legacy item-upgrade effect catalog is absent. The later C_ItemUpgrade names '
                'are themselves publication gaps in the pinned 9.1.0 register; no effect '
                'records or input/output contract established by this source.')
    if symbol == 'CanTrackBattlePets':
        return ('Legacy battle-pet tracking query is absent. Later C_Minimap.CanTrackBattlePets '
                'also remains a reviewed publication gap; minimap tracking capability/data '
                'is not inferred from a pet-journal fixture.')
    if symbol == 'SetLootSpecialization':
        return ('No legacy loot-specialization setter or loot-award policy backing. '
                'GetLootSpecialization is only a temporary constant-zero default; selecting '
                'class/spec IDs cannot be modeled by aliasing character specialization.')
    if symbol == 'GetBattlegroundPoints':
        return ('No battleground team score/objective producer or legacy point-query contract. '
                'The source names the function but supplies no result layout/fixture; '
                'do not substitute rated PvP statistics or fabricate zero team points.')
    if symbol == 'GetLFGRoleUpdateBattlegroundInfo':
        return ('No historical battleground role-update invitation state/query structure. '
                'Stored lfg_roles booleans support GetPVPRoles/SetPVPRoles only, not '
                'queue invitation metadata or role-update lifecycle.')
    return ('Current publication is absent; this source supplies identity only, not a '
            'complete modeled input/output/lifecycle contract for ' + symbol +
            '. No compatibility shim or guessed payload added.')


def extract_rows():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    plaintext = (SOURCES / '5.3.0-api-changes.txt').read_text()
    rows = module.seed_rows(plaintext, '5.3.0')
    lines = plaintext.splitlines()
    for row in rows:
        if row['status'] != 'audit-pending':
            continue
        source = lines[int(row['source_id'].rsplit('-', 1)[1]) - 1]
        row['note'] = source
        if source.startswith('[Transcluded source:'):
            row['status'] = 'metadata-only'
            row['note'] += ' Separately pinned diff inventoried; this pointer gives no runtime credit.'
            continue
        if 'specialization' in source.lower() and 'loot' in source.lower():
            row['note'] += (' No award-policy state or setter; constant-zero GetLootSpecialization '
                            'is publication only, not modeled selection.')
        elif 'stable' in source.lower():
            row['note'] += (' Historical five-page/fifty-slot stable layout is not demonstrated '
                            'by current stable APIs/fixtures. No 2013 roster/page model introduced.')
        elif 'browser' in source.lower():
            row['note'] += (' Current Browser creation maps to a generic frame, with no embedded '
                            'web engine, support-service persistence or native browsing. '
                            'Intentional unsupported browser domain; not modeled parity.')
        elif 'encoding' in source.lower():
            row['note'] += (' No before/after four-field trade-link fixture or Cooking '
                            'specialization encoding supplied. Current hyperlink handling '
                            'does not prove the historical bit/field layout migration.')
        elif 'OpenToCategory' in source:
            row['note'] += (' Historical InterfaceOptions panel list/update ordering and '
                            'wrong-panel bug are not reproducible in current Settings UI. '
                            'Do not monkey-patch Blizzard Lua with the page double-call workaround.')
        else:
            row['note'] += ' Retained source context; transcluded diff is separately pinned and inventoried.'
    for number, line in enumerate((SOURCES / '5.3.0-api-changes.diff.wikitext').read_text().splitlines(), 1):
        if line.startswith('|+'):
            rows.append({'source_id': f'diff-caption-{number:03}', 'status': 'metadata-only',
                         'capabilities': [], 'note': '2013 build context, not API behavior: ' + line[2:].strip()})
    return rows


def main():
    register = read(SOURCES / '5.3.0-wikitext-register.json')
    observed = read(HERE / 'p530-discovery-results.json')
    assert set(observed) == {entry['id'] for entry in register['entries']}
    rows, gaps = [], []
    for entry in register['entries']:
        result = observed[entry['id']]
        note = ('Current publication/absence after ordered later-retail supersession only; '
                'not historical signature/output/security/domain parity.')
        capabilities = ['publication-absence'] if result['ok'] else []
        if not result['ok']:
            note = gap_reason(entry)
            gaps.append({'source_id': entry['id'], 'symbol': entry['symbol'],
                         'reason': note, 'observed': result})
        elif entry['symbol'] in ('GetPVPRoles', 'SetPVPRoles'):
            note += (' Existing lfg_roles backing is tested across tank/dps, healer-only '
                     'and all-false transitions in bare and cached environments. '
                     'No historical queue/role-update event semantics credited.')
            capabilities.append('bounded-role-state')
        elif entry['symbol'] == 'GetLootSpecialization':
            note += ' Temporary inert constant-zero default, not loot-selection backing.'
        elif entry['symbol'] == 'GetWebTicket':
            note += ' Temporary nil default, not web-ticket service backing.'
        elif entry['symbol'] == 'C_PetBattles.CanAcceptQueuedPVPMatch':
            note += ' Temporary pet-battle queue compatibility value, not network PvP-match lifecycle.'
        elif entry['symbol'].startswith('Browser'):
            note += (' Generic shared frame/script publication or inert navigation only; '
                     'no native browser engine or focus/history semantics credited.')
        if result['expected']['superseded_by']:
            note += ' Superseded by ' + result['expected']['superseded_by'] + '.'
        rows.append({'source_id': entry['id'], 'status': 'bounded-coverage' if result['ok'] else 'audit-pending',
                     'capabilities': capabilities, 'note': note})
    rows.extend(extract_rows())
    assert len({row['source_id'] for row in rows}) == len(rows)
    write(SOURCES / '5.3.0-page-coverage.json', {
        'patch': '5.3.0', 'source_revid': 4065122, 'diff_revid': 3188422,
        'proof_policy': 'Publication is distinct from bounded existing backing; unsupported historical contracts remain pending.',
        'source_rows': rows,
        'non_inventory_source': {'path': 'data/patch-api/sources/5.3.0-api-changes.txt'},
    })
    write(ROOT / 'tests/data/patch_5_3_0_sweep_known_gaps.json', [row['source_id'] for row in gaps])
    write(HERE / 'p530-gap-review.json', gaps)
    write(HERE / 'p530-accounting-summary.json', {
        'inventory': len(register['entries']), 'accounted_ids': len(rows), 'publication_gaps': len(gaps),
        'statuses': {status: sum(row['status'] == status for row in rows)
                     for status in sorted({row['status'] for row in rows})},
        'pending_prose': [row['source_id'] for row in rows if row['status'] == 'audit-pending'
                          and not row['source_id'].startswith('diff-wt-')],
    })
    print(json.dumps(read(HERE / 'p530-accounting-summary.json')))


if __name__ == '__main__':
    main()
