"""Portable dispatcher for unchanged historical 5.0.4 proof."""
from pathlib import Path
import runpy

runpy.run_path(str(Path(__file__).resolve().parent / 'integrated/replay_history.py'), run_name='__main__')
