"""Rebuild occurrence accounting from retained source and actual sweep observations."""
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
PATCH = '8.2.5'


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


REASONS = {
    'DefragmentGPU': 'Command record absent. Simulator GPU atlas allocation has no native console GPU-memory defragmentation operation; adding a command record/no-op would not model it.',
    'BNGetFriendInfo': 'Legacy inert_global_defaults registration and system_api_seeded call remain. No cached consumer, but deleting this cross-profile global needs a separate classic-safe registration/test migration; not treated as unused namespace cleanup.',
    'C_Calendar.EventRemoveInviteByGuid': 'No raw GUID-based invite mutation. Closure needs calendar-event invitation ownership, GUID-to-invite mapping and update-event semantics; indexed editor calls are not proof of GUID removal.',
    'C_ClubFinder.GetPlayerSettingsFocusFlagsSelectedCount': 'No raw selection-count producer. Focus-bit selection/storage and the selected-count rule need a concrete populated settings model; namespace lookup fabrication is not a count implementation.',
    'C_ClubFinder.GetRecruitingClubInfoFromClubID': 'No raw club-ID recruiting DTO producer. Requires a populated club-ID-to-posting index with missing-ID semantics; existing GUID queries do not prove this lookup.',
    'C_ClubFinder.GetRecruitingClubInfoFromFinderGUID': 'No raw finder-GUID recruiting DTO producer. Requires populated posting identity mapping and a complete recruiting-info result, not an empty table.',
    'C_Commentator.GetIndirectSpellID': 'No raw tracked-to-indirect spell mapping. Requires spectator spell relationships and unknown-ID behavior; returning the input would invent the relationship.',
    'C_Commentator.GetMatchDuration': 'No raw match clock producer. Needs start/end snapshots and clock units/lifecycle, rather than an arbitrary duration constant.',
    'C_Commentator.GetPlayerAuraInfo': 'No raw spectator aura DTO producer. Needs team/player identity and aura-instance state; normal player aura state is not a spectator team mapping.',
    'C_Commentator.GetTrackedSpellID': 'No raw indirect-to-tracked spell mapping. Requires spectator canonical spell identity and missing-ID behavior.',
    'C_Commentator.GetTrackedSpells': 'No raw spectator tracked-spell collection. Needs concrete per-player tracked spell lists and defensive/offensive categorization.',
    'C_Commentator.IsTrackedSpell': 'No raw tracked-membership predicate. Requires the tracked-spell collection and category policy; a constant boolean is not modeled membership.',
    'C_Commentator.SetRequestedDebuffCooldowns': 'No raw spectator request mutation. Needs requested-debuff selection storage and the resulting cooldown delivery lifecycle.',
    'C_LFGInfo.GetRoleCheckDifficultyDetails': 'No raw role-check difficulty DTO. Needs active role-check context, difficulty metadata and missing-context outputs; generic LFG activity rows do not establish this result.',
    'C_PartyInfo.CanInvite': 'No raw invitation-eligibility predicate. Roster count alone lacks leader/assistant, target, restriction and pending-invite policy; a capacity-only boolean would claim incomplete eligibility.',
    'C_PartyInfo.ConfirmConvertToRaid': 'No raw conversion-confirmation mutation. Needs a pending conversion prompt and accepted/declined transition, not direct conversion or no-op acknowledgment.',
    'C_PartyInfo.ConfirmInviteTravelPass': 'No raw Battle.net travel-pass confirmation. Needs pending account invitation identity and acceptance/decline request delivery.',
    'C_PartyInfo.ConfirmInviteUnit': 'No raw unit-invite confirmation. Needs pending unit identity, acceptance/decline and roster/invitation events.',
    'C_PartyInfo.ConfirmLeaveParty': 'No raw leave-confirmation mutation. Needs pending leave prompt ownership and confirmed/declined state transition; existing LeaveParty is not prompt confirmation.',
    'C_PartyInfo.ConfirmRequestInviteFromUnit': 'No raw request-invite confirmation. Needs pending request identity and accepted/declined delivery, rather than a generic invite call.',
    'C_PartyInfo.ConvertToParty': 'No raw raid-to-party mutation. Requires conversion eligibility, excess-member handling and roster-update lifecycle; namespace autostub does not change group state.',
    'C_PartyInfo.ConvertToRaid': 'No raw party-to-raid mutation. Requires leadership eligibility and coherent raid roster transition/events.',
    'C_PartyInfo.InviteUnit': 'No raw unit-invitation producer. Requires pending invitations, target resolution and eligibility/error events; adding roster members immediately is not an invitation.',
    'C_PartyInfo.RequestInviteFromUnit': 'No raw outgoing join-request producer. Requires request target/expiry/response state rather than a no-op or immediate roster join.',
    'C_PvP.GetMatchPVPStatColumns': 'No raw match-stat column DTO producer. Needs active-match column definitions and ordered statistic metadata; score values alone cannot reconstruct columns.',
    'C_QuestLog.GetQuestDifficultyLevel': 'No raw difficulty-level query. Needs quest-ID metadata and scaling/difficulty rules tied to player/session context; fixed levels would fabricate quest data.',
    'C_QuestLog.IsQuestReplayedRecently': 'No raw replay-recency query. Needs replay completion timestamps and the native recency boundary, not generic completion flags.',
    'C_QuestLog.IsQuestTrivial': 'No raw quest triviality predicate. Needs effective quest difficulty and player-level triviality thresholds; quest completion/disabled state is not triviality.',
    'C_QuestSession.CanStop': 'No raw stop-eligibility producer. Needs active session ownership and pending start/stop lifecycle, not a constant based only on session existence.',
    'C_QuestSession.GetAvailableSessionCommand': 'No raw available-command producer. Needs role/session lifecycle and command enum eligibility; pending-command defaults are not available commands.',
    'C_QuestSession.GetSuperTrackedQuest': 'No raw session supertracked selection. Needs session-owned selected quest identity and clear/missing semantics distinct from ordinary quest tracking.',
    'C_QuestSession.HasJoined': 'No raw joined-state producer. Needs participant acceptance/join state separate from session existence.',
    'C_QuestSession.HasPendingCommand': 'No raw command-pending predicate. Needs pending command state and response/clear transitions.',
    'C_QuestSession.RequestSessionStart': 'No raw session-start request producer. Needs member consent, pending command and session-created/joined lifecycle; immediate toggling would omit consent.',
    'C_QuestSession.RequestSessionStop': 'No raw session-stop request producer. Needs ownership validation, pending stop and destruction/left event lifecycle.',
    'C_QuestSession.SendSessionBeginResponse': 'No raw begin-response mutation. Needs pending participant response identity and accepted/declined progression.',
    'C_QuestSession.SetQuestIsSuperTracked': 'No raw supertracked selection mutation. Needs session-owned selection storage and corresponding query/notification lifecycle.',
    'C_RecruitAFriend.ClaimActivityReward': 'No raw reward-claim mutation. Needs earned activity reward state, eligibility, consumption and entitlement delivery; a no-op would not claim anything.',
    'C_RecruitAFriend.ClaimNextReward': 'No raw next-reward claim. Needs ordered unlocked/claimed rewards and eligibility/entitlement transition.',
    'C_RecruitAFriend.GenerateRecruitmentLink': 'No raw recruitment-link producer. Native token issuance is server-backed; inventing a URL is not a modeled enrollment token.',
    'C_RecruitAFriend.GetRecruitActivityRequirementsText': 'No raw requirement-text producer. Needs activity requirement metadata and localization inputs, not an arbitrary string.',
    'C_RecruitAFriend.RemoveRAFRecruit': 'No raw recruit removal. Needs linked recruit identity and relationship mutation/update events.',
    'C_RecruitAFriend.RequestUpdatedRecruitmentInfo': 'No raw asynchronous recruitment refresh. Needs host snapshot/request completion and RAF_INFO_UPDATED delivery, not an empty callback.',
    'C_SocialRestrictions.IsMuted': 'No raw social mute-state producer. Voice-chat mute is a different domain; requires account/chat restriction snapshot and transitions.',
    'C_SpellBook.IsSpellDisabled': 'Current cached retail consumer in Blizzard_Deprecated remains (whole-word/bare scans retained). Do not retire despite later-page absence expectation; no raw producer and ordinary autostub lookup remains an exact gap.',
    'C_UIWidgetManager.GetCaptureZoneVisualizationInfo': 'No raw capture-zone visualization DTO. Needs server widget ID/type payload with capture-zone fields; existing widget DTO types do not prove this shape.',
    'ConfirmBNRequestInviteFriend': 'No global Battle.net invite-request confirmation. Needs pending friend/account request state with accept/decline lifecycle; a placeholder would discard the response.',
    'GetExpansionForLevel': 'Global absent on retail. Mists-only fixed thresholds are not evidence for current retail post-squish expansion-level mapping; needs profile-correct level metadata and boundary fixtures.',
    'LeaveParty': 'Current cached retail whole-word/bare consumers and modeled legacy group callers remain. Retirement prohibited; retain precise historical absence gap rather than deleting caller behavior.',
}
BATTLE_NET = {
    'GetAccountInfoByID': 'account-ID to account DTO lookup',
    'GetFriendGameAccountInfo': 'friend-index/game-account-index DTO lookup',
    'GetFriendNumGameAccounts': 'friend-index game-account collection count',
    'GetGameAccountInfoByID': 'game-account-ID to game-account DTO lookup',
}


