"""Rebuild audit-owned occurrence accounting from retained publication evidence."""
import hashlib
import importlib.util
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read_json(path):
    return json.loads(path.read_text())


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


MODEL_BOUNDARIES = {
    'C_AchievementInfo': 'Guild-achievement metadata is not provided by the achievement ID/count surface.',
    'C_CampaignInfo': 'Requires campaign quest icon/sorting policy, not a constant quest visibility flag.',
    'C_ClassTalents': 'Requires config creation/import/commit/rename/delete state, capacity constraints and shared-action-bar persistence; existing talent queries are not this lifecycle.',
    'C_CraftingOrders': 'Requires customer catalog/options/favorite state and customer interaction lifecycle; existing local order records do not supply these contracts.',
    'C_CurrencyInfo': 'Requires currency description metadata, distinct from count/name/quantity publication.',
    'C_EditMode': 'Requires a documented layout serialization format and layout add/delete/exit lifecycle; current anchors are not serialized layouts.',
    'C_GossipInfo': 'Requires optionID-based gossip selection and its resulting interaction/quest transition, not a no-op selector.',
    'C_Item': 'Requires inventory GUID identity and location resolution shared with real item inputs, not a made-up GUID decoder.',
    'C_LegendaryCrafting': 'Requires populated Runeforge modifier DTO metadata; a nil/empty tuple is not the changed description-table contract.',
    'C_MajorFactions': 'Requires weekly renown cap policy and character progress inputs, distinct from faction identity.',
    'C_Minimap': 'Requires tracking/filter state, battle-pet/hidden-quest capability inputs and POI world-effect/texture metadata.',
    'C_MountJournal': 'Requires dragonriding classification of the collected mount inventory, not all collected mounts.',
    'C_MythicPlus': 'Requires display-season identity inputs distinct from a generic season number.',
    'C_PaperDollInfo': 'Requires inspected player rated-solo-shuffle DTOs, not local-player stats.',
    'C_PlayerInteractionManager': 'Requires NPC interaction validation, confirmation, replacement and reopening transitions tied to unit/NPC state.',
    'C_ProfSpecs': 'Requires profession specialization path/perk/tab graph, entries/currencies/ranks and refund/unlock eligibility state.',
    'C_PvP': 'Requires solo-shuffle per-spec rating/reward/minimum-item-level/brawl inputs and policies.',
    'C_QuestItemUse': 'Requires quest-item/object eligibility derived from quest and object inputs, not unconditional permission.',
    'C_QuestLog': 'Requires quest-to-unit relationship and major-faction reputation reward metadata.',
    'C_QuestOffer': 'Requires offered-quest major-faction reputation reward DTOs.',
    'C_Texture': 'Requires fileDataID-to-filename metadata, not forward asset resolution or invented paths.',
    'C_TradeSkillUI': 'Requires profession recipe/reagent quality, operation/recraft/salvage/enchant catalogs or learned/unlearned/first-craft filter state specific to this endpoint; existing recipe publication is insufficient.',
    'C_Traits': 'Requires config staging/commit/rollback, refund/repurchase history and inspect/talent-test state transitions; current read-only trait records do not implement these operations.',
    'C_UIWidgetManager': 'Requires populated FillUpFrames visualization DTOs and their owner widget inputs.',
    'C_WeeklyRewards': 'Requires weekly chest retirement policy/season inputs, not a generic vault reward count.',
    'C_XMLUtil': 'Requires enumeration of runtime XML template identities, not a hardcoded empty list.',
}
GLOBAL_BOUNDARIES = {
    'CheckTalentMasterDist': 'Current expectation requests publication after later supersession; talent-master distance needs NPC/range interaction state.',
    'GetGraphicsCVarValueForQualityLevel': 'Requires graphics quality-level profiles and CVar value mapping; renderer defaults do not supply these profiles.',
    'GetUnitEmpowerMinHoldTime': 'Requires empower minimum-hold timing from the unit spell cast state.',
    'IsGraphicsCVarValueSupported': 'Requires device/profile-specific graphics CVar value capability validation.',
    'IsGraphicsSettingValueSupported': 'Requires device/profile-specific graphics setting capability validation.',
    'IsPlayerInGuildFromGUID': 'Requires GUID-to-guild membership resolution, not local-player guild membership.',
    'IsSpecializationActivateSpell': 'Requires spell-to-specialization activation metadata.',
    'JoinRatedSoloShuffle': 'Requires a rated solo-shuffle queue admission/state model, not a no-op join.',
    'ReleaseAction': 'Requires action key press/release state and execution semantics.',
    'SetUnitCursorTexture': 'Requires unit cursor texture state consumed by cursor rendering.',
    'TargetToggle': 'Requires target toggle selection semantics and associated state/events.',
    'UnitIsInteractable': 'Requires unit interaction eligibility inputs and range/restriction policy.',
}
WIDGET_BOUNDARIES = {
    'ModelSceneActor:SetModelByHyperlink': 'Hyperlink-to-3D-model loading belongs to the intentionally unsupported 3D domain; no new permanent stub is credited.',
    'MovieFrame:StartMovieByName': 'Requires named movie lookup/playback and completion behavior; publication cannot be fabricated.',
    'OffScreenFrame:TestPrintToFile': 'Native offscreen diagnostic/file-output contract is not modeled and must not silently perform filesystem writes.',
    'Path:GetCurveType': 'Animation owner constructs successfully, but curve-type query is absent; path curve state/contract is not established by generic animation fields.',
    'Path:GetMaxControlPointOrder': 'Animation owner constructs successfully, but ordered control-point limit query is absent; control-point contract requires behavioral work.',
    'Path:SetCurveType': 'Animation owner constructs successfully, but curve-type mutation is absent; requires interpolation semantics, not an ignored setter.',
    'Scale:GetScaleFrom': 'Animation owner constructs successfully; new getter absent despite existing setter. Shared scale axis/time semantics require a separate behavioral regression before implementation.',
    'Scale:GetScaleTo': 'Animation owner constructs successfully; new getter absent despite existing setter. Shared scale axis/time semantics require a separate behavioral regression before implementation.',
    'WorldFrame:OnModelCleared': 'Existing WorldFrame lookup is nil; page presents a method-like widget row, distinct from the separately proven actor HasScript row. Do not turn a callback into a callable stub.',
}


