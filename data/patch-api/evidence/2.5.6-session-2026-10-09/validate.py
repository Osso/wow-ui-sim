#!/usr/bin/env python3
"""Replay only sealed Patch 2.5.6 SOURCE/configuration accounting."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import tomllib

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
SOURCE = ROOT / 'data/patch-api/sources'
TRANSCLUSION = '{{:Patch_1.15.9/API_changes}}'
BOUNDARY = '[Transcluded source: Patch_1.15.9/API_changes; not expanded]'
BASE = '864b4f7e46319704732ba82eef8ffc0757771610'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def check_seals():
    seals = json.loads((EVIDENCE / 'seals.json').read_bytes())
    for path, expected in seals.items():
        assert digest((ROOT / path).read_bytes()) == expected, f'seal: {path}'
    return len(seals)


def load_tool(name):
    path = EVIDENCE / f'historical-{name}.py'
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def render_source(raw):
    # The stock extractor rejects this literal transclusion. Replace only its
    # marker locally, without retrieving, expanding or asserting linked content.
    assert raw.count(TRANSCLUSION) == 1, 'literal transclusion'
    extractor = load_tool('extract_patch_non_inventory')
    return extractor.extract_text(
        raw.replace(TRANSCLUSION, BOUNDARY), canonical_patch_navigation=True
    )


def inventory_counts(raw):
    generator = load_tool('gen_patch_wikitext_register')
    sections = generator.split_sections(raw)
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        parsed, counts = generator.parse_section(section, sections.get(section, []))
        entries.extend(parsed)
        headers.extend(counts)
    assert not generator.TEMPLATE.search(raw), 'unexpected explicit API reference'
    return {
        'enumerated_api_occurrences': len(entries), 'header_counts': headers,
        'register_created': False,
        'removals': sum(entry['direction'] == 'removed' for entry in entries),
        'explicit_signatures': 0, 'local_summary_contracts': 0,
    }


def expand_features(features, feature):
    enabled = {feature}
    for child in features[feature]:
        if child in features:
            enabled.update(expand_features(features, child))
    return enabled


def configured_profiles():
    """Read the literal copied Rust arms/Cargo feature graph and manifest bytes.

    Bounded to this historical syntax, not a new runtime/profile classifier.
    Manifests contain paths, not TOC contents or measured interface versions.
    """
    config = EVIDENCE / 'configured-inputs'
    rust = (config / 'src/client_profile.rs').read_text()
    features = tomllib.loads((config / 'Cargo.toml').read_text())['features']
    arms = re.search(r'pub fn cache_subdir.*?match self \{(.*?)\n        \}',
                     rust, re.S)[1]
    names = re.findall(r'ClientProfile::(\w+) => "(\w+)"', arms)
    interface_arms = re.search(
        r'pub const fn interface_version.*?match self \{(.*?)\n        \}',
        rust.split('impl ClientProfile {', 1)[1], re.S
    )[1]
    epochs = dict(re.findall(r'RetailApiEpoch::(\w+) => (\d+)', rust))
    interfaces = {}
    for variants, value in re.findall(
        r'(ClientProfile::\w+(?: \| ClientProfile::\w+)*) => (\w+)',
        interface_arms
    ):
        for variant in re.findall(r'ClientProfile::(\w+)', variants):
            interfaces[variant] = value
    profiles = []
    for variant, name in names:
        feature = f'client-{name}'
        enabled = expand_features(features, feature)
        value = interfaces[variant]
        if value == 'RETAIL_API_INTERFACE_VERSION':
            versions = [int(version) for epoch, version in epochs.items()
                        if 'retail-' + epoch.removeprefix('Retail').replace('_', '-') in enabled]
            interface = max(versions)
        else:
            interface = int(value)
        path = config / f'data/blizzard-ui-files/{name}.txt'
        lines = path.read_text().splitlines()
        profiles.append({
            'variant': variant, 'feature': feature, 'cache_subdir': name,
            'configured_interface': interface,
            'manifest': {
                'path': path.relative_to(EVIDENCE).as_posix(),
                'sha256': digest(path.read_bytes()), 'entries': len(lines),
                'toc_entries': sum(line.lower().endswith('.toc') for line in lines),
                'mainline_toc_entries': sum(line.endswith('_Mainline.toc') for line in lines),
            },
        })
    return profiles


def validate_profile(profile):
    expected = {
        'code_revision': BASE, 'source_toc': 20506, 'source_interface_family': '205xx',
        'native_profile_correspondence': 'UNPROVEN', 'unsupported_client_diagnosis': None,
        'measurements': {key: 'not-performed' for key in
                         ['cache', 'runtime', 'model', 'native', 'full_ui']},
    }
    assert {key: profile[key] for key in expected} == expected, 'profile evidence'
    profiles = configured_profiles()
    assert profile['configured_profiles'] == profiles, 'configured profiles'
    config = EVIDENCE / 'configured-inputs'
    hashes = {path.relative_to(config).as_posix(): digest(path.read_bytes())
              for path in sorted(config.rglob('*')) if path.is_file()}
    assert profile['original_file_hashes'] == hashes, 'configuration input hashes'
    return len(profiles)


def validate(*, response, raw, pin, ledger, text, profile):
    page = json.loads(response)['query']['pages']['685353']
    revision, = page['revisions']
    for key, expected in [('pageid', 685353), ('title', 'Patch 2.5.6/API changes')]:
        assert page[key] == pin[key] == ledger['source'][key] == expected, key
    for key, expected in [('revid', 6778086), ('timestamp', '2026-07-22T05:42:00Z')]:
        assert revision[key] == pin[key] == ledger['source'][key] == expected, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert digest(raw) == pin['wikitext_sha256'] == '9ea30f11808982a5c3f0a051b5a41bd8f7472ec7450c8e3576cdb515a4f9978a', 'source hash'
    assert digest(response) == pin['response_sha256'] == '4b5c835e8f549a1e578be9a57b31c995c455d1f35560dffb44d74989d3a711f0', 'response hash'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'
    manifest = json.loads((EVIDENCE / 'frozen-manifest.json').read_bytes())
    assert pin == ledger['source'] == next(
        row for row in manifest['pages'] if row['version'] == '2.5.6'
    ), 'frozen source pin'
    registry_bytes = (EVIDENCE / 'frozen-registry.json').read_bytes()
    assert digest(registry_bytes) == manifest['registry_sha256'], 'frozen registry hash'
    registry = json.loads(registry_bytes)['pages']
    assert registry[-1]['version'] == '1.0.0', 'registry boundary'
    registered = next(row for row in registry if row['version'] == '2.5.6')
    assert all(registered[key] == pin[key] for key in registered), 'registry identity'
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'],
            ledger['scope'], ledger['later_registers']) == (
        'patch-source-accounting/v1', '2.5.6', 'tbc-classic', None,
        'source-and-configured-profile-accounting', []), 'client history'
    assert '* TOC: <code>20506</code>' in raw.decode().splitlines(), 'literal source TOC'
    expected_text = render_source(raw.decode())
    assert text == expected_text.encode(), 'plaintext reproduction'
    assert ledger['non_inventory_source']['sha256'] == digest(text), 'plaintext hash'
    assert ledger['non_inventory_source']['extractor_flags'] == [
        '--text-only', '--canonical-patch-navigation'
    ], 'extractor flags'
    assert ledger['non_inventory_source']['local_transform'] == {
        'from': TRANSCLUSION, 'to': BOUNDARY
    }, 'local transform'
    seeds = load_tool('extract_patch_non_inventory').seed_rows(expected_text, '2.5.6')
    rows = ledger['source_rows']
    nonblank = [(i, line) for i, line in enumerate(raw.decode().splitlines(), 1) if line.strip()]
    assert [row['source_id'] for row in rows] == [row['source_id'] for row in seeds], 'row accounting'
    assert len(rows) == len(nonblank), 'row accounting'
    for row, seed, (number, literal) in zip(rows, seeds, nonblank):
        assert (row['wikitext_line'], row['text_line'], row['source_text']) == (
            number, number, literal), 'literal row accounting'
        status = 'UNPROVEN' if literal == TRANSCLUSION else seed['status']
        assert row['status'] == status, 'source proof limit'
        assert row['capabilities'] == [], 'no publication/behavior credit'
        assert row['note'], 'explicit proof limit'
    inventory = inventory_counts(raw.decode())
    assert ledger['inventory'] == inventory, 'inventory accounting'
    transclusion_row, = [row for row in rows if row['source_text'] == TRANSCLUSION]
    expected_contract = {
        'source_id': transclusion_row['source_id'], 'status': 'UNPROVEN',
        'kind': 'unexpanded-blue-post-transclusion', 'target': 'Patch_1.15.9/API_changes',
        'member_identities': None, 'arguments': None, 'returns': None,
        'event_payloads': None, 'state_transitions': None, 'security_rules': None,
        'native_equivalence': None,
    }
    assert ledger['contracts'] == [expected_contract], 'transclusion contract'
    profile_count = validate_profile(profile)
    return {
        'source_rows': len(rows), 'statuses': dict(Counter(row['status'] for row in rows)),
        'enumerated_api_occurrences': inventory['enumerated_api_occurrences'],
        'header_counts': inventory['header_counts'], 'removals': inventory['removals'],
        'explicit_signatures': inventory['explicit_signatures'],
        'local_summary_contracts': inventory['local_summary_contracts'],
        'unexpanded_contracts': len(ledger['contracts']), 'configured_profiles': profile_count,
        'runtime_observations': 0, 'native_observations': 0, 'model_observations': 0,
    }


def main():
    sealed_inputs = check_seals()
    summary = validate(
        response=(EVIDENCE / 'source-response.json').read_bytes(),
        raw=(SOURCE / '2.5.6-api-changes.wikitext').read_bytes(),
        pin=json.loads((EVIDENCE / 'source-pin.json').read_bytes()),
        ledger=json.loads((SOURCE / '2.5.6-page-coverage.json').read_bytes()),
        text=(SOURCE / '2.5.6-api-changes.txt').read_bytes(),
        profile=json.loads((EVIDENCE / 'profile-observation.json').read_bytes()),
    )
    print(json.dumps(dict(summary, sealed_inputs=sealed_inputs), sort_keys=True))


if __name__ == '__main__':
    main()
