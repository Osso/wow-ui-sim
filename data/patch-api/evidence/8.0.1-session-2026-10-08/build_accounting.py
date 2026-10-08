#!/usr/bin/env python3
"""Write 8.0.1 occurrence accounting from retained full-UI observations."""
import hashlib
import importlib.util
import json
import re
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCES = ROOT / 'data/patch-api/sources'

GAP_REASONS = {
    'C_ChatInfo.CanReportPlayer': 'No report eligibility model for report type, target identity, evidence or account restrictions. Autostub lookup is not a raw API; no constant eligibility promoted.',
    'C_ChatInfo.GetChannelRosterInfo': 'No indexed channel roster snapshot with names/ownership/moderator membership; active chat channel count does not supply roster records.',
    'C_ChatInfo.GetRegisteredAddonMessagePrefixes': 'No environment-local prefix registration catalog to enumerate. Existing fallback returns no real registered-prefix state; requires populated registration, deduplication and isolation fixtures.',
    'C_ChatInfo.IsAddonMessagePrefixRegistered': 'No prefix-interest registry/query state. Ordinary namespace lookup synthesizes a callable; no actual membership or registration lifecycle backs it.',
    'C_ChatInfo.RegisterAddonMessagePrefix': 'No prefix-interest registration model. Current cached documentation uses RegisterAddonMessagePrefixResult (not historical boolean); type/length/capacity/security/result transitions need a specified registry contract, not promotion of a true/no-op stub.',
    'C_ChatInfo.IsPartyChannelType': 'No channel-type membership/classification catalog. Requires exact channel enum membership including party/raid/instance distinctions; no guessed classifier published.',
    'C_ChatInfo.SendAddonMessage': 'Existing state-backed namespace sender in addon_messages.rs is explicitly client-wowforever-only. Retail argument security, result/routing and inbound-vs-outbound semantics are not established by the permissive legacy global sender; no alias or shim promoted.',
    'C_ChatInfo.SendAddonMessageLogged': 'Existing logged outbound-intent sender is client-wowforever-only. Retail logged-message acceptance, nil result and CHAT_MSG_ADDON_LOGGED delivery/throttling lifecycle lack a backing model; no fabricated logging sender published.',
    'C_Map.GetBountySetIDForMap': 'No map-keyed bounty-set association records. Existing map hierarchy/art metadata cannot produce bounty-set identity; requires concrete associated/unassociated map fixtures.',
    'C_Map.GetMapArtHelpTextPosition': 'No map-art help-text coordinate metadata. Map layer dimensions do not establish normalized help-text placement; no guessed vector returned.',
    'C_Map.GetMapDisplayInfo': 'No display metadata snapshot for UiMap presentation. MapData hierarchy/name/art fields do not specify display scale/offset or associated fields; no generic fabricated record.',
    'C_Map.GetMapHighlightInfoAtPosition': 'No normalized-point highlight texture/atlas/geometry lookup records. Child rect map identity is not highlight visual metadata; requires populated regions and misses.',
    'C_Map.GetMapLevels': 'No ordered floor/level catalog associated with maps. Hierarchy child maps are not documented dungeon levels; no guessed floor list or nil shim published.',
    'GLYPH_ADDED': 'Cached retail has zero whole-word hits, but glyph_state.rs still emits this event and c_glyph_globals tests consume it. Retirement would break an existing modeled glyph application lifecycle. Historical/current mismatch retained, not solved by altering vendor Lua or silently weakening tests.',
    'GLYPH_REMOVED': 'Cached retail has zero whole-word hits, but glyph_state.rs emits this event and c_glyph_globals tests consume it. Removing registration requires changing the existing glyph-removal lifecycle; recorded problematic instead of a destructive event gate.',
    'SPELL_TEXT_UPDATE': 'Later 11.0.0 register removes this event, but the simulator still registers it (same unresolved failure as the 11.0.0 audit). No new event producer/retirement is implemented here; historical addition is superseded, not credited as modern publication.',
}

PROSE_REASONS = {
    7: 'Namespace migration needs real prefix registry and retail send semantics. Old globals remain for existing simulator consumers/deprecation compatibility; the source statement is retained, not claimed as a complete native migration.',
    8: 'Combat-log OnEvent no-loadout/current-event retrieval requires an actual combat producer and per-dispatch snapshot fixture; registration alone cannot prove event argument absence or snapshot timing.',
    9: 'Cached full UI rejects an unknown event without registering it and accepts UNIT_POWER_UPDATE. Exact native error text and all event names are not claimed.',
    10: 'Cached full UI rejects a string aura index; actual Blizzard AuraUtil.FindAuraByName is callable and returns nil for a missing aura. Populated name lookup/order/filter parity is not established by this bounded input test.',
    11: 'UNIT_POWER removal and UNIT_POWER_UPDATE publication are inventory-probed; actual power-change producer/payload/order across units remains unproven.',
    12: 'Twenty joined-channel limit needs environment-local join/leave membership, capacity boundary and error/result fixtures. GetNumActiveChannels publication does not prove a channel-capacity model.',
    13: 'Whole C_Vignettes replacement requires namespace retirement and C_VignetteInfo snapshot semantics. No full namespace migration/successor population proof is inferred from an external category link.',
    16: 'The phrase all map API has no exhaustive member list. Explicit named removals are inventory-probed; unnamed historical map functions and total native map migration are not claimed.',
    17: 'GetCurrentMapAreaID absence and C_Map.GetBestMapForUnit publication are probed; current WorldMapFrame display identity and native uiMapID behavior across zones require a transition fixture.',
    18: 'Global GetPlayerMapPosition absence is probed. Existing C_Map.GetPlayerMapPosition models player position but does not establish party-member positions or native map-coordinate behavior.',
    19: 'EJ_GetCurrentInstance absence is probed; EJ_GetInstanceForMap composition must be verified against explicit player map/instance snapshots, not a fabricated encounter-journal default.',
    20: 'GLYPH_UPDATED absence is probed, but GLYPH_ADDED/REMOVED remain registered for existing simulator glyph producers/tests. Aggregate removal sentence therefore remains unresolved.',
    21: 'Three named globals are absent in the full-UI inventory observations. Historical removed functionality and native errors are not claimed.',
}


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def rendered_raw_line(extractor, line):
    line = re.sub(r'\{\{ref web\|([^{}]+)\}\}', r'[Reference: \1]', line)
    line = re.sub(r'^(=+)\s*(.*?)\s*\1$', r'\1 \2 \1', line)
    return extractor.render_line(line)


