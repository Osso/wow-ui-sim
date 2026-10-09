"""Separate historical publication, existing backing behavior and unresolved contracts."""
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'

GAP_REASONS = {
    'C_PetBattles.GetForfeitPenalty': 'Namespace lookup fabricates a function but no raw member/backing forfeit-penalty lifecycle exists. Pet-battle forfeiture and penalty state are not modeled; no shim added.',
    'DeclineChannelInvite': 'Legacy channel-invitation decline global is absent; no modeled invitation acceptance/decline lifecycle or pending inviter state. Chat sending is not invite handling.',
    'GetArenaTeamIndexBySize': 'Historical arena-team roster/size indexing is absent. Current rated PvP queries do not establish the 2013 persistent arena-team contract.',
    'GetBlacklistMapName': 'Historical random-battleground blacklist slots/map catalog are absent. Existing PvP fixtures do not model blacklist selection or names.',
    'GetQuestChoiceInfo': 'Historical global is absent. Existing missing_surface/quest_choice.rs publishes C_QuestChoice with a fixed fixture; restoring the global would be an unrequested compatibility alias, not proof of a 2013 quest producer.',
    'GetQuestChoiceOptionInfo': 'Historical global is absent. Modern C_QuestChoice fixture uses response IDs/zero-based option lookup, but the source page supplies no 2013 argument/output contract. No fabricated global alias.',
    'GetRandomBGInfo': 'Historical global is absent; current PvP backing exposes a seeded C_PvP random-battleground row, not this historical global. No legacy alias or random queue/service parity claim.',
    'IsAllowedToUserTeleport': 'Source spelling retained literally (User, not Use); raw member absent. No documented argument/teleport permission policy or consumer found; do not invent a predicate.',
    'IsTradeSkillReady': 'Historical global is absent; modern C_TradeSkillUI readiness does not establish historical global publication. No shim alias; profession loading behavior remains scoped to the current namespace.',
    'PrepVoidStorageForTransmogrify': 'Void-storage transmog preparation global is absent. The void-storage source-to-transmog pending transaction and selection ownership are unmodeled; current outfit models are not that service.',
    'UseVoidItemForTransmogrify': 'Void-storage transmog use global is absent. No source-item/slot selection, pending transmog transaction or transfer lifecycle is modeled; no no-op implementation.',
    'SetRaidDifficultyID': 'Getter is state-backed, but this setter global is absent. The page does not define group-leader permissions, supported IDs, in-instance restrictions or change events. Getter proof does not imply setter parity.',
    'UpdateWorldMapArrow': 'Historical map-arrow update global is absent. Current map APIs do not model the old arrow frames/camera-position ownership; the removed arrow family is not restored.',
    'ControlGetActiveCvarValue': 'Historical FrameXML helper is absent in the SharedXML preload. Source gives no active-control/CVar comparison contract; current CVar lookup is not sufficient. No Blizzard patch or substitute helper.',
}


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def inventory_reason(entry, result):
    symbol = entry['symbol']
    if symbol.startswith('C_MapBar.'):
        return ('No raw publication for ' + symbol + '; lookup-only namespace defaults are not credit. '
                'No scenario map-bar participation/value/phase/tag producer or state model exists; the 2013 map-bar backing cannot be replaced with a plausible default.')
    if symbol.startswith('Model:'):
        return ('Historical removal conflicts with the simulator common frame method surface: ' + symbol +
                ' remains published. Model/PlayerModel 3D is intentionally unsupported. Current whole-word bare-name consumers and shared method dispatch forbid removing common methods merely to satisfy a 2013 owner migration. No rendering or historical method-ownership parity claim.')
    if symbol.startswith('PlayerModel On'):
        return ('HasScript is false for ' + symbol +
                '. The simulator has no 3D model animation/update producer. Advertising a script without that producer would be a shortcut; intentional 3D exclusion retained.')
    assert symbol in GAP_REASONS, symbol
    return GAP_REASONS[symbol]


