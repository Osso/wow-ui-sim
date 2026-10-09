"""Separate exact same-Era publication precedence; never mutate frozen ledgers."""
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
    assert [s['patch'] for s in inputs] == [s['patch'] for s in frozen['successors']], 'same-Era successor order'
    records = []
    summaries = []
    for item in inputs:
        assert item['history'] == 'classic-era' and (not item['native_credit']) and (not item['model_credit']), 'history/credit boundary'
        data = (E / item['path']).read_bytes()
        assert audit.digest(data) == item['sha256'], 'actual successor ledger hash'
        ledger = json.loads(data)
        assert ledger['patch'] == item['patch'], 'successor patch'
        source = (E / 'successors' / item['patch'] / 'source.wikitext').read_text().splitlines()
        occurrences = ledger[item['inventory_field']]
        contextual = item['inventory_field'] == 'api_occurrences'
        inventory = [] if contextual else occurrences
        matches = []
        for row in inventory:
            n = row['line'] if 'line' in row else row['wikitext_line']
            assert row['literal'] == source[n - 1], 'literal successor row'
            section = SECTIONS.get(row['section'], row['section'])
            matching = [r for r in frozen['inventory'] if r['symbol'] == row['symbol'] and r['section'] == section]
            for prior in matching:
                record = dict(patch=item['patch'], source_id=prior['id'], source_line=prior['line'], symbol=prior['symbol'], section=section, later_line=n, later_literal=row['literal'], later_direction=row['direction'], proof='SOURCE publication precedence only; original semantic/native/model limits remain UNPROVEN')
                records.append(record)
                matches.append(record)
        summaries.append(dict(patch=item['patch'], ledger_sha256=item['sha256'], inventory_occurrences=len(inventory), contextual_occurrences=len(occurrences) if contextual else 0, contextual_limit='No added/removed/changed signature semantics inferred from contextual name mentions.', exact_overlap_occurrences=len(matches)))
    occurrence_status = []
    for row in frozen['inventory']:
        overlapping = [r for r in records if r['source_id'] == row['id']]
        occurrence_status.append(dict(source_id=row['id'], symbol=row['symbol'], overlaps=overlapping, latest=overlapping[-1] if overlapping else None, semantic_status='UNPROVEN', native_credit=False, model_credit=False))
    return dict(schema='separate-same-era-publication-precedence/v1', patch='1.13.4', base_revision=audit.BASE, inputs=inputs, successor_summaries=summaries, overlaps=records, occurrence_status=occurrence_status, totals=dict(actual_registers=len(inputs), source_inventory=len(occurrence_status), overlap_occurrences=len(records), source_occurrences_with_overlap=sum((bool(r['overlaps']) for r in occurrence_status)), meaningful_historical_closures=0, native_closures=0), scope='No linked/transcluded expansion; no mutation of frozen queue, ledger, seals; no imported runtime proof.')

def validate(obj):
    assert obj == build(), 'serialized successor precedence'
    return obj['totals']
if __name__ == '__main__':
    if sys.argv[1:] == ['capture']:
        path = E / 'successor-application.json'
        assert not path.exists(), 'refuse application overwrite'
        path.write_text(json.dumps(build(), indent=2, ensure_ascii=False) + '\n')
    else:
        print(json.dumps(validate(audit.read_json('successor-application.json')), indent=2))
