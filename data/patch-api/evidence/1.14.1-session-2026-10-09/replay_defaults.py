"""Replay unchanged extractor defaults against retained raw bytes."""
import importlib.util
from pathlib import Path
import sys

E = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('historical_extract', E / 'historical-tools/extract_patch_non_inventory.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
text = module.extract_text((E / 'source.wikitext').read_text())
if sys.argv[1:] == ['capture']:
    assert not (E / 'default-extract.txt').exists()
    (E / 'default-extract.txt').write_text(text)
else:
    assert text.encode() == (E / 'default-extract.txt').read_bytes(), 'default extract differs'
    print('default extract byte-identical')
