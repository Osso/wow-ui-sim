"""Portable historical 5.4.1 gate; replay original invariants from mapped pins."""
from pathlib import Path
import runpy
import sys

sys.dont_write_bytecode = True
runpy.run_path(str(Path(__file__).resolve().parent / 'integrated/replay_history.py'),
               run_name='__main__')
