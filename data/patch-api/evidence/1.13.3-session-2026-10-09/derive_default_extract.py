"""Record historical default extractor outcome without a repository lookup."""
import importlib.util
import json
from pathlib import Path
E = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('extractor', E / 'historical-tools/extract_patch_non_inventory.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
try:
    text = module.extract_text((E / 'source.wikitext').read_text())
except ValueError as error:
    result = dict(result='existing-failure', text=None, error_type='ValueError', error=str(error), flags=[])
else:
    result = dict(result='success', text=text, error_type=None, error=None, flags=[])
assert not (E / 'default-extract-outcome.json').exists(), 'refuse overwrite'
(E / 'default-extract-outcome.json').write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
if result['result'] == 'success':
    (E / 'default-extract.txt').write_text(result['text'])
print(json.dumps(result, indent=2))