def review_gap(entry, observation, consumer):
    symbol = entry['symbol']
    section = entry['section']
    removed = observation['expected']['publication'] == 'absent'
    if removed:
        qualified = consumer['scans']['qualified']['matches'] if consumer else []
        bare = consumer['scans']['bare']['matches'] if consumer else []
        if qualified:
            reason = ('Current cached Lua contains exact qualified references listed below; retained, not retired. '
                      'References may be declarations/deprecation wrappers or profile-specific files; a textual match alone is not proof of a loaded retail caller. Preserve vendor wrappers and current callers.')
        elif bare:
            reason = ('No exact namespace/owner-qualified cache match; bare-name references listed below require successor/deprecation/profile attribution. '
                      'Do not describe bare matches as old API consumers. Deferred outside a zero-match retirement batch.')
        else:
            reason = ('Qualified and bare-name cached Lua searches both empty. Retirement remains deferred: this accounting-first audit does not change runtime registration or shared classic methods. '
                      'A separate retail-only retirement with repeated-lookup and exact caller migration proof is required; no live consumer is falsely claimed.')
    elif section == 'cvars':
        reason = ('Console Command catalog lacks this native diagnostic operation; requires meaningful FPS logging/customization export behavior, not a fabricated catalog record.'
                  if entry.get('kind') == 'command' else
                  'Historical CVar is absent from current value/default registry; current retail publication policy/default provenance needs establishing before registering an obsolete variable.')
    elif section == 'widgets':
        reason = WIDGET_BOUNDARIES[symbol]
    else:
        reason = GLOBAL_BOUNDARIES[symbol] if '.' not in symbol else MODEL_BOUNDARIES[symbol.split('.')[0]]
    return {'source_id': entry['id'], 'symbol': symbol, 'direction': entry['direction'],
            'expectation': observation['expected'], 'observed': observation['observed'],
            'outcome': 'retained-gap', 'reason': reason,
            'cached_references': consumer['scans'] if consumer else {},
            'source_evidence': 'p1000-gap-source-scans.json#' + entry['id'],
            'current_metadata_context': 'p1000-native-contracts.json#' + entry['id']}


