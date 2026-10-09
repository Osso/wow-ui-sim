"""Deterministic literal 3.0.2 ledger; frozen source is the only contract input."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ACTUAL = ['3.3.0', '3.3.3', '3.3.5', '4.0.1']
QUEUE = ['3.0.3', '3.0.8', '3.1.0', '3.2.0']


def digest(data):
    return hashlib.sha256(data).hexdigest()


def tool(evidence, name):
    spec = importlib.util.spec_from_file_location(name, evidence / ('historical-' + name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def reason(entry):
    symbol, text = entry['symbol'], entry['annotation']
    if entry['direction'] == 'removed':
        return ('Literal historical removal only. No modern-retail caller/consumer scan or runtime absence measurement; '
                'no retirement or Classic credit. Replacement text is not a successor proof.')
    if '??' in text or '?' in text:
        return 'Source explicitly marks unknown/uncertain fields or syntax; no complete argument/return/domain contract or measured result.'
    if 'kind' in entry:
        return {'event': 'Literal event name/payload prose retained; no causal model transition, dispatch-order or native event measurement.',
                'cvar': 'Literal synchronization CVar retained; server persistence/sync model and native setting transitions unmeasured.',
                'click-modifier': 'Literal FOCUSCAST modifier retained; no default is specified, focus casting transition unmeasured.',
                'widget-script': 'Literal widget handler retained; achievement tooltip ownership/dispatch lifecycle unmeasured.'}[entry['kind']]
    if symbol.startswith(('Model:', 'PlayerModel:')):
        return '3D model display is intentionally unsupported; literal historical signature retained without 3D behavior or native credit.'
    if symbol.startswith(('Calendar', 'GetComparison', 'GetCriteriaComparison', 'GetLatestUpdatedComparison', 'GetLatestCompletedComparison')):
        return 'Named historical server/calendar/comparison operation lacks bounded state transitions and native fixture; no placeholder model.'
    if symbol in ('GetCVarBool', 'GetCVarInfo', 'UnitAura', 'UnitBuff', 'UnitDebuff', 'CancelPlayerBuff', 'UnitMana', 'UnitManaMax'):
        return 'Explicit historical return/alias contract is not the selected modern retail epoch; no historical runtime/profile or matching native capture. Do not overwrite modern behavior.'
    if symbol.startswith(('Texture:', 'Slider:', 'Button:', 'GameTooltip:')):
        return 'Literal widget contract retained. Existing modern method names alone prove neither historical outputs nor state/render/dispatch behavior; no reproduced in-scope bug or historical native fixture.'
    if symbol == 'UnitExists':
        return 'Literal Petlover/name-prefix requirement retained. Current token-only resolver has no general visible-name identity policy; page says unit functions plural, not a bounded one-function alias contract. No invented name precedence/case/visibility model.'
    if symbol in ('BackupMacros', 'RestoreMacros', 'UploadSettings', 'DownloadSettings'):
        return 'Requires server-stored backup/sync state and combat restriction transitions; source does not define backup failure/ownership semantics. No local copy shim.'
    if symbol.startswith('InterfaceOptionsFrame_'):
        return 'Historical FrameXML rename, not native publication. No linked implementation expansion or deletion of current wrappers.'
    return ('Literal identity/signature retained; no bounded behavioral observation for this occurrence. '
            'Arguments/returns alone do not establish state, failure or lifecycle behavior; no speculative model or publication-only parity claim.')


def prose_reason(number, entries):
    notes = {
        4: 'self/local arguments replace this/argN; no named complete handler signatures, hook ordering or obsolete-argument boundary specified.',
        6: 'SecureStateHeader replacement is not named; linked SecureHeadersGuide not expanded. No secure-template execution contract invented.',
        7: 'External guide URL is a source boundary only, not fetched documentation or a native contract.',
        9: 'Server-stored bindings/macros/settings require persistence and account/server transition fixtures; none measured.',
        11: 'Button Font-object migration is qualified as appears; no complete per-state ownership/lifecycle contract or native comparison.',
        13: 'Crafting-family retirement/enchanting migration has no complete renamed mapping; explicit removals elsewhere retained individually.',
        24: '36 account-wide macro slots is literal historical capacity, not the modern selected retail epoch; backup/combat/persistence behavior unmeasured.',
        27: 'GetPlayerBuff family is not expanded into guessed APIs. UnitAura addition and explicit rows elsewhere retained; historical aura layout not modern parity.',
        29: 'Introduces the following literal partial aura signatures; no separate behavior or native measurement.',
        40: 'Six aura filters, space/pipe chaining, implicit HELPFUL and alias ignoring rules retained; historical index/name/rank model and failure cases unmeasured.',
        41: 'Addon advice to cache expiration and avoid OnUpdate re-query is not an API retirement or simulator optimization permission.',
        42: 'untilCanceled is described but absent from the listed return tuple; inconsistent source cannot establish a return position.',
        45: 'Spell-based mount/pet migration retained; forum post #57 not expanded, ownership/cursor/summon/cooldown transitions undefined.',
        52: 'PlayerModel method introduction is context; exact SetCreature row retained. 3D display intentionally unsupported.',
        97: 'panel.refresh is a literal optional panel callback after defaults/before show; no named registration API, argument signature, or hook/reentrancy policy specified.',
        100: 'Item strings include character level but no full grammar, placement or backward-compatibility contract is given.',
        357: 'Six inviteStatus numeric labels and absence of global constants retained; not enum publication/native proof or calendar invite state transitions.',
        358: 'Four literal sort criteria retained; ordering/ties/case/reverse semantics not specified.',
    }
    if number in notes:
        return notes[number]
    if number in list(range(60, 65)) + list(range(67, 72)):
        return 'Threat state thresholds/tanking colors and detailed fields retained; no mob/actor identity, distance scaling or causal event transition fixture. Do not replace threat state with fixed scalars.'
    related = next((entry for entry in entries if entry['wikitext_line'] == number), None)
    return reason(related) if related else 'Exact prose retained; no bounded state-transition test or native observation. Linked domains are not expanded.'


def build_ledger(evidence):
    original = evidence / 'original'
    raw = (original / 'source.wikitext').read_text()
    lines = raw.splitlines()
    entries = tool(evidence, 'gen_patch_wikitext_register').parse_wrath_launch_inventory(raw)
    inventory = [dict(row, status='UNPROVEN', capabilities=[], note=reason(row)) for row in entries]
    byline = {}
    for row in entries:
        byline.setdefault(row['wikitext_line'], []).append(row['id'])
    headers, prose, signatures, source_rows = [], [], [], []
    context = ''
    for number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        heading = re.fullmatch(r'\s*(=+)\s*(.*?)\s*\1\s*', line)
        ids = byline.get(number, [])
        if heading:
            context = heading[2]
            headers.append({'id': f'header-{number}', 'wikitext_line': number, 'source_text': line,
                            'title': context, 'level': len(heading[1]), 'literal_count': None,
                            'status': 'metadata-only'})
        metadata = bool(heading or line.startswith('{{apichanges|'))
        if not metadata:
            # Every non-inventory statement is retained, including threats, URL boundaries,
            # inconsistent untilCanceled prose and unexpanded family/template descriptions.
            labeled = re.match(r'\s*\*?\s*(NEW|UPDATED|MODIFIED|REMOVED)\b', line)
            annotation = re.search(r'(?:\s+--\s+|\)\s+-\s+|\s+\(replaced by\s+)(.*)', line)
            explanation = annotation[1] if annotation else None
            if not ids or not labeled or explanation:
                prose.append({'id': f'prose-{number}', 'wikitext_line': number,
                              'source_text': line, 'context': context,
                              'literal_annotation': explanation, 'status': 'UNPROVEN', 'capabilities': [],
                              'note': prose_reason(number, entries)})
        source_rows.append({'id': f'source-{number}', 'wikitext_line': number, 'source_text': line,
                            'context': context, 'inventory_ids': ids,
                            'status': 'metadata-only' if metadata else 'UNPROVEN',
                            'category': 'metadata' if metadata else 'inventory' if ids else 'prose',
                            'capabilities': []})
    for row in entries:
        if row['section'] not in ('global-api', 'widgets', 'framexml') or row.get('kind') == 'widget-script':
            continue
        line = row['annotation']
        normalized = re.sub(r'\{\{api\|([^{}|]+)\}\}', r'\1', line)
        match = re.search(re.escape(row['symbol']) + r'\(([^)]*)\)', normalized)
        signatures.append({'id': 'signature-' + row['id'], 'inventory_id': row['id'],
                           'symbol': row['symbol'], 'wikitext_line': row['wikitext_line'],
                           'source_text': line, 'literal_arguments': match[1] if match else None,
                           'literal_return_prefix': normalized.split('=', 1)[0].strip() if '=' in normalized else None,
                           'status': 'UNPROVEN', 'capabilities': [],
                           'note': ('Literal syntax only; optional brackets/typos/??/ellipsis are not repaired. '
                                    'No explicit signature' if not match else
                                    'Literal signature text only; optional brackets/typos/??/ellipsis are not repaired.') +
                                   '; arity, accepted types, return types/positions, secrecy and failure behavior unmeasured.'})
    return {'schema': 'patch-source-accounting/v1', 'patch': '3.0.2', 'client_line': 'retail',
            'profile': 'retail', 'scope': 'frozen-source-not-runtime',
            'source': json.loads((original / 'source-pin.json').read_bytes()),
            'actual_successors': ACTUAL, 'queued_successors': QUEUE,
            'successor_policy': 'Main inserts actual queued registers in patch order; no queue supersession credit, no Classic/TBC/Era crossline.',
            'inventory_rows': inventory, 'prose_ledger': prose, 'signature_ledger': signatures,
            'header_ledger': headers, 'source_rows': source_rows}


def counts(ledger):
    return {'source_rows': len(ledger['source_rows']),
            'source_statuses': dict(Counter(r['status'] for r in ledger['source_rows'])),
            'inventory_occurrences': len(ledger['inventory_rows']),
            'kinds': dict(Counter(r.get('kind', r['section']) for r in ledger['inventory_rows'])),
            'directions': dict(Counter(r['direction'] for r in ledger['inventory_rows'])),
            'prose_limits': len(ledger['prose_ledger']), 'signature_limits': len(ledger['signature_ledger']),
            'explicit_signature_texts': sum(r['literal_arguments'] is not None for r in ledger['signature_ledger']),
            'headers': len(ledger['header_ledger']), 'literal_command_occurrences': 0,
            'actual_successors': ACTUAL, 'queued_successors': QUEUE,
            'native_observations': 0, 'runtime_observations': 0, 'meaningful_closures': 0}
