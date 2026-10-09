"""Replay sealed historical source bytes; independent of Git/target/runtime."""
import json

from accounting import (
    EVIDENCE, ROOT, SOURCE, digest, extracted_text, frozen_inputs,
    register_for, validate_ledger,
)


def main():
    seals = json.loads((EVIDENCE / 'seals.json').read_bytes())
    for name, expected in seals.items():
        assert digest((ROOT / name).read_bytes()) == expected, f'seal: {name}'
    raw = frozen_inputs()
    ledger = json.loads((SOURCE / '2.3.0-page-coverage.json').read_bytes())
    result = validate_ledger(raw, ledger)
    for name, value in [('2.3.0-page-coverage.json', ledger),
                        ('2.3.0-wikitext-register.json', register_for(raw, ledger))]:
        expected = (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()
        assert (SOURCE / name).read_bytes() == expected, f'serialized bytes: {name}'
    assert (SOURCE / '2.3.0-api-changes.txt').read_bytes() == extracted_text(raw).encode(), 'extract bytes'
    print(json.dumps(dict(result, sealed_inputs=len(seals)), sort_keys=True))


if __name__ == '__main__':
    main()
