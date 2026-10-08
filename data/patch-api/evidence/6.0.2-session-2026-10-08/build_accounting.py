"""Account for pinned inventory, every retained prose line and diff enum member."""
import importlib.util
import json
from pathlib import Path
from collections import Counter

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def gap_reason(entry, result, decisions):
    symbol = entry['symbol']
    expected = result['expected']
    observed = result['observed']['detail']
    if expected['publication'] == 'absent':
        decision = decisions.get(symbol)
        if decision and (decision['cached_consumers'] or decision['caller_scans'] or decision['later_additions']):
            reason = 'Retirement prohibited: current cached consumers, simulator callers or later re-additions remain; preserve the live contract.'
        else:
            reason = 'Historical removal conflicts with current publication; no destructive change without native/successor evidence.'
    else:
        domain = symbol.split('.')[0]
        reasons = {
            'C_Garrison': 'Historical garrison building/follower/mission/recruitment/shipment contract lacks a registered modeled provider for this member; namespace lookup alone is not backing behavior.',
            'C_LFGList': 'Existing listing/search/application models do not implement this applicant/invite/role-check/social-data operation; no synthesized server responses.',
            'C_ToyBox': 'Historical toy filter/cursor operation lacks a state-backed provider; collection presence does not implement its filter or pickup semantics.',
            'C_Trophy': 'Garrison monument selection/appearance persistence has no backing subsystem; no fabricated monument inventory.',
            'C_Questline': 'Historical indexed questline catalog contract differs from current questline lookup; no indexed catalog fixture or translation proved.',
            'C_TaskQuest': 'Historical task quest text getter has no registered provider; modern task query presence is not a historical signature proof.',
            'C_MountJournal': 'Legacy pickup/summon contract lacks a modeled provider and has simulator successor callers; no retirement or placeholder added.',
            'C_Scenario': 'Scenario objective supersession/dungeon classification contract is not represented by the existing step/reward model.',
            'C_Vignettes': 'Historical vignette GUID lookup lacks a modeled instance-to-GUID catalog; no fabricated identity.'}
        if domain in reasons:
            reason = reasons[domain]
        elif ':' in symbol:
            reason = 'Historical widget method conflicts with later surface expectation or lacks registration; preserve current tooltip consumers rather than remove methods.'
        elif entry['section'] == 'events':
            reason = 'Historical event name is not in the current retail event registry; historical payload/native timing not modeled.'
        else:
            domains = [
                (('AntiAliasing', 'Graphics', 'Multisample'), 'Hardware graphics-setting selection/capability state is not modeled on this headless host.'),
                (('GMEuropa', 'BNSummon', 'Commentator', 'WarGame', 'Skirmish'), 'Native support/customer-service/social matchmaking or spectator operation has no backing server workflow.'),
                (('Amplify', 'Armor', 'Cleave', 'Multistrike', 'Readiness', 'Bladed', 'Penalty'), 'Retired source-era combat stat/formula has no explicit modeled input or era-specific evaluation.'),
                (('Talent', 'Draenor', 'ShouldHideTalents'), 'Historical talent/zone-ability selection and source-era catalog are not represented by the current specialization model.'),
                (('Quest', 'Task', 'RewardCurrencies', 'Story'), 'Historical quest/task/objective query schema or current successor migration is not implemented for this exact global.'),
                (('Map', 'Continent', 'MicroDungeon', 'Closest', 'Taxi'), 'Historical geographic hierarchy/nearest-entity or taxi slot catalog has no exact modeled provider.'),
                (('EquipmentSet', 'LootInfo', 'ToyByName'), 'Historical equipment ignore-set/loot tuple or name-to-toy lookup lacks the corresponding modeled catalog and operation.'),
                (('LegacyDifficulty', 'Role'), 'Source-era group role restriction/legacy raid difficulty policy has no explicit transition or eligibility fixture.'),
            ]
            reason = next((message for fragments, message in domains
                           if any(fragment in symbol for fragment in fragments)),
                          'Exact legacy global is missing or conflicts with a later removal; recorded current publication cannot substitute for native successor/signature evidence.')
    return symbol + ': ' + reason + ' Observed: ' + observed


