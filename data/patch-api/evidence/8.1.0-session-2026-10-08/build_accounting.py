"""Account for every raw API identity and every retained extract occurrence."""
import hashlib
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
from reproduce_sources import load_extractor

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'

REASONS = {
    'gxMTOpaque': 'CVar not registered. The page gives no default or simulator render-thread control contract; registering an arbitrary string would only fabricate publication.',
    'C_AchievementInfo.GetSupercedingAchievements': 'No raw achievement supersession query. Achievement records lack an ordered supersession relation; previous/next achievements are not proof of superseding achievements.',
    'C_Calendar.EventGetClubId': 'No selected calendar-event club identity producer. Requires a selected event with club ownership and missing-selection semantics.',
    'C_Calendar.EventSetClubId': 'No selected calendar-event club mutation. Requires selected event ownership, accepted club IDs and coherent update notifications.',
    'C_Calendar.GetEventIndexInfo': 'No calendar event-index DTO producer. Needs the selected event calendar offset/day/index relation, not a fixed tuple.',
    'C_Calendar.GetNextClubId': 'No pending/new-event club-ID state query. Needs staged calendar editor state distinct from current event ownership.',
    'C_Calendar.SetNextClubId': 'No pending/new-event club-ID state mutation. Needs editor state and its consumption by event creation, not a no-op setter.',
    'C_ChatInfo.ResetDefaultZoneChannels': 'No raw zone-channel reset. Needs zone channel policy, joined-channel state and join/leave transitions; current channel defaults do not model reset.',
    'C_ChatInfo.SwapChatChannelsByChannelIndex': 'No raw ordered chat-channel swap. Needs channel IDs/indexes and the resulting persisted order; unrelated chat messages are not channel ordering.',
    'C_Club.AddClubStreamChatChannel': 'No club stream-to-chat channel join. Requires club/stream identity, membership and coherent channel state/events.',
    'C_Club.Flush': 'No raw club cache flush. Needs cached request/snapshot ownership and invalidation semantics; an empty function would not flush anything.',
    'C_CurrencyInfo.DoesWarModeBonusApply': 'No per-currency war-mode eligibility metadata query. A global war-mode boolean cannot establish whether this currency gets a bonus.',
    'C_Debug.DashboardIsEnabled': 'No debug-dashboard state/domain. Native client diagnostics are not the simulator GUI; do not publish a constant as an enabled-state model.',
    'C_Garrison.IsEnvironmentCountered': 'No environment-counter computation. Requires mission environment mechanics and follower ability matching; mission presence alone is insufficient.',
    'C_Item.CanScrapItem': 'No item scrapping eligibility metadata/policy. Inventory presence does not identify scrappable classes, restrictions or context.',
    'C_Item.CanViewItemPowers': 'No item power-view eligibility metadata. Item links and Azerite defaults do not establish whether this item supports power viewing.',
    'C_Loot.IsLegacyLootModeEnabled': 'No legacy-loot mode state. Requires instance expansion/level/eligibility rules and selected loot context, not an arbitrary boolean.',
    'C_Map.GetBountySetMaps': 'No bounty-set to ordered map index. Map records lack bounty-set membership, so returning every map or an empty table would fabricate the relation.',
    'C_MountJournal.GetMountFromItem': 'No item-ID to mount-ID relation. Existing mount spell IDs cannot be treated as item IDs; needs item association metadata and unknown-ID output.',
    'C_PartyInfo.GetInviteReferralInfo': 'No pending party invitation referral DTO. Needs inviter/referral identity and request lifetime rather than the joined roster.',
    'C_PetJournal.GetPetInfoByItemID': 'No item-to-battle-pet species relation. Current species journal records do not supply item association metadata.',
    'C_PlayerInfo.GUIDIsPlayer': 'No raw player-GUID classifier. Existing PlayerLocation temporary provider is not an unconditional C API producer; needs accepted GUID grammar/security contract without name or unit existence assumptions.',
    'C_PvP.CanToggleWarModeInArea': 'No area-specific war-mode toggle policy. War-mode state alone lacks sanctuary/capital/map eligibility metadata.',
    'C_PvP.GetRewardItemLevelsByTierEnum': 'No tier-enum reward level table. Needs season/bracket/tier item-level metadata; rating records alone cannot derive reward levels.',
    'C_PvP.GetWeeklyChestInfo': 'No historical PvP chest progress/reward DTO. Needs season-owned weekly entitlement state and historical fields; current weekly vault state is a different contract.',
    'C_QuestLine.RequestQuestLinesForMap': 'No questline request lifecycle. Needs a host map-to-questline index, response snapshot and completion events, not a no-op request.',
    'C_ReportSystem.CanReportPlayer': 'No player-report eligibility policy. Needs target/report context and permission restrictions; account presence is insufficient.',
    'C_ReportSystem.ReportPlayer': 'No player-report submission lifecycle. Current report-system state does not store target, report type and submission/result transition for this legacy member.',
    'C_ReportSystem.ReportServerLag': 'No server-lag reporting transport/request state. Simulator network statistics are not a report submission; native backend remains unmodeled.',
    'C_SummonInfo.CancelSummon': 'No pending summon cancellation mutation. Needs a pending offer and its rejected/cleared transition with summon notifications.',
    'C_SummonInfo.ConfirmSummon': 'No pending summon acceptance mutation. Needs pending inviter/destination, expiration and accepted travel transition; teleporting immediately would bypass offer state.',
    'C_SummonInfo.GetSummonConfirmAreaName': 'No pending summon destination name producer. Current player area is not the offered summon destination.',
    'C_SummonInfo.GetSummonConfirmSummoner': 'No pending summon inviter identity producer. Needs the active offer sender, not a constant or player name.',
    'C_UIWidgetManager.GetSpellDisplayVisualizationInfo': 'No spell-display visualization DTO. Requires concrete widget-ID/type payload including spell-specific fields; other widget shapes cannot prove it.',
}
FRIEND_OPERATIONS = {
    'AddFriend': 'friend request creation and response events',
    'AddIgnore': 'ignore-list identity insertion',
    'AddOrDelIgnore': 'ignore-list membership toggle',
    'AddOrRemoveFriend': 'friend membership toggle with pending request semantics',
    'DelIgnore': 'ignore-list identity removal',
    'DelIgnoreByIndex': 'ordered ignore-list removal by index',
    'GetFriendInfo': 'name/identity-based friend DTO lookup distinct from index lookup',
    'GetSelectedFriend': 'selected friend index and empty-selection behavior',
    'GetSelectedIgnore': 'selected ignore index and empty-selection behavior',
    'IsIgnored': 'ignore membership by accepted player identity',
    'IsIgnoredByGuid': 'GUID-to-ignore identity mapping and membership',
    'RemoveFriend': 'friend removal by accepted player identity',
    'RemoveFriendByIndex': 'ordered friend removal by index',
    'SendWho': 'who request/query lifecycle and response delivery; native hardware-event policy unverified',
    'SetFriendNotes': 'identity-based friend note mutation',
    'SetFriendNotesByIndex': 'index-based friend note mutation',
    'SetSelectedFriend': 'persistent selected friend index updates',
    'SetSelectedIgnore': 'persistent selected ignore index updates',
    'SortWho': 'stable who-result ordering with field/direction semantics',
}


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def reason(result):
    symbol = result['expected']['symbol']
    if result['expected']['publication'] == 'absent':
        if symbol == 'C_ConfigurationWarnings.GetConfigurationWarningSeen':
            return ('No cached consumer, but simulator temporary provider and its unit tests still publish/call this getter after C API registration. '
                    'Do not retire a live provider path; profile-safe provider/test migration needs a separate modeled contract. '
                    'Retirement test exposed raw republication; candidate removed from retirement list, complete caller evidence retained.')
        if symbol.startswith('C_ConfigurationWarnings.'):
            return ('Retirement prohibited: current cached Blizzard Lua consumes this member. '
                    'Keep modeled configuration warning state and report historical absence mismatch; '
                    'complete qualified/bare/caller scans retained.')
        return ('Later register expects absence, but ordinary namespace lookup still fabricates this member. '
                'That later-page removal has a retained gap; this audit does not silently implement another '
                'page retirement without consumer/later-readdition proof. Supersession ID: '
                + str(result['expected']['superseded_by']))
    if symbol.startswith('C_FriendList.'):
        member = symbol.split('.')[1]
        return ('No raw friend-list producer for ' + FRIEND_OPERATIONS[member] + '. '
                'Requires coherent friend/ignore/who backing state; existing indexed/read-only fixtures '
                'and namespace autostubs do not implement this operation.')
    return REASONS[symbol]


