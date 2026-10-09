"""Literal frozen retail 2.4.2 contracts; publication is not native/model parity."""
import re

from gen_patch_wikitext_register import parse_retail_tbc_summary

RAW_NOTES = {
    5: 'Level-grant acceptance lacks a modeled pending grant/level lifecycle; signature unspecified.',
    6: 'Object-filter helper has no bare retail publication; temporary combat-log compatibility is not loaded here and is not historical parity.',
    7: 'Level-grant decline lacks a modeled pending grant lifecycle; signature unspecified.',
    8: 'Legacy GetCoinText is absent in the bare factory. Current C_CurrencyInfo amount/separator behavior passes separately; no historical alias or native output credit.',
    9: 'Quest-log spell rewards lack established spell-link identity and return contracts.',
    10: 'Selected quest spell rewards lack established spell-link identity and return contracts.',
    11: 'strreplace has no stated signature, replacement rule, or error contract; no guessed alias.',
    15: 'Last-number singular/plural rendering and elimination of unspecified *_P1 constants lack runtime proof; preserve both examples and placeholder literally.',
    18: 'Six literal constant names retained separately; callable classifier does not measure constant values, retirement or plural-localization migrations.',
    19: 'Strict officer-note escape evaluation lacks literal escape grammar and roster write/backend proof.',
    20: 'PingLocation publication is not protected-call behavior; targeting AoE security transition unproven.',
    21: 'Negative/zero combat-log cursor shift lacks real retained-log history/native indexing proof; no placeholder model credit.',
    22: 'AddMessage publication is not addToStart behavior; frozen page does not give argument position/default, so no guessed stack slot.',
    26: 'GUID/name request throttle (128 per 30 seconds, queued) lacks resolver, clock and network lifecycle model.',
    28: 'AH getall resolver saturation and 5-10 minute Unknown names are workload/context observations, not exact timing proof; no auction/resolver integration.',
}


def row(source_id, number, literal, note, status='audit-pending', capabilities=None):
    return dict(source_id=source_id, wikitext_line=number, literal=literal,
                capabilities=capabilities or [], status=status, note=note)


def account_source(raw, register, observations):
    entries = parse_retail_tbc_summary(raw)
    assert register['entries'] == entries, 'literal inventory'
    assert set(observations) == {entry['id'] for entry in entries}, 'observation set'
    rows = []
    for number, line in enumerate(raw.splitlines(), 1):
        if not line.strip():
            continue
        metadata = line.startswith(('{{apichanges|', '=')) or line == '{{reflist}}'
        rows.append(row(f'raw-{number:03}', number, line,
                        'Navigation/editorial heading/reference list; no runtime credit.' if metadata
                        else RAW_NOTES[number], 'metadata-only' if metadata else 'audit-pending'))
    for entry in entries:
        observed = observations[entry['id']]
        ok = observed['ok']
        rows.append(row(entry['id'], entry['wikitext_line'], entry['annotation'],
                        ('Supported current bare-retail factory callable publication only; no signature, model, cached Game or historical native credit.'
                         if ok else RAW_NOTES[entry['wikitext_line']]),
                        'bounded-coverage' if ok else 'audit-pending',
                        ['current-retail-factory-publication'] if ok else []))
        # Empty parentheses are an abbreviated source signature, not a zero-argument guarantee.
        tail = re.sub(r'^.*?(?:\]\]|\}\})', '', entry['annotation'], count=1)
        signature = re.match(r'\([^)]*\)', tail)
        literal = entry['symbol'] + (signature[0] if signature else ' [signature unspecified]')
        rows.append(row(f"signature-{entry['symbol']}-{entry['wikitext_line']}",
                        entry['wikitext_line'], literal,
                        'Literal abbreviated/partial signature only; return arity, types, defaults, errors and native acceptance remain unproven.'))
    for direction, names in [('removed', ('GOLD', 'SILVER', 'COPPER')),
                             ('added', ('GOLD_AMOUNT', 'SILVER_AMOUNT', 'COPPER_AMOUNT'))]:
        for name in names:
            rows.append(row(f'constant-{name}-18', 18, name,
                            f'Literal {direction} constant name; value/localization and current-publication/retirement unmeasured.'))
    for number, names in [(21, ('CombatLogGetNumEntries',)),
                          (28, ('QueryAuctionItems', 'GetAuctionItemInfo'))]:
        for name in names:
            rows.append(row(f'context-api-{name}-{number}', number, name,
                            'Contextual API reference, not a new/changed publication occurrence; linked documentation not expanded.'))
    assert len(rows) == len({item['source_id'] for item in rows}), 'duplicate source IDs'
    return rows