def prose_reason(literal):
    if 'Transcluded source:' in literal:
        return 'Diff is independently pinned and its four API inventories and explicit enum members are separately accounted; no transclusion expansion at runtime.'
    if 'Unknown' in literal:
        return 'Source itself leaves this item-bonus action unknown; no semantics invented.'
    if 'anchor' in literal.lower() or 'width or height' in literal:
        return 'Historical zero/negative-size anchor restriction conflicts with supported current layout behavior; native epoch evidence required before restricting simulator anchors.'
    if 'BonusID' in literal or 'bonusID' in literal or 'ItemBonus' in literal or 'ItemAppearance' in literal:
        return 'Historical item-bonus/link schema requires source-era DB2 and upgrade/stat/appearance evaluation; no game install or native item-bonus fixture on this host.'
    if 'GUID' in literal or 'Player-' in literal or 'BattlePet-' in literal or 'Vignette-' in literal or 'Creature-' in literal or 'Unit Type' in literal:
        return 'Historical GUID format statement retained; no complete player/pet/object/vignette generator and identity round-trip parity proof.'
    if 'CinematicModel' in literal:
        return 'CinematicModel publication is swept; 3D cinematic rendering is intentionally unsupported, not a modeled display contract.'
    if 'C_Timer' in literal or 'handle' in literal or 'callback' in literal:
        return 'Timer publication swept separately; this audit does not claim native scheduling/cancellation/GC parity.'
    if 'XML' in literal or 'keyValue' in literal or 'ParentKey' in literal or 'atlas' in literal or 'file=' in literal:
        return 'XML/atlas statement retained; no historical XML grammar/size/parent-field parity proof in this audit.'
    if 'difficulty' in literal.lower() or 'flex' in literal or 'Mythic 20' in literal:
        return 'Source-era raid difficulty/size mapping retained; no historical raid eligibility/native transition proof.'
    if any(word in literal for word in ['returns', 'return', 'args:', 'true/false', 'parameter', 'SetChecked', 'SetText', 'TargetUnit', 'SetDesaturated', 'InCombatLockdown']):
        return 'Historical argument/tuple/boolean or event-payload contract retained; publication alone does not prove this behavior.'
    return 'Historical domain/prose statement retained verbatim; addon/file migration or source-era behavior is not inferred from current publication.'


def main():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    register = json.loads((SOURCES / '6.0.2-wikitext-register.json').read_text())
    results = json.loads((HERE / 'patch_6_0_2_publication_sweep-results.json').read_text())
    decisions = {r['symbol']: r for r in json.loads((HERE / 'p602-retirement-decisions.json').read_text())['members']}
    rows, gaps = [], []
    for entry in register['entries']:
        result = results[entry['id']]
        note = ('Exact current publication/absence with later-register supersession only; no signature/output/security/domain parity.'
                if result['ok'] else gap_reason(entry, result, decisions))
        modeled = entry['symbol'] in ('C_Scenario.GetBonusSteps', 'C_Scenario.GetBonusStepRewardQuestID')
        if modeled:
            note = 'Existing ordered ScenarioState.steps backs bonus enumeration/optional rewards; bare and cached state-transition tests prove the bounded contract.'
        rows.append({'source_id': entry['id'], 'status': 'bounded-coverage' if result['ok'] else 'audit-pending',
                     'capabilities': ['scenario-bonus-state'] if modeled else (['publication-absence'] if result['ok'] else []), 'note': note})
        if not result['ok']:
            gaps.append({'source_id': entry['id'], 'symbol': entry['symbol'], 'reason': note, 'observed': result['observed']})
    text = (SOURCES / '6.0.2-api-changes.txt').read_text()
    scout = []
    for row in extractor.seed_rows(text, '6.0.2'):
        line = int(row['source_id'].rsplit('-', 1)[1])
        literal = text.splitlines()[line - 1]
        if row['status'] != 'metadata-only':
            row['note'] = prose_reason(literal)
        scout.append(dict(row, literal=literal, extract_line=line))
        rows.append(row)
    enums = json.loads((HERE / 'p602-enum-register.json').read_text())
    for entry in enums:
        rows.append({'source_id': entry['source_id'], 'status': 'audit-pending', 'capabilities': [],
                     'note': entry['symbol'] + ': historical ' + entry['direction'] + ' constant value/absence retained; current value or source-era default is not inferred. No runtime constant removal performed.'})
    # Enum group headings are editorial context, not additional runtime values.
    raw_diff = (SOURCES / '6.0.2-api-changes.diff.wikitext').read_text()
    for number, line in enumerate(raw_diff.splitlines(), 1):
        if number >= 679 and line.startswith(': [['):
            rows.append({'source_id': f'diff-enum-group-{number:03}', 'status': 'metadata-only',
                         'capabilities': [], 'note': 'Enum group heading: ' + line + '; explicit members accounted separately.'})
    dump(SOURCES / '6.0.2-page-coverage.json', {'patch': '6.0.2', 'source_revid': register['source']['revid'],
         'proof_policy': 'Current publication plus bounded scenario behavior only; every historical contract retained without fabricated parity.', 'source_rows': rows})
    dump(HERE / 'p602-gap-review.json', gaps)
    dump(HERE / 'p602-extract-scout.json', scout)
    summary = {'inventory_rows': len(register['entries']), 'inventory_gaps': len(gaps),
               'inventory_ok': len(register['entries']) - len(gaps), 'prose_rows': len(scout),
               'enum_members': len(enums), 'ledger_rows': len(rows), 'statuses': dict(Counter(r['status'] for r in rows))}
    dump(HERE / 'p602-accounting-summary.json', summary)
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
