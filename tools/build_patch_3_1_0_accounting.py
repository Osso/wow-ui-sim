"""Build source-only 3.1.0 ledgers from literal pins and own observed publication.

No native or modeled behavior is inferred from a callable/absent identity.
"""
from collections import Counter
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
SOURCES = ROOT / 'data/patch-api/sources'
EVIDENCE = ROOT / 'data/patch-api/evidence/3.1.0-session-2026-10-09'
CALL = re.compile(r'([A-Za-z_]\w*(?::[A-Za-z_]\w*)?)\([^)]*\)')


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def semantic_limit(section, subsection):
    if section == 'Talent Functions':
        return 'UNPROVEN: historical 1-based talent/glyph groups, inspect/pet selection, active-group defaults, preview allocation/prerequisites and learning lifecycle; modern specialization presence is not this contract.'
    if section == 'Slash Commands':
        return 'UNPROVEN: /useTalents 1|2 and [spec:1] macro selection, parser dispatch and action behavior; examples are not executed by publication probes.'
    if subsection in ('UnitAura', 'DEPRECATED isMine()'):
        return 'UNPROVEN: legacy aura return tuple, caster unit-token/controller fallback, arena tokens and player-owned-first ordering; modern aura names or IDs do not prove legacy semantics.'
    if subsection == 'Animation and Mouse Hover' or section == 'Secure Handlers Updates':
        return 'UNPROVEN: ownerless secure SetUpAnimation removal, unnamed restored methods/Show-Hide driver, hover expiration/reset, explicit-hide/move cancellation, child rectangles and noncombat restriction. Source explicitly reports PTR bugs; linked secure guide is unexpanded.'
    if subsection == 'GetInventoryItemsForSlot()':
        return 'UNPROVEN: slot-indexed base-item-ID numerical-location table and optional supplied-table mutation; publication does not prove location encoding or item identities.'
    if subsection == 'GameTooltip:SetGlyph()':
        return 'UNPROVEN: historical talentGroup input/selection and glyph tooltip output; empty parentheses do not specify full argument list.'
    if subsection == 'GetPlayerFacing() UPDATE':
        return 'UNPROVEN: historical/native PTR correction and radians return; constant publication cannot prove player-orientation state.'
    if subsection == 'Target Facing':
        return 'Source explicitly declines facing for other units; question/example carries no new GetTargetFacing publication or signature contract.'
    return 'UNPROVEN: identity-only historical source supplies no complete argument/coercion, return-value or server/state-transition contract; current publication is separate evidence.'


def main():
    raw = (SOURCES / '3.1.0-api-changes.wikitext').read_text()
    register = json.loads((SOURCES / '3.1.0-wikitext-register.json').read_text())
    observed = json.loads((EVIDENCE / 'own-sweep-green-results.json').read_text())
    entries = register['entries']
    by_line = {row['wikitext_line']: row for row in entries}
    assert set(observed) == {row['id'] for row in entries}
    inventory = []
    for entry in entries:
        result = observed[entry['id']]
        note = ('Publication/absence only; historical call semantics remain UNPROVEN.' if result['ok']
                else 'UNPROVEN publication mismatch: ' + json.dumps(result, ensure_ascii=False))
        inventory.append(dict(source_id=entry['id'], status='bounded-coverage' if result['ok'] else 'audit-pending',
                              capabilities=['publication-only'] if result['ok'] else [], note=note))
    signatures, raw_rows = [], []
    section = subsection = ''
    for number, line in enumerate(raw.splitlines(), 1):
        if not line.strip():
            continue
        heading = re.fullmatch(r'(={1,4})\s*([^=]+?)\s*\1', line)
        if heading:
            if len(heading[1]) <= 2:
                section, subsection = heading[2].strip(), ''
            else:
                subsection = heading[2].strip()
        limit = semantic_limit(section, subsection)
        calls = list(CALL.finditer(line))
        for ordinal, call in enumerate(calls, 1):
            identity_only = not call[0].split('(', 1)[1][:-1].strip()
            signatures.append(dict(source_id=f'signature-{number:03}-{ordinal}', symbol=call[1],
                                   wikitext_line=number, source_text=line, fragment=call[0],
                                   source_role='identity/question/heading/empty-call' if identity_only else 'literal-arguments',
                                   status='audit-pending', capabilities=[], note=limit))
        metadata = bool(heading or number in by_line or number < 12)
        note = ('Literal source line; inventory/signature IDs carry separate proof limits. ' + limit
                if number in by_line else ('Source context/navigation/editorial only.' if metadata else limit))
        raw_rows.append(dict(source_id=f'raw-line-{number:03}', wikitext_line=number, source_text=line,
                             section=section, subsection=subsection,
                             status='metadata-only' if metadata else 'audit-pending', capabilities=[], note=note))
    spec = importlib.util.spec_from_file_location('extract', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text = (SOURCES / '3.1.0-api-changes.txt').read_text()
    extracts = extractor.seed_rows(text, '3.1.0')
    for row in extracts:
        row.update(status='metadata-only', capabilities=[], note='Rendered full-source mirror. Original nonblank raw-line and signature IDs retain all semantics and precise limits; no duplicate credit.')
    signature_ledger = [{k: row[k] for k in ('source_id', 'status', 'capabilities', 'note')} for row in signatures]
    ledger = inventory + extracts + raw_rows + signature_ledger
    assert len({row['source_id'] for row in ledger}) == len(ledger)
    write_json(SOURCES / '3.1.0-signatures.json', dict(schema='patch-api-literal-signatures/v1', signatures=signatures))
    write_json(SOURCES / '3.1.0-page-coverage.json', dict(schema='patch-api-page-coverage/v1', patch='3.1.0', client_line='retail',
        proof_policy='Own observed publication only. Raw lines and literal fragments include questions/headings/examples without promoting them to contracts. Native/model/security parity UNPROVEN.', source_rows=ledger))
    print(json.dumps(dict(inventory=len(inventory), extracts=len(extracts), raw_nonblank=len(raw_rows),
                          signatures=len(signatures), ledger=len(ledger), statuses=dict(Counter(row['status'] for row in ledger)))))


if __name__ == '__main__':
    main()