def main():
    register = read(SOURCES / '5.2.0-wikitext-register.json')
    observed = read(HERE / 'discovery-results.json')
    assert set(observed) == {r['id'] for r in register['entries']}
    rows, gaps = [], []
    for entry in register['entries']:
        result = observed[entry['id']]
        note = 'Current retail publication/absence after explicit later-retail supersession only; no 2013 output, signature, security or domain parity.'
        capabilities = ['publication-absence'] if result['ok'] else []
        if not result['ok']:
            note = inventory_reason(entry, result)
            gaps.append({'source_id': entry['id'], 'symbol': entry['symbol'], 'reason': note, 'observed': result})
        elif entry['symbol'] == 'GetRaidDifficultyID':
            capabilities.append('state-backed-instance-difficulty')
            note += ' New integration and prefork proof reads distinct state values 14/16; no setter/permissions claim.'
        elif entry['symbol'] == 'Cooldown:GetCooldownDuration':
            capabilities.append('state-backed-cooldown-duration')
            note += ' Existing cooldown_widget tests assert timing updates and clear transitions in milliseconds; no 2013-specific rate semantics claim.'
        elif entry['symbol'] == 'GetSchoolString':
            capabilities.append('bounded-school-name-mapping')
            note += ' Existing C_Spell mapping and unmodified cached deprecated alias; scoped school integration case proves Physical school mask 1, not historical locale/composite-mask completeness.'
        elif entry['symbol'] in ('UnitGetTotalAbsorbs', 'UnitStagger', 'C_PetJournal.GetPetCooldownByGUID'):
            note += ' Fixed-zero/default implementation is NOT modeled credit; absorption, stagger or pet-cooldown producer/state remains pending.'
        elif entry['symbol'].startswith('PlayerModel:'):
            note += ' Permanent 3D/model compatibility surface only; rendering is intentionally unsupported.'
        rows.append({'source_id': entry['id'], 'status': 'bounded-coverage' if result['ok'] else 'audit-pending',
                     'capabilities': capabilities, 'note': note})
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    prose = extractor.seed_rows((SOURCES / '5.2.0-api-changes.txt').read_text(), '5.2.0')
    notes = {
        '004': 'Dual text/percent status-bar option is a vendor UI/CVar presentation contract. No current 2013 configuration/render observation; publication does not establish it.',
        '005': 'UnitStagger is a temporary zero default in unit_stagger_defaults.rs; no rolling ten-second monk stagger damage pool/producer or bar rendering proof.',
        '006': 'UnitGetTotalAbsorbs returns constant zero in utility_system_spell/spell_api.rs; no stacked absorb pool, mutation producer, UNIT_ABSORB_AMOUNT_CHANGED payload lifecycle or health-bar overlay proof.',
        '007': 'SecureActionButtonTemplate togglemenu execution requires protected action dispatch and unit-popup authorization, not mere template existence. No source-era security/input proof; no vendor monkey-patch.',
        '008': 'Void-storage transmog preparation/use transaction is unmodeled; inventory existence and outfit models are not void-storage source transfer.',
        '011': 'GetRaidDifficultyID reads state values 14/16 in new tests; SetRaidDifficultyID is absent. Renaming and setter permission/change-event contract remain pending.',
        '012': 'GetDifficultyInfo is a temporary lookup/default in difficulty_pvp_util_defaults.rs, not proof of a complete 2013 difficulty catalog and outputs.',
        '013': 'PetJournal GetPetCooldownByGUID returns fixed 0/0/false. No per-GUID cooldown state or expiry transitions; no model credit.',
        '014': 'Current C_Spell school-name mask mapping is behaviorally covered; cached deprecated alias is unmodified. SchoolStringTable has no cache/src/tests consumer in supplemental whole-word scans. Historical localization/composite-mask completeness remains unproven.',
        '015': 'Source says almost all (if not all) PetJournal functions: no complete member list/argument signatures. Cannot invent a universal isWild removal assertion.',
        '016': 'Source nil-PetID failure contract spans unspecified PetJournal members; current default cooldown function accepts/ignores arguments. No blanket pcall-validation shim; exact per-member error parity remains pending.',
        '017': 'Legacy GetCurrencyInfo full texture-path return is source-era output semantics, not current modern icon ID publication. No captured 2013 currency/texture fixture; no fabricated path.',
        '018': 'Reported GetTradeSkillReagentItemLink breakage is a historical client defect. Do not replace working current profession backing with always-nil behavior.',
        '019': 'Source uncertainty (appears to always return nil) for IsConsumableItem is retained. Do not turn an unverified historical bug report into a current modeled contract.',
        '022': 'MAX_BLACKLIST_BATTLEGROUNDS relocation is PVPUI load-order scope, not global retirement. Supplemental whole-word cache/src/tests scans find no consumer; no 2013 PVPUI dependency/order fixture.',
        '023': 'MapBarFrame_GetString depends on historical MAP_BAR_* localized keys and phase/tag formatting. MapBar model is missing; no fabricated string helper.',
        '026': 'Expansion instructions are page presentation only; no runtime credit.',
    }
    for row in prose:
        suffix = row['source_id'].rsplit('-', 1)[-1]
        if suffix in ('026', '028'):
            row['status'] = 'metadata-only'
            row['note'] = ('Expansion instructions only; no runtime credit.' if suffix == '026' else
                           'Transclusion separately fetched/pinned; every inventory row accounted above.')
        elif row['status'] == 'audit-pending':
            assert suffix in notes, row
            row['note'] = notes[suffix]
    rows.extend(prose)
    for number, line in enumerate((SOURCES / '5.2.0-api-changes-diff.wikitext').read_text().splitlines(), 1):
        if line.startswith('|+'):
            rows.append({'source_id': f'diff-caption-{number:03}', 'status': 'metadata-only', 'capabilities': [],
                         'note': 'Pinned retail diff build context, not runtime credit: ' + line[2:].strip(), 'wikitext_line': number})
    assert len({r['source_id'] for r in rows}) == len(rows)
    dump(SOURCES / '5.2.0-page-coverage.json', {'patch': '5.2.0', 'source_revid': register['source']['revid'],
         'proof_policy': 'Publication/absence, bounded existing backing and pending 2013 contracts remain distinct.',
         'source_rows': rows, 'non_inventory_source': {'path': 'data/patch-api/sources/5.2.0-api-changes.txt'}})
    dump(ROOT / 'tests/data/patch_5_2_0_sweep_known_gaps.json', [r['source_id'] for r in gaps])
    dump(HERE / 'gap-review.json', gaps)
    summary = {'inventory': len(observed), 'publication_gaps': len(gaps), 'total_ids': len(rows),
               'statuses': {s: sum(r['status'] == s for r in rows) for s in sorted({r['status'] for r in rows})},
               'pending_prose': [r['source_id'] for r in prose if r['status'] == 'audit-pending'],
               'modeled_inventory': [r['source_id'] for r in rows if len(r['capabilities']) > 1], 'retirements': 0}
    dump(HERE / 'accounting-summary.json', summary)
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
