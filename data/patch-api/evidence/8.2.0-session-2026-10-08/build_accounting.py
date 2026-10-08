#!/usr/bin/env python3
"""Build occurrence accounting from pinned input and actual sweep observations."""
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '8.2.0'


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


REASONS = {
    'BNSetFriendFavoriteFlag': 'No Battle.net friend favorite mutation producer; requires friend/account identity, persisted favorite flag and friend-list notification fixtures, not a no-op.',
    'C_AzeriteEssence.CanDeactivateEssence': 'Existing essence model has milestones and active IDs, but lacks a specified deactivation permission policy (forge/rest/change restrictions). Generated documentation supplies milestoneID and boolean only; do not invent eligibility from activation rules.',
    'C_ChatInfo.GetClubStreamIDs': 'No raw club-stream enumeration producer; needs per-club stream identity/catalog visibility fixtures rather than a fabricated empty list.',
    'C_ClubFinder.CheckAllPlayerApplicantSettings': 'No applicant-settings aggregate check producer; needs matching filter/settings policy and concrete differing applicant fixtures.',
    'C_ClubFinder.ClearAllFinderCache': 'No finder cache invalidation lifecycle; clearing all must distinguish postings, applicant and pending results and notifications.',
    'C_ClubFinder.ClearClubApplicantsCache': 'No per-club applicant cache eviction producer; requires keyed cached results and refresh lifecycle.',
    'C_ClubFinder.ClearClubFinderPostingsCache': 'No postings cache eviction producer; requires posting query results and refresh lifecycle independent of applicants.',
    'C_ClubFinder.ReturnCommunityApplicantList': 'No raw community applicant snapshot producer; requires community-keyed applicant data, ordering and pending status.',
    'C_ClubFinder.ReturnGuildApplicantList': 'No raw guild applicant snapshot producer; requires guild-keyed applicant data and result ordering.',
    'C_ClubFinder.ReturnPendingCommunityApplicantList': 'No pending community applicant snapshot producer; pending request and accepted/rejected transitions are not specified by existing placeholder lookup.',
    'C_ClubFinder.ReturnPendingGuildApplicantList': 'No pending guild applicant snapshot producer; requires pending membership lifecycle and guild identity.',
    'C_ClubFinder.ShouldShowClubFinder': 'No modeled finder eligibility/display policy; constant true would not model account restrictions or feature availability.',
    'C_Commentator.GetElapsedMs': 'No spectator match clock producer; needs match start/pause/end state and deterministic time input.',
    'C_EncounterJournal.SetPreviewMythicPlusLevel': 'No preview-level mutation backing encounter journal outputs; setting a field without scaled preview consumers would not model behavior.',
    'C_EncounterJournal.SetPreviewPvpTier': 'No PvP-tier preview mutation backing journal outputs; requires tier-dependent item preview fixtures.',
    'C_ItemUpgrade.GetItemHyperlink': 'No selected upgrade-item hyperlink producer; needs current upgrade session item identity and hyperlink context, not a generic item link.',
    'C_MountJournal.ApplyMountEquipment': 'Temporary guarded no-op exists in collections_journal_namespace.rs but is not raw-published in full-UI discovery. Requires inventory consumption, equipment eligibility and apply-result lifecycle; no shim promoted as a fix.',
    'C_PaperDollInfo.GetInspectItemLevel': 'No inspect equipment snapshot/item-level aggregation producer for inspected unit; player average item level is not inspect state.',
    'C_PvP.CanDisplayDeaths': 'No match scoreboard visibility policy for death columns; constant flag would not distinguish map/mode.',
    'C_PvP.CanDisplayHonorableKills': 'No match scoreboard honorable-kill column policy; requires map/mode metadata.',
    'C_PvP.DoesMatchOutcomeAffectRating': 'No match-specific rating eligibility policy; existing rated predicates do not specify outcome/rating effects.',
    'C_PvP.GetActiveMatchBracket': 'No active-match bracket producer with team size and bracket identity.',
    'C_PvP.GetActiveMatchDuration': 'No active-match start/end clock producer; constant duration is not a match lifecycle.',
    'C_PvP.GetActiveMatchWinner': 'No match outcome/team winner producer including draw and incomplete states.',
    'C_PvP.GetMatchPVPStatColumn': 'No keyed match-stat column metadata producer; requires map-specific names/order/visibility.',
    'C_PvP.GetMatchPVPStatIDs': 'No match-stat ID catalog producer; fabricated empty list would conceal missing scoreboard domain.',
    'C_PvP.GetPostMatchCurrencyRewards': 'No post-match currency reward records or delivery state; requires populated currency-ID/quantity fixtures.',
    'C_PvP.GetPostMatchItemRewards': 'No post-match item reward records or delivery lifecycle; requires populated item identity/quantity fixtures.',
    'C_PvP.GetPVPActiveMatchPersonalRatedInfo': 'No player-rated-match result/rating delta snapshot producer.',
    'C_PvP.GetScoreInfo': 'No structured indexed scoreboard producer; deprecated wrappers depend on its populated contract and cannot establish native publication.',
    'C_PvP.GetScoreInfoByPlayerGuid': 'No GUID-keyed scoreboard index producer; requires concrete player score records and unknown-GUID behavior.',
    'C_PvP.GetTeamInfo': 'No match team score/name/rating record producer; generic namespace lookup is fabricated.',
    'C_PvP.IsActiveMatchRegistered': 'No match registration lifecycle producer; a published active-match-state default is not registration state.',
    'C_PvP.IsArena': 'No raw arena match classifier; needs match/instance mode distinction and populated fixture, not aliasing a plausible constant.',
    'C_PvP.IsBattleground': 'No raw battleground match classifier; requires match/instance metadata and inactive cases.',
    'C_PvP.IsMatchFactional': 'No per-match factional-team policy; arena/battleground flags alone do not establish factional behavior.',
    'C_SpellBook.ContainsAnyDisenchantSpell': 'No spellbook disenchant classification metadata/query; needs known/unknown spell membership and disenchant-tag fixtures.',
    'C_UIWidgetManager.GetTextureAndTextRowVisualizationInfo': 'No texture/text-row widget visualization records; requires indexed texture/text/state rows, not a nil default.',
    'C_UIWidgetManager.GetTextureAndTextVisualizationInfo': 'No texture/text widget visualization snapshot producer with concrete visibility/text/texture state.',
    'C_UIWidgetManager.GetZoneControlVisualizationInfo': 'No zone-control visualization producer; requires faction/control/progress widget snapshot lifecycle.',
    'C_VoiceChat.IsChannelJoinPending': 'Voice connecting flag is connection-wide, not per-channel pending join requests; no channel-keyed request lifecycle.',
    'SetMoveEnabled': 'No movement-input enable gate consumed by movement commands; registering a setter without input integration is a no-op shortcut.',
    'SetTurnEnabled': 'No turn-input enable gate consumed by directional/turn commands; requires input suppression/resumption fixtures.',
    'ShouldShowIslandsWeeklyPOI': 'No island weekly quest/POI eligibility producer; requires weekly completion and map context.',
    'ShouldShowSpecialSplashScreen': 'No special splash eligibility/acknowledgment state; requires account/build-specific display lifecycle.',
    'UpdateWindow': 'No Lua window-update host contract; console command and Lua global are distinct identities, and host window refresh is not modeled.',
    'Checkout:CopyExternalLink': 'No Checkout external-link/clipboard lifecycle; absent object method cannot be replaced by a no-op in the generic frame surface.',
    'Checkout:OnButtonUpdate': 'Checkout HasScript rejects handler despite global handler enum; no embedded checkout button state/event producer.',
    'Checkout:OnEditFocusGained': 'Checkout HasScript rejects handler; EditBox focus dispatch is not embedded Checkout field focus lifecycle.',
    'Checkout:OnEditFocusLost': 'Checkout HasScript rejects handler; needs embedded checkout field focus transition dispatch.',
    'Checkout:OnError': 'Checkout HasScript rejects handler; needs embedded checkout error lifecycle and payload fixtures.',
    'Checkout:OnExternalLink': 'Checkout HasScript rejects handler; requires external-link activation/callback lifecycle.',
    'TabardModel:GetLowerEmblemFile': 'TabardModel has no lower emblem asset producer. 3D model rendering is intentionally unsupported; no fabricated file ID.',
    'TabardModel:GetUpperEmblemFile': 'TabardModel has no upper emblem asset producer. 3D model rendering is intentionally unsupported; no fabricated file ID.',
    'azeriteEssenceSwapTutorial': 'CVar lacks registered value/default. Page supplies no default or tutorial lifecycle; no guessed persisted state.',
    'GxuSwapChainSize': 'No Command-kind console catalog entry or swap-chain sizing host operation; a CVar placeholder is not command publication.',
    'GxuWindowSize': 'No Command-kind console catalog entry or host window sizing operation.',
    'ToggleWindowMode': 'No Command-kind console catalog entry or host window-mode mutation contract.',
    'command:UpdateWindow': 'No Command-kind console catalog entry or host refresh operation; keep distinct from Lua global UpdateWindow.',
}