def reason(symbol):
    if symbol in REASONS:
        return REASONS[symbol]
    if symbol.startswith('C_BattleNet.'):
        operation = BATTLE_NET[symbol.split('.')[1]]
        return f'No raw {operation}. Needs populated Battle.net account/friend/game-account identity indexes and missing-ID/index behavior; GUID/default queries cannot prove this producer.'
    if symbol.startswith('ModelSceneActor:'):
        return f'{symbol.split(":")[1]} missing on ModelSceneActor. Native 3D actor mesh/equipment/mount/sheathe domain is intentionally unsupported (AGENTS.md 3D scope); no new shim or claim of 3D behavior.'
    raise ValueError(f'Unreviewed gap: {symbol}')


def build_inventory(raw_lines, register, results):
    ledger, reviews = [], []
    for row in register['entries']:
        result = results[row['id']]
        if not result['ok']:
            note = reason(row['symbol'])
            status, capabilities = 'audit-pending', []
            reviews.append({'source_id': row['id'], 'symbol': row['symbol'],
                            'wikitext_line': row['wikitext_line'],
                            'literal': raw_lines[row['wikitext_line'] - 1],
                            'expectation': result['expected'], 'observation': result['observed'],
                            'reason': note})
        elif result['expected']['publication'] == 'absent':
            status, capabilities = 'bounded-coverage', ['current-retail-absence']
            superseded = result['expected']['superseded_by']
            note = ('Current-retail absence only; superseded by later removal ' + superseded
                    if superseded else 'Current-retail absence only; no native historical behavior claim.')
        else:
            status, capabilities = 'partial-development-green', ['current-retail-publication']
            note = 'Current-retail publication/event registration only; no signature, payload, populated-output, security or historical behavior parity.'
        ledger.append({'source_id': row['id'], 'status': status,
                       'capabilities': capabilities, 'note': note})
    dump(EVIDENCE / 'p825-gap-review.json', reviews)
    return ledger


