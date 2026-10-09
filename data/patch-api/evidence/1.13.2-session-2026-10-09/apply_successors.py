"""Separate actual same-Era identity precedence; original SOURCE/seals immutable."""
from collections import defaultdict
import json
from pathlib import Path
import sys
import audit
E = Path(__file__).resolve().parent
SECTIONS = {'Global API': 'global-api', 'Events': 'events', 'CVars': 'cvars', 'Widgets': 'widgets'}

def build(inputs=None):
    original = audit.read_json('applied-successors/inputs.json')
    inputs = original if inputs is None else inputs
    assert inputs == original, 'actual ordered successor inputs'
    frozen = audit.build()
    assert [i['patch'] for i in inputs] == [i['patch'] for i in frozen['successors']], 'same-Era order'
    by_identity = defaultdict(list)
    for row in frozen['inventory']:
        by_identity[row['section'], row['symbol']].append(row)
    records, summaries = ([], [])
    for item in inputs:
        assert item['history'] == 'classic-era' and (not item['native_credit']) and (not item['model_credit']), 'history/credit boundary'
        data = (E / item['path']).read_bytes()
        assert audit.digest(data) == item['sha256'], 'actual successor ledger hash'
        ledger = json.loads(data)
        assert ledger['patch'] == item['patch'], 'successor patch'
        pin = next((i['pin'] for i in frozen['successors'] if i['patch'] == item['patch']))
        assert ledger['source'] == pin, 'successor source authority'
        source = (E / 'successors' / item['patch'] / 'source.wikitext').read_text().splitlines()
        occurrences = ledger[item['inventory_field']]
        contextual = item['inventory_field'] == 'api_occurrences'
        inventory = [] if contextual else occurrences
        matches = []
        for row in inventory:
            n = row['line'] if 'line' in row else row['wikitext_line']
            assert row['literal'] == source[n - 1], 'literal successor row'
            section = SECTIONS.get(row['section'], row['section'])
            for prior in by_identity[section, row['symbol']]:
                record = dict(patch=item['patch'], source_id=prior['id'], source_line=prior['line'], symbol=prior['symbol'], section=section, source_direction=prior['direction'], later_line=n, later_literal=row['literal'], later_direction=row['direction'], proof='Exact SOURCE identity/direction precedence only; no historical semantics/model/native closure.')
                records.append(record)
                matches.append(record)
        summaries.append(dict(patch=item['patch'], ledger_sha256=item['sha256'], inventory_occurrences=len(inventory), contextual_occurrences=len(occurrences) if contextual else 0, contextual_limit='Contextual names are not added/removed declarations; no direction, linked subset or signature inference.', exact_overlap_occurrences=len(matches)))
    by_source = defaultdict(list)
    for record in records:
        by_source[record['source_id']].append(record)
    occurrence_status = [dict(source_id=row['id'], symbol=row['symbol'], section=row['section'], source_direction=row['direction'], overlaps=by_source[row['id']], latest=by_source[row['id']][-1] if by_source[row['id']] else None, semantic_status='UNPROVEN', native_credit=False, model_credit=False) for row in frozen['inventory']]
    return dict(schema='separate-same-era-publication-precedence/v1', patch='1.13.2', base_revision=audit.BASE, inputs=inputs, successor_summaries=summaries, overlaps=records, occurrence_status=occurrence_status, totals=dict(actual_registers=len(inputs), source_inventory=len(occurrence_status), overlap_occurrences=len(records), source_occurrences_with_overlap=sum((bool(i['overlaps']) for i in occurrence_status)), overlaps_by_patch={i['patch']: i['exact_overlap_occurrences'] for i in summaries}, meaningful_historical_closures=0, native_closures=0), scope='Separate receipt: actual integrated same-Era ledgers only, never foreign semantic supersession or original ledger/queue/seal mutation.')

def validate(obj, expected=None):
    assert obj == (build() if expected is None else expected), 'serialized actual successor precedence'
    return obj['totals']
if __name__ == '__main__':
    if sys.argv[1:] == ['capture']:
        path = E / 'successor-application.json'
        assert not path.exists(), 'refuse receipt overwrite'
        path.write_text(json.dumps(build(), indent=2, ensure_ascii=False) + '\n')
    else:
        print(json.dumps(validate(audit.read_json('successor-application.json')), indent=2))