PROSE_REASONS = {
    16: 'File-path sandbox statement needs addon texture/sound/file lookup denial tests across Interface and non-Interface roots; no native path-security parity claim.',
    17: 'GetFileIDFromPath non-Interface restriction needs denied path resolution tests and actual asset mappings; this host has no WoW install.',
    18: 'Fonts path exception needs concrete native font-path resolution fixtures; asset availability and sandbox exception are not proven by publication.',
    19: 'Sound folder override prohibition plus mute/unmute lifecycle needs real asset precedence and audio integration; no WoW install on host.',
    20: 'Protected reporting requires secure/insecure callers, reporting session state and blocked automated sends; publication of dialog is not security proof.',
    21: 'Nameplate anchor-family restriction needs family identity and rejection behavior in SetPoint; generic layout support does not prove this boundary.',
    22: 'Multiple customizable tooltip textures per line requires concrete multi-texture line layout/output fixtures, not AddTexture publication.',
    23: 'WorldStateFrame to UI Widget Manager migration is a broad replacement claim; needs historical producer/consumer fixtures rather than surface lookup.',
    24: 'ShowUIPanel/HideUIPanel combat protection requires secure/insecure dispatch and combat-state mutation tests; publication is not protection parity.',
    25: 'Frame-stack display naming and fstack_preferParentKeys interaction requires concrete inspector output for both CVar settings; CVar publication alone is insufficient.',
}