def build_extract(raw_lines, text, extractor):
    rows = extractor.seed_rows(text, PATCH)
    scout = []
    reasons = {
        5: 'Hardware-event protection of C_FriendList.SendWho is retained verbatim; no native hardware-event differential or historical protection test is credited by the publication sweep.',
        6: 'SendChatMessage partial hardware-event restriction statement retained; publication proves neither hardware-event gating nor instance-dependent delivery.',
        7: 'Exact channel/SAY/YELL/instance restriction and inline reference retained; historical cited behavior needs hardware-event and instance/raid differential proof. No native security parity credited.',
        8: 'Party Sync and Recruit-A-Friend overhaul spans consent/session/reward/server lifecycles; missing inventory producers remain explicit. A broad overhaul cannot be established by publication alone.',
    }
    for row in rows:
        number = int(row['source_id'].rsplit('-', 1)[1])
        literal = text.splitlines()[number - 1]
        if number in (12, 13, 14, 17):
            row['status'] = 'metadata-only'
            row['note'] = 'Build comparison, external diff URL or References marker; editorial source context, no runtime credit.'
        elif number in reasons:
            row['note'] = reasons[number]
        candidates = []
        for raw_number, raw in enumerate(raw_lines, 1):
            rendered = extractor.render_line(re.sub(r'<ref>\{\{ref web\|([^{}]+)\}\}</ref>', r'[Reference: \1]', raw))
            if rendered.strip() == literal.strip():
                candidates.append(raw_number)
        if not candidates and number == 1:
            candidates = [1]
        if not candidates and literal.startswith('=='):
            candidates = [i for i, raw in enumerate(raw_lines, 1)
                          if raw.replace(' ', '') == literal.replace(' ', '')]
        assert candidates, (row, literal)
        raw_number = candidates[0]
        scout.append({'source_id': row['source_id'], 'extract_line': number,
                      'literal': literal, 'wikitext_line': raw_number,
                      'raw_literal': raw_lines[raw_number - 1],
                      'status': row['status'], 'reason': row['note']})
    dump(EVIDENCE / 'p825-extract-scout.json', scout)
    return rows


