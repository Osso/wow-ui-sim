#!/usr/bin/env python3
"""Measure native exact-name invocations inside one admitted service."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
from contextlib import nullcontext


def open_memory_peak():
    relative = next(line.split(':', 2)[2].strip()
                    for line in Path('/proc/self/cgroup').read_text().splitlines()
                    if line.startswith('0:'))
    return (Path('/sys/fs/cgroup') / relative.lstrip('/') / 'memory.peak').open('r+')


def measure_native(command, peak):
    if peak is not None:
        peak.seek(0)
        peak.write('0')
        peak.flush()
    started = time.monotonic()
    with tempfile.TemporaryFile() as output:
        process = subprocess.Popen(command, stdout=output, stderr=subprocess.STDOUT)
        try:
            _, status, usage = os.wait4(process.pid, 0)
            process.returncode = os.waitstatus_to_exitcode(status)
        finally:
            if process.returncode is None:
                process.kill()
                process.wait()
        elapsed = time.monotonic() - started
        output.seek(0)
        text = output.read().decode('utf-8', errors='replace')
    memory = None
    if peak is not None:
        peak.seek(0)
        memory = int(peak.read())
    return {'exit': process.returncode, 'seconds': elapsed,
            'user_cpu_seconds': usage.ru_utime, 'system_cpu_seconds': usage.ru_stime,
            'cpu_seconds': usage.ru_utime + usage.ru_stime,
            'max_rss_kib': usage.ru_maxrss, 'memory_peak_bytes': memory}, text


def run_batch(manifest):
    records = []
    names = manifest.get('names')
    context = open_memory_peak() if manifest.get('measure_memory', False) else nullcontext()
    with context as peak:
        for name in names if names is not None else [None]:
            command = [manifest['binary']]
            if name is not None:
                command += ['--test-threads=1', '--format=pretty', '--color=never', '--exact', name]
            publish(manifest['result'], records, name)
            record, text = measure_native(command, peak)
            sys.stdout.write(text)
            sys.stdout.flush()
            if name is not None:
                outcomes = re.findall(r'^test ' + re.escape(name) + r' \.\.\. (ok|FAILED|ignored)\b', text, re.M)
                record['name'] = name
                record['status'] = ('failed' if record['exit'] else 'invalid' if not outcomes
                                    else 'ignored' if outcomes[-1] == 'ignored' else 'passed')
                if record['status'] == 'invalid':
                    record['exit'] = 2
                    print(f'Invalid native result: selected case {name} was not reported', file=sys.stderr)
            records.append(record)
            publish(manifest['result'], records, None)
    return next((r['exit'] for r in records if r['exit']), 0)


def publish(path, records, current):
    result = {'exit': next((r['exit'] for r in records if r['exit']), 0),
              'records': records, 'current_name': current}
    destination = Path(path)
    temporary = destination.with_suffix('.tmp')
    temporary.write_text(json.dumps(result, indent=2) + '\n')
    temporary.replace(destination)


if __name__ == '__main__':
    raise SystemExit(run_batch(json.loads(Path(sys.argv[1]).read_text())))
