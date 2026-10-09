"""Historical input/seal/receipt validator; no Git, target or current tools."""
import importlib.util
import json
from pathlib import Path
import re
import unittest
import audit
import apply_successors
E = Path(__file__).resolve().parent

def suite_count(filename):
    spec = importlib.util.spec_from_file_location(filename.removesuffix('.py'), E / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return unittest.defaultTestLoader.loadTestsFromModule(module).countTestCases()

def validate_tests(prefix, filename):
    receipt = audit.read_json(prefix + '-proof.json')
    log = (E / (prefix + '.log')).read_text()
    count = suite_count(filename)
    assert receipt['exit_code'] == 0, 'test receipt exit'
    assert re.search('Ran ' + str(count) + ' tests? in ', log), 'derived test count'
    assert log.rstrip().endswith('OK'), 'test success log'
    assert receipt['start_epoch'] <= receipt['end_epoch'], 'test epochs'
    return count

def validate():
    seals = audit.check_seals()
    audit.check_defaults()
    source = audit.validate_ledger(audit.read_json('ledger.json'))
    successor = apply_successors.validate(audit.read_json('successor-application.json'))
    source_tests = validate_tests('source-green', 'test_source_accounting.py')
    successor_tests = validate_tests('successor-green', 'test_successors.py')
    era = audit.read_json('era-totems-green-proof.json')
    era_log = (E / 'era-totems-green.log').read_text()
    results = re.findall('test result: ok\\. (\\d+) passed; (\\d+) failed;', era_log)
    assert era['exit_code'] == 0 and len(results) == 1 and (results[0][1] == '0'), 'current Era lifecycle receipt'
    assert era['start_epoch'] <= era['end_epoch'], 'Era epochs'
    argv = era['argv']
    assert '--offline' in argv and '--locked' in argv and ('--no-default-features' in argv), 'Era offline/locked profile'
    assert argv[argv.index('--features') + 1] == 'client-era', 'Era profile'
    return dict(original_seals=seals, source_totals=source, successor_totals=successor, source_tests=source_tests, successor_tests=successor_tests, current_era_tests=int(results[0][0]), native_historical_closures=0, scope='Historical copied inputs/receipts only; no rerun of current Era or final gates.')
if __name__ == '__main__':
    print(json.dumps(validate(), indent=2))