def main():
    extractor = load_extractor()
    register_path = SOURCES / '8.1.0-wikitext-register.json'
    register = json.loads(register_path.read_text())
    results = json.loads((EVIDENCE / 'patch_8_1_0_publication_sweep-results.json').read_text())
    raw_lines = (SOURCES / '8.1.0-api-changes.wikitext').read_text().splitlines()
    text_path = SOURCES / '8.1.0-api-changes.txt'
    text = text_path.read_text()
    rows, review = [], []
    retired = {'C_Map.GetBountySetIDForMap', 'C_Calendar.EventGetClubID', 'C_Calendar.EventSetClubID'}
    for entry in register['entries']:
        key = entry['id']
        observation = results[key]
        ok = observation['ok']
        modeled = entry['symbol'] == 'C_DateAndTime.CompareCalendarTime'
        status = ('audit-pending' if not ok else 'bounded-coverage'
                  if entry['symbol'] in retired or modeled or observation['expected']['publication'] == 'absent'
                  else 'partial-development-green')
        note = (reason(observation) if not ok else
                'Bounded supplied-civil-time ordering; rhs sign follows Blizzard documentation, weekday ignored, inputs unchanged. Invalid civil-date/security/native parity not claimed.' if modeled else
                'Consumer-free retail absence; bare/repeated/cached checks and Mists preservation. No native signature or behavior claim.' if entry['symbol'] in retired else
                'Publication/absence observation only; no signature, populated-output, payload, security or behavior parity credit.')
        capabilities = [] if not ok else [{'kind': 'calendar-ordering' if modeled else 'publication/absence',
                                         'proof': 'tests/patch_8_1_0_calendar_compare.rs' if modeled else 'tests/patch_8_1_0_publication_sweep.rs'}]
        rows.append({'source_id': key, 'source': entry['symbol'], 'status': status,
                     'capabilities': capabilities, 'note': note})
        if not ok:
            review.append({'source_id': key, 'symbol': entry['symbol'], 'wikitext_line': entry['wikitext_line'],
                           'literal': raw_lines[entry['wikitext_line'] - 1], 'expectation': observation['expected'],
                           'observation': observation['observed'], 'reason': note})
    rendered = []
    for number, raw in enumerate(raw_lines, 1):
        if raw.startswith('* ') and '{{api|' in raw:
            continue
        value = re.sub(r'<ref>\{\{ref web\|([^{}]+)\}\}</ref>', r'[Reference: \1]', raw)
        value = extractor.render_line(re.sub(r'^(=+)\s*(.*?)\s*\1$', r'\1 \2 \1', value))
        rendered.append((number, raw, value))
    scout = []
    cursor = 0
    for row in extractor.seed_rows(text, '8.1.0'):
        extract_line = int(row['source_id'].rsplit('-', 1)[-1])
        literal = text.splitlines()[extract_line - 1]
        while cursor < len(rendered) and rendered[cursor][2] != literal:
            cursor += 1
        assert cursor < len(rendered), literal
        number, raw_literal, _ = rendered[cursor]
        cursor += 1
        unknown = literal == '?'
        note = ('Source explicitly supplies only "?" for this section; no enumerated contract to implement. '
                'Retain source uncertainty, not behavioral credit.' if unknown else
                'Source/build/section/namespace/caption/reference context retained literally; inventory identities audited separately. '
                'No whole-namespace absence, linked-page content or behavioral credit.')
        row.update(status='audit-pending' if unknown else 'metadata-only', capabilities=[], note=note)
        rows.append(row)
        scout.append({'source_id': row['source_id'], 'extract_line': extract_line, 'literal': literal,
                      'wikitext_line': number, 'raw_literal': raw_literal,
                      'status': row['status'], 'reason': note})
    coverage = {'schema': 'patch-page-coverage/v1', 'patch': '8.1.0',
                'source_sha256': hashlib.sha256(register_path.read_bytes()).hexdigest(),
                'non_inventory_source': {'path': str(text_path.relative_to(ROOT)),
                                         'sha256': hashlib.sha256(text_path.read_bytes()).hexdigest()},
                'proof_policy': 'Current retail publication/absence plus bounded supplied calendar comparison, never historical/native parity.',
                'source_rows': rows}
    dump(SOURCES / '8.1.0-page-coverage.json', coverage)
    dump(EVIDENCE / 'p810-gap-review.json', review)
    dump(EVIDENCE / 'p810-extract-scout.json', scout)
    summaries = []
    for path in sorted(EVIDENCE.glob('patch_*_publication_sweep-results.json')):
        observations = json.loads(path.read_text())
        summaries.append({'file': path.name, 'rows': len(observations),
                          'ok': sum(r['ok'] for r in observations.values()),
                          'gaps': sum(not r['ok'] for r in observations.values())})
    dump(EVIDENCE / 'p810-sweep-summary.json', summaries)
    print(json.dumps({'inventory': len(register['entries']), 'extract': len(scout),
                      'gaps': len(review), 'ledger': len(rows)}))


if __name__ == '__main__':
    main()
