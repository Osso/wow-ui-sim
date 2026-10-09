"""Resume only unproved gates after canonicalizing the master pin."""
import sys
from finalize_integration import read, finish_gates
comparison = read('gap-comparison.json')
observations = sum(row['observations'] for row in comparison if row['patch'] != '5.4.0')
fixtures = sum(int(count) for row in read('command-results.json').values() for count in row['python_tests'])
sys.exit(finish_gates(observations, fixtures))