def main():
    register = read(SOURCES / f'{PATCH}-wikitext-register.json')
    results = read(HERE / 'p820-reviewed-discovery-results.json')
    raw_lines = (SOURCES / f'{PATCH}-api-changes.wikitext').read_text().splitlines()
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text_path = SOURCES / f'{PATCH}-api-changes.txt'
    text = text_path.read_text()
    rows, review = [], []
    for entry in register['entries']:
        result = results[entry['id']]
        absent = result['expected']['publication'] == 'absent'
        if result['ok']:
            reason = ('Current-retail absence/deprecation/supersession only; no historical signature or behavior parity.'
                      if absent else 'Current-retail publication/HasScript/event registration only; no populated-output, security, callback or historical parity.')
            status = 'bounded-coverage' if absent else 'partial-development-green'
            capabilities = ['current-retail-absence' if absent else 'current-retail-publication']
            if entry['symbol'] == 'C_VoiceChat.SetMasterVolumeScale':
                capabilities.append('normalized-volume-round-trip')
                reason = 'State-backed normalized voice ducking setter/getter; invalid-input rejection preserves simulator invariant, not claimed native edge/security parity.'
        else:
            key = 'command:UpdateWindow' if entry.get('kind') == 'command' and entry['symbol'] == 'UpdateWindow' else entry['symbol']
            reason = REASONS[key]
            status, capabilities = 'audit-pending', []
            review.append({'source_id':entry['id'], 'symbol':entry['symbol'], 'wikitext_line':entry['wikitext_line'],
                           'literal':raw_lines[entry['wikitext_line']-1], 'expectation':result['expected'],
                           'observation':result['observed'], 'reason':reason, 'decision':'recorded-problematic',
                           'caller_scan':'p820-gap-caller-scout.json'})
        rows.append({'source_id':entry['id'],'status':status,'capabilities':capabilities,'note':reason})
    scout = []
    for row in extractor.seed_rows(text, PATCH):
        number = int(row['source_id'].rsplit('-',1)[1])
        literal = text.splitlines()[number-1]
        matches = [index for index,line in enumerate(raw_lines,1)
                   if extractor.extract_text(line, legacy_api_tables=True).strip() == literal.strip()]
        assert len(matches) == 1, (literal,matches)
        raw_number = matches[0]
        reason = PROSE_REASONS.get(raw_number, 'Editorial heading/source/build/TOC or deprecated-source link; no runtime credit; linked pages not expanded.')
        row.update(status='audit-pending' if raw_number in PROSE_REASONS else 'metadata-only', note=reason)
        rows.append(row)
        scout.append({'source_id':row['source_id'],'extract_line':number,'literal':literal,'wikitext_line':raw_number,
                      'raw_literal':raw_lines[raw_number-1],'status':row['status'],'reason':reason})
    contexts = []
    for number,line in enumerate(raw_lines,1):
        if line.startswith('|+'):
            key=f'caption-{number:03}'
            contexts.append({'source_id':key,'wikitext_line':number,'literal':line})
            rows.append({'source_id':key,'status':'metadata-only','capabilities':[],
                         'note':'Literal historical comparison caption; no runtime credit.'})
    ledger = {'schema':'patch-api-page-coverage/v1','patch':PATCH,'source':f'data/patch-api/sources/{PATCH}-wikitext-register.json',
              'source_sha256':hashlib.sha256((SOURCES/f'{PATCH}-wikitext-register.json').read_bytes()).hexdigest(),
              'non_inventory_source':{'path':str(text_path.relative_to(ROOT)),'sha256':hashlib.sha256(text_path.read_bytes()).hexdigest()},
              'proof_policy':'Occurrence accounting only; publication/absence is not native signature, populated output, payload, security or historical parity. 8.2.5 unmerged register reserved for integration.',
              'source_rows':rows}
    write(SOURCES/f'{PATCH}-page-coverage.json',ledger)
    write(ROOT/'tests/data/patch_8_2_0_sweep_known_gaps.json',sorted(r['source_id'] for r in review))
    write(HERE/'p820-gap-review.json',review)
    write(HERE/'p820-extract-scout.json',scout)
    write(HERE/'p820-build-context.json',contexts)
    print(json.dumps({'inventory':len(register['entries']),'gaps':len(review),'extract':len(scout),'captions':len(contexts),'ledger':len(rows)}))


if __name__ == '__main__':
    main()