def main():
    register = read_json(SOURCES / '10.0.0-wikitext-register.json')
    observations = read_json(EVIDENCE / '10.0.0-sweep.json')
    consumers = read_json(EVIDENCE / 'p1000-exact-removal-consumers.json')
    gaps = {key: value for key, value in observations.items() if not value['ok']}
    assert set(observations) == {entry['id'] for entry in register['entries']}
    review = [review_gap(entry, observations[entry['id']], consumers.get(entry['id']))
              for entry in register['entries'] if entry['id'] in gaps]
    write_json(EVIDENCE / 'p1000-gap-review.json', review)
    write_json(ROOT / 'tests/data/patch_10_0_0_sweep_known_gaps.json', sorted(gaps))
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text_path = SOURCES / '10.0.0-api-changes.txt'
    text = text_path.read_text()
    extract_rows = extractor.seed_rows(text, '10.0.0')
    lines = text.splitlines()
    section = 'source context'
    scout = []
    for row in extract_rows:
        line_number = int(row['source_id'].rsplit('-', 1)[1])
        literal = lines[line_number - 1]
        if literal.startswith('='):
            section = literal.strip('= ').strip()
        if row['status'] == 'audit-pending':
            row['note'] = (f'Literal {section} occurrence retained; {literal.strip()!r}. '
                           'Requires occurrence-specific behavioral/identity proof; adjacent publication does not prove this statement or example.')
        scout.append({'source_id': row['source_id'], 'extract_line': line_number,
                      'literal': literal, 'section': section, 'status': row['status'],
                      'reason': row['note'], 'behavioral_credit': False})
    write_json(EVIDENCE / 'p1000-extract-scout.json', scout)
    raw_lines = (SOURCES / '10.0.0-api-changes.wikitext').read_text().splitlines()
    # Inventory extraction intentionally excludes these lines; account without rewriting prior extracts.
    context = [{'source_id': f'wt-context-{number}', 'wikitext_line': number,
                'literal': raw_lines[number - 1], 'capabilities': [],
                'status': 'metadata-only' if number == 220 else 'audit-pending',
                'note': 'Build transition caption; no runtime credit.' if number == 220 else
                'Inventory-section prose excluded by shared extractor; retained literally here. Successor migration behavior remains unproved.'}
               for number in (220, 594, 595, 596)]
    write_json(EVIDENCE / 'p1000-inventory-context.json', context)
    inventory = []
    for entry in register['entries']:
        observation = observations[entry['id']]
        gap = next((row for row in review if row['source_id'] == entry['id']), None)
        inventory.append({'source_id': entry['id'],
                          'capabilities': [] if gap else ['publication-sweep-10-0-0'],
                          'status': 'audit-pending' if gap else 'bounded-coverage',
                          'note': gap['reason'] if gap else
                          'Current cached publication/absence/event registration/CVar queries only. Signature annotations, populated outputs, event payloads, security and native/historical parity remain unproved.'})
    rows = inventory + extract_rows + [{key: row[key] for key in ('source_id', 'capabilities', 'status', 'note')} for row in context]
    assert len(rows) == len({row['source_id'] for row in rows})
    coverage = {'schema': 'patch-page-coverage/v1', 'patch': '10.0.0',
                'source_register': 'data/patch-api/sources/10.0.0-wikitext-register.json',
                'source_sha256': hashlib.sha256((SOURCES / '10.0.0-wikitext-register.json').read_bytes()).hexdigest(),
                'audit_status': 'in-progress',
                'proof_policy': 'Complete occurrence accounting, not complete compatibility. Current retail 12.1.0 with all 25 later registers; publication-only bounded credit. Pending prose/examples/annotations are not behavior proof. No production API changes or independent agent/model acceptance.',
                'capabilities': [{'id': 'publication-sweep-10-0-0', 'symbols': ['639 inventory occurrences'],
                                  'scope': 'Current cached publication/absence/event registration/CVar queries only.',
                                  'spec': 'docs/specs/patch-10-0-0-publication-sweep.md',
                                  'tests': ['tests/patch_10_0_0_publication_sweep.rs'],
                                  'compiled_revision': 'aa860f1d5',
                                  'proof': '26 current-scope sweeps, factory regression, exact negative control, helper regressions, format/default/Mists checks and exit-0 startup [].',
                                  'ledger': 'data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-proof.json'}],
                'non_inventory_source': {'path': str(text_path.relative_to(ROOT)),
                                         'sha256': hashlib.sha256(text_path.read_bytes()).hexdigest(),
                                         'wikitext_revid': 6789768, 'wikitext_sha256': register['source']['sha256'],
                                         'extract_tool': 'tools/extract_patch_non_inventory.py', 'preserve_examples': True},
                'inventory_context_source': 'data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-inventory-context.json',
                'source_rows': rows}
    write_json(SOURCES / '10.0.0-page-coverage.json', coverage)
    print(json.dumps({'inventory': len(inventory), 'extract': len(extract_rows), 'inventory_context': len(context),
                      'gaps': len(gaps), 'ledger': len(rows), 'statuses': dict(Counter(row['status'] for row in rows))}, indent=2))


if __name__ == '__main__':
    main()
