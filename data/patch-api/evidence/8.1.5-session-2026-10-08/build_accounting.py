"""Build occurrence accounting from retained source and observed sweep output."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '8.1.5'


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


REASONS = {
    'BNGetDisplayName': 'Missing Battle.net display-name lookup by GUID/account identity. Needs account display-name data and invalid/missing identity behavior, not a player-name constant.',
    'C_AccountInfo.GetIDFromBattleNetAccountGUID': 'Missing Battle.net account-GUID to account-ID mapping and validation. Character GUID parsing does not supply the account identity producer.',
    'C_ChatInfo.CanReportPlayer': 'No qualified cached consumer, but two bare CanReportPlayer consumers under C_ReportSystem prohibit retirement under the conservative scan rule. Requires namespace-specific migration/publication policy, not an autostub tombstone.',
    'C_Club.SetCommunityID': 'Historical removal superseded by 10.2.7 addition wt-global-api-C_Club.SetCommunityID-52. Zero cached consumers; initial test match is a gap fixture only. Needs community selection state/model. A trial retirement could not satisfy the later published expectation and was reverted; prior inputs restored.',
    'C_EncounterJournal.IsEncounterComplete': 'Missing encounter completion lookup for the selected journal/instance context. Requires completion history keyed by encounter and instance/difficulty, not a false-return shim.',
    'C_MerchantFrame.IsMerchantItemRefundable': 'Missing merchant item refund eligibility/expiry metadata and current merchant index lookup. Ownership or price alone does not establish refundability.',
    'C_PlayerInfo.UnitIsSameServer': 'Missing unit-to-realm/server identity relationship. Existing fixed local SimRealm is not a populated remote-unit realm producer; connected-realm and invalid-token semantics need evidence.',
    'C_PvP.GetAvailableBrawlInfo': 'Missing available brawl schedule/eligibility and brawl DTO producer. Existing battleground fixture does not model the next available brawl.',
    'C_ReportSystem.InitiateReportPlayer': 'Later 9.2.5 removal expects absence, but modeled report-token production and help/report/XML tests still use this legacy API. Requires explicit successor migration; not unused-member cleanup.',
    'C_ReportSystem.SendReportPlayer': 'Later 9.2.5 removal expects absence, but modeled report-token submission and help/report/XML tests still use this legacy API. Requires explicit successor migration; not unused-member cleanup.',
    'C_UIWidgetManager.GetDoubleStateIconRowVisualizationInfo': 'Missing native double-state icon-row widget DTO, paired icon states and visibility/layout metadata. Generic widget defaults cannot supply populated visualization data.',
    'InActiveBattlefield': 'No cached consumer; existing inert-global registration and a Mists world-map caller remain. Needs retail-only publication removal with classic preservation, not cross-profile deletion.',
    'UnitPvpClassification': 'Missing per-unit PvP classification state and enum mapping (honor/leader/flag-carrier context), including invalid/absent tokens. Unit reaction is not this classification.',
    'UnitSelectionType': 'Missing per-unit selection/highlight category state and native selection-type mapping. Friendly/hostile unit predicates do not determine every selection class.',
    'UnitWidgetSet': 'Missing unit-to-widget-set association and lifecycle. Global UI widget-set data cannot identify a particular unit widget set.',
    'gcinfo': 'Native rilua Lua 5.1 primitive remains published. Zero cached/source callers, but removal needs retail-only VM/global publication policy with classic preservation; not namespace autostub cleanup.',
}


def gap_reason(symbol):
    if symbol in ('GetCVar', 'GetCVarBitfield', 'GetCVarBool', 'GetCVarDefault',
                  'RegisterCVar', 'ResetTestCvars', 'SetCVar', 'SetCVarBitfield'):
        return ('Whole-word cached bare-name consumers remain (including C_CVar namespace/deprecation references); simulator globals and tests also call this CVar API. Consumer rule forbids retirement. Requires explicit legacy-global/deprecation ownership and caller migration, not deleting shared CVar state.')
    return REASONS[symbol]


def extract_scout(extractor, raw_lines, text):
    scout = []
    cursor = 0
    for row in extractor.seed_rows(text, PATCH):
        number = int(row['source_id'].rsplit('-', 1)[1])
        literal = text.splitlines()[number - 1]
        for index in range(cursor, len(raw_lines)):
            raw_literal = raw_lines[index]
            rendered = extractor.render_line(re.sub(r'^(=+)\s*(.*?)\s*\1$', r'\1 \2 \1', raw_literal))
            if rendered == literal:
                cursor = index + 1
                break
        else:
            raise AssertionError(('no raw occurrence', literal))
        reason = ('Unspecified ? marker: page supplies no widget/change/CVar contract to implement. Retained as editorial incomplete-source context, not behavior coverage.'
                  if literal == '?' else
                  'Namespace migration note; exact members are independently inventoried. No behavioral credit.'
                  if literal.startswith('New C_') else
                  'Editorial heading/build/deprecated-source link; linked pages not expanded and no runtime credit.')
        scout.append({'source_id': row['source_id'], 'extract_line': number,
                      'wikitext_line': index + 1, 'literal': literal,
                      'raw_literal': raw_literal, 'status': 'metadata-only', 'reason': reason})
    return scout


def build():
    register_path = SOURCES / f'{PATCH}-wikitext-register.json'
    register = read(register_path)
    results = read(EVIDENCE / 'patch_8_1_5_publication_sweep-results.json')
    raw_lines = (SOURCES / f'{PATCH}-api-changes.wikitext').read_text().splitlines()
    text_path = SOURCES / f'{PATCH}-api-changes.txt'
    rows, reviews = [], []
    for entry in register['entries']:
        result = results[entry['id']]
        if not result['ok']:
            reason = gap_reason(entry['symbol'])
            reviews.append({'source_id': entry['id'], 'symbol': entry['symbol'],
                            'wikitext_line': entry['wikitext_line'],
                            'literal': raw_lines[entry['wikitext_line'] - 1],
                            'expectation': result['expected'], 'observation': result['observed'],
                            'reason': reason})
            status, capabilities = 'audit-pending', []
        elif entry['symbol'] == 'C_ToyBoxInfo.NeedsFanfare':
            status, capabilities = 'bounded-coverage', ['current-retail-publication', 'toy-acquisition-fanfare-clear']
            reason = 'Shared acquisition/clear backing state agrees with GetToyInfo fifth result. Lua integration proves collection transitions; native NEW_TOY_ADDED event/payload and persistence parity not claimed.'
        elif result['expected']['publication'] == 'absent':
            status, capabilities = 'bounded-coverage', ['current-retail-absence']
            reason = 'Current-retail absence/deprecation provenance only; no historical behavior credit.'
        else:
            status, capabilities = 'partial-development-green', ['current-retail-publication']
            reason = 'Publication/event registration only. Existing functions may be compatibility defaults; no populated-output, security, signature, payload or historical parity credit.'
        if result['expected']['superseded_by']:
            reason += ' Superseded by ' + result['expected']['superseded_by'] + '.'
        rows.append({'source_id': entry['id'], 'status': status, 'capabilities': capabilities, 'note': reason})
    scout = extract_scout(load_extractor(), raw_lines, text_path.read_text())
    rows.extend({'source_id': row['source_id'], 'status': row['status'],
                 'capabilities': [], 'note': row['reason']} for row in scout)
    dump(SOURCES / f'{PATCH}-page-coverage.json', {
        'schema': 'patch-api-page-coverage/v1', 'patch': PATCH,
        'source': str(register_path.relative_to(ROOT)), 'source_sha256': sha(register_path),
        'non_inventory_source': {'path': str(text_path.relative_to(ROOT)), 'sha256': sha(text_path)},
        'proof_policy': 'Exhaustive occurrence accounting. Bounded toy/admin behavior and consumer-free absence only; publication does not prove native/historical parity. Incomplete ? markers retained without invented contracts.',
        'source_rows': rows})
    dump(EVIDENCE / 'p815-gap-review.json', reviews)
    dump(EVIDENCE / 'p815-extract-scout.json', scout)
    dump(EVIDENCE / 'p815-build-context.json', [])
    summaries = []
    for path in sorted(SOURCES.glob('*-wikitext-register.json')):
        patch = path.name.removesuffix('-wikitext-register.json')
        result_path = EVIDENCE / ('patch_' + patch.replace('.', '_') + '_publication_sweep-results.json')
        observed = read(result_path)
        known = read(ROOT / ('tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        summaries.append({'patch': patch, 'rows': len(observed), 'ok': sum(row['ok'] for row in observed.values()),
                          'gaps': len(known), 'exact_gap_identity': sorted(key for key, row in observed.items() if not row['ok']) == sorted(known)})
    dump(EVIDENCE / 'p815-sweep-summary.json', summaries)
    print(json.dumps({'inventory': len(register['entries']), 'extract': len(scout), 'ledger': len(rows), 'gaps': len(reviews)}))


if __name__ == '__main__':
    build()
