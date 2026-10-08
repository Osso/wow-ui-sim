"""Summarize exact selectors, terminal counts and identical assertion failures."""
import hashlib
import json
from pathlib import Path
import re
from run_checks import SELECTORS

HERE = Path(__file__).resolve().parent


def read(name):
    return (HERE / name).read_text()


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def cases(name):
    return set(re.findall(r'^(\S+): test$', read(name + '.txt'), re.M))


def failures(name):
    return sorted(set(re.findall(r'^test (\S+) \.\.\. FAILED', read(name + '.txt'), re.M)))


def proof(name):
    return json.loads(read(name + '.proof.json'))


def result(name):
    text = read(name + '.txt')
    panics = []
    for case, location, message in re.findall(r"thread '([^']+)' \([^\n]+\) panicked at ([^\n]+):\n(.*?)(?=\n\n|\Z)", text, re.S):
        location = re.sub(r'^.*?/(tests|src)/', r'\1/', location)
        message = message.split('\nnote: run with ')[0].split('\nLua error ')[0]
        panics.append({'case': case, 'location': location, 'message': message})
    return {'receipt': name + '.proof.json', 'exit': proof(name)['exit'],
            'summaries': re.findall(r'^test result: .+$', text, re.M), 'failures': failures(name),
            'panics': sorted(panics, key=lambda row: (row['case'], row['location']))}


if __name__ == '__main__':
    rows = []
    for target in ['integration', 'prefork_full_ui']:
        branch_cases = cases(target + '-list')
        master_cases = cases('master-' + target + '-list')
        selectors = []
        for selector in SELECTORS:
            selected = sorted(case for case in branch_cases if selector in case)
            old = sorted(case for case in master_cases if selector in case)
            selectors.append({'selector': selector, 'branch_count': len(selected), 'master_count': len(old),
                              'branch_only': sorted(set(selected) - set(old)), 'master_only': sorted(set(old) - set(selected))})
        if target == 'integration':
            branch = result(target + '-regressions')
            master = result('master-' + target + '-regressions')
            assert branch['failures'] == master['failures'] and branch['panics'] == master['panics']
            rows.append({'target': target, 'selectors': selectors, 'branch': branch, 'master': master})
        else:
            checks = []
            for selector in SELECTORS:
                branch = result('prefork-' + selector)
                master = result('master-prefork-' + selector)
                assert branch['failures'] == master['failures'] and branch['panics'] == master['panics'], selector
                checks.append({'selector': selector, 'branch': branch, 'master': master})
            rows.append({'target': target, 'selectors': selectors, 'checks': checks})
    for target in ['integration', 'prefork_full_ui']:
        selectors = ['cvar', 'taint']
        branch_cases = cases('mists-' + target + '-list')
        matched = {selector: sorted(case for case in branch_cases if selector in case) for selector in selectors}
        if target == 'integration':
            branch = result('mists-' + target + '-regressions')
            master = result('master-mists-' + target + '-regressions')
            assert branch['failures'] == master['failures'] and branch['panics'] == master['panics']
            rows.append({'profile': 'mists', 'target': target, 'selectors': matched, 'branch': branch, 'master': master})
        else:
            checks = []
            for selector in selectors:
                branch = result('mists-prefork-' + selector)
                master = result('master-mists-prefork-' + selector)
                assert branch['failures'] == master['failures'] and branch['panics'] == master['panics'], selector
                checks.append({'selector': selector, 'branch': branch, 'master': master})
            rows.append({'profile': 'mists', 'target': target, 'selectors': matched, 'supported': False,
                         'reason': 'Cargo target requires client-retail; no Mists prefork tests exist', 'checks': checks})
    dump('regression-comparison.json', rows)
    startup = {}
    for label in ['branch-startup', 'master-startup']:
        output = read(label + '.txt')
        assert proof(label)['exit'] == 0 and output.rstrip().endswith('[]')
        assert 'Status: CLEAN' in output
        startup[label] = {'errors': [], 'addons_enabled': '--no-addons' not in proof(label)['command'],
                          'receipt': label + '.proof.json'}
    dump('startup-comparison.json', startup)
    receipts = {p.name.removesuffix('.proof.json'): json.loads(p.read_text()) for p in sorted(HERE.glob('*.proof.json'))}
    dump('receipts.json', receipts)
    print(json.dumps({'regression_rows': len(rows), 'receipts': len(receipts), 'startup': '[] / []'}))
