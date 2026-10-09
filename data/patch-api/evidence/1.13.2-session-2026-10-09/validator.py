"""Separate current receipts; derive counts without Git/target/current-tool dependencies."""
import json
from pathlib import Path
import audit
import apply_successors
E = Path(__file__).resolve().parent

def validate_model(observations):
    cases = audit.read_json('current-model-cases.json')
    assert cases, 'nonempty explicit current model cases'
    expected = [dict(name=c['name'], input=c['input'], observed=[True, c['input'], c['boolean']], rust_store=c['input']) for c in cases]
    assert observations['observations'] == expected, 'exact current seeded storage observations'
    assert observations['raw_types'] == ['function', 'function', 'function'], 'real registered current reads/writes'
    assert observations['configured_interface'] == audit.profiles()[0]['configured_interface'], 'current configured profile'
    assert observations['source_interface'] == audit.read_json('ledger.json')['source_toc'], 'source/current epochs distinct'
    assert observations['runtime_changes'] == 0, 'no runtime implementation change'
    for key in ['native_credit', 'source_default_credit', 'historical_signature_credit', 'cvar_effect_credit', 'source_ledger_mutated']:
        assert observations[key] is False, f'no credit: {key}'
    return len(expected)

def check_receipt_seals():
    seals = audit.read_json('receipt-seals.json')
    assert 'seals.json' in seals and 'originals.tar.gz' in seals, 'anchor unchanged original map/archive in separate receipts'
    for name, expected in seals.items():
        assert audit.digest((E / name).read_bytes()) == expected, f'receipt seal: {name}'
    return len(seals)

def check_portable_receipts():
    proof = audit.read_json('portable-green-proof.json')
    assert proof['exit_code'] == 0, 'portable GREEN'
    result = audit.read_json('portable-receipts.json')
    assert result['failures'] == 0 and result['errors'] == 0, 'all copied controls pass'
    cases = {r['test'] for r in result['runs'] if 'test' in r}
    assert len(cases) == result['tests_run'], 'derived portable test count'
    originals = audit.read_json('seals.json')
    restores = {r['restored']: r for r in result['runs'] if 'restored' in r}
    assert set(restores) == {'ledger.json', 'green.log'}, 'both serialized controls restored'
    for name, restore in restores.items():
        assert restore['sha256'] == originals[name], 'exact restored original bytes'
        assert restore['all_original_seals'] == len(originals), 'all original seals preserved'
        failures = [r for r in result['runs'] if 'test' in r and r['exit_code'] != 0 and (f'seal: {name}' in r['stderr'])]
        assert failures, f'serialized rejection receipt: {name}'
    return (result['tests_run'], sorted(restores))

def validate():
    original_seals = audit.check_seals()
    receipt_seals = check_receipt_seals()
    audit.check_defaults()
    source = audit.read_json('ledger.json')
    source_totals = audit.validate_ledger(source)
    assert source['measurements'] == dict(runtime=0, model=0, native=0), 'SOURCE is not historical runtime/model/native credit'
    assert audit.read_json('source-green-proof.json')['exit_code'] == 0, 'original SOURCE GREEN'
    assert audit.read_json('successor-green-proof.json')['exit_code'] == 0, 'separate successor GREEN'
    successor_totals = apply_successors.validate(audit.read_json('successor-application.json'))
    transitions = validate_model(audit.read_json('current-model-observations.json'))
    portable_tests, restored = check_portable_receipts()
    return dict(original_seals=original_seals, separate_receipt_seals=receipt_seals, source_totals=source_totals, actual_successor_totals=successor_totals, current_model_transitions=transitions, portable_tests=portable_tests, exact_restored_files=restored, historical_credit=source['measurements'], scope='bounded frozen page SOURCE and separate receipts, not main integration/native/final acceptance')
if __name__ == '__main__':
    print(json.dumps(validate(), indent=2))
