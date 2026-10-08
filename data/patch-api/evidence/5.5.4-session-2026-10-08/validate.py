"""Portable historical Mists Classic gate; replay original unchanged invariants."""
from pathlib import Path
import runpy
import sys

sys.dont_write_bytecode = True
runpy.run_path(str(Path(__file__).resolve().parent / 'integrated/replay_history.py'),
               run_name='__main__')
