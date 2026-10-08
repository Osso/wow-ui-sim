"""Account every inventory/extract identity without turning publication into parity."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'

PROSE = {
    4: 'Line region publication is exercised through its real frame factory. UiCamera publication is present but native camera/model behavior and 3D rendering are intentionally unsupported; inheritance/native behavior is not proved.',
    5: 'Region:GetDebugName publication is present; page gives no naming format or unnamed-region debug identity contract. Publication is not exact formatting parity.',
    6: 'CreateLine and flatten-layer getters/setters are published. Complete flatten-rendering semantics and historical draw-order parity are outside this publication test; no rendering credit inferred.',
    7: 'SetColorTexture is published. Existing color storage is not an independent native Legion visual or legacy numeric-SetTexture comparison.',
    8: 'AdventureMap namespace publication is present. No historical Legion adventure-map mission fixture or scenario flow is supplied by this page; complete new-addon behavior is not proved.',
    9: 'ArtifactUI namespace publication is present. No historical artifact unlock/power/relic fixture or Legion server lifecycle is supplied; no domain implementation inferred from its name.',
    10: 'Transmog and TransmogCollection namespaces are published. Historical wardrobe acquisition/appearance/cost behavior is not specified or independently probed here.',
    12: 'TradeSkillUI namespace and explicitly listed methods are accounted independently. The implemented name filter has concrete backing behavior; remaining recipe/source/category/favorite/Obliterum contracts need distinct models/fixtures. Table presence is not full domain parity.',
    13: 'GetNumQuestLogRewardSpells publication does not establish dynamic quest reward spell cardinality or native reward fixtures; this page supplies no concrete quest reward data.',
    16: 'Literal item-link example is preserved. Serializer-wide omission of zero fields and compatibility across actual item producers are not established by publication tests; no native item-string probe is available.',
    17: 'Complete nameplate creation changed without a lifecycle contract. The simulator intentionally has no 3D world nameplate renderer; no event/frame lifecycle model is fabricated.',
    18: 'Blizzard_Nameplates is named as an addon but historical Legion contents are not in the active retail cache. Addon naming alone is not historical loading or lifecycle proof.',
    19: 'All three nameplate events are registered and publication-probed. Creation/removal payload ordering and 3D plate association have no supported world-nameplate backing system.',
    20: 'Source calls the destination C_Nameplates, while its explicit members use C_NamePlate. Preserve this spelling discrepancy; do not fabricate the plural namespace or infer retirement of old globals from a migration summary.',
    21: 'SetColorTexture publication exists. Source says numeric SetTexture fails in many cases but does not identify the exact failing cases; no guessed legacy overload restriction added.',
    22: 'Explicit tiling methods are publication-accounted. Source qualifies replacements as possibly intended and omits precise UV/clamping behavior; no guessed GPU/native tiling compatibility claim.',
    23: 'SetToFileData is already absent and SetTexture is published. FileDataID texture resolution/native output requires a WoW CASC install unavailable on this host; no runtime or vendor workaround added.',
    24: 'GetSpellInfo publication does not independently prove third-return fileDataID for historical spell fixtures; current spell metadata/native probes are outside this bounded sweep.',
    25: 'Alpha endpoints use the real animation factory. SetChange remains because current animation behavior tests call it; deleting shared dispatch would regress live callers. Historical removal remains an exact gap, not hidden by a shim.',
    26: 'GetInboxItem is published; exact six-return mail layout with itemID needs concrete native mailbox input and behavioral parity proof. Publication alone receives no mail contract credit.',
    27: 'GetSendMailItem is published; exact five-return attachment layout with itemID needs concrete native send-mail input and parity proof. Publication alone receives no mail contract credit.',
    28: 'GetNumDungeonMapLevels is absent; current map backing data does not model the removed historical dungeon floor-array API. No guessed array or fake count added.',
    29: 'GetMapLandmarkInfo is absent; historical landmark index/type ordering has no concrete archive map fixture. Modern map/area-POI queries are not assumed equivalent.',
    30: 'Third-party Minimap creation/XML rejection is a loader ownership/security contract, not mere CreateFrame publication. It is not proved here; do not change shared frame creation from this summary alone.',
    31: 'RecipeReagentItem successor is published. SetTradeSkillItem is retained for live tooltip tests; historical recipe-index versus recipeID translation and reagent fixtures are not proved.',
    32: 'GetQuestLogRewardSpell is published, but required-index validation/return/security parity is not exercised with native quest rewards. No fabricated reward fixture added.',
    33: 'Garrison wildcard migration names neither concrete members, LE_* constants nor new return fields. No guessed changes to the current Garrison model or current consumers.',
    34: 'Mount rename summary is expanded into the three explicit old/successor pairs below; no additional unnamed function is inferred.',
    35: 'Consumer-free GetMountInfo is retired from both raw and synthesized lookup. GetMountInfoByID stays published; no historical display-index versus ID behavioral equivalence claim.',
    36: 'Consumer-free GetMountInfoExtra is retired from both raw and synthesized lookup. GetMountInfoExtraByID stays published; no historical extra-info tuple parity claim.',
    37: 'SummonByID is published; Summon remains for live collection callers. Historic old-index versus ID summoning behavior is not proved or deleted to silence this sweep.',
    40: 'Three explicit glyph names are publication-accounted; etc. has no member catalog. Cached InspectUI uses a SetGlyph method, so broad name-based removal is unsafe; no unnamed glyph model or deletion inferred.',
    41: 'Both gem globals are already absent. Parenthetical itemString extraction is qualified by apparently and gives no exact format/parsing contract; no gem parser parity inferred.',
    42: 'GetQuestLogRewardTalents is already absent; no code retirement required.',
    43: 'All four helm/cloak globals are explicitly re-added in the 12.0.0 register and have current modeled transition callers. Preserve later semantics; do not retire from this historical page.',
    44: 'Multistrike and Amplify are unnamed domains, not member catalogs. No API identities, return contracts or replacement semantics are supplied; no guessed retirements.',
    45: 'Frame:AllowAttributeChanges is already absent; no shared frame dispatch change required.',
    46: 'UNIT_COMBO_POINTS is already unregistered. UNIT_POWER is named as a successor, but old/new event payload ordering is not supplied or modeled by this publication check.',
    47: 'floatingCombatTextComboPoints is already absent after later removal. Successor CVar default/behavior and combat text routing are not specified by this historical summary.',
    48: 'GetModel remains for live model state tests; GetModelFileID is published. Native 3D behavior is intentionally unsupported and shared state callers must not be broken to force historical absence.',
}
METADATA_PROSE = {34}
BOUNDED_PROSE = {35: 'mount-retirement-lookup', 36: 'mount-retirement-lookup',
                 42: 'current-retail-absence', 45: 'current-retail-absence',
                 43: 'later-register-publication-supersession'}


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def gap_reason(row):
    symbol = row['expected']['symbol']
    if symbol in ('Alpha:SetChange', 'Model:GetModel', 'GameTooltip:SetTradeSkillItem', 'C_MountJournal.Summon'):
        return 'Historical removal conflicts with current concrete src/tests callers retained in whole-word scans; no deletion or vendor patch. ' + row['observed']['detail']
    if row['expected']['publication'] == 'absent':
        return ('Later register ' + str(row['expected']['superseded_by']) +
                ' requests absence, but ordinary lookup still synthesizes/retains a function. No later-page retirement without its own current-consumer review. ' + row['observed']['detail'])
    if symbol.startswith('C_NamePlate.'):
        return 'Historical plural size API has no modeled 3D world-nameplate size/lifecycle system; singular current shim is not an equivalent implementation. No new shim. ' + row['observed']['detail']
    if 'Obliterate' in symbol or 'Obliterum' in symbol:
        return 'Legion Obliterum forge has no cursor conversion, eligibility, yield/cost or forge lifecycle model/fixture. No guessed value or no-op producer. ' + row['observed']['detail']
    if symbol.startswith('C_TradeSkillUI.'):
        return 'No concrete raw producer/backing state for this historical recipe filter/category/source/favorite/search/repeat/output contract; name search is modeled separately, not a substitute. ' + row['observed']['detail']
    if symbol in ('GetNameplateFrames', 'GetNumNamePlateMotionTypes'):
        return 'Old global is absent; migration summary does not explicitly retire it. No historical nameplate motion/frame catalog or 3D plate lifecycle; retain exact source gap.'
    if symbol == 'GetNumDungeonMapLevels':
        return 'Historical dungeon floor-array API absent; no archive dungeon floor fixture in the current map model.'
    if symbol == 'GetMapLandmarkInfo':
        return 'Historical landmark index/type tuple absent; no archive landmark fixture or demonstrated modern equivalent.'
    raise ValueError('unclassified gap: ' + symbol)


def main():
    register_path = SOURCES / '7.0.3-wikitext-register.json'
    register = read(register_path)
    results = read(HERE / 'p703-own-fixed-results.json')
    known = set(read(ROOT / 'tests/data/patch_7_0_3_sweep_known_gaps.json'))
    assert set(results) == {entry['id'] for entry in register['entries']}
    assert {key for key, row in results.items() if not row['ok']} == known
    dump(HERE / 'patch_7_0_3_publication_sweep-results.json', results)
    rows, gaps = [], []
    for entry in register['entries']:
        key = entry['id']
        reason = gap_reason(results[key]) if key in known else 'Current-retail publication/absence only; no signature, output, security or historical behavior parity credit.'
        caps = [] if key in known else ['current-retail-publication-or-absence']
        if entry['symbol'] == 'C_TradeSkillUI.SetRecipeItemNameFilter' and key not in known:
            caps.append('state-backed-recipe-name-filter')
            reason = 'Concrete name-filter state, nil clear, case-insensitive existing learned catalogue search and list-update dispatch; cached and bare behavioral proof. Other filters/native locale collation not implied.'
        rows.append({'source_id': key, 'status': 'audit-pending' if key in known else 'bounded-coverage',
                     'capabilities': caps, 'note': reason})
        if key in known:
            gaps.append({'source_id': key, 'symbol': entry['symbol'], 'reason': reason,
                         'expected': results[key]['expected'], 'observed': results[key]['observed']})
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text_path = SOURCES / '7.0.3-api-changes.txt'
    text = text_path.read_text()
    raw_lines = (SOURCES / '7.0.3-api-changes.wikitext').read_text().splitlines()
    rendered = [extractor.render_line(re.sub(r'^(=+)\s*(.*?)\s*\1$', r'\1 \2 \1', line)) for line in raw_lines]
    scout, cursor = [], 0
    for seed in extractor.seed_rows(text, '7.0.3'):
        number = int(seed['source_id'].rsplit('-', 1)[1])
        literal = text.splitlines()[number - 1]
        while cursor < len(rendered) and rendered[cursor] != literal:
            cursor += 1
        assert cursor < len(rendered), literal
        raw_number = cursor + 1
        cursor += 1
        if seed['status'] != 'metadata-only':
            reason = PROSE[number]
            status = 'metadata-only' if number in METADATA_PROSE else ('bounded-coverage' if number in BOUNDED_PROSE else 'audit-pending')
            seed.update(status=status, capabilities=[BOUNDED_PROSE[number]] if number in BOUNDED_PROSE else [], note=reason)
        rows.append(seed)
        scout.append({'source_id': seed['source_id'], 'extract_line': number,
                      'wikitext_line': raw_number, 'literal': literal,
                      'raw_literal': raw_lines[raw_number - 1], 'status': seed['status'],
                      'reason': seed['note']})
    dump(SOURCES / '7.0.3-page-coverage.json', {
        'schema': 'patch-api-page-coverage/v1', 'patch': '7.0.3',
        'source': str(register_path.relative_to(ROOT)),
        'source_sha256': hashlib.sha256(register_path.read_bytes()).hexdigest(),
        'non_inventory_source': {'path': str(text_path.relative_to(ROOT)), 'sha256': hashlib.sha256(text_path.read_bytes()).hexdigest()},
        'proof_policy': 'Every retained occurrence accounted; publication is not behavioral/native parity. Only named consumer-free removals; 7.1.0 first-position integration placeholder remains.',
        'source_rows': rows,
    })
    dump(HERE / 'p703-extract-scout.json', scout)
    dump(HERE / 'p703-gap-review.json', gaps)
    dump(HERE / 'p703-problematic-contracts.json', [row for row in scout if row['status'] == 'audit-pending'])
    dump(HERE / 'p703-accounting-summary.json', {
        'inventory_rows': len(register['entries']), 'inventory_ok': len(results) - len(known),
        'inventory_gaps': len(known), 'extract_rows': len(scout), 'ledger_rows': len(rows),
        'ledger_statuses': dict(Counter(row['status'] for row in rows)),
        'pending_prose': sum(row['status'] == 'audit-pending' for row in scout),
    })
    print((HERE / 'p703-accounting-summary.json').read_text())


if __name__ == '__main__':
    main()
