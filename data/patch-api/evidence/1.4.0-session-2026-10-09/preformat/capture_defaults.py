"""Capture own-base default outputs once; never overwrite frozen captures."""

import json
from pathlib import Path
import audit

EVIDENCE = Path(__file__).resolve().parent


def write_once(name, data):
    path = EVIDENCE / name
    assert not path.exists(), f"refuse overwrite: {name}"
    path.write_bytes(data)


def main():
    extractor = audit.load_historical("extract_patch_non_inventory")
    generator = audit.load_historical("gen_patch_wikitext_register")
    raw = (EVIDENCE / "source.wikitext").read_text()
    write_once("default-extract.txt", extractor.extract_text(raw).encode())
    errors = []
    for tool, function, argument in [
        (generator, "parse_symbol", "not a reference"),
        (extractor, "extract_text", None),
    ]:
        try:
            getattr(tool, function)(argument)
        except Exception as error:
            errors.append(
                {
                    "function": function,
                    "argument": argument,
                    "type": type(error).__name__,
                    "message": str(error),
                }
            )
        else:
            raise AssertionError("expected historical malformed-input error")
    write_once("default-errors.json", (json.dumps(errors, indent=2) + "\n").encode())
    print(
        json.dumps(
            {
                "extract_bytes": len(extractor.extract_text(raw).encode()),
                "errors": errors,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