def main():
    register_path = SOURCES / '8.0.1-wikitext-register.json'
    register = read(register_path)
    results = read(HERE / 'patch_8_0_1_publication_sweep-results.json')
    known = set(read(ROOT / 'tests/data/patch_8_0_1_sweep_known_gaps.json'))
    assert {key for key, row in results.items() if not row['ok']} == known
    raw_lines = (SOURCES / '8.0.1-api-changes.wikitext').read_text().splitlines()
    ledger, review = [], []
    for entry in register['entries']:
        identifier, symbol = entry['id'], entry['symbol']
        observed = results[identifier]
        if identifier in known:
            status, caps, note = 'audit-pending', [], GAP_REASONS[symbol]
            review.append({'source_id': identifier, 'symbol': symbol,
                           'wikitext_line': entry['wikitext_line'],
                           'literal': raw_lines[entry['wikitext_line'] - 1],
                           'expectation': observed['expected'],
                           'observation': observed['observed'], 'reason': note})
        elif observed['expected']['publication'] == 'absent':
            status, caps = 'bounded-coverage', ['current-retail-absence']
            supersession = observed['expected']['superseded_by']
            note = ('Superseded by ' + supersession + '; ' if supersession else '')
            note += 'Current full-UI absence/deprecated fallback only; no historical behavior parity or new retirement.'
        else:
            status, caps = 'partial-development-green', ['current-retail-publication']
            note = 'Full-UI publication/event registration only; no signature, payload, populated output or native security parity.'
            if symbol == 'C_Map.GetMapPosFromWorldPos':
                status = 'bounded-coverage'
                caps += ['explicit-world-rectangle-projection']
                note = 'Explicit known-map rectangles project finite world points into normalized coordinates; continent/override/miss/overlap/reversed-axis/degenerate/isolation fixtures. No native geography or automatic overlapping hierarchy selection.'
        ledger.append({'source_id': identifier, 'status': status, 'capabilities': caps, 'note': note})
    extractor = load_extractor()
    text_path = SOURCES / '8.0.1-api-changes.txt'
    text_lines = text_path.read_text().splitlines()
    rendered = [rendered_raw_line(extractor, line) for line in raw_lines]
    scout = []
    for seed in extractor.seed_rows(text_path.read_text(), '8.0.1'):
        number = int(seed['source_id'].rsplit('-', 1)[1])
        literal = text_lines[number - 1]
        assert literal in rendered, literal
        raw_number = rendered.index(literal) + 1
        metadata = (seed['status'] == 'metadata-only' or number in (3, 4, 25, 28, 32, 33))
        if metadata:
            seed.update(status='metadata-only', note='Editorial heading/source reference or added-vs-initially-documented qualifier; linked sources are not expanded and carry no runtime credit.')
        else:
            seed['note'] = PROSE_REASONS[number]
            if number in (9, 10, 21):
                seed.update(status='bounded-coverage', capabilities=['cached-input-change' if number in (9, 10) else 'named-global-absence'])
        ledger.append(seed)
        scout.append({'source_id': seed['source_id'], 'extract_line': number,
                      'literal': literal, 'wikitext_line': raw_number,
                      'raw_literal': raw_lines[raw_number - 1],
                      'status': seed['status'], 'reason': seed['note']})
    assert len(ledger) == len({row['source_id'] for row in ledger})
    write(SOURCES / '8.0.1-page-coverage.json', {
        'schema': 'patch-api-page-coverage/v1', 'patch': '8.0.1',
        'source': str(register_path.relative_to(ROOT)), 'source_sha256': digest(register_path),
        'non_inventory_source': {'path': str(text_path.relative_to(ROOT)), 'sha256': digest(text_path)},
        'proof_policy': 'Complete occurrence accounting; bounded local behavior is not native parity. 8.1.5 merged; 8.1.0 integration placeholder remains. Problematic cases retain literal reasons without shims.',
        'source_rows': ledger,
    })
    write(HERE / 'p801-gap-review.json', review)
    write(HERE / 'p801-extract-scout.json', scout)
    write(HERE / 'p801-accounting-summary.json', {
        'inventory_rows': len(register['entries']), 'inventory_ok': len(results) - len(known),
        'inventory_gaps': len(known), 'extract_rows': len(scout), 'ledger_rows': len(ledger),
        'ledger_statuses': dict(Counter(row['status'] for row in ledger)),
        'pending_prose': sum(row['status'] == 'audit-pending' for row in scout),
    })
    print((HERE / 'p801-accounting-summary.json').read_text())


if __name__ == '__main__':
    main()
