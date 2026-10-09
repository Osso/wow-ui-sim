"""Replay unchanged historical default extraction bytes without current tools."""
import importlib.util
from pathlib import Path
import sys
sys.dont_write_bytecode = True
E = Path(__file__).resolve().parent
path = E/'historical-tools/extract_patch_non_inventory.py'
spec = importlib.util.spec_from_file_location('historical_extract',path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
raw = (E/'source.wikitext').read_text()
text = module.extract_text(raw)
assert text.encode() == (E/'default-extract.txt').read_bytes(), 'default extract differs'
print('default extract byte-identical; no opt-in flags, no linked expansion')
