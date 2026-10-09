"""Portable 5.4.0 history and integrated proof entrypoint."""
from pathlib import Path
import runpy
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent / 'integrated'
runpy.run_path(str(HERE / 'replay_history.py'), run_name='__main__')
runpy.run_path(str(HERE / 'validate.py'), run_name='__main__')
