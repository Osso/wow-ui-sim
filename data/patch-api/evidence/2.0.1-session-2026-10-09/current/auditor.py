#!/usr/bin/env python3
"""Literal SOURCE-only 2.0.1 accounting. No shared parser or runtime changes.

Occurrences are lexical mentions, not reconstructed publication inventory.
Every nonblank line is retained, including examples and later-patch qualifiers.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

HEADER = re.compile(r'^(={2,})\s*(.*?)\s*\1$')
CALL = re.compile(r'\b([A-Za-z_][\w]*(?:[.:][A-Za-z_][\w]*)*)\(')
EVENT = re.compile(r'\b[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)*(?:_\*|\*)|\b[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)+')
COMMAND = re.compile(r'(?<![\w:/<.])/[a-z]+\*?')
HANDLER = re.compile(r'\b(?:On[A-Z][A-Za-z]+|PreClick|PostClick)\b')
CONTEXT = re.compile(r'\b(?:[A-Z][a-z]+(?:[A-Z][A-Za-z0-9]*)+|FrameXML|SecureXML|Region|GameTooltip|Button|Frame|MovePad)\b\*?')
REFERENCES = [re.compile(pattern) for pattern in (
    r'\{\{[^{}]*\}\}', r'\[\[[^\]]+\]\]', r'\[https?://[^\]]+\]',
)]
LANGUAGE = {'function', 'pairs', 'table.getn'}


def digest(content):
    return hashlib.sha256(content).hexdigest()


def read_call(line, start, opening):
    """Preserve exact bytes; incomplete/quoted forms never gain a closing token."""
    depth, quote = 0, None
    for index in range(opening, len(line)):
        char = line[index]
        if quote:
            if char == quote and (index == 0 or line[index - 1] != '\\'):
                quote = None
        elif char in ('"', "'"):
            # Wikitext emphasis is not a Lua string delimiter.
            if not (char == "'" and (line[index:index + 2] == "''" or line[index - 1:index + 1] == "''")):
                quote = char
        elif char == '(':
            depth += 1
        elif char == ')':
            depth -= 1
            if depth == 0:
                return line[start:index + 1], 'literal-fragment'
    return line[start:], 'malformed-unclosed'


def direction_at(line, offset):
    prefix = line[:offset]
    if re.search(r'Formerly\s+\S*$', prefix):
        return 'former-name'
    if re.search(r'Replaces\s+[\w/]*$', prefix):
        return 'replaced-name'
    label = re.match(r'\s*\*+\s*(NEW|UPDATED|RENAMED)\b', line)
    return {'NEW': 'added', 'UPDATED': 'changed', 'RENAMED': 'renamed'}.get(
        label[1] if label else '', 'mentioned')


def find_occurrences(line, number, heading, qualifier):
    found, occupied = [], []

    def append(match, kind, signature=None, signature_status='unspecified', symbol=None, span=None):
        start, end = span or (match.span(1) if match.re is CALL else match.span())
        if any(start >= left and end <= right for left, right in occupied):
            return
        occupied.append((start, end))
        found.append({
            'id': f'occ-2.0.1-{number:03}-{start:04}', 'line': number,
            'column': start, 'symbol': symbol or line[start:end], 'kind': kind,
            'direction': direction_at(line, start), 'heading': heading,
            'patch_qualifier': qualifier, 'literal': line,
            'signature': signature, 'signature_status': signature_status,
            'declaration_literal': line,
            'status': 'UNPROVEN', 'capabilities': [],
            'limit': 'Literal mention/fragment only; publication, full signature, execution and native behavior unmeasured.',
        })

    for match in CALL.finditer(line):
        symbol = match[1]
        opening = line.find('(', match.end(1))
        signature, state = read_call(line, match.start(1), opening)
        if symbol == 'function':
            handler = re.match(r'\*\*\s*(\w+)\s*:', line)
            if handler:
                append(match, 'widget-script-signature', signature, state, handler[1], handler.span(1))
            else:
                append(match, 'language-example', signature, state)
        else:
            kind = ('language-example' if symbol in LANGUAGE else
                    'widget-method' if ':' in symbol else 'global-api')
            append(match, kind, signature, state)
    for match in EVENT.finditer(line):
        symbol = match[0]
        kind = ('constant' if symbol == 'CURRENT_ACTIONBAR_PAGE' else
                'event-family' if '*' in symbol else 'event')
        append(match, kind)
    for match in COMMAND.finditer(line):
        code_start = line.rfind('<code>', 0, match.start())
        code_end = line.find('</code>', match.end())
        inside_code = code_start >= 0 and '</code>' not in line[code_start:match.start()]
        fragment = line[match.start():code_end] if inside_code and code_end >= 0 else None
        prefix = line[code_start:match.start()] if inside_code else ''
        kind = ('macro-option-suffix' if 'reset=' in prefix else
                'command-placeholder' if match[0] == '/command' else
                'command-family' if '*' in match[0] else 'command')
        append(match, kind, fragment,
               'literal-command-fragment' if fragment is not None else 'unspecified')
    for match in HANDLER.finditer(line):
        # Named wrappers already have an occurrence at the function fragment.
        if re.match(r'\*\*\s*' + re.escape(match[0]) + r'\s*:\s*function\(', line):
            continue
        append(match, 'widget-script')
    for match in CONTEXT.finditer(line):
        name = match[0]
        kind = ('widget-template' if name.endswith('Template') else
                'global-api-reference' if direction_at(line, match.start()) == 'replaced-name' else
                'api-family' if name == 'TargetNearest*' else 'context-identifier')
        append(match, kind)
    for match in re.finditer(r"''string''\.(?:trim|split|join)", line):
        append(match, 'string-method-reference')
    return sorted(found, key=lambda row: row['column'])


def account(raw):
    rows, headers, occurrences, references = [], [], [], []
    heading, section_qualifier = '', None
    for number, line in enumerate(raw.splitlines(), 1):
        if not line.strip():
            continue
        header = HEADER.fullmatch(line)
        if header:
            heading = header[2]
            patch = re.search(r'\b2\.0\.\d+\b', heading)
            section_qualifier = patch[0] if patch else None
            headers.append({'id': f'header-2.0.1-{number:03}', 'line': number,
                            'title': heading, 'literal': line, 'count_claim': None})
        patches = re.findall(r'\*2\.0\.\d+\*', line)
        qualifier = patches[-1].strip('*') if patches else section_qualifier
        metadata = bool(header) or number == 1
        rows.append({
            'id': f'raw-2.0.1-{number:03}', 'line': number, 'literal': line,
            'heading': heading, 'patch_qualifier': qualifier,
            'status': 'metadata-only' if metadata else 'UNPROVEN', 'capabilities': [],
            'limit': ('Header/navigation only; no expanded contract.' if metadata else
                      'UNPROVEN: complete literal prose/example retained; no runtime or native observation.'),
        })
        if not header:
            occurrences.extend(find_occurrences(line, number, heading, qualifier))
        references.extend({
            'id': f'ref-2.0.1-{number:03}-{match.start():04}', 'line': number,
            'literal': match[0], 'status': 'UNPROVEN',
            'limit': 'Unexpanded linked/transcluded content; no inferred contracts.',
        } for match in sorted((match for pattern in REFERENCES for match in pattern.finditer(line)),
                              key=lambda match: (match.start(), -len(match[0]))))
    kinds = Counter(row['kind'] for row in occurrences)
    return {
        'schema': 'patch-api-literal-source-accounting/v1', 'patch': '2.0.1',
        'client_line': 'historical-retail', 'source_sha256': digest(raw.encode()),
        'source_rows': rows, 'headers': headers, 'occurrences': occurrences,
        'signature_rows': [row for row in occurrences if row['signature'] is not None],
        'references': references,
        'counts': {
            'nonblank_rows': len(rows), 'headers': len(headers),
            'occurrences': len(occurrences), 'by_kind': dict(sorted(kinds.items())),
            'signature_fragments': sum(row['signature'] is not None for row in occurrences),
            'prose_contract_rows': sum(row['status'] == 'UNPROVEN' for row in rows),
            'references': len(references), 'cvars': 0, 'runtime_observations': 0,
            'meaningful_closures': 0, 'native_observations': 0,
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    args.output.write_text(json.dumps(account(args.source.read_text()), indent=2,
                                      ensure_ascii=False) + '\n')


if __name__ == '__main__':
    main()