def main():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    register = read(SOURCES / f'{PATCH}-wikitext-register.json')
    results = read(EVIDENCE / 'patch_8_2_5_publication_sweep-results.json')
    raw_lines = (SOURCES / f'{PATCH}-api-changes.wikitext').read_text().splitlines()
    text = (SOURCES / f'{PATCH}-api-changes.txt').read_text()
    inventory = build_inventory(raw_lines, register, results)
    extract = build_extract(raw_lines, text, extractor)
    contexts = [{'source_id': f'build-context-{i:03}', 'wikitext_line': i, 'literal': line}
                for i, line in enumerate(raw_lines, 1) if line.startswith('|+')]
    dump(EVIDENCE / 'p825-build-context.json', contexts)
    context_rows = [{'source_id': row['source_id'], 'capabilities': [], 'status': 'metadata-only',
                     'note': 'Consolidated comparison caption; retained separately, no runtime credit.'}
                    for row in contexts]
    ledger = {'schema': 'patch-api-page-coverage/v1', 'patch': PATCH,
              'source': f'data/patch-api/sources/{PATCH}-wikitext-register.json',
              'source_sha256': digest(SOURCES / f'{PATCH}-wikitext-register.json'),
              'non_inventory_source': {'path': f'data/patch-api/sources/{PATCH}-api-changes.txt',
                                       'sha256': digest(SOURCES / f'{PATCH}-api-changes.txt')},
              'proof_policy': 'Exhaustive occurrence accounting; publication/absence only. Substantive prose remains pending. No fabricated producer or historical/native parity credit.',
              'source_rows': inventory + extract + context_rows}
    dump(SOURCES / f'{PATCH}-page-coverage.json', ledger)
    summaries = []
    for path in sorted(SOURCES.glob('*-wikitext-register.json'), key=lambda p: tuple(map(int, p.name.split('-')[0].split('.')))):
        patch = path.name.removesuffix('-wikitext-register.json')
        observed = read(EVIDENCE / ('patch_' + patch.replace('.', '_') + '_publication_sweep-results.json'))
        gaps = sum(not row['ok'] for row in observed.values())
        summaries.append({'patch': patch, 'rows': len(observed), 'ok': len(observed) - gaps,
                          'gaps': gaps, 'result': 'pass'})
    dump(EVIDENCE / 'p825-sweep-summary.json', summaries)


if __name__ == '__main__':
    main()
